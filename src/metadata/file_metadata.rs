use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::css_metadata::{css_attribute::CssAttribute, CssMetaData};

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct FormattedCssFile {
	pub absolute_path: String,
    pub included_files: Vec<FileMetaData>,
	pub styles: Vec<CssStyleExtended>,
	pub lines: Vec<LineInformation>,
}

impl FormattedCssFile {
	pub fn new() -> FormattedCssFile {
		FormattedCssFile {
			absolute_path: String::new(),
			included_files: Vec::new(),
			styles: Vec::new(),
			lines: Vec::new()
		}
	}

	pub fn generate_formatted_file(metadata_files: &Vec<CssMetaData>) -> FormattedCssFile {
		// Tag , Attributes
		let mut formatted_file = FormattedCssFile::new();
		let mut css_map: HashMap<String, Vec<CssAttributeExtended>> = HashMap::new();
	
		for metadata_file in metadata_files {
			if let Some(styles) = &metadata_file.styles {
				styles
				.iter()
				.for_each( |style| {
					let key = style.tag.clone();
	
					let existing_attributes = css_map.entry(key).or_insert(Vec::new());
	
					style.attributes.iter().for_each(|attribute| {
						existing_attributes.append(&mut CssAttributeExtended::from_attribute(attribute.clone(), metadata_file.id));
					});
				})
			}

			formatted_file.included_files.push(FileMetaData {
				id: metadata_file.id,
				file_name: metadata_file.file_name.clone(),
				absolute_path: metadata_file.absolute_path.clone(),
			})
		}
	
		let mut css_vec: Vec<CssStyleExtended> = css_map
		.iter_mut()
		.map(|(key, value)| {
			value.sort_by_key(|attribute| attribute.name.clone());
			
			CssStyleExtended {
				owner: {
					if let Some(first) = value.first() {
						if value.iter().all(|attribute| attribute.owner == first.owner) {
							Some(first.owner)
						} else {
							None
						}
					} else {
						get_owner(&key, metadata_files)
					}
				},
				tag: key.clone(),
				attributes: value.clone(),
			}
		})
		.collect();
	
		css_vec.sort_by_key(|x| x.tag.clone());
	
		formatted_file.styles = css_vec;

		formatted_file.update_lines();

		formatted_file
	}
	
	pub fn to_css_string(&self) -> String {
		let mut css_string = String::new();
		let mut is_last_attribute = false;
	
		self.styles
		.iter()
		.for_each(|style| {
			css_string.push_str(&style.tag);
			css_string.push_str(" {\n\t");
			style
			.attributes
			.iter()
			.enumerate()
			.for_each(|(attr_index, attribute)| {
				if attr_index == style.attributes.len() - 1 { is_last_attribute = true }
				
				css_string.push_str(&attribute.name);
				css_string.push_str(": ");
				css_string.push_str(&attribute.value);
				if is_last_attribute {
					css_string.push_str(";\n"); //TODO: Add a function that tracks indentation
				} else {
					css_string.push_str(";\n\t");
				}
			});
			css_string.push_str("}\n");
			is_last_attribute = false;
		});
	
		css_string
	}

