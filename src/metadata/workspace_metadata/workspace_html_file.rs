use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{metadata::html_metadata::HtmlMetaData, HTML_METADATA_PATH};

use super::{id_to_json_file_name, WorkspaceMetaData};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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

	pub fn parse(html_metadata: &HtmlMetaData) -> WorkspaceHtmlFile {
        WorkspaceHtmlFile {
            id: html_metadata.id.clone(),
            file_name: html_metadata.file_name.clone(),
            absolute_path: html_metadata.absolute_path.clone(),
            css_files: {
				if let Some(sheets) = &html_metadata.css_sheets {
					Some(sheets.into_iter().map(|sheet| sheet.id).collect())
				} else {
					None
				}
			},
        }
    }

	pub fn update(&self, workspace_metadata: &WorkspaceMetaData) -> Result<WorkspaceHtmlFile, String> {
		let metadata_path = PathBuf::from(&workspace_metadata.workspace_path).join(HTML_METADATA_PATH).join(id_to_json_file_name(&self.id));

		let html_file_contents = match fs::read_to_string(&metadata_path) {
			Ok(value) => value,
			Err(error) => return Err(format!("Error trying to read HTML Metadata File ({}): {:?}", &self.absolute_path, error))
		};

		let mut html_metadata: HtmlMetaData = match serde_json::from_str(&html_file_contents) {
			Ok(value) => value,
			Err(error) => return Err(format!("Error trying to deserialize HTML Metadata File ({}): {:?}", &self.absolute_path, error))
		};

		// Updates the HtmlMetaData and returns a new version of WorkspaceHtmlFile to reflect its contents
		let workspace_html_file = match html_metadata.update_metadata(&metadata_path, workspace_metadata) {
			Ok(value) => value,
			Err(error) => return Err(error)
		};

		Ok(workspace_html_file)
	} 
}
