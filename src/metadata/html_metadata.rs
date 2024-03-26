use std::{fs, path::PathBuf};

use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::file::{create_dir_and_file, get_css_file_paths};

use super::{css_metadata::css_file::CssFile, workspace_metadata::{workspace_html_file::WorkspaceHtmlFile, WorkspaceMetaData}};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct HtmlMetaData {
	pub id: u32,
	pub file_name: String,
	pub absolute_path: String,

	#[serde(with = "ts_seconds")]
	pub last_updated: DateTime<Utc>,
	
	#[serde(skip_serializing_if = "Option::is_none")]
	pub css_sheets: Option<Vec<CssFile>>,
}

impl HtmlMetaData {
	pub fn new() -> HtmlMetaData {
		HtmlMetaData {
			id: 0,
			file_name: String::new(),
			absolute_path: String::new(),
			last_updated: Utc::now(),
			css_sheets: None,
		}
	}

	/// Update the `HtmlMetaData.css_sheets` to contain all the necessary imported sheets.
	pub fn update_css_sheets(&mut self, workspace_metadata: &WorkspaceMetaData) -> Result<WorkspaceHtmlFile, String> {
		let html_string = match fs::read_to_string(&self.absolute_path) {
			Ok(value) => value,
			Err(error) => return Err(format!("Error occurred trying to open HTML file ({}): {:?}", self.absolute_path, error))
		};
		
		let css_file_paths = match get_css_file_paths(&PathBuf::from(&self.absolute_path), &html_string) {
			Ok(value) => value,
			Err(error) => return Err(error)
		};
		
		if css_file_paths.is_empty() {
			self.css_sheets = None;
		} else {
			for file_path in css_file_paths {
				if let Some(id) = workspace_metadata.get_css_file_id(&file_path) {
					if let Some(sheets) = &mut self.css_sheets {
						sheets.retain(|sheet| sheet.id != id);
					}

					let css_file = CssFile { 
						id: id, 
						file_name: file_path.file_name().unwrap().to_str().unwrap().to_string(), 
						absolute_path: file_path.to_str().unwrap().to_string() 
					};

					if let Some(sheets) = &mut self.css_sheets {
						sheets.push(css_file);
					} else {
						self.css_sheets = Some(vec![css_file]);
					}
				}
			}
		}

		if let Some(sheets) = &mut self.css_sheets {
			if sheets.is_empty() {
				self.css_sheets = None
			}
		}

		Ok(WorkspaceHtmlFile {
			id: self.id.clone(),
			file_name: self.file_name.clone(),
			absolute_path: self.absolute_path.clone(),
			css_files: {
				if let Some(sheets) = &self.css_sheets {
					Some(sheets.into_iter().map(|sheet| sheet.id).collect())
				} else {
					None
				}
			}
		})

	}

	pub fn create_metadata(metadata_path: &PathBuf, file_path: &PathBuf, id: &u32) -> Result<HtmlMetaData, String> {
		match create_dir_and_file(&metadata_path) {
			Ok(_) => (),
			Err(error) => return Err(error)
		};

		let mut metadata = HtmlMetaData::new();

        metadata.id = *id;
		metadata.file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();
		metadata.absolute_path = file_path.to_str().unwrap().to_string();
		metadata.last_updated = Utc::now();

		match fs::write(&metadata_path, serde_json::to_string_pretty(&metadata).unwrap()) {
			Ok(_) => return Ok(metadata),
			Err(error) => return Err(format!("Error writing metadata to file: ({:?}) {:?}", &metadata_path, error))
		};
	}
	
	pub fn update_metadata(&mut self, metadata_path: &PathBuf, workspace_metadata: &WorkspaceMetaData) -> Result<WorkspaceHtmlFile, String> {
        let mut new_metadata = self.clone();

		let new_workspace_metadata = match new_metadata.update_css_sheets(workspace_metadata) {
			Ok(value) => value,
			Err(error) => return Err(error)
		};

        new_metadata.last_updated = Utc::now();

        match fs::write(&metadata_path, serde_json::to_string_pretty(&new_metadata).unwrap()) {
			Ok(_) => return Ok(new_workspace_metadata),
			Err(error) => return Err(format!("Error writing metadata to file: ({:?}) {:?}", &metadata_path, error))
		};
	}

}
