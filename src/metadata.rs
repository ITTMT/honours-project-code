use std::{collections::HashMap, ffi::OsStr, fs, path::{Path, PathBuf}};

use self::css_metadata::CssMetaData;

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

pub trait Metadata<T> {
	fn create_metadata(metadata_path: &PathBuf, file_path: &PathBuf) -> Result<T, String>;

	fn update_metadata(&mut self, metadata_save_path: &PathBuf) -> Result<T, String>;
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
		unorganised_files
		.iter()
		.for_each(|file_path| match file_path.extension().and_then(OsStr::to_str) {
			Some("css") => self.css_files.push(file_path.clone()),
			Some("html") => self.html_files.push(file_path.clone()),
			Some("json") => {
				// we discard json files that are not in .bhc/.meta
				match file_path.parent() {
					Some(path) => {
						if path.ends_with(".bhc/.meta") {
							self.json_files.meta_file = file_path.clone();
						} else if path.ends_with(".bhc/.meta/css") {
							self.json_files.css_files.push(file_path.clone())
						} else if path.ends_with(".bhc/.meta/html") {
							self.json_files.html_files.push(file_path.clone())
						}
					},
					None => ()
				}
			},
			
			_ => (),
		})
	}

	// Key = Actual CSS File PathBuf
	// Value = Corresponding CSS Metadata File PathBuf
	pub fn map_css_files(&self) -> HashMap<PathBuf, PathBuf> {
		let mut css_map: HashMap<PathBuf, PathBuf> = HashMap::new();
		
		self.json_files
		.css_files
		.iter()
		.for_each(|css_file|{
			let json_string = fs::read_to_string(css_file).unwrap(); // File will exist (only time it might not is if the person deletes a file between the creation of the vector and the reading of the file)

			let parsed_css_file: CssMetaData = serde_json::from_str(&json_string).unwrap();

			let metadata_path = Path::new(&parsed_css_file.absolute_path).to_path_buf();

			css_map.insert(css_file.clone(), metadata_path);
		});
		
		css_map
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
	use std::path::{Path, PathBuf};
	use crate::metadata::{GroupedFiles, GroupedJsonFiles};

	#[test]
	fn sort_files_test() {
		let path1: PathBuf = Path::new("C:/random/file.css").to_path_buf();
		let path2: PathBuf = Path::new("C:/random/file.html").to_path_buf();
		let path3: PathBuf = Path::new("C:/random/.bhc/.meta/file.json").to_path_buf();


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