	pub fn update_lines(&mut self) {
		self.lines.clear();

		let mut line_count: u32 = 0;

		self
		.styles
		.iter()
		.for_each(|style| {
			self.lines.push(LineInformation::new_line(line_count, style.owner));
			line_count += 1;
			
			style
			.attributes
			.iter()
			.for_each(|attribute| {
				self.lines.push(LineInformation::new_line(line_count, Some(attribute.owner)));
				line_count += 1;
			});

			self.lines.push(LineInformation::new_line(line_count, style.owner));
			line_count +=1;
		});
	}

}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct CssStyleExtended {
	pub owner: Option<u32>,
	pub tag: String,
    pub attributes: Vec<CssAttributeExtended>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct CssAttributeExtended {
	pub owner: u32,
	pub name: String,
    pub value: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<u32>, // 0 if inline style in HTML, id otherwise. If it is missing, then it is an original from the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_overwritten: Option<bool>, // will be None if it's not an imported style, true if the value is different to that of the source file, and false if it's the same.
}

impl CssAttributeExtended {
	// Have to destack the values to value to get individual line owner
	pub fn from_attribute(attribute: CssAttribute, owner_id: u32) -> Vec<CssAttributeExtended> {
		attribute
		.values
		.iter()
		.map(|value| {
			CssAttributeExtended {
				owner: owner_id,
				name: attribute.name.clone(),
				value: value.clone(),
				source: attribute.source,
				is_overwritten: attribute.is_overwritten
			}
		})
		.collect()
	}
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct LineInformation {
	pub line_number: u32,
	pub owner: Option<u32>
}

impl LineInformation {
	pub fn new_line(line_number: u32, owner: Option<u32>) -> LineInformation {
		LineInformation {
			line_number: line_number,
			owner: owner
		}
	}
}



#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct FileMetaData {
    pub id: u32,
    pub file_name: String,
	pub absolute_path: String,
}

fn get_owner(tag_name: &str, metadata_files: &Vec<CssMetaData>) -> Option<u32> {
	for metadata_file in metadata_files {
		if let Some(styles) = &metadata_file.styles {
			if styles.iter().any(|x| x.tag == tag_name) {
				return Some(metadata_file.id)
			}
		}
	}

	None
}




#[cfg(test)]
mod tests {
	#[test]
    fn merge_css_metadata_test() {

        /*
        h1 {
            background-color: red;
            background-color: green;
            font-size: 14pt;
            font-family: ...;
        }
        */

        // let attribute_1 = CssAttribute {
        //     name: String::from("background-color"),
        //     values: vec![String::from("red"), String::from("green")],
        //     source: None,
        //     is_overwritten: None,
        // };

        // let style_1 = CssStyle {
        //     tag: String::from("h1"),
        //     attributes: vec![attribute_1.clone()]
        // };

        // let attribute_2 = CssAttribute {
        //     name: String::from("font-size"),
        //     values: vec![String::from("14pt")],
        //     source: None,
        //     is_overwritten: None
        // };

        // let style_2 = CssStyle {
        //     tag: String::from("h1"),
        //     attributes: vec![attribute_2.clone()]
        // };

        // let css_metadata_1 = CssMetaData {
        //     id: 0,
        //     file_name: String::new(),
        //     absolute_path: String::new(),
        //     last_updated: Utc::now(),
        //     imported_sheets: None,
        //     styles: Some(vec![style_1])
        // };

        // let css_metadata_2 = CssMetaData {
        //     id: 1,
        //     file_name: String::new(),
        //     absolute_path: String::new(),
        //     last_updated: Utc::now(),
        //     imported_sheets: None,
        //     styles: Some(vec![style_2])
        // };

        // let expected_style = CssStyle {
        //     tag: String::from("h1"),
        //     attributes: vec![attribute_1.clone(), attribute_2.clone()]
        // };


        // let expected: Vec<CssStyle> = vec![expected_style];


        // let output = merge_css_metadata(&vec![css_metadata_1, css_metadata_2]);

        // println!("{output:?}");

        // assert_eq!(expected, output);
    }

    #[test]
    fn generate_css_string_test() {
    //     let attribute_1 = CssAttribute {
    //         name: String::from("background-color"),
    //         values: vec![String::from("red"), String::from("green")],
    //         source: None,
    //         is_overwritten: None,
    //     };

    //     let style_1 = CssStyle {
    //         tag: String::from("h1"),
    //         attributes: vec![attribute_1.clone()]
    //     };

    //     let attribute_2 = CssAttribute {
    //         name: String::from("font-size"),
    //         values: vec![String::from("14pt")],
    //         source: None,
    //         is_overwritten: None
    //     };

    //     let style_2 = CssStyle {
    //         tag: String::from("h1"),
    //         attributes: vec![attribute_2.clone()]
    //     };

    //     let css_metadata_1 = CssMetaData {
    //         id: 0,
    //         file_name: String::new(),
    //         absolute_path: String::new(),
    //         last_updated: Utc::now(),
    //         imported_sheets: None,
    //         styles: Some(vec![style_1])
    //     };

    //     let css_metadata_2 = CssMetaData {
    //         id: 1,
    //         file_name: String::new(),
    //         absolute_path: String::new(),
    //         last_updated: Utc::now(),
    //         imported_sheets: None,
    //         styles: Some(vec![style_2])
    //     };

    // let expected = "h1 {\n\tbackground-color: red;\n\tbackground-color: green;\n\tfont-size: 14pt;\n}\n";

    // assert_eq!(generate_css_string(&vec![css_metadata_1, css_metadata_2]), expected);
    // }
	}
}