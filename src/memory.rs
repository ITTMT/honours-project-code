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
			true => Some(self.workspace_folders[0].clone()),
			false => None,
		}
	}
}

#[cfg(test)]
mod tests {
    use std::path::{self, PathBuf};

    use super::Memory;

	#[test]
	fn test_get_only_folder() {
		let mut folder: Memory = Memory{ workspace_folders: Vec::new()};

		let path: PathBuf = [r"C:\", "windows", "system32.dll"].iter().collect();
		folder.workspace_folders.push(path.clone());

		assert_eq!(folder.get_only_folder().unwrap(), path);
	}

	#[test]
	fn test_get_only_folder_empty() {
		let memory = Memory{ workspace_folders: Vec::new()};

		assert!(memory.get_only_folder() == None);
	}

	#[test]
	fn test_get_only_folder_multiple() {
		let mut memory = Memory{ workspace_folders: Vec::new()};

		let path1: PathBuf = [r"C:\", "windows", "system32"].iter().collect();
		let path2: PathBuf = [r"C:\", "windows", "system32.dll"].iter().collect();

		memory.workspace_folders.push(path1);
		memory.workspace_folders.push(path2);

		assert!(memory.get_only_folder() == None);
	}


}