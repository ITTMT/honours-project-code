mod lexer;

use std::fs::File;
use std::io::{BufReader, Read};
use html5tokenizer::{BasicEmitter, NaiveParser, Token};

fn main() {
    let file_as_string = open_file("D:/University/honours-project-code/html_files/test.html");

    // println!("{}", file_as_string);

    tokenize_html(&file_as_string);
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

fn tokenize_html(file_contents :&String) {
    let emitter = BasicEmitter::default();

    for tokens in html5tokenizer::Tokenizer::new(file_contents, emitter) {
        println!("{:?}", tokens)
    }

    // This kind of works, but there will be some hacky work needed to be done to convert it to JSON.

    // Example output
    // Ok(Token(StartTag(StartTag { name: "html", self_closing: false, attributes: {"lang": "en"} })))
}

//TODO: Open a file (Done), read its contents(Done), turn it into tokens

//TODO: Implement the Client and Server component to detect when a file has been opened and parse it.

// https://github.com/microsoft/vscode/tree/main/extensions/html-language-features/server/src
// It's Microsoft's TypeScript implementation of HTML, it's a mess to try and follow.
// Looking at what an ideal JSON layout might be for it. What information is required to be recorded.
