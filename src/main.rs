mod lexer;

use std::fs::File;
use std::io::{BufReader, Read};

fn main() {
    let file_as_string = open_file("D:/University/honours-project-code/html_files/test.html");

    println!("{}", file_as_string);
}

fn open_file(file :&str) -> String {
    let file = match File::open(file) {
        Ok(file) => file,
        Err(error) => panic!("Error opening file : {:?}", error)
    };
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).expect("TODO: panic message");


    return contents;
}

//TODO: Open a file (Done), read its contents(Done), turn it into tokens

//TODO: Implement the Client and Server component to detect when a file has been opened.