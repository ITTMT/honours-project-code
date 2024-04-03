use html5gum::{HtmlString, Token, Tokenizer};
use std::{
    ffi::OsStr, fs::{self, File}, ops::Deref, path::{Component, PathBuf}};
use tower_lsp::lsp_types::{DidOpenTextDocumentParams, TextDocumentItem};

use crate::{metadata::{css_metadata::get_metadata_files, file_metadata::FormattedCssFile}, Backend, VIRTUAL_PATH};


//TODO: I need to make metadata for the virtual file to act as a "staging" area for changes that are made but save has not been pressed
//TODO: Think about what is needed to send back to the client to get the colouring of lines correct
// for each line we need to know its owner, 

impl Backend {
    pub async fn get_css_file(&self, params: DidOpenTextDocumentParams) -> Result<Option<FormattedCssFile>, String> {
        let file_path = file_to_pathbuf(&params.text_document);

        if let Some(file_path_root) = file_path.parent(){
            let file_pathbuf = file_path_root.to_path_buf();

            let workspace_path = match self.get_workspace_folder(&file_path).await {
                Ok(value) => value,
                Err(error) => return Err(error),
            };

            let file_destination = get_full_path(&file_path, &workspace_path);
    
            let css_files = match get_css_file_paths(&file_pathbuf, &params.text_document.text) {
                Ok(value) => value,
                Err(error) => return Err(error),
            };

            match css_files.len() {
                0 => return Ok(None),
                1 => {
                    let css_metadata_files = match get_metadata_files(&css_files, &workspace_path) {
                        Ok(value) => value,
                        Err(error) => return Err(error)
                    };

                    if let Some(metadata) = css_metadata_files {
                        let mut formatted_file = FormattedCssFile::generate_formatted_file(&metadata);

                        formatted_file.absolute_path = css_files.first().unwrap().to_str().unwrap().to_string();
                    
                        return Ok(Some(formatted_file))
                    }
                },
                _ => {
                    let css_metadata_files = match get_metadata_files(&css_files, &workspace_path) {
                        Ok(value) => value,
                        Err(error) => return Err(error)
                    };

                    if let Some(metadata) = css_metadata_files {
                        let mut formatted_file = FormattedCssFile::generate_formatted_file(&metadata);

                        let css_string = formatted_file.to_css_string();
                        
                        match save_css_file(&css_string, &file_destination) {
                            Ok(value) => {
                                formatted_file.absolute_path = value.to_str().unwrap().to_string();

                                return Ok(Some(formatted_file))
                            },
                            Err(error) => return Err(error)
                        };
                    } 
                }
            }
        }; 

        Err(format!("Could not get css file for file: {:?}", file_path))
    }
}

pub fn get_css_file_paths(absolute_path_of_html: &PathBuf, file_contents: &str) -> Result<Vec<PathBuf>, String> {
    let tag_name: HtmlString = HtmlString(b"link".to_vec());
    let href: HtmlString = HtmlString(b"href".to_vec());

    let mut css_vec: Vec<PathBuf> = Vec::new();

    let mut error_occurred = false;
    let mut error_string: String = String::new();

    Tokenizer::new(file_contents)
        .infallible()
        .filter_map(|token| match token {
            Token::StartTag(tag) => {
                if tag.name == tag_name {
                    Some(tag)
                } else {
                    None
                }
            }
            _ => None,
        })
        .for_each(|link| match link.attributes.get_key_value(&href) {
            Some((_, value)) => {
                let s = value.deref().to_vec();
                let string_result = String::from_utf8_lossy(&s);
                let pathbuf = PathBuf::from(&string_result.to_string());

                match find_absolute_path(&absolute_path_of_html, &pathbuf) {
                    Ok(value) => css_vec.push(value),
                    Err(error) => {
                        error_occurred = true;
                        error_string = error;
                        return;
                    }
                }
            }
            None => (),
        });

    if error_occurred {
        return Err(error_string);
    }

    Ok(css_vec)
}

