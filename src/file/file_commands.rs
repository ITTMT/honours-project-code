use std::{fs::File, io::{BufReader, Read}};

use tower_lsp::lsp_types::Url;

pub fn open_file(file_path: Url) -> Result<String, String> {
	let file_path_string = file_path.as_str();

	let file = match File::open(&file_path_string) {
		Ok(file) => file,
		Err(error) => return Err(format!("File Opening Error: Unable to open file ({}) - {}", &file_path_string, error))
	};

	let mut buf_reader = BufReader::new(file);
	let mut contents = String::new();
	match buf_reader.read_to_string(&mut contents){
		Ok(buffer) => buffer,
		Err(error) => return Err(format!("File Opening Error: Unable to read file ({}) - {}", &file_path_string, error))
	};

	Ok(contents)
}

pub fn open_file_string(file_path: &str) -> Result<String, String> {

	let file = match File::open(&file_path) {
		Ok(file) => file,
		Err(error) => return Err(format!("File Opening Error: Unable to open file ({}) - {}", &file_path, error))
	};

	let mut buf_reader = BufReader::new(file);
	let mut contents = String::new();
	match buf_reader.read_to_string(&mut contents){
		Ok(buffer) => buffer,
		Err(error) => return Err(format!("File Opening Error: Unable to read file ({}) - {}", &file_path, error))
	};

	Ok(contents)
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;

    use super::open_file;

	#[test]
	fn it_works() {
		let path = "C:/Users/Ollie/Documents/serverexampletest/html_files/test.html";

		let test = Url::parse("C:/Users/Ollie/Documents/serverexampletest/html_files/test.html").expect("");
		let test_str = test.as_str();

		println!("{:?}", open_file(test));
	}
}