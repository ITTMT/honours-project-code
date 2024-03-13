use std::{ffi::OsStr, path::PathBuf};

pub mod css_metadata;
pub mod html_metadata;
pub mod workspace_metadata;

#[derive(Debug, PartialEq)]
pub struct GroupedFiles {
	pub html_files: Vec<PathBuf>,
	pub css_files: Vec<PathBuf>,
	pub json_files: Vec<PathBuf>,
}

pub trait Metadata<T> {
	fn create_metadata(metadata_path: &PathBuf, file_path: &PathBuf) -> Result<T, String>;
}

impl GroupedFiles {
	pub fn new() -> GroupedFiles {
		GroupedFiles {
			html_files: Vec::new(),
			css_files: Vec::new(),
			json_files: Vec::new(),
		}
	}

	pub fn sort_files(&mut self, unorganised_files: &Vec<PathBuf>) {
		unorganised_files
		.iter()
		.for_each(|x| match x.extension().and_then(OsStr::to_str) {
			Some("css") => self.css_files.push(x.clone()),
			Some("html") => self.html_files.push(x.clone()),
			Some("json") => self.json_files.push(x.clone()),
			
			_ => (),
		})
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
	use crate::metadata::GroupedFiles;

	#[test]
	fn sort_files_test() {
		let path1: PathBuf = Path::new("C:/random/file.css").to_path_buf();
		let path2: PathBuf = Path::new("C:/random/file.html").to_path_buf();
		let path3: PathBuf = Path::new("C:/random/file.json").to_path_buf();


		let paths: Vec<PathBuf> = vec![path1.clone(), path2.clone(), path3.clone()];

		let expected = GroupedFiles {
			css_files: vec![path1.clone()],
			html_files: vec![path2.clone()],
			json_files: vec![path3.clone()],
		};

		let mut actual = GroupedFiles::new();

		actual.sort_files(&paths);

		assert_eq!(actual, expected);
	}
}