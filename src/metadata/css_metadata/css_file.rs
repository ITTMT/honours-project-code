use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CssFile {
    pub id: u32,
    pub file_name: String,
    pub absolute_path: String,
}

impl CssFile {
    pub fn new() -> CssFile {
        CssFile {
            id: 0,
            file_name: String::new(),
            absolute_path: String::new(),
        }
    }
}