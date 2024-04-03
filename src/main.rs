mod bhc_commands;
mod file;
mod logging;
mod metadata;
mod workspace;

use bhc_commands::BhcShowDocumentParams;
use logging::Logging;
use metadata::file_metadata::FormattedCssFile;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};


const METADATA_PATH: &'static str = ".bhc/.meta/meta.json";
const CSS_METADATA_PATH: &'static str = ".bhc/.meta/css";
const HTML_METADATA_PATH: &'static str = ".bhc/.meta/html";
const SHARED_PATH: &'static str = ".bhc/.shared";
const VIRTUAL_PATH: &'static str = ".bhc/.virtual";

const EXT_HTML: &'static str = "html";
const EXT_CSS: &'static str = "css";

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

        // Give time to attach debugger
        // std::thread::sleep(std::time::Duration::from_secs(5));

        self.log_info("BHC language server initialized!").await;

        let workspaces = match self.get_workspaces().await {
            Ok(value) => value,
            Err(error) => {
                self.log_error(error).await;
                return
            }
        };

        for workspace in workspaces {
            self.initialize_workspace(&workspace).await;
        }
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.log_info(format!("File Opened: {}", params.text_document.uri)).await;

        // TODO: If HTML, then we need to check the HTML file metadata, and any imported CSS files from it. If there are more than 1 imported css file in the html file, we need to make a "virtual file" which gets saved to VIRTUAL_PATH, IT CURRENTLY READS THE HTML FILE DIRECTLY (WRONG)
        match params.text_document.language_id.as_str() {
            EXT_HTML => {
                //TODO: If it contains multiple then we put it into the .bhc/.virtual folder.

                let css_file_path = match self.get_css_file(params).await {
                    Ok(value) => value,
                    Err(error) => {
                        self.log_error(error).await;
                        return
                    }
                };

                // There are no CSS files to open anything
                if css_file_path.is_none() {
                    return
                }

                //TODO: This needs to pass back more information to colour the page.

                if let Some(css_path) = css_file_path {
                    let css_file_url = Url::parse(css_path.to_str().unwrap()).unwrap();

                    let params = BhcShowDocumentParams { 
                        uri: css_file_url,
                        file: FormattedCssFile::new() 
                    };
    
                    match self.client.send_request::<bhc_commands::BhcShowDocumentRequest>(params).await {
                        Ok(_) => (),
                        Err(error) => self.log_error(format!("Error occurred trying to open CSS file: {}", error)).await,
                    };
                }
            },

            EXT_CSS => {
                // TODO: If CSS, then we need to pass back the metadata to colour the lines and give right click options to reformat...
                
            }
            _ => ()
           
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.log_info(format!("File Closed: {}", params.text_document.uri)).await;
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        self.log_info("Workspace Folder Changed.").await;

        let added_workspaces = self.parse_workspaces(&params.event.added).await;

        for workspace in added_workspaces {
            self.initialize_workspace(&workspace).await;
        }
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

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend { client }).finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}