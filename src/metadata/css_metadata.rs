pub mod css_attribute;
pub mod css_file;
pub mod css_style;

use std::{collections::HashMap, fs, path::PathBuf};
use chrono::{DateTime, serde::ts_seconds, Utc};
use cssparser::{ParseError, Parser, ParserInput, Token};
use serde::{Deserialize, Serialize};
use crate::file::create_dir_and_file;
use self::{css_attribute::CssAttribute, css_file::CssFile, css_style::CssStyle};
use super::workspace_metadata::{workspace_css_file::WorkspaceCssFile, WorkspaceMetaData};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CssMetaData {
    pub id: u32,
	pub file_name: String,
	pub absolute_path: String,

	#[serde(with = "ts_seconds")]
	pub last_updated: DateTime<Utc>,
	
	#[serde(skip_serializing_if = "Option::is_none")]
	pub imported_sheets: Option<Vec<CssFile>>, // imported files from .bhc/.shared/
	
	#[serde(skip_serializing_if = "Option::is_none")]
	pub styles: Option<Vec<CssStyle>>,
}

impl CssMetaData {
	pub fn new() -> CssMetaData {
		CssMetaData {
            id: 0,
			file_name: String::new(),
			absolute_path: String::new(),
			last_updated: Utc::now(),
			imported_sheets: None,
			styles: None,
		}
	}


    /// For the provided mutable `self`, modify all the `CssStyle`'s. The styles will be updated based on the contents of `new_styles`, if a style is not present in `new_styles`, that is indicative that is has been deleted and will be removed. 
    pub fn update_styles(&mut self, new_styles: Vec<CssStyle>) {
        let mut new_styles_map: HashMap<String, CssStyle> = HashMap::new();

        for new_style in new_styles {
            let existing_style = new_styles_map.entry(new_style.tag.clone()).or_insert(CssStyle {
                tag: new_style.tag.clone(),
                attributes: new_style.attributes.clone(),
            });

            existing_style.update_attributes(new_style.attributes.clone())
        }

        if let Some(styles) = &mut self.styles{
            for original_style in styles {
                if let Some(new_style) = new_styles_map.remove(&original_style.tag) {
                    // Replace the original attribute with the new one
                    original_style.update_or_insert(&new_style);
                }
            }
        }

        for (_, style) in new_styles_map.into_iter() {
            if let Some(styles) = &mut self.styles {
                styles.push(style);
            } else {
                self.styles = Some(vec![style]);
            }
        }
    }

	pub fn create_metadata(metadata_path: &PathBuf, file_path: &PathBuf, id: &u32) -> Result<CssMetaData, String> {

		match create_dir_and_file(&metadata_path) {
			Ok(_) => (),
			Err(error) => return Err(error)
		};

		let css_string = match fs::read_to_string(&file_path) {
			Ok(value) => value,
			Err(error) => return Err(format!("Error trying to read css file: ({:?}) {:?}", &file_path, error))
		};

		let mut metadata = CssMetaData::new();

        metadata.id = *id;
		metadata.file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();
		metadata.absolute_path = file_path.to_str().unwrap().to_string();
		metadata.last_updated = Utc::now();

		let mut parser_input = ParserInput::new(&css_string);
		let mut parser = Parser::new(&mut parser_input);

		metadata.styles = parse_sheet(&mut parser).unwrap();

		match fs::write(&metadata_path, serde_json::to_string_pretty(&metadata).unwrap()) {
			Ok(_) => return Ok(metadata),
			Err(error) => return Err(format!("Error writing metadata to file: ({:?}) {:?}", &metadata_path, error))
		};
	}
	
	pub fn update_metadata(&mut self, metadata_path: &PathBuf, workspace_metadata: &WorkspaceMetaData) -> Result<WorkspaceCssFile, String> {
        let file_path = PathBuf::from(&self.absolute_path);
        
        let mut new_metadata = self.clone();

        new_metadata.last_updated = Utc::now();

        let css_string = match fs::read_to_string(&file_path) {
			Ok(value) => value,
			Err(error) => return Err(format!("Error trying to read css file: ({:?}) {:?}", &file_path, error))
		};

        let mut parser_input = ParserInput::new(&css_string);
		let mut parser = Parser::new(&mut parser_input);

        new_metadata.styles = parse_sheet(&mut parser).unwrap();

        if let Some(styles) = new_metadata.styles.clone() {
            self.update_styles(styles.clone());
        } else {
            self.styles = None
        }

        match fs::write(&metadata_path, serde_json::to_string_pretty(&new_metadata).unwrap()) {
			Ok(_) => (),
			Err(error) => return Err(format!("Error writing metadata to file: ({:?}) {:?}", &metadata_path, error))
		};

        Ok(WorkspaceCssFile::parse(&new_metadata))
	}
}

//TODO: Generate a css string from a Vec<CssMetaData>, will need to turn it into a dictionary and then back to sorted vec grouping all the same tags 
pub fn merge_css_metadata(metadata_files: &Vec<CssMetaData>) -> Result<Vec<CssStyle>, String> {

    // Tag , Attributes

    /*
        h1 {
            background-color: red;
            background-color: green;
            font-size: 14pt;
            font-family: ...;
        }
    
     */
    let mut css_map: HashMap<String, Vec<CssAttribute>> = HashMap::new();

    for metadata_file in metadata_files {
        if let Some(styles) = &metadata_file.styles {
            styles
            .iter()
            .for_each( |style| {
                let key = style.tag.clone();

                let existing_style = css_map.entry(key).or_insert(Vec::new());

                style.attributes.iter().for_each(|attribute| {
                    existing_style.push(attribute.clone());
                });
            })
        }
    }

    let mut css_vec: Vec<CssStyle> = css_map
    .iter_mut()
    .map(|(kv, x)| {
        x.sort_by_key(|x| x.name.clone());
        
        CssStyle {
            tag: kv.clone(),
            attributes: x.clone(),
        }
    })
    .collect();

    css_vec.sort_by_key(|x| x.tag.clone());


    Ok(css_vec)
}


