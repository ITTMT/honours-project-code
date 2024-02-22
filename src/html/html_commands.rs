use std::ops::Deref;

use html5gum::{HtmlString, Token, Tokenizer};

pub fn get_css_files(file_contents :&String) -> Vec<String> {
    let tag_name: HtmlString = HtmlString(b"link".to_vec());
    let href: HtmlString = HtmlString(b"href".to_vec());

    let mut css_vec: Vec<String> = vec![];

    for token in Tokenizer::new(file_contents).infallible() {
        match token {
            Token::StartTag(tag) => {
                if tag.name == tag_name {
                    match tag.attributes.get_key_value(&href){
                        Some((_, value)) => {
                            let s = value.deref().to_vec();
                            let string_result = String::from_utf8_lossy(&s);
                            let string_value = string_result.to_string();

                            css_vec.push(string_value);
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

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;

    use crate::{file::file_commands, html::html_commands::get_css_files};

	#[test]
	fn it_works() {
		let path = "C:/Users/Ollie/Documents/serverexampletest/html_files/test.html";
		let file_path = Url::parse(path).expect("");

		let test = file_commands::open_file(file_path).expect("");


		println!("{:?}", get_css_files(&test));

	}
}