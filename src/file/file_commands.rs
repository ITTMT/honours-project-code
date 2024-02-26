use std::{fs::File, io::{BufReader, Read}};

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