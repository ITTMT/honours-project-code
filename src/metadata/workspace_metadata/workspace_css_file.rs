use serde::{Deserialize, Serialize};

use crate::{metadata::css_metadata::CssMetaData, SHARED_PATH};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkspaceCssFile {
    pub id: u32,
    pub file_name: String,
    pub absolute_path: String,
    pub is_shared: bool, // is the file in the .bhc/.shared/ folder

    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_files: Option<Vec<u32>>, // none if is_shared, or no html files reference it
}

impl WorkspaceCssFile {
    pub fn new() -> WorkspaceCssFile {
        WorkspaceCssFile {
            id: 0,
            file_name: String::new(),
            absolute_path: String::new(),
            is_shared: false,
            html_files: None
        }
    }

    pub fn parse(css_metadata: &CssMetaData) -> WorkspaceCssFile {
        WorkspaceCssFile {
            id: css_metadata.id.clone(),
            file_name: css_metadata.file_name.clone(),
            absolute_path: css_metadata.absolute_path.clone(),
            is_shared: check_is_shared(&css_metadata.absolute_path),
            html_files: None,
        }
    }
}

fn check_is_shared(absolute_path: &str) -> bool {
    absolute_path.contains(SHARED_PATH)
}