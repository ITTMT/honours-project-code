mod bhc_commands;
mod file;
mod logging;
mod memory;
mod metadata;

use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use bhc_commands::BhcShowDocumentParams;
use logging::Logging;
use memory::Memory;
use once_cell::sync::Lazy;
use tower_lsp::jsonrpc::Result;
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
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info:     None,
            offset_encoding: None,
            capabilities:    ServerCapabilities {
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
                        supported:            Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations:   None,
                }),
                workspace_symbol_provider: None,
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.log_info("BHC language server initialized!").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.log_info(format!("File Opened: {}", params.text_document.uri)).await;

        self.first_time_setup_check().await;

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
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
