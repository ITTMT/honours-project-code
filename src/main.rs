mod css;
mod file;
mod html;

use css::css_commands;
use html::html_commands;
use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
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
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["bhc.open_file".to_string()],
                    work_done_progress_options: Default::default(),
                }),
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
        self.client
            .log_message(MessageType::INFO, "BHC language server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, format!("File Opened: {}", params.text_document.uri))
            .await;

        let value = css_commands::produce_css_file(html_commands::get_css_files(&params.text_document.text));

        let x = ShowDocumentParams{
            uri: Url::parse(&"C:/Users/Ollie/Documents/serverexampletest/css_files/stylesheet_1.css").expect(""),
            external: None, 
            take_focus: None, 
            selection: None
        };
        
        let _ = self.client.show_document(x)
        .await;

        self.client
        .log_message(MessageType::INFO, format!("Testing 123 C:/Users/Ollie/Documents/serverexampletest/css_files/stylesheet_1.css")).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, format!("File Closed: {}", params.text_document.uri))
            .await;
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, "Workspace Folder Changed.")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.client
            .log_message(MessageType::INFO, "watched files have changed!")
            .await;
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        self.client
        .log_message(MessageType::INFO, format!("Command Executed {:?}", params))
        .await;

        
        return Ok(Some(Value::String("C:/Users/Ollie/Documents/serverexampletest/css_files/stylesheet_1.css".to_string())));
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