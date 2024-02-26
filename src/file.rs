use crate::{Backend, Logging};

pub mod file_commands;

pub async fn get_workspace_paths(backend: &Backend) -> Vec<String> {
	let mut x: Vec<String> = Vec::new();

	match backend.client.workspace_folders().await {
		Ok(value) => match value {
			Some(value) => {
				value
				.iter()
				.for_each(|workspace_paths| x.push(workspace_paths.uri.path().to_string()));
			},
			None => backend.log_info("Could not find any workspace folders").await // This occurs when a file is created and saved without a folder being opened.
		},
		Err(error) => backend.log_error(error).await
	}

	return x;
}