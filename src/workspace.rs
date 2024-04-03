use std::{collections::HashMap, fs, path::PathBuf};

use chrono::{DateTime, Utc};
use tower_lsp::lsp_types::{TextDocumentItem, WorkspaceFolder};

use crate::{file::{contains_web_documents, recursive_file_search}, logging::Logging, metadata::{css_metadata::CssMetaData, html_metadata::HtmlMetaData, workspace_metadata::{create_workspace_metadata, id_to_json_file_name, open_workspace_metadata, workspace_css_file::WorkspaceCssFile, workspace_html_file::WorkspaceHtmlFile, WorkspaceMetaData}, GroupedFiles}, Backend, CSS_METADATA_PATH, HTML_METADATA_PATH, METADATA_PATH, SHARED_PATH};

impl Backend {
	/// Get the workspaces that are currently open. Calls into the LSP [`workspace/workspaceFolders`](https://microsoft.github.io/language-server-protocol/specification#workspace_workspaceFolders)
	/// Returns `Ok(Vec<PathBuf>)` if there are workspaces and was able to transform from `WorkspaceFolder` to `PathBuf`.
	/// Returns `Err(String)` if there was no workspaces found or able to be converted.
	
	pub async fn get_workspaces(&self) -> Result<Vec<PathBuf>, String> {
        let mut workspace_pathbufs: Vec<PathBuf> = Vec::new(); 
        
        match self.client.workspace_folders().await {
            Ok(value) => {
                if let Some(workspaces) = value {
                    workspace_pathbufs = self.parse_workspaces(&workspaces).await;
                }
            },
            Err(error) => {
                return Err(format!("Error occurred trying to get workspace folders: {:?}", error))
            },
        };

        if workspace_pathbufs.is_empty() {
            return Err(String::from("There are no workspaces open"))
        }

		Ok(workspace_pathbufs)
    }

	pub async fn parse_workspaces(&self, workspace_folders: &Vec<WorkspaceFolder>) -> Vec<PathBuf> {
		let mut workspaces: Vec<PathBuf> = Vec::new();

		for workspace in workspace_folders {
			match workspace.uri.to_file_path() {
				Ok(value) => workspaces.push(value),
				Err(error) => {
					self.log_error(format!("Error occurred trying to transform URI to PathBuf ({:?}) {:?}", workspace.uri, error)).await;
					continue
				}
			}
		}

		workspaces
	}


	/// For the supplied `file_path`, get the workspace `PathBuf` it belongs to.
	/// Returns `Ok(PathBuf)` if it was able to find it.
	/// Returns `Err(String)` if it couldn't find the workspace. This means that the file was opened externally, and not belonging to any currently open workspace.
	pub async fn get_workspace_folder(&self, file_path: &PathBuf) -> Result<PathBuf, String> {
       let workspaces = match self.get_workspaces().await {
			Ok(value) => value,
			Err(error) => return Err(error)
	   };

        match workspaces.first() {
            Some(value) => Ok(value.clone()),
            None => Err(format!("Could not find workspace for file: {:?}", file_path)),
        }
    }

	/// For the supplied `workspace_path`, get the workspace metadata, 
	/// Returns `Ok(WorkspaceMetaData)` on success.
	/// Returns `Err(String)` if it was unable to find the file, or had trouble deseralizing it.
	pub async fn get_workspace_metadata(&self, workspace_path: &PathBuf) -> Result<WorkspaceMetaData, String> {
		match get_workspace_metadata(&workspace_path) {
			Ok(value) => Ok(value),
			Err(error) => Err(error)
		}
	}

	/// For the supplied `text_document`, get the CSS metadata, 
	/// Returns `Ok(CssMetaData)` on success.
	/// Returns `Err(String)` if it was unable to find the file, or had trouble deseralizing it.
	pub async fn get_css_metadata(&self, text_document: &TextDocumentItem) -> Result<CssMetaData, String> {
		let workspace_path = match self.get_workspace(text_document).await {
			Ok(value) => value, 
			Err(error) => {
				return Err(error)
			}
		};

		let file_path = text_document.uri.to_file_path().unwrap();

		match get_css_metadata(&workspace_path, &file_path) {
			Ok(value) => return Ok(value),
			Err(error) => {
				return Err(error);
			}
		}
	}

	// pub async fn get_html_metadata(&self, text_document: &TextDocumentItem) -> HtmlMetaData {
	// 	panic!("todo");
	// }


