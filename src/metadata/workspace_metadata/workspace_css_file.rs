use serde::{Deserialize, Serialize};

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
}