use html5gum::{HtmlString, Token, Tokenizer};
use mktemp::Temp;
use tower_lsp::lsp_types::WorkspaceFolder;
use crate::{Backend, Logging};
use std::{env, fs::{self, File}, io::{self, BufReader, Read, Write}, ops::Deref, path::{Path, PathBuf}};

pub trait Files {
	async fn get_workspace_paths(&self, folders: Option<Vec<WorkspaceFolder>>) -> Vec<PathBuf>;

	async fn open_file(&self, file_path: &str) -> Option<String>;

	fn create_dotfolder(&self, workspace_path: &PathBuf) -> Option<PathBuf>;

	fn get_css_files(&self, absolute_path_of_html: &str, file_contents :&str) -> Vec<String>;

	fn find_absolute_path(&self, original_file_path: &str, css_path: &str) -> Option<String>;

	fn get_only_filename(&self, url :&str) -> Result<String, String>;

	async fn produce_css_file(&self, html_filename: &str , save_folder: &PathBuf, url_as_string: Vec<String>) -> Result<PathBuf, String>;
}

impl Files for Backend {
	async fn get_workspace_paths(&self, folders: Option<Vec<WorkspaceFolder>>) -> Vec<PathBuf> {

		let mut workspace_paths: Vec<PathBuf> = Vec::new();
		let mut error_occured = false;
			
		match folders{
			Some(value) => {
				value
				.iter()
				.for_each(|path| match path.uri.to_file_path(){
					Ok(result) => workspace_paths.push(result),
					Err(_) => {
						error_occured = true;
					}
				});
			},
			None => workspace_paths.push(env::temp_dir())
		}

		if error_occured {
			self.log_error("Error transforming uri to file path.").await;
		}
	
		return workspace_paths;
	}

	async fn open_file(&self, file_path: &str) -> Option<String> {

		let file = match File::open(&file_path) {
			Ok(result) => result,
			Err(error) => {
				self.log_error(format!("File Opening Error: Unable to open file ({}) - {}", &file_path, error)).await;
				return None;
			}
		};
	
		let mut buf_reader = BufReader::new(file);
		let mut contents = String::new();
		match buf_reader.read_to_string(&mut contents){
			Ok(buffer) => buffer,
			Err(error) => {
				self.log_error(format!("File Opening Error: Unable to read file ({}) - {}", &file_path, error)).await;
				return None;
			}
		};
	
		Some(contents)
	}

	fn create_dotfolder(&self, workspace_path: &PathBuf) -> Option<PathBuf> {
		let mut desired_path = workspace_path.clone();
		desired_path.push("/.bhc");
	
		match fs::create_dir_all(&desired_path) {
			Ok(_) => Some(desired_path),
			Err(_) => None
		}
	}
	
	fn get_css_files(&self, absolute_path_of_html: &str, file_contents :&str) -> Vec<String> {
		let tag_name: HtmlString = HtmlString(b"link".to_vec());
		let href: HtmlString = HtmlString(b"href".to_vec());
	
		let mut css_vec: Vec<String> = Vec::new();
	
		for token in Tokenizer::new(file_contents).infallible() {
			match token {
				Token::StartTag(tag) => {
					if tag.name == tag_name {
						match tag.attributes.get_key_value(&href){
							Some((_, value)) => {
								let s = value.deref().to_vec();
								let string_result = String::from_utf8_lossy(&s);
								let string_value = string_result.to_string();
	
								let absolute_css_path = self.find_absolute_path(&absolute_path_of_html, &string_value).expect("");
	
								css_vec.push(absolute_css_path);
							},
							None => continue
						}
					}
				}
				_ => continue,
			}
		}
	
		css_vec
	}
	
