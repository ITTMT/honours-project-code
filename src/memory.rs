use std::path::PathBuf;

#[derive(Debug)]
pub struct Memory {
    pub workspace_folders: Vec<PathBuf>,
}

impl Memory {

	pub fn add_workspaces(&mut self, mut workspaces: Vec<PathBuf>) {
		self.workspace_folders.append(&mut workspaces);
	}

	pub fn get_workspace_folder(self, i: &PathBuf) -> Option<PathBuf> {
		if self.workspace_folders.contains(i) {
			let x: Vec<PathBuf> = self.workspace_folders
			.iter()
			.filter(|&x| x == i)
			.cloned()
			.collect();

			match x.first() {
				Some(value) => Some(value),
				None => None
			};
		}

		None
	}

	pub fn workspace_folder_length(&self) -> u32 {
		self.workspace_folders.len() as u32
	}

	pub fn get_only_folder(&self) -> Option<PathBuf> {
		match self.workspace_folders.len() == 1 {
			true => Some(PathBuf::new()),
			false => None,
		}
	}

	
}