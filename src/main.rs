mod bhc_commands;
mod file;
mod logging;
mod memory;
mod metadata;
mod error;


use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{env, fs};

use bhc_commands::BhcShowDocumentParams;
use error::Error;
use logging::Logging;
use metadata::workspace_metadata::{create_workspace_metadata, open_workspace_metadata, WorkspaceMetaData};
use memory::Memory;
use once_cell::sync::Lazy;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

static MEMORY: Lazy<Arc<Mutex<Memory>>> = Lazy::new(|| {
    let memory = Memory::new();
    Arc::new(Mutex::new(memory))
});

/* #region Language Server - tower_lsp */

#[derive(Debug, Clone)]
pub struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            offset_encoding: None,
            capabilities: ServerCapabilities {
                call_hierarchy_provider: None,
                code_action_provider: None,
                code_lens_provider: None,
                color_provider: None,
                completion_provider: None,
                declaration_provider: None,
                definition_provider: None,
                diagnostic_provider: None,
                document_formatting_provider: None,
                document_highlight_provider: None,
                document_link_provider: None,
                document_on_type_formatting_provider: None,
                document_range_formatting_provider: None,
                document_symbol_provider: None,
                execute_command_provider: None,
                experimental: None,
                folding_range_provider: None,
                hover_provider: None,
                implementation_provider: None,
                inlay_hint_provider: None,
                inline_value_provider: None,
                linked_editing_range_provider: None,
                moniker_provider: None,
                position_encoding: None,
                references_provider: None,
                rename_provider: None,
                selection_range_provider: None,
                semantic_tokens_provider: None,
                signature_help_provider: None,
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                type_definition_provider: None,
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                workspace_symbol_provider: None,
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.log_info("BHC language server initialized!").await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.log_info(format!("File Opened: {}", params.text_document.uri)).await;

        self.first_time_setup_check().await;

        if params.text_document.language_id == "html" {
            let mut error_string: String = "".to_string();
            let mut error_occurred = false;
            let mut css_file_path: PathBuf = Path::new("").to_path_buf();

            let memory = Arc::clone(&MEMORY);

            match self.produce_css_file(params, &memory.lock().unwrap()) {
                Ok(value) => css_file_path = value,
                Err(error) => {
                    error_occurred = true;
                    error_string = error;
                }
            };

            if error_occurred {
                self.log_error(error_string).await;
                return;
            }

            let css_file_url = Url::parse(css_file_path.to_str().unwrap()).unwrap();

            let params = BhcShowDocumentParams { uri: css_file_url };

            match self.client.send_request::<bhc_commands::BhcShowDocumentRequest>(params).await {
                Ok(_) => (),
                Err(error) => self.log_error(format!("Error occurred trying to open CSS file: {}", error)).await,
            };
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.log_info(format!("File Closed: {}", params.text_document.uri)).await;
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        self.log_info("Workspace Folder Changed.").await;

        let memory = Arc::clone(&MEMORY);

        let added_workspaces = match self.get_workspace_paths(params.event.added) {
            Ok(value) => value,
            Err(error) => {
                self.log_error(error).await;
                return;
            }
        };

        let removed_workspaces = match self.get_workspace_paths(params.event.removed) {
            Ok(value) => value,
            Err(error) => {
                self.log_error(error).await;
                return;
            }
        };

        memory.lock().unwrap().add_workspaces(added_workspaces);
        memory.lock().unwrap().remove_workspaces(removed_workspaces);
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.log_info("watched files have changed!").await;
    }

    async fn will_save(&self, params: WillSaveTextDocumentParams) {
        self.log_info(format!("Saving File: {}", params.text_document.uri)).await;
    }

    async fn did_save(&self, x: DidSaveTextDocumentParams) {
        self.log_info(format!("Saved files changed: {}", x.text_document.uri)).await;
    }
}
/* #endregion */

impl Backend {
    pub async fn first_time_setup_check(&self) {
        let memory = Arc::clone(&MEMORY);

        if !memory.lock().unwrap().is_ready() {
            let workspace_paths = match self.client.workspace_folders().await {
                Ok(value) => match value {
                    Some(value) => value,
                    None => {
                        memory.lock().unwrap().add_workspace(env::temp_dir());
                        return;
                    }
                },
                Err(error) => {
                    self.log_error(format!("Error occurred trying to get workspace folders: {}", error)).await;
                    return;
                }
            };

            match self.get_workspace_paths(workspace_paths) {
                Ok(value) => memory.lock().unwrap().add_workspaces(value),
                Err(error) => self.log_error(error).await,
            };

            // need to iterate through each html and css file to create the metadata.

            let workspace_paths = memory.lock().unwrap().get_all_workspaces();

            let mut error_handler = Error::new();

            workspace_paths
            .into_iter()
            .for_each(|path| {
                let mut found_paths: Vec<PathBuf> = Vec::new();
                recursive_file_search(&path, &mut found_paths);

                // need to first of all check if the workspace is a web project, otherwise I don't need to do anything for it.

                let mut metadata_file_path = path.clone();
                metadata_file_path.push(".bhc/.meta/meta.json");

                let mut x = if found_paths.contains(&metadata_file_path) {
                    match open_workspace_metadata(&metadata_file_path) {
                        Ok(value) => value,
                        Err(error) => {
                            error_handler.handle_error(error);
                            return;
                        }
                    }
                } else {
                    match create_workspace_metadata(&metadata_file_path) {
                        Ok(value) => value,
                        Err(error) => {
                            error_handler.handle_error(error);
                            return;
                        }
                    }
                };

                // I've got the workspace metadata primed.
                // I need to read it to see if it contains all the css and html files in it
                // I need to check if each file has been saved more recently than this file has been last_updated
                // if the file has, then I need to update the metadata
                // DO NOT change the last_updated until the end of the iteration.
                // because there is a 2-way link in the html and css files, that will need to be done at the end with a map(?)
                // 

                
                // get all the css files
                // get all the html files
                // get all the json files

                // 


            });

            if error_handler.error_occurred {
                self.log_error(error_handler.error_string).await;
            }
        }
    }
}

fn error_handle<S: Into<String>>(error_string: &mut String, error_flag: &mut bool, new_error: String){
    error_string = new_error;
    error_flag = true;
}

pub fn recursive_file_search(path: &PathBuf, found_paths: &mut Vec<PathBuf>) {
    match fs::read_dir(path) {
        Ok(value) => value.for_each(|res| match res {
            Ok(value) => {
                if value.path().is_dir() {
                    recursive_file_search(&value.path(), found_paths);
                } else if value.path().is_file() {
                    found_paths.push(value.path())
                }
            }
            Err(_) => (),
        }),
        Err(_) => (),
    }
}



#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend { client }).finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use crate::recursive_file_search;

    #[test]
    fn test123() {
        let path = Path::new("D:/programming/web-dev/xd/html").to_path_buf();

        let xd = fs::read_dir(path).unwrap();

        xd.for_each(|x| println!("{:?}", x))
    }

    #[test]
    fn test_recursive_file_search() {
        let mut x: Vec<PathBuf> = Vec::new();

        let path = Path::new("D:/programming/web-dev/xd").to_path_buf();

        recursive_file_search(path, &mut x);

        x.iter().for_each(|path| println!("{:?}", path))
    }
}