	/// Initializes the metadata on startup, this includes creating metadata for the first time if it didn't exist, and updating any existing metadata since the last time the workspace was opened. 
	/// This will return nothing and logs any errors that occurs throughout the process.
	/// It will try to create as many metadata files and skip any that throw an error
	pub async fn initialize_workspace(&self, workspace_path: &PathBuf) {
		let files = recursive_file_search(&workspace_path);

		if !contains_web_documents(&files) {
			return
		}

		let css_metadata_path = &workspace_path.join(CSS_METADATA_PATH);
		let html_metadata_path = &workspace_path.join(HTML_METADATA_PATH);
		let shared_path = &workspace_path.join(SHARED_PATH);
		let workspace_metadata_path = &workspace_path.join(METADATA_PATH);

		// this is just for initialising, it doesn't need to be a provider of truth
		let grouped_files: GroupedFiles = files.into();

		// this is a source of truth and will be used to save back at the end.
		let mut workspace_metadata = if grouped_files.contains_workspace_metadata() {
			match open_workspace_metadata(workspace_metadata_path) {
				Ok(value) => value,
				Err(error) => {
					self.log_error(error).await;
					return
				}
			}
		} else {
			match create_workspace_metadata(workspace_metadata_path, &workspace_path) {
				Ok(value) => value,
				Err(error) => {
					self.log_error(error).await;
					return
				}
			}
		};

		workspace_metadata.workspace_path = workspace_path.clone().into_os_string().into_string().unwrap();

		// create a hashmap of css files to their json metadata files. If the file key doesn't appear in the list, it means we have to create its metadata file from scratch
		let css_metadata_map: HashMap<PathBuf, PathBuf> = grouped_files.map_css_files();

		for css_file in &grouped_files.css_files {
			match css_metadata_map.get_key_value(css_file) {
				Some((_,css_metadata_file_path)) => {
					match fs::read_to_string(css_metadata_file_path) {
						Ok(contents) => {
							// it exists so we can deserialize it
							let mut css_metadata: CssMetaData = serde_json::from_str(&contents).unwrap();

							// get the metadata of the original file
							let css_file_metadata = fs::metadata(&css_metadata.absolute_path).unwrap();

							let file_last_modified: DateTime<Utc> = css_file_metadata.modified().unwrap().into();

							// if the original file has been updated more recently than the proclaimed last_updated time then we need to update the contents of 
							if file_last_modified > css_metadata.last_updated {
								match css_metadata.update_metadata(css_metadata_file_path) {
									Ok(_) => (),
									Err(error) => {
										self.log_error(error).await;
										continue
									}
								};
							}
						},
						Err(error) => {
							self.log_error(format!("Error trying to read file ({:?}): {:?}", &css_metadata_file_path, error)).await;
							continue
						}
					}

				}, 
				None => {
					// we cannot know what sheets have been imported ahead of time
					// we can know what styles there are though
					// get the next available id
					let id = workspace_metadata.get_next_available_css_id();

					let file_name = id_to_json_file_name(&id);

					let mut save_path = css_metadata_path.clone();
					save_path.push(&file_name);

					// create the file <id>.json
					let css_metadata = match CssMetaData::create_metadata(&save_path, css_file, &id) {
						Ok(value) => value,
						Err(error) => {
							self.log_error(error).await;
							continue
						}
					};


					// We need to save the css_metadata 

					let css_file_metadata = WorkspaceCssFile::parse(&css_metadata);

					workspace_metadata.add_css_file(css_file_metadata)
				} 
			}
		}

		let html_metadata_map: HashMap<PathBuf, PathBuf> = grouped_files.map_html_files();

		//TODO: Will have to find a way to re-pair up orphaned files that have been moved externally.

		for html_file in &grouped_files.html_files {
			match html_metadata_map.get_key_value(html_file) {
				Some((_,html_metadata_file_path)) => {
					match fs::read_to_string(html_metadata_file_path) {
						Ok(contents) => {
							// it exists so we can deserialize it
							let mut html_metadata: HtmlMetaData = serde_json::from_str(&contents).unwrap();

							// get the metadata of the original file
							let html_file_metadata = fs::metadata(&html_metadata.absolute_path).unwrap();

							let file_last_modified: DateTime<Utc> = html_file_metadata.modified().unwrap().into();

							// if the original file has been updated more recently than the proclaimed last_updated time then we need to update the contents of 
							if file_last_modified > html_metadata.last_updated {
								match html_metadata.update_metadata(html_metadata_file_path, &workspace_metadata) {
									Ok(_) => (),
									Err(error) => {
										self.log_error(error).await;
										continue
									}
								};
							}
						},
						Err(error) => {
							self.log_error(format!("Error trying to read file ({:?}): {:?}", &html_metadata_file_path, error)).await;
							continue
						}
					}

				}, 
				None => {
					let id = workspace_metadata.get_next_available_html_id();

					let file_name = id_to_json_file_name(&id);

					let mut save_path = html_metadata_path.clone();
					save_path.push(&file_name);

					// create the file <id>.json
					let html_metadata = match HtmlMetaData::create_metadata(&save_path, html_file, &id) {
						Ok(value) => value,
						Err(error) => {
							self.log_error(error).await;
							continue
						}
					};

					let html_file_metadata = WorkspaceHtmlFile::parse(&html_metadata);

					workspace_metadata.add_html_file(html_file_metadata)
				} 

			}
		} 

		// Update all the associations to each file
		// HTML first, then CSS
		let mut html_workspace_metadata_map: HashMap<usize, WorkspaceHtmlFile> = HashMap::new();
		
		for (index, html_file) in workspace_metadata.html_files.iter().enumerate() {
			// This transforms a basic HTML file into one that contains any included stylesheets inside it
			let new_metadata = match html_file.update(&workspace_metadata){
				Ok(value) => value,
				Err(error) => {
					self.log_error(error).await;
					continue
				}
			};

			html_workspace_metadata_map.insert(index, new_metadata);
		}

		for (index, metdata_file) in html_workspace_metadata_map {
			match workspace_metadata.modify_html_file(&metdata_file, &index) {
				Ok(_) => (),
				Err(error) => {
					self.log_error(error).await;
					continue
				}
			}
		}

		// TODO: do the references to eachother part

		// TODO: Remove any files that no longer exist 
		// they will have the same filename, but different id's
		match workspace_metadata.update_metadata(workspace_metadata_path) {
			Ok(_) => (),
			Err(error) => {
				self.log_error(error).await;
				return 
			}
		};

	}

