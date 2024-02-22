use std::{fs::{self, File}, io::{self, Write}, path::Path};
use mktemp::Temp;


use crate::file::file_commands::open_file_string;

pub fn produce_css_file(url_as_string: Vec<String>) -> String {

	let mut test: String = String::new(); 

	for s in url_as_string {
		test.push_str(&open_file_string(&s).expect(""));
	}

	// url_as_string
	// 	.iter()
	// 	.map(|file_name| test.push_str(&open_file_string(&file_name).expect("")));

	let new_path = "C:/Users/Ollie/Documents/serverexampletest/css_files/TestFile.css";
	if !Path::new(new_path).exists() {
		let _ = File::create(new_path);
	} else {
		let _ = fs::remove_file(new_path);
		let _ = File::create(new_path);
	}

	let tmp_path = Temp::new_file().expect("");

    // Open temp file for writing
    let mut tmp = File::create(&tmp_path).expect("");
    // Open source file for reading
    let mut src = File::open(&new_path).expect("");
    // Write the data to prepend
    tmp.write_all(test.as_bytes()).expect("");
    // Copy the rest of the source file
    io::copy(&mut src, &mut tmp).expect("");
    fs::remove_file(&new_path).expect("");
    fs::rename(&tmp_path, &new_path).expect("");

	return new_path.to_string();
	
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;

    use crate::{file::file_commands, html::html_commands::get_css_files};

    use super::produce_css_file;


	#[test]
	fn it_works() {
		let path = "C:/Users/Ollie/Documents/serverexampletest/html_files/test.html";
		let file_path = Url::parse(path).expect("");

		let test = file_commands::open_file(file_path).expect("");


		produce_css_file(get_css_files(&test));

	}
}