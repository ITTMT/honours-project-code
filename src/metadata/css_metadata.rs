use std::{fs, path::PathBuf};

use chrono::{DateTime, serde::ts_seconds, Utc};
use cssparser::{BasicParseError, ParseError, Parser, ParserInput, Token};
use serde::{Deserialize, Serialize};

use crate::file::create_dir_and_file;

use super::Metadata;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CssMetaData {
    pub file_name: String,
    pub absolute_path: String,

    #[serde(with = "ts_seconds")]
    pub last_updated: DateTime<Utc>,
    pub imported_sheets: Vec<CssFile>, // imported files from .bhc/.shared/
    pub styles: Vec<Style>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CssFile {
    pub id: u32,
    pub file_name: String,
    pub absolute_path: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Style {
    pub tag: String,
    pub attributes: Vec<Attribute>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Attribute {
    pub name: String,
    pub value: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<u32>, // 0 if inline style in HTML, id otherwise. If it is missing, then it is an original from the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_overwritten: Option<bool>, // will be None if it's not an imported style, true if the value is different to that of the source file, and false if it's the same.
}

impl CssMetaData {
    fn new() -> CssMetaData {
        CssMetaData {
            file_name: String::new(),
            absolute_path: String::new(),
            last_updated: Utc::now(),
            imported_sheets: Vec::new(),
            styles: Vec::new(),
        }
    }

    fn new_<S: Into<String>>(
        file_name: S, absolute_path: S, last_updated: i64, styles: Vec<Style>, imported_sheets: Vec<CssFile>,
    ) -> CssMetaData {
        CssMetaData {
            file_name: file_name.into(),
            absolute_path: absolute_path.into(),
            last_updated: DateTime::from_timestamp(last_updated, 0).unwrap(),
            styles: styles,
            imported_sheets: imported_sheets,
        }
    }
}

impl CssFile {
    fn new<S: Into<String>>(id: u32, file_name: S, absolute_path: S) -> CssFile {
        CssFile {
            id: id,
            file_name: file_name.into(),
            absolute_path: absolute_path.into(),
        }
    }
}

impl Style {
    fn new() -> Style {
        Style{
            tag: String::new(),
            attributes: Vec::new()
        }
    }

    fn new_<S: Into<String>>(tag: S, attributes: Vec<Attribute>) -> Style {
        Style {
            tag: tag.into(),
            attributes: attributes,
        }
    }
}

impl Attribute {
    fn new() -> Attribute {
        Attribute {
            name: String::new(),
            value: String::new(),
            source: None,
            is_overwritten: None,
        }
    }
    fn new_<S: Into<String>>(name: S, value: S, source: Option<u32>, is_overwritten: Option<bool>) -> Attribute {
        Attribute {
            name: name.into(),
            value: value.into(),
            source: source,
            is_overwritten: is_overwritten,
        }
    }
}

impl Metadata<CssMetaData> for CssMetaData {
    fn create_metadata(metadata_path: &PathBuf, file_path: &PathBuf) -> Result<CssMetaData, String> {

        match create_dir_and_file(&metadata_path) {
            Ok(_) => (),
            Err(error) => return Err(error)
        };

        let css_string = match fs::read_to_string(&file_path) {
            Ok(value) => value,
            Err(error) => return Err(format!("Error trying to read css file: ({:?}) {:?}", &file_path, error))
        };

        let last_updated = fs::metadata(&file_path).unwrap();

        let mut metadata = CssMetaData::new();

        metadata.file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();
        metadata.absolute_path = file_path.to_str().unwrap().to_string();
        metadata.last_updated = last_updated.modified().unwrap().into();

        let mut parser_input = ParserInput::new(&css_string);
        let mut parser = Parser::new(&mut parser_input);


        Ok(metadata)
    }
}

pub fn parse_sheet<'a>(parser: &mut Parser) -> Result<Vec<Style>, ParseError<'a, String>> {
    let mut styles: Vec<Style> = Vec::new();
    let mut style = Style::new();

    while !parser.is_exhausted() {
        match parser.next() {
            Ok(token) => match token {
                Token::Ident(value) => {
                    style = Style::new();
                    style.tag = value.to_string()
                },
                Token::CurlyBracketBlock => {
                    let attributes = parser.parse_nested_block(|inner_parser| {
                        parse_attributes(inner_parser)
                    }).unwrap();

                    style.attributes = attributes;

                    styles.push(style.clone());
                },
                _ => (),
            },

            Err(_) => ()
        }
    }

    Ok(styles)
}

fn parse_attributes<'a>(parser: &mut Parser) -> Result<Vec<Attribute>, ParseError<'a, String>> {
    let mut attributes: Vec<Attribute> = Vec::new();
    let mut attribute = Attribute::new();

    while !parser.is_exhausted() {
        match parser.next() {
            Ok(token) => match token {
                Token::Ident(value) => {
                    attribute.name = value.to_string();
                },
                Token::Colon => {
                    let attribute_value = parse_attribute_value(parser).unwrap();

                    attribute.value = attribute_value;

                    attributes.push(attribute.clone());
                }
                _ => (),
            },

            Err(_) => ()
        }
    }


    Ok(attributes)
}

fn parse_attribute_value<'a>(parser: &mut Parser) -> Result<String, ParseError<'a, String>> {
    let mut attribute_value = String::new();

    while !parser.is_exhausted() {
        match parser.next() {
            Ok(token) => match token {
                Token::Ident(value) => {
                    attribute_value = value.to_string();
                },
                Token::Dimension { has_sign, value, int_value, unit } => {
                    attribute_value.push_str(&value.to_string());
                    attribute_value.push_str(&unit);
                }
                Token::Semicolon => {
                    break
                }
                _ => (),
            },

            Err(_) => ()
        }
    }
    Ok(attribute_value)
}

// string = match parser.next() {
// Ok(token) => match token {
// Token::Ident(value) => value.to_string(),
// Token::Dimension { has_sign, value, int_value, unit } => {
// let mut string = String::new();
// string.push_str(&value.to_string());
// string.push_str(&unit);
//
// string
// },
// _ => String::new(),
// },
//
// Err(_) => String::new(),
// };

/* #region Unit Tests */
#[cfg(test)]
mod tests {
    use std::{fs, time::UNIX_EPOCH};
    use cssparser::{Parser, ParserInput};


    use super::{Attribute, CssFile, CssMetaData, parse_sheet, Style};

    #[test]
    fn test() {
        let attr = fs::metadata("D:/programming/web-dev/xd/.bhc/html/test.css").unwrap();

        let attr2 = fs::metadata("D:/programming/web-dev/xd/css/test1.css").unwrap();

        let x = attr.modified().unwrap();

        println!("{:?}", x);

        let z = attr2.modified().unwrap();
        let abc = z.duration_since(UNIX_EPOCH).unwrap().as_secs();
        println!("{:?}", z);
        println!("{:?}", abc);

        let y = 1;
    }

    #[test]
    fn test2() {
        let attribute1 = Attribute::new_("background-color", "red", None, None);
        let attribute2 = Attribute::new_("background-color", "green", Some(2), Some(false));
        let attribute3 = Attribute::new_("font-size", "11pt", Some(2), Some(true));

        let attributes1: Vec<Attribute> = vec![attribute1];
        let attributes2: Vec<Attribute> = vec![attribute2, attribute3];

        let style1 = Style::new_("h1", attributes1);
        let style2 = Style::new_("p", attributes2);

        let styles: Vec<Style> = vec![style1, style2];

        let file1 = CssFile::new(2, "base.css", "D:/programming/web-dev/xd/.bhc/.shared/base.css");

        let files = vec![file1];

        let metadata = CssMetaData::new_("test.css", "D:/programming/web-dev/xd/css/test.css", 1710090300, styles, files);

        let serialized = serde_json::to_string(&metadata).unwrap();

        println!("serialized = {}", serialized);

        let deserialized: CssMetaData = serde_json::from_str(&serialized).unwrap();

        println!("deserialized = {:?}", deserialized);

        assert_eq!(deserialized, metadata);
    }

    #[test]
    fn test3() {
        let css_string = r#"
h1 {
    background-color: red;
    background-color: green;
    font-size: 100pt;
    xd: 100px;
}

p {
    font-size: 14pt;
}"#;

        let mut parserinput = ParserInput::new(&css_string);
        let mut parser = Parser::new(&mut parserinput);

        let mut metadata = CssMetaData::new();

        metadata.styles = parse_sheet(&mut parser).unwrap();

        println!("{metadata:?}");
    }
}
/* #endregion */

// Now I need to tokenize the CSS to be able to feed it into the metadata format.
// Also need to make the workspace metadata format.

/*
This means I have to update these metadata files whenever
    a html file is opened
    a html file is saved
    a css file is opened
    a css file is saved
    one of the concatenated css files are saved. They should never be opened without the html file being opened.


    I will need to do a check then when a file opens,

    if it's html do x
    css do y


When we save a html file, what can change that we need to keep track of for the sake of making concatenated files
    what linked css files are contained in it
    what inline styles there are


When we save a css file...
    what styles there are / were


When we save a concatenated css file
    what file the style belongs to might change

    if we have a style sheet that is used by multiple pages we want to
        provide warning that changing a style might affect other pages
        try to find style sheets unique to each page and move the original "shared" style into it

            e.g. we have a base.css file
                h1 {
                    background-color = red;
                }

            this file is referenced by 3 other pages
            we want to change it to green on one page but keep it red on the other 2
            so we need to move the style from the base.css file into individual files for each page.

            It might be easier to provide a folder inside .bhc for storing base css style sheets that will automatically apply to each page
            in the virtual view and save them individually into the actual files.

            Rules being, every html file has a unique css that belongs to them, they can be lego blocked with shared style sheets stored inside
            .bhc/.shared/ by some magic, maybe some autocomplete inside the css file to do include <x> and it automatically pastes the styles into the style sheet.


    I've now made more work for myself,
    if someone is installing this extension, they likely still have more than 1 file importing in a html file
    do i need to check then if the file contains a unique imported file or just tell them, they should create a unique file
    probably the latter,
                Create warning saying: create a unique file somewhere and link it
                Set up shared styles in the .bhc/.shared/ folder
                to import a shared style sheet into the unique one, type !import <x> and it will paste the contents into the file
                with this, it means I no longer need to create a separate virtual view... maybe. That's only true if they do follow that guideline,
                otherwise multiple imports still need to be concatenated. Do a check then... if a file contains multiple imports, then we concatenate and make
                a temp virtual file, otherwise we open the actual file, but we need to enforce a rule that in order for it to work, the sheets need to be unique
                and not imported inside multiple html files.
*/
