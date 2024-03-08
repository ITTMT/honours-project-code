mod file;
mod bhc_commands;
mod logging;
mod memory;

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use file::Files;
use logging::Logging;
use memory::Memory;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};


// This will last the entire lifetime of the server.
// Reduces the need to call for workspace changes every time, as it only needs to be known at the start
// and when it gets changed.
static MEMORY: Mutex<Memory> = Mutex::new(Memory { workspace_folders: Vec::new(), });

fn access_memory() -> MutexGuard<'static, Memory> {
    MEMORY.lock().unwrap()
}

#[derive(Debug)]
pub struct Backend{
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
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
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
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

        let workspace_paths = match self.client.workspace_folders().await {
            Ok(value) => value,
            Err(error) => {
                self.log_error(format!("Error occurred trying to get workspace folders: {}", error)).await;
                None
            }
        };

        let x = self.get_workspace_paths(workspace_paths).await;

        access_memory().add_workspaces(x);
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.log_info(format!("File Opened: {}", params.text_document.uri)).await;

        let folder = match access_memory().get_only_folder(){
            Some(value) => value,
            None => PathBuf::new()
        };



// TODO: Need to change this to pass in the html PathBuf
// Create dotfolder is pointless, it can just create the entire path, the 
// file structure of the .bhc folder should be mirror of the actual file structure.
// 

        let save_folder = self.create_dotfolder(&folder).expect("");
        let file_name = self.get_only_filename(&params.text_document.uri.to_string()).expect("");
        let absolute_css_paths = self.get_css_files(&params.text_document.uri.to_string(), &params.text_document.text);

        let value = self.produce_css_file(&file_name, &save_folder, absolute_css_paths).await.unwrap();

        let abc = value.into_os_string().into_string().unwrap();
        let y = bhc_commands::BhcShowDocumentParams {
            uri: Url::parse(&abc).unwrap(),
        };

        match self.client.send_request::<bhc_commands::BhcShowDocumentRequest>(y).await {
            Ok(_) => (),
            Err(error) => self.log_error(format!("Error occurred trying to open CSS file: {}", error)).await
        };
        
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.log_info(format!("File Closed: {}", params.text_document.uri)).await;
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.log_info("Workspace Folder Changed.").await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.log_info("watched files have changed!").await;
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