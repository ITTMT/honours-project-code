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

// https://www.w3.org/TR/2011/WD-html5-20110113/tokenization.html

// it might work better if it was something like
// open tag char detected -> begin open state -> accumulate chars until white-space / special / invalid / close tag
// char -> parse


// inline js and css might be a pain in the arse, especially if I want to add quick editing rules, like
// move to file. I would maybe need to write a tokenizer for both of those as well to be accessed under the same LSP.
// HTML5Tokenizer might not be enough since it is made to only work with HTML, the goal of the project is to have
// better interoperability between html and css, so the LSP would be better if it was context-aware of the two.
