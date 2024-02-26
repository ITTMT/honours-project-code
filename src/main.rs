mod css;
mod file;
mod html;

use std::fmt::Display;
use std::path::Path;

use css::css_commands;
use html::html_commands;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
pub struct Backend{
    client: Client,
}

trait Logging {
    async fn log_info<M: Display>(&self, message:M);

    async fn log_error<M: Display>(&self, message:M);
} 

impl Logging for Backend {
    async fn log_info<M: Display>(&self, message: M) {
        self.client
        .log_message(MessageType::INFO, message)
        .await;
    }

    async fn log_error<M:Display>(&self, message: M) {
        self.client
        .log_message(MessageType::ERROR, message)
        .await;
    }
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
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.log_info(format!("File Opened: {}", params.text_document.uri)).await;

        let value = css_commands::produce_css_file(html_commands::get_css_files(&params.text_document.text));

        let x = ShowDocumentParams{
            uri: Url::from_file_path(Path::new(&value)).expect(""),
            external: None, 
            take_focus: Some(false), 
            selection: None
        };
        
        match self.client.show_document(x)
        .await{
            Ok(value) => if value == false {
                self.log_error("Unable to open CSS file").await
            },
            Err(error) => 
                self.log_error(format!("Error occurred trying to open CSS file: {}", error)).await,
        };

        let x = file::get_workspace_paths(self).await;

        self.log_info(format!("{:?}", x)).await;
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