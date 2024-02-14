use std::fs::File;
use std::io::{BufReader, Read};
use std::ops::Deref;
use html5gum::{HtmlString, Token, Tokenizer};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult::default())
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initalized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}

// fn main() {
//     let file_as_string = open_file("C:\\Users\\Ollie\\Documents\\CS\\honours-project-code\\html_files\\test.html");
//     let test = tokenize_html(&file_as_string);

//     println!("{:?}", test)
// }

// fn open_file(file :&str) -> String {
//     let file = match File::open(file) {
//         Ok(file) => file,
//         Err(error) => panic!("Error opening file : {:?}", error)
//     };
//     let mut buf_reader = BufReader::new(file);
//     let mut contents = String::new();
//     buf_reader.read_to_string(&mut contents).expect("TODO: panic message");

//     return contents;
// }

// fn tokenize_html(file_contents :&String) -> Vec<String> {
//     let tag_name: HtmlString = HtmlString(b"link".to_vec());
//     let href: HtmlString = HtmlString(b"href".to_vec());

//     let mut css_vec: Vec<String> = vec![];

//     for token in Tokenizer::new(file_contents).infallible() {
//         match token {
//             Token::StartTag(tag) => {
//                 if tag.name == tag_name {
//                     match tag.attributes.get_key_value(&href){
//                         Some((_, value)) => {
//                             let s = value.deref().to_vec();
//                             let string_result = String::from_utf8_lossy(&s);
//                             let string_value = string_result.to_string();

//                             css_vec.push(string_value);
//                         },
//                         None => continue
//                     }
//                 }
//             }
//             _ => continue,
//         }
//     }

//     css_vec
// }




// fn return_css_files(tokens: &Token) -> Vec<str> {
//     tokens
//     . 
// }

//TODO: Open a file (Done), read its contents(Done), turn it into tokens

//TODO: Implement the Client and Server component to detect when a file has been opened and parse it.
