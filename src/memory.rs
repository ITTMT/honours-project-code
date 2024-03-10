use std::{env, path::PathBuf};

use crate::{logging::Logging, Backend};

pub struct Memory {
    workspace_paths: Vec<PathBuf>,
    ready:           bool,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            workspace_paths: Vec::new(),
            ready:           false,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn add_workspace(&mut self, workspace: PathBuf) {
        self.workspace_paths.push(workspace);
        self.ready = true;
    }

    pub fn add_workspaces(&mut self, mut workspaces: Vec<PathBuf>) {
        self.workspace_paths.append(&mut workspaces);
        self.ready = true;
    }

    pub fn remove_workspaces(&mut self, workspaces: Vec<PathBuf>) {
        self.workspace_paths.retain(|x| !workspaces.contains(x))
    }

    

    pub fn get_workspace_folder(&self, file_path: &PathBuf) -> Option<PathBuf> {
        let x: Vec<PathBuf> = self.workspace_paths.clone().into_iter().filter(|x| file_path.starts_with(x)).collect();

        match x.first() {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }

    pub async fn first_time_setup(&mut self, backend: &Backend) {
        let workspace_paths = match backend.client.workspace_folders().await {
            Ok(value) => match value {
                Some(value) => value,
                None => {
                    self.add_workspace(env::temp_dir());
                    return;
                }
            },
            Err(error) => {
                backend.log_error(format!("Error occurred trying to get workspace folders: {}", error)).await;
                return;
            }
        };

        match backend.get_workspace_paths(workspace_paths) {
            Ok(value) => self.add_workspaces(value),
            Err(error) => backend.log_error(error).await,
        };
    }
}

/* #region Unit Tests */
#[cfg(test)]
mod tests {}
/* #endregion */
