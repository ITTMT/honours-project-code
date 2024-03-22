mod bhc_commands;
mod file;
mod logging;
mod memory;
mod error;
mod metadata;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::fs;

use bhc_commands::BhcShowDocumentParams;
use chrono::{DateTime, Utc};
use error::Error;
use logging::Logging;
use metadata::workspace_metadata::id_to_json_file_name;
use metadata::{css_metadata::CssMetaData, workspace_metadata::{create_workspace_metadata, open_workspace_metadata}};
use memory::Memory;
use metadata::{css_metadata, GroupedFiles, Metadata};
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
            let mut error_string = String::new();
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
                    None => return
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

            let workspace_paths = memory.lock().unwrap().get_all_workspaces().clone();

            let mut error_handler = Error::new();

            workspace_paths
            .iter()
            .for_each(|workspace_path| {
                let mut css_metadata_path = workspace_path.clone();
                css_metadata_path.push(".bhc/.meta/css");
                let mut html_metadata_path = workspace_path.clone();
                html_metadata_path.push(".bhc/.meta/html");

                let mut found_paths: Vec<PathBuf> = Vec::new();
                recursive_file_search(workspace_path, &mut found_paths);

                // grouped files are all the files that can be found in the workspace that are either .html, .css, or .json
                let grouped_files: GroupedFiles = found_paths.clone().into();

                // need to first of all check if the workspace is a web project, otherwise I don't need to do anything for it.

                let mut metadata_file_path = workspace_path.clone();
                metadata_file_path.push(".bhc/.meta/meta.json");

                let mut metadata = if found_paths.contains(&metadata_file_path) {
                    match open_workspace_metadata(&metadata_file_path) {
                        Ok(value) => value,
                        Err(error) => {
                            error_handler.handle_error(error);
                            return;
                        }
                    }
                } else {
                    match create_workspace_metadata(&metadata_file_path, &workspace_path) {
                        Ok(value) => value,
                        Err(error) => {
                            error_handler.handle_error(error);
                            return;
                        }
                    }
                };

                let css_metadata_map: HashMap<PathBuf, PathBuf> = grouped_files.map_css_files(); 

                // for each file in the grouped files, we need to find it's corresponding metadata file, we get this by going through the grouped_file filename (pathbuf) and seeing if any of the metadata css_files share the same name, the problem is that the css metadata file names will be their <id>.json (1.json, 2.json... etc) so we need to extract(?) the associated filename with their Id's, so maybe we need to make a Map of KVP's? (Key: PathBuf, value: int ID?)

                // TODO: Need to check the metadata files if it exists.

                grouped_files
                .css_files
                .iter()
                .for_each(|css_file| match css_metadata_map.get_key_value(css_file) {
                    Some((_,css_metadata_file_path)) => {
                        match fs::read_to_string(css_metadata_file_path) {
                            Ok(contents) => {

                                // it exists so we can deserialize it
                                let mut css_metadata: CssMetaData = serde_json::from_str(&contents).unwrap();

                                // get the metadata of the original file
                                let css_file_metadata = fs::metadata(&css_metadata.absolute_path).unwrap();

                                let file_last_modified: DateTime<Utc> = css_file_metadata.modified().unwrap().into();

                                // if the original file has been updated more recently than the proclaimed last_updated time then we need to update the contents of 
                                if file_last_modified > css_metadata.last_updated {
                                    match css_metadata.update_metadata(css_metadata_file_path) {
                                        Ok(_) => (),
                                        Err(error) => {
                                            error_handler.handle_error(error);
                                            return
                                        }
                                    };
                                }
                            },
                            Err(error) => ()
                        }

                    }, 
                    None => {
                        // we cannot know what sheets have been imported ahead of time
                        // we can know what styles there are though
                        // get the next available id
                        let id = metadata.get_next_available_css_id();

                        let file_name = id_to_json_file_name(&id);

                        let mut save_path = css_metadata_path.clone();
                        save_path.push(&file_name);

                        // create the file <id>.json
                        match CssMetaData::create_metadata(&save_path, css_file) {
                            Ok(_) => (),
                            Err(error) => {
                                error_handler.handle_error(error);
                                return
                            }
                        };
                        
                        // 
                    } // We need to create the metadata file in the .bhc/.meta/css folder. We will get the necessary <id>.json from the list in the metadata variable 
                    
                    // TODO: Add the cssmetadata to the metadata so the loop can continue
                    // TODO: Once we are done all the files we save the metadata.
                });

                if error_handler.error_occurred {
                    return
                }

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

        recursive_file_search(&path, &mut x);

        x.iter().for_each(|path| println!("{:?}", path))
    }
}
