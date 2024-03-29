use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct FormattedCssFile {
    pub included_files: Vec<FileMetaData>,
    pub lines: Vec<Option<FileMetaData>>, // Line number = index + 1 -> Some if line contains anything, nothing if white space
}

impl FormattedCssFile {
	pub fn new() -> FormattedCssFile {
		FormattedCssFile {
			included_files: Vec::new(),
			lines: Vec::new(),
		}
	}


	//TODO: Add function to format the files 
}



#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct FileMetaData {
    pub id: u32,
    pub file_name: String,
    pub is_shared: bool,
}

impl FileMetaData {
	pub fn new() -> FileMetaData {
		FileMetaData {
			id: 0,
			file_name: String::new(),
			is_shared: false,
		}
	}
}
