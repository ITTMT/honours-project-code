use crate::{Backend, VIRTUAL_PATH};
use html5gum::{HtmlString, Token, Tokenizer};
use path_absolutize::Absolutize;
use std::{
    collections::HashMap, ffi::OsStr, fs::{self, File}, io::{BufReader, Read}, ops::Deref, path::PathBuf};
use tower_lsp::lsp_types::{DidOpenTextDocumentParams, TextDocumentItem};

struct FormattedCssFile {
    file_path: PathBuf,
    included_files: Vec<FileMetaData>,
    lines: HashMap<u32, FileMetaData>,
}

struct FileMetaData {
    id: u32,
    file_name: String,
    is_shared: bool,
}

//TODO: Think about what is needed to send back to the client to get the colouring of lines correct
// for each line we need to know its owner, 

impl Backend {
    fn open_file(&self, file_path: &PathBuf) -> Result<String, String> {
        let file = match File::open(file_path) {
            Ok(result) => result,
            Err(error) => return Err(format!("File Opening Error: Unable to open file ({:?}) - {}", file_path, error)),
        };

        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        match buf_reader.read_to_string(&mut contents) {
            Ok(buffer) => buffer,
            Err(error) => return Err(format!("File Opening Error: Unable to read file ({:?}) - {}", file_path, error)),
        };

        Ok(contents)
    }

    fn get_css_file_paths(&self, absolute_path_of_html: &PathBuf, file_contents: &str) -> Result<Vec<PathBuf>, String> {
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

                    match self.find_absolute_path(&absolute_path_of_html, &pathbuf) {
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
    fn find_absolute_path(&self, document_path: &PathBuf, css_path: &PathBuf) -> Result<PathBuf, String> {
        if css_path.exists() && css_path.is_absolute() {
            return Ok(css_path.clone());
        }

        if css_path.is_relative() {
            match css_path.absolutize_from(document_path) {
                Ok(value) => return Ok(value.to_path_buf()),
                Err(error) => return Err(format!("Error trying` to find absolute path for {:?}: {:?}", css_path, error)),
            };
        }

        Err(format!("Error trying to find CSS File: {:?}", css_path))
    }

    
    pub async fn get_css_file(&self, params: DidOpenTextDocumentParams) -> Result<PathBuf, String> {
        let file_path = file_to_pathbuf(&params.text_document);

        if let Some(file_path_root) = file_path.parent(){
            let file_pathbuf = file_path_root.to_path_buf();

            let workspace_path = match self.get_workspace_folder(&file_path).await {
                Ok(value) => value,
                Err(error) => return Err(error),
            };

            let file_destination = get_full_path(&file_path, &workspace_path);
    
            let css_files = match self.get_css_file_paths(&file_pathbuf, &params.text_document.text) {
                Ok(value) => value,
                Err(error) => return Err(error),
            };

            //TODO: Add logic here to make new file if multiple css files, or just return normal file

            match css_files.len() {
                0 => return Ok(PathBuf::new()),
                1 => return Ok(css_files[0].clone()), //TODO: Add logic to return file with metadata.
                _ => {
                    let css_string = match self.generate_css_string(&css_files) {
                        Ok(value) => value,
                        Err(error) => return Err(error),
                    };

                    return self.save_css_file(&css_string, &file_destination);
                }
            }
        }; 

        Err(format!("Could not get css file for file: {:?}", file_path))
    }

    fn generate_css_string(&self, css_files: &Vec<PathBuf>) -> Result<String, String> {
        let mut concatenated_string: String = String::new();

        for (i, file) in css_files.iter().enumerate() {
            match self.open_file(&file) {
                Ok(value) => concatenated_string.push_str(&value),
                Err(error) => return Err(error),
            }

            if i != css_files.len() - 1 {
                concatenated_string.push_str("\n");
            }
        }

        Ok(concatenated_string)
    }

    fn save_css_file(&self, css_string: &str, save_path: &PathBuf) -> Result<PathBuf, String> {

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

    use tower_lsp::LspService;

    use crate::{file::get_full_path, Backend};

    #[test]
    fn test_find_absolute_path() {
        let (service, _) = LspService::build(|client| Backend { client }).finish();

        let x = service.inner().find_absolute_path(
            &PathBuf::from("C:\\Windows\\System32\\en-GB"),
            &PathBuf::from("..\\en\\AuthFWSnapIn.Resources.dll"),
        );

        assert_eq!(PathBuf::from("C:\\Windows\\System32\\en\\AuthFWSnapIn.Resources.dll"), x.unwrap());

        let y = service.inner().find_absolute_path(
            &PathBuf::from("C:\\Windows\\System32\\en-GB"),
            &PathBuf::from("C:\\Windows\\System32\\en\\AuthFWSnapIn.Resources.dll"));

        assert_eq!(PathBuf::from("C:\\Windows\\System32\\en\\AuthFWSnapIn.Resources.dll"), y.unwrap());
    }

    #[test]
    fn test_get_css_file_paths() {
        let (service, _) = LspService::build(|client| Backend { client }).finish();

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

        let x = service.inner().get_css_file_paths(&absolute_path, file_contents).unwrap();

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
        let (service, _) = LspService::build(|client| Backend { client }).finish();

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

        assert_eq!(service.inner().save_css_file(file_contents, &save_path).unwrap(), save_path.clone());
    }
}
/* #endregion */