/// For a given `document_path`, which will be an absolute path of a HTML document, and a `css_path` which may or may not be an absolute path, 
/// Returns `Ok(PathBuf)` if it was able to find the path. This will be the absolute path of the `css_path`
/// Returns `Err(String)` if it was unable to find the absolute path for the provided `css_path`
fn find_absolute_path(document_path: &PathBuf, css_path: &PathBuf) -> Result<PathBuf, String> {
    if css_path.exists() && css_path.is_absolute() {
        return Ok(css_path.clone());
    }

    if css_path.is_relative() {
        let css_components = css_path.components();
        let mut actual_path = PathBuf::new();
        
        let mut document_dir: PathBuf = if document_path.is_file() {
            document_path.parent().unwrap().to_path_buf()
        } else {
            document_path.clone()
        };

        for component in css_components {
            if component == Component::ParentDir {
                document_dir = document_dir.parent().unwrap().to_path_buf()
            } else {
                actual_path.push(component);
            }
        }

        let final_path = document_dir.clone().join(actual_path);

        return Ok(final_path)
    }

    Err(format!("Error trying to find CSS File: {:?}", css_path))
}

pub fn save_css_file(css_string: &str, save_path: &PathBuf) -> Result<PathBuf, String> {

    match create_dir_and_file(&save_path) {
        Ok(_) => (),
        Err(error) => return Err(error)
    };

    match fs::write(save_path, css_string) {
        Ok(_) => (),
        Err(error) => return Err(format!("Error occurred trying to write to the file ({:?}), {:?}", save_path, error)),
    };

    Ok(save_path.clone())
}

pub fn create_dir_and_file(file_path: &PathBuf) -> Result<(), String> {
    let directory = file_path.parent().unwrap().to_path_buf();

    match fs::create_dir_all(directory) {
        Ok(_) => (),
        Err(error) => return Err(format!("Error occurred trying to create the save directory: {:?}", error)),
    };

    match File::create(file_path) {
        Ok(_) => (),
        Err(error) => return Err(format!("Error occurred trying to create the file: ({:?}), {:?}", file_path, error)),
    }

    Ok(())
}

fn file_to_pathbuf(document: &TextDocumentItem) -> PathBuf {
    let file_path = match document.uri.to_file_path() {
        Ok(value) => value,
        Err(_) => PathBuf::new(), // Need a default, if file doesn't exist, this should never happen
    };

    file_path
}

/// Get the full PathBuf for a virtual file. The virtual file gets created when a HTML file contains more than one CSS link inside it so we can concatenate all of its contents.
fn get_full_path(file_pathbuf: &PathBuf, workspace_pathbuf: &PathBuf) -> PathBuf {
    let extra_path = file_pathbuf.strip_prefix(workspace_pathbuf);

    let mut final_path: PathBuf = workspace_pathbuf.join(VIRTUAL_PATH);

    match extra_path {
        Ok(value) => {
            final_path.push(value);
            final_path.set_extension("css");
        }
        Err(_) => return PathBuf::new(),
    }

    final_path
}

/// For a given path, return all of the files it contains as a `Vec<PathBuf>`
pub fn recursive_file_search(path: &PathBuf) -> Vec<PathBuf> {
    let mut found_paths: Vec<PathBuf> = Vec::new();
    
    match fs::read_dir(path) {
        Ok(value) => value.for_each(|res| match res {
            Ok(value) => {
                if value.path().is_dir() {
                    inner_recursive_file_search(&value.path(), &mut found_paths);

                } else if value.path().is_file() {
                    found_paths.push(value.path())
                }
            }
            Err(_) => (),
        }),
        Err(_) => (),
    }

    found_paths
}

