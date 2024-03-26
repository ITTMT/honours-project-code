use std::{collections::HashMap, ffi::OsStr, fs, path::PathBuf};

use crate::{CSS_METADATA_PATH, HTML_METADATA_PATH, METADATA_PATH, VIRTUAL_PATH};

use self::{css_metadata::CssMetaData, html_metadata::HtmlMetaData};

pub mod css_metadata;
pub mod workspace_metadata;
pub mod html_metadata;


#[derive(Debug, PartialEq)]
pub struct GroupedFiles {
	pub html_files: Vec<PathBuf>,
	pub css_files: Vec<PathBuf>,
	pub json_files: GroupedJsonFiles,
}

#[derive(Debug, PartialEq)]
pub struct GroupedJsonFiles {
	pub meta_file: PathBuf,
	pub html_files: Vec<PathBuf>,
	pub css_files: Vec<PathBuf>,
}

impl GroupedJsonFiles {
	pub fn new() -> GroupedJsonFiles {
		GroupedJsonFiles {
			meta_file: PathBuf::new(),
			html_files: Vec::new(),
			css_files: Vec::new(),
		}
	}
}

impl GroupedFiles {
	pub fn new() -> GroupedFiles {
		GroupedFiles {
			html_files: Vec::new(),
			css_files: Vec::new(),
			json_files: GroupedJsonFiles::new(),
		}
	}

	pub fn sort_files(&mut self, unorganised_files: &Vec<PathBuf>) {
		for file_path in unorganised_files {
			if let Some(path) = file_path.to_str() {
				if path.contains(VIRTUAL_PATH){
					continue
				}
			}

			match file_path.extension().and_then(OsStr::to_str) {
				Some("css") => self.css_files.push(file_path.clone()),
				Some("html") => self.html_files.push(file_path.clone()),
				Some("json") => {
					if file_path.ends_with(METADATA_PATH) {
						self.json_files.meta_file = file_path.clone();
					} else {
						if let Some(path) = file_path.parent() {
							if path.ends_with(CSS_METADATA_PATH) {
								self.json_files.css_files.push(file_path.clone())
							} else if path.ends_with(HTML_METADATA_PATH) {
								self.json_files.html_files.push(file_path.clone())
							}
						}
					}
				},
				
				_ => (),
			}
		}
	}

	/// From the JSON CSS Files, create a HashMap of `CSS Absolute Path` Key and `Metadata File Path` Values
	pub fn map_css_files(&self) -> HashMap<PathBuf, PathBuf> {
		let mut css_map: HashMap<PathBuf, PathBuf> = HashMap::new();
		
		self.json_files
		.css_files
		.iter()
		.for_each(|metadata_file_path|{
			let json_string = fs::read_to_string(metadata_file_path).unwrap(); // File will exist (only time it might not is if the person deletes a file between the creation of the vector and the reading of the file)

			let parsed_css_file: CssMetaData = serde_json::from_str(&json_string).unwrap();

			let css_file_path = PathBuf::from(&parsed_css_file.absolute_path);

			css_map.insert(css_file_path,metadata_file_path.clone());
		});
		
		css_map
	}

	/// From the JSON HTML Files, create a HashMap of `HTML Absolute Path` Key and `Metadata File Path` Values
	pub fn map_html_files(&self) -> HashMap<PathBuf, PathBuf> {
		let mut html_map: HashMap<PathBuf, PathBuf> = HashMap::new();
		
		for metadata_file_path in &self.json_files.html_files {
			let json_string = fs::read_to_string(metadata_file_path).unwrap(); // File will exist (only time it might not is if the person deletes a file between the creation of the vector and the reading of the file)
	
			let parsed_html_file: HtmlMetaData = serde_json::from_str(&json_string).unwrap();
	
			let html_file_path = PathBuf::from(&parsed_html_file.absolute_path);
	
			html_map.insert(html_file_path,metadata_file_path.clone());
		}
		
		html_map
	}

	pub fn contains_workspace_metadata(&self) -> bool {
		self.json_files.meta_file.components().count() > 0
	}
}

impl Into<GroupedFiles> for Vec<PathBuf> {
	fn into(self) -> GroupedFiles {
		let mut grouped_files = GroupedFiles::new();

		grouped_files.sort_files(&self);

		grouped_files
	}
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;
	use crate::metadata::{GroupedFiles, GroupedJsonFiles};

	#[test]
	fn sort_files_test() {
		let path1: PathBuf = PathBuf::from("C:/random/file.css");
		let path2: PathBuf = PathBuf::from("C:/random/file.html");
		let path3: PathBuf = PathBuf::from("C:/random/.bhc/.meta/file.json");


		let paths: Vec<PathBuf> = vec![path1.clone(), path2.clone(), path3.clone()];

		let expected = GroupedFiles {
			css_files: vec![path1.clone()],
			html_files: vec![path2.clone()],
			json_files: GroupedJsonFiles {
				meta_file: path3.clone(),
				html_files: Vec::new(),
				css_files: Vec::new()
			},
		};

		let mut actual = GroupedFiles::new();

		actual.sort_files(&paths);

		assert_eq!(actual, expected);
	}
}