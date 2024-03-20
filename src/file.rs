use crate::{memory::Memory, Backend};
use html5gum::{HtmlString, Token, Tokenizer};
use path_absolutize::Absolutize;
use std::{
    fs::{self, File},
    io::{BufReader, Read},
    ops::Deref,
    path::{Path, PathBuf},
};
use tower_lsp::lsp_types::{DidOpenTextDocumentParams, TextDocumentItem, WorkspaceFolder};

impl Backend {
    pub fn get_workspace_paths(&self, folders: Vec<WorkspaceFolder>) -> Result<Vec<PathBuf>, String> {
        let mut workspace_paths: Vec<PathBuf> = Vec::new();
        let mut error_occured = false;
        let mut error_string: String = String::new();

        folders.iter().for_each(|path| match path.uri.to_file_path() {
            Ok(result) => workspace_paths.push(result),
            Err(_) => {
                error_occured = true;
                error_string = path.uri.to_string();
            }
        });

        if error_occured {
            return Err(format!("Error transforming uri to file path: {}", error_string));
        }

        Ok(workspace_paths)
    }

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
                    let pathbuf = Path::new(&string_result.to_string()).to_path_buf();

                    match self.find_absolute_path(&absolute_path_of_html, &pathbuf) {
                        Ok(value) => match value {
                            Some(value) => css_vec.push(value),
                            None => (),
                        },
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

    fn find_absolute_path(&self, document_path: &PathBuf, css_path: &PathBuf) -> Result<Option<PathBuf>, String> {
        if css_path.exists() && css_path.is_absolute() {
            return Ok(Some(css_path.clone()));
        }

        if css_path.is_relative() {
            match css_path.absolutize_from(document_path) {
                Ok(value) => return Ok(Some(value.to_path_buf())),
                Err(error) => return Err(format!("Error trying to find absolute path for {:?}: {:?}", css_path, error)),
            };
        }

        Ok(None)
    }

    pub fn produce_css_file(&self, params: DidOpenTextDocumentParams, memory: &Memory) -> Result<PathBuf, String> {
        let file_path = file_to_pathbuf(&params.text_document);

        let file_path_root = file_path.parent().unwrap().to_path_buf(); // add option check

        let workspace_path = match memory.get_workspace_folder(&file_path) {
            Some(value) => value,
            None => return Err(format!("Unable to find workspace path for given file: {:?}", file_path)),
        };

        let file_destination = get_full_path(&file_path, &workspace_path);

        let css_files = match self.get_css_file_paths(&file_path_root, &params.text_document.text) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };

        let css_string = match self.generate_css_string(css_files) {
            Ok(value) => value,
            Err(error) => return Err(error),
        };

        return self.save_css_file(&css_string, &file_destination);
    }

    fn generate_css_string(&self, css_files: Vec<PathBuf>) -> Result<String, String> {
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

fn get_full_path(file_pathbuf: &PathBuf, workspace_pathbuf: &PathBuf) -> PathBuf {
    let extra_path = file_pathbuf.strip_prefix(workspace_pathbuf);

    let mut final_path: PathBuf = workspace_pathbuf.clone();
    final_path.push(".bhc");

    match extra_path {
        Ok(value) => {
            final_path.push(value);
            final_path.set_extension("css");
        }
        Err(_) => return PathBuf::new(),
    }

    final_path
}

/* #region Unit Tests */

#[cfg(test)]
mod tests {

    use std::path::{Path, PathBuf};

    use tower_lsp::LspService;

    use crate::{file::get_full_path, memory::Memory, Backend};

    #[test]
    fn test_find_absolute_path() {
        let (service, _) = LspService::build(|client| Backend { client }).finish();

        let x = service.inner().find_absolute_path(
            &Path::new("C:\\Windows\\System32\\en-GB").to_path_buf(),
            &Path::new("..\\en\\AuthFWSnapIn.Resources.dll").to_path_buf(),
        );

        assert_eq!(Path::new("C:\\Windows\\System32\\en\\AuthFWSnapIn.Resources.dll").to_path_buf(), x.unwrap().unwrap());

        let y = service.inner().find_absolute_path(
            &Path::new("C:\\Windows\\System32\\en-GB").to_path_buf(),
            &Path::new("C:\\Windows\\System32\\en\\AuthFWSnapIn.Resources.dll").to_path_buf(),
        );

        assert_eq!(Path::new("C:\\Windows\\System32\\en\\AuthFWSnapIn.Resources.dll").to_path_buf(), y.unwrap().unwrap());
    }

    #[test]
    fn test_get_css_file_paths() {
        let (service, _) = LspService::build(|client| Backend { client }).finish();

        let absolute_path = Path::new("C:/Users/Ollie/Documents/serverexampletest/html_files/test.html").to_path_buf();

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
        expected.push(Path::new("C:/Users/Ollie/Documents/serverexampletest/css_files/base.css").to_path_buf());
        expected.push(Path::new("C:/Users/Ollie/Documents/serverexampletest/css_files/stylesheet_1.css").to_path_buf());

        let x = service.inner().get_css_file_paths(&absolute_path, file_contents).unwrap();

        x.iter().for_each(|y| println!("{:?}", y));

        assert_eq!(x, expected);
    }

    #[test]
    fn test_get_file_path() {
        tokio_test::block_on(async {
            let (_, _) = LspService::build(|client| Backend { client }).finish();

            let mut memory = Memory::new();

            let workspace_path = Path::new("C:/temp/random/path").to_path_buf();
            let a = workspace_path.clone();

            memory.add_workspace(workspace_path);

            let file_pathbuf = Path::new("C:/temp/random/path/html/myfile.html").to_path_buf();

            assert_eq!(memory.get_workspace_folder(&file_pathbuf).unwrap(), a);
        })
    }

    #[test]
    fn test_get_full_path() {
        let workspace_path = Path::new("C:/temp/random/path").to_path_buf();

        let a = Path::new("C:/temp/random/path/.bhc/html/myfile.css").to_path_buf();

        let file_pathbuf = Path::new("C:/temp/random/path/html/myfile.html").to_path_buf();

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

        let save_path = Path::new("C:\\Users\\Ollie\\Documents\\testing123\\new_file.css").to_path_buf();

        assert_eq!(service.inner().save_css_file(file_contents, &save_path).unwrap(), save_path.clone());
    }
}
/* #endregion */
