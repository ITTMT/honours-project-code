use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::css_attribute::CssAttribute;


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CssStyle {
    pub tag: String,
    pub attributes: Vec<CssAttribute>,
}

impl CssStyle {
    pub fn new() -> CssStyle {
        CssStyle{
            tag: String::new(),
            attributes: Vec::new()
        }
    }

    pub fn update_attributes(&mut self, new_attributes: Vec<CssAttribute>) {
        let mut new_attributes_map: HashMap<String, CssAttribute> = HashMap::new();

        for new_attribute in new_attributes {
            let existing_attribute = new_attributes_map.entry(new_attribute.name.clone()).or_insert(CssAttribute {
                name: new_attribute.name.clone(),
                values: Vec::new(),
                source: None,
                is_overwritten: None,
            });

            //TODO: Maybe need to update is_overwritten if the value is the same as the source. Would require extra logic.

            existing_attribute.values.extend(new_attribute.values);
            if let Some(source) = new_attribute.source {
                existing_attribute.source = Some(source);
            }
            if let Some(is_overwritten) = new_attribute.is_overwritten {
                existing_attribute.is_overwritten = Some(is_overwritten);
            }
        }

        for original_attribute in &mut self.attributes {
            if let Some(new_attribute) = new_attributes_map.remove(&original_attribute.name) {
                // Replace the original attribute with the new one
                original_attribute.update_or_insert(&new_attribute);
            }
        }

        // Add any extra new attributes
        for (_, attr) in new_attributes_map.into_iter() {
            self.attributes.push(attr);
        }
    }

    pub fn update_or_insert(&mut self, new_attribute: &CssStyle) {
        self.attributes = new_attribute.attributes.clone();
    }
}

#[cfg(test)]
mod tests {
    use crate::metadata::css_metadata::css_attribute::CssAttribute;

    use super::CssStyle;


    #[test]
    fn update_style_test() {
        let mut old_style = CssStyle::new();

        let old_attribute1 = CssAttribute {
            name: String::from("background-color"),
            values: vec![String::from("red")],
            source: None,
            is_overwritten: None,
        };

        let old_attribute2 = CssAttribute {
            name: String::from("font-size"),
            values: vec![String::from("12pt")],
            source: Some(1),
            is_overwritten: Some(false),
        };

        old_style.attributes = vec![old_attribute1.clone(), old_attribute2.clone()];

        let new_attribute1 = CssAttribute {
            name: String::from("background-color"),
            values: vec![String::from("green")],
            source: None,
            is_overwritten: None,
        };

        let mut new_attributes = vec![new_attribute1.clone()];

        old_style.update_attributes(new_attributes);

        let mut new_style = CssStyle::new();
        new_style.attributes = vec![new_attribute1.clone(), old_attribute2.clone()];

        assert_eq!(old_style, new_style);

        let new_attribute2 = CssAttribute {
            name: String::from("font-size"),
            values: vec![String::from("12pt"), String::from("14pt")],
            source: None,
            is_overwritten: None,
        };

        new_attributes = vec![new_attribute1.clone(), new_attribute2.clone()];

        old_style.update_attributes(new_attributes);

        new_style.attributes = vec![new_attribute1.clone(), new_attribute2.clone()];
        new_style.attributes[1].source = Some(1);
        new_style.attributes[1].is_overwritten = Some(false);

        assert_eq!(old_style, new_style);
    }
}

