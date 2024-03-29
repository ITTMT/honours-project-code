use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::{request::Request, Url};

use crate::metadata::file_metadata::FormattedCssFile;

#[derive(Debug)]
pub enum BhcShowDocumentRequest {}

impl Request for BhcShowDocumentRequest {
    type Params = BhcShowDocumentParams;
    type Result = ();
    const METHOD: &'static str = "bhc/ShowDocumentRequest";
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct BhcShowDocumentParams {
    /// The actual message
    pub uri: Url,
    pub file: FormattedCssFile
}