fn inner_recursive_file_search(path: &PathBuf, found_paths: &mut Vec<PathBuf>) {
    match fs::read_dir(path) {
        Ok(value) => value.for_each(|res| match res {
            Ok(value) => {
                if value.path().is_dir() {
                    inner_recursive_file_search(&value.path(), found_paths);

                } else if value.path().is_file() {
                    found_paths.push(value.path())
                }
            }
            Err(_) => (),
        }),
        Err(_) => (),
    }
}

pub fn contains_web_documents(file_paths: &Vec<PathBuf>) -> bool {
    for file_path in file_paths {
        if let Some(file_extension) = file_path.extension().and_then(OsStr::to_str) {
            if ["html", "css"].contains(&file_extension) {
                return true
            }
        }
    }

    false
}

/* #region Unit Tests */

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use crate::file::{find_absolute_path, get_css_file_paths, get_full_path, save_css_file};

    #[test]
    fn test_find_absolute_path() {
        // let x = find_absolute_path(
        //     &PathBuf::from("C:\\Windows\\System32\\en-GB"),
        //     &PathBuf::from("..\\en\\AuthFWSnapIn.Resources.dll"),
        // );

        // assert_eq!(PathBuf::from("C:\\Windows\\System32\\en\\AuthFWSnapIn.Resources.dll"), x.unwrap());

        // let y = find_absolute_path(
        //     &PathBuf::from("C:\\Windows\\System32\\en-GB"),
        //     &PathBuf::from("C:\\Windows\\System32\\en\\AuthFWSnapIn.Resources.dll"));

        // assert_eq!(PathBuf::from("C:\\Windows\\System32\\en\\AuthFWSnapIn.Resources.dll"), y.unwrap());

        let z = find_absolute_path(&PathBuf::from("d:\\programming\\web-dev\\xd\\html\\test.html"), &PathBuf::from("../css/test.css"));

        assert_eq!(PathBuf::from("d:\\programming\\web-dev\\xd\\css\\test.css"), z.unwrap());
    }

    #[test]
    fn test_get_css_file_paths() {
        let absolute_path = PathBuf::from("C:/Users/Ollie/Documents/serverexampletest/html_files/test.html");

        let file_contents = r#"<!DOCTYPE html>
		<html lang="en#">
		<head>
			<meta charset="UTF-8">
			<title style="font-size: 14em">Title</title>
			<link rel="stylesheet" type="text/css" href="C:/Users\Ollie/Documents\serverexampletest/css_files/base.css"/>
			<link rel="stylesheet" type="text/css" href="../css_files/stylesheet_1.css"/>
		</head>
		<body>
			<h1>My Title</h1>
			<p>Lorem Ipsum Dolores</p>
		</body>
		</html>"#;

        let mut expected: Vec<PathBuf> = Vec::new();
        expected.push(PathBuf::from("C:/Users/Ollie/Documents/serverexampletest/css_files/base.css"));
        expected.push(PathBuf::from("C:/Users/Ollie/Documents/serverexampletest/css_files/stylesheet_1.css"));

        let x = get_css_file_paths(&absolute_path, file_contents).unwrap();

        x.iter().for_each(|y| println!("{:?}", y));

        assert_eq!(x, expected);
    }

    #[test]
    fn test_get_full_path() {
        let workspace_path = PathBuf::from("C:/temp/random/path");

        let a = PathBuf::from("C:/temp/random/path/.bhc/html/myfile.css");

        let file_pathbuf = PathBuf::from("C:/temp/random/path/html/myfile.html");

        assert_eq!(get_full_path(&file_pathbuf, &workspace_path), a);
    }

    #[test]
    fn test_save_css_file() {
        let file_contents = r#"
h1 {
	font-size: 14ex;
	background-color: red;
}

p {
	font-size: 10ex;
	background-color: green;
}
"#;

        let save_path = PathBuf::from("C:\\Users\\Ollie\\Documents\\testing123\\new_file.css");

        assert_eq!(save_css_file(file_contents, &save_path).unwrap(), save_path.clone());
    }
}
/* #endregion */