	/// For a given `text_document``, we return the correct workspace pathbuf.
	/// Returns `Ok(PathBuf)` on success.
	/// Returns `Err(String)` if it was unable to find the workspace path the `text_document` belongs to.
	/// 
	async fn get_workspace(&self, text_document: &TextDocumentItem) -> Result<PathBuf, String> {
		let workspace_paths = match self.get_workspaces().await {
			Ok(value) => value,
			Err(error) => return Err(error)
		};

		let file_path = text_document.uri.to_file_path().unwrap();

		for workspace in workspace_paths {
			if file_path.starts_with(&workspace) {
				return Ok(workspace);
			}
		}

		return Err(format!("Could not find any workspace for the given path: {:?}", file_path))
	}
}

/// For the given `workspace_path`, return the WorkspaceMetaData. This can be found at `{workspace_path}/.bhc/.meta/meta.json`.
/// Returns `Ok(WorkspaceMetaData)` if it was able to find the file and deserialize it
/// Returns `Err(String)` if the file doesn't exist or was unable to deserialize it.
fn get_workspace_metadata(workspace_path: &PathBuf) -> Result<WorkspaceMetaData, String> {
	let mut final_path: PathBuf = workspace_path.clone();

	final_path.push(METADATA_PATH);

	let metadata_string: String = match fs::read_to_string(&final_path) {
		Ok(value) => value,
		Err(error) => {
			return Err(format!("Error trying to read workspace metadata file ({:?}): {:?}", &final_path, error))
		}
	};

	match serde_json::from_str(&metadata_string) {
		Ok(value) => return Ok(value),
		Err(error) => {
			return Err(format!("Error deseralizing workspace metadata file ({:?}): {:?}", &final_path, error))
		}
	};
}

/// For the given `workspace_path`, and the provided `file_path`, get the CssMetaData for it. The `file_path` should be the actual file absolute path.
/// Returns `Ok(CssMetaData)` if it was able to find the file and deserialize it
/// Returns `Err(String)` if the file doesn't exist or was unable to deserialize it.
fn get_css_metadata(workspace_path: &PathBuf, file_path: &PathBuf) -> Result<CssMetaData, String> {
	let workspace_metadata = match get_workspace_metadata(&workspace_path) {
		Ok(value) => value,
		Err(error) => return Err(error)
	};

	for css_file in workspace_metadata.css_files {
		if &PathBuf::from(&css_file.absolute_path) == file_path {
			let metadata_string: String = match fs::read_to_string(file_path) {
				Ok(value) => value,
				Err(error) => {
					return Err(format!("Error trying to read CSS metadata file ({:?}): {:?}", file_path, error))
				}
			};
		
			match serde_json::from_str(&metadata_string) {
				Ok(value) => return Ok(value),
				Err(error) => {
					return Err(format!("Error deseralizing CSS metadata file ({:?}): {:?}", file_path, error))
				}
			};
		}
	}

	Err(format!("Could not find a CSS metadata file at {:?}", file_path))
}


// fn get_html_metadata(workspace_path: &PathBuf, file_path: &PathBuf) -> HtmlMetadata {}