// Also sorts the styles and attributes in alphabetical order
pub fn parse_sheet<'a>(parser: &mut Parser) -> Result<Option<Vec<CssStyle>>, ParseError<'a, String>> {
    let mut styles: Vec<CssStyle> = Vec::new();
    let mut style = CssStyle::new();

    while !parser.is_exhausted() {
        match parser.next() {
            Ok(token) => match token {
                Token::Ident(value) => {
                    style = CssStyle::new();
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

    if styles.len() > 0 {
        styles.sort_by_key(|style| style.tag.clone());

        return Ok(Some(styles))
    }

    Ok(None)

}

fn parse_attributes<'a>(parser: &mut Parser) -> Result<Vec<CssAttribute>, ParseError<'a, String>> {
    let mut attribute_map: HashMap<String, CssAttribute> = HashMap::new();
    
    let mut attribute = CssAttribute::new();
    
    while !parser.is_exhausted() {
        match parser.next() {
            Ok(token) => match token {
                Token::Ident(value) => {
                    let new_value = value.to_string();

                    match attribute_map.get_key_value(&new_value) {
                        Some((_, val)) => attribute = val.clone(),
                        None => {
                            attribute = CssAttribute::new();
                            attribute.name = new_value.clone();
                        },
                    }                    
                },
                Token::Colon => {
                    let attribute_value = parse_attribute_value(parser).unwrap();

                    attribute.values.push(attribute_value);

                    attribute_map.insert(attribute.clone().name, attribute.clone());
                }
                _ => (),
            },

            Err(_) => ()
        }
    }

    let mut attributes: Vec<CssAttribute> = attribute_map.values().cloned().collect();
    attributes.sort_by_key(|attr| attr.name.clone());

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
                Token::Dimension { has_sign: _, value, int_value: _, unit } => {
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


/* #region Unit Tests */
#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use cssparser::{Parser, ParserInput};

    use super::{merge_css_metadata, parse_sheet, CssAttribute, CssFile, CssMetaData, CssStyle};

    #[test]
    fn test_serialize_deserialize() {
        let attribute1 = CssAttribute{
            name: String::from("background-color"),
            values: vec![String::from("red")],
            source: None,
            is_overwritten: None, 
        };
        let attribute2 = CssAttribute{
            name: String::from("background-color"), 
            values: vec![String::from("green")],
            source: Some(2), 
            is_overwritten: Some(false),
        };
        let attribute3 = CssAttribute{
            name: String::from("font-size"),
            values: vec![String::from("11pt")],
            source: Some(2),
            is_overwritten: Some(true),
        };

        let attributes1: Vec<CssAttribute> = vec![attribute1];
        let attributes2: Vec<CssAttribute> = vec![attribute2, attribute3];

        let style1 = CssStyle{
            tag: String::from("h1"),
            attributes: attributes1,
        };
        let style2 = CssStyle{
            tag: String::from("p"), 
            attributes: attributes2,
        };

        let styles: Vec<CssStyle> = vec![style1, style2];

        let file1 = CssFile {
            id: 2, 
            file_name: String::from("base.css"), 
            absolute_path: String::from("D:/programming/web-dev/xd/.bhc/.shared/base.css"),
        };

        let files = vec![file1];

        let metadata = CssMetaData{
            id: 0,
            file_name: String::from("test.css"), 
            absolute_path: String::from("D:/programming/web-dev/xd/css/test.css"), 
            last_updated: DateTime::from_timestamp(1710090300, 0).unwrap(), 
            styles: Some(styles), 
            imported_sheets: Some(files),
        };

        let serialized = serde_json::to_string(&metadata).unwrap();

        println!("serialized = {}", serialized);

        let deserialized: CssMetaData = serde_json::from_str(&serialized).unwrap();

        println!("deserialized = {:?}", deserialized);

        assert_eq!(deserialized, metadata)
    }

    #[test]
    fn test_parse_sheet() {
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

        let mut expected = CssMetaData::new();
        let mut style1 = CssStyle::new();
        style1.tag = String::from("h1");
        let mut attribute1 = CssAttribute::new();
        attribute1.name = String::from("background-color");
        attribute1.values = vec![String::from("red"), String::from("green")];
        let mut attribute2 = CssAttribute::new();
        attribute2.name = String::from("font-size");
        attribute2.values = vec![String::from("100pt")];
        let mut attribute3 = CssAttribute::new();
        attribute3.name = String::from("xd");
        attribute3.values = vec![String::from("100px")];

        style1.attributes = vec![attribute1, attribute2, attribute3];

        let mut style2 = CssStyle::new();
        style2.tag = String::from("p");
        let mut attribute4 = CssAttribute::new();
        attribute4.name = String::from("font-size");
        attribute4.values = vec![String::from("14pt")];

        style2.attributes = vec![attribute4];

        expected.styles = Some(vec![style1, style2]);

        expected.last_updated = metadata.last_updated;

        assert_eq!(expected, metadata);

    }

    #[test]
    fn merge_css_metadata_test() {
        let attribute_1 = CssAttribute {
            name: String::from("background-color"),
            values: vec![String::from("red"), String::from("green")],
            source: None,
            is_overwritten: None,
        };



        let input: Vec<CssMetaData> = vec![];

        let output = merge_css_metadata(&input).unwrap();
    }


}
/* #endregion */