	fn find_absolute_path(&self, original_file_path: &str, css_path: &str) -> Option<String> {
		if Path::new(css_path).exists() || Path::new(css_path).exists(){
			return Some(css_path.replace('\\', "/").to_string());
		}
	
		if css_path.starts_with("..") {
			let mut absolute_path: String = String::new();
	
			let css_relative_path: Vec<&str> = css_path.split(['\\', '/']).collect();
			let css_relative_path_backstep_removed: Vec<&str> = css_relative_path.into_iter().filter(|&abc| abc != "..").collect();
	
			let number_of_backsteps = css_path.matches("..").count();
	
			let original_split: Vec<&str> = original_file_path.split(['\\', '/']).collect();
	
			// -1 to remove the file name, and number_of_backsteps is how many ../ there are 
			let number_of_steps_from_original = original_split.len() - 1 - number_of_backsteps;
	
			for i in 0..number_of_steps_from_original {
				absolute_path.push_str(original_split[i]);
				absolute_path.push_str("/");
			}
	
			for (i, ele) in css_relative_path_backstep_removed.iter().enumerate() {
				absolute_path.push_str(ele);
				
				if i != css_relative_path_backstep_removed.len() - 1 {
					absolute_path.push_str("/");
				}
			}
	
			if Path::new(&absolute_path).exists() {
				return Some(absolute_path);
			}
	
			// let a  = original_split.last().expect("").to_string();
			// let b = a.len();
			// let c = &a[..b-5];
			
			// absolute_path.push_str(c);
			// absolute_path.push_str(".css");
	
			// println!("{number_of_steps_from_original}");
		}
	
		None
	}
	
	fn get_only_filename(&self, url :&str) -> Result<String, String> {
	
	
	
		return Ok("".to_string());
	}
	
	async fn produce_css_file(&self, html_filename: &str , save_folder: &PathBuf, url_as_string: Vec<String>) -> Result<PathBuf, String> {
	
		let mut concatenated_css: String = String::new(); 
	
		for s in url_as_string {
			concatenated_css.push_str(&self.open_file(&s).await.unwrap());
	
			concatenated_css.push_str("\n");
		}
	
		let mut new_file_path = save_folder.clone();
		new_file_path.push(html_filename);
		new_file_path.set_extension("css");
	
	
		// need to change this to not have the race condition
		if !Path::new(&new_file_path).exists() {
			let _ = File::create(&new_file_path);
		} else {
			let _ = fs::remove_file(&new_file_path);
			let _ = File::create(&new_file_path);
		}
	
		let tmp_path = Temp::new_file().expect("");
	
		self.log_info(format!("{:?}", &new_file_path)).await;

		// Open temp file for writing
		let mut tmp = File::create(&tmp_path).unwrap();
		// Open source file for reading
		let mut src = File::open(&new_file_path).unwrap();
		// Write the data to prepend
		tmp.write_all(concatenated_css.as_bytes()).expect("");
		// Copy the rest of the source file
		io::copy(&mut src, &mut tmp).expect("");
		fs::remove_file(&new_file_path).expect("");
		fs::rename(&tmp_path, &new_file_path).expect("");
	
		return Ok(new_file_path);
		
	}
}




#[cfg(test)]
mod tests {

    use tower_lsp::LspService;

    use crate::Backend;

    use super::Files;

	#[test]
	fn test_aboslute_paths() {
		let (service, _) = LspService::build(|client| Backend {
			client,
		})
		.finish();

		let x = service.inner().find_absolute_path("C:\\Users/Ollie\\Documents/serverexampletest/html_files/test.html", 
		"../css_files/stylesheet_1.css");
		
		assert_eq!("C:/Users/Ollie/Documents/serverexampletest/css_files/stylesheet_1.css", x.unwrap());

		let y = service.inner().find_absolute_path("C:\\Users/Ollie\\Documents/serverexampletest/html_files/test.html", 
		"C:\\Users/Ollie\\Documents/serverexampletest/css_files/stylesheet_1.css");
		assert_eq!("C:/Users/Ollie/Documents/serverexampletest/css_files/stylesheet_1.css", y.unwrap());
	}

	#[test]
	fn test_get_css_files() {
		let (service, _) = LspService::build(|client| Backend {
			client,
		})
		.finish();

		let absolute_path = "C:/Users/Ollie/Documents/serverexampletest/html_files/test.html";

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

		let mut expected: Vec<String> = Vec::new();
		expected.push("C:/Users/Ollie/Documents/serverexampletest/css_files/base.css".to_string());
		expected.push("C:/Users/Ollie/Documents/serverexampletest/css_files/stylesheet_1.css".to_string());

		let x = service.inner().get_css_files(&absolute_path, file_contents);

		x.iter().for_each(|y| println!("{y}"));
		
		assert_eq!(x, expected);


	}
}