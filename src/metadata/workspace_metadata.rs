use std::{fs::{self, File}, path::PathBuf};

use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{self, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkspaceMetaData {
    pub workspace_path: String,

    #[serde(with = "ts_seconds")]
    pub last_updated: DateTime<Utc>,

    pub html_files: Vec<WorkspaceHtmlFile>,
    pub css_files: Vec<WorkspaceCssFile>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkspaceHtmlFile {
    pub id: u32,
    pub file_name: String,
    pub absolute_path: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_files: Option<Vec<u32>>, // none if there are no link href css files in them.
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkspaceCssFile {
    pub id: u32,
    pub file_name: String,
    pub absolute_path: String,
    pub is_shared: bool, // is the file in the .bhc/.shared/ folder

    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_files: Option<Vec<u32>>, // none if is_shared, or no html files reference it
}

pub fn create_workspace_metadata(metadata_path: &PathBuf) -> Result<WorkspaceMetaData, String> {
    let metadata_dir = match metadata_path.parent() {
        Some(value) => value.to_path_buf(),
        None => {
            return Err(format!("Error occurred trying to get the workspace metadata folder: {:?}", &metadata_path))
        }
    };

    match File::open(&metadata_path) {
        Ok(_) => (),
        Err(_) => {
            match fs::create_dir_all(&metadata_dir){
                Ok(_) =>  {
                    match File::create(&metadata_path) {
                        Ok(_) => (),
                        Err(error) => return Err(format!("Couldn't create necessary directories: ({:?}) {:?}", &metadata_path, error))
                    }
                },
                Err(error) => return Err(format!("Couldn't create metadata file: ({:?}) {:?}", &metadata_path, error))
            };
        }
    };

    match open_workspace_metadata(&metadata_path) {
        Ok(value) => Ok(value),
        Err(error) => Err(error)
    }
   
}

pub fn open_workspace_metadata(metadata_path: &PathBuf) -> Result<WorkspaceMetaData, String> {
    let metadata_json_string = match fs::read_to_string(&metadata_path) {
        Ok(value) => value,
        Err(error) => return Err(format!("Error reading metadata file to string: ({:?}) {:?}", &metadata_path, error))
    };


    let metadata_json: WorkspaceMetaData = match serde_json::from_str(&metadata_json_string) {
        Ok(value) => value,
        Err(error) => return Err(format!("Error deserializing metadata json: ({:?}) \n JSON String = {} \n {:?}",&metadata_path, metadata_json_string, error))
    };

    Ok(metadata_json)
}
