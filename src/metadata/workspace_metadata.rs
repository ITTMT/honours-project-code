pub mod workspace_css_file;
pub mod workspace_html_file;

use std::{fs::{self, File}, path::PathBuf};
use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{self, Deserialize, Serialize};

use crate::file::create_dir_and_file;

use self::{workspace_css_file::WorkspaceCssFile, workspace_html_file::WorkspaceHtmlFile};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WorkspaceMetaData {
    pub workspace_path: String,

    #[serde(with = "ts_seconds")]
    pub last_updated: DateTime<Utc>,

    pub html_files: Vec<WorkspaceHtmlFile>,
    pub css_files: Vec<WorkspaceCssFile>,
}

impl WorkspaceMetaData {
    pub fn new() -> WorkspaceMetaData {
        WorkspaceMetaData {
            workspace_path: String::new(),
            last_updated: Utc::now(),
            html_files: Vec::new(),
            css_files: Vec::new(),
        }
    }

    pub fn get_next_available_css_id(&self) -> u32 {
        let mut smallest_id: u32 = 1;

        self.css_files
        .iter()
        .for_each(|x| {
            if x.id == smallest_id {
                smallest_id += 1;
            } else {
                return
            }
        });

        smallest_id
    }

    pub fn get_next_available_html_id(&self) -> u32 {
        let mut smallest_id: u32 = 1;

        self.html_files
        .iter()
        .for_each(|x| {
            if x.id == smallest_id {
                smallest_id += 1;
            } else {
                return
            }
        });

        smallest_id
    }

    // pub fn get_css_metadata(&self, css_file_path: &PathBuf) -> CssMetaData {
        


    // }

    
    pub fn add_css_file(&mut self, css_file_metadata: WorkspaceCssFile) {
        self.css_files.push(css_file_metadata)
    }

    /// Save the WorkspaceMetaData back to `meta.json`
    /// Returns `Ok(())` if it succeeds
    /// Returns `Err(String)` if it is unable to save the file. Effectively meaning the extension won't work... 
    pub fn update_metadata(&mut self, file_path: &PathBuf) -> Result<(), String> {
        
        match create_dir_and_file(&file_path) {
			Ok(_) => (),
			Err(error) => return Err(error)
		};

		self.last_updated = Utc::now();

		match fs::write(&file_path, serde_json::to_string_pretty(&self).unwrap()) {
			Ok(_) => return Ok(()),
			Err(error) => return Err(format!("Error writing workspace metadata to file: ({:?}) {:?}", &file_path, error))
		};

    }
}

pub fn id_to_json_file_name(id: &u32) -> String {
    let mut file_name = String::from(id.to_string());
    file_name.push_str(".json");

    file_name
}

pub fn create_workspace_metadata(metadata_path: &PathBuf, workspace_path: &PathBuf) -> Result<WorkspaceMetaData, String> {
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

    match fs::write(&metadata_path, create_default_metadata(workspace_path)) {
        Ok(_) => (),
        Err(error) => {
            return Err(format!("Error trying to initialize metadata file: ({:?}) {:?}", &metadata_path, error))
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
        Err(error) => return Err(format!("Error deserializing metadata json: ({:?})\n JSON String = {}\n {:?}",&metadata_path, metadata_json_string, error))
    };

    Ok(metadata_json)
}

fn create_default_metadata(workspace_path: &PathBuf) -> String {
    let mut metadata = WorkspaceMetaData::new();
    metadata.workspace_path = workspace_path.to_str().unwrap().to_string();
    metadata.last_updated = Utc::now();

    serde_json::to_string_pretty(&metadata).unwrap()
}


#[cfg(test)]
mod test {
    use super::{WorkspaceCssFile, WorkspaceMetaData};


    #[test]
    fn get_next_available_css_id_test() {
        let mut metadata = WorkspaceMetaData::new();

        let mut css_file_1 = WorkspaceCssFile::new();
        css_file_1.id = 1;
        let mut css_file_2 = WorkspaceCssFile::new();
        css_file_2.id = 2;
        let mut css_file_3 = WorkspaceCssFile::new();
        css_file_3.id = 3;
        let mut css_file_4 = WorkspaceCssFile::new();
        css_file_4.id = 4;

        metadata.css_files = vec![css_file_1, css_file_2, css_file_3, css_file_4];

        assert_eq!(5, metadata.get_next_available_css_id());

        metadata.css_files[3].id = 5;

        assert_eq!(4, metadata.get_next_available_css_id());
    }
}