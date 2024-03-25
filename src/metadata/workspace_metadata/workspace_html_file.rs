use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkspaceHtmlFile {
    pub id: u32,
    pub file_name: String,
    pub absolute_path: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_files: Option<Vec<u32>>, // none if there are no link href css files in them.
}

impl WorkspaceHtmlFile {
	pub fn new() -> WorkspaceHtmlFile {
		WorkspaceHtmlFile {
			id: 0,
			file_name: String::new(),
			absolute_path: String::new(),
			css_files: None,
		}
	}
}
