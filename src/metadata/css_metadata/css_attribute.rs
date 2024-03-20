use ::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CssAttribute {
    pub name: String,
    pub values: Vec<String>, // This needs to be a vector, because there might be some times when there are multiple values for the same style, which would just mean the last one is actually styled.

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<u32>, // 0 if inline style in HTML, id otherwise. If it is missing, then it is an original from the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_overwritten: Option<bool>, // will be None if it's not an imported style, true if the value is different to that of the source file, and false if it's the same.
}


impl CssAttribute {
    pub fn new() -> CssAttribute {
        CssAttribute {
            name: String::new(),
            values: Vec::new(),
            source: None,
            is_overwritten: None,
        }
    }
    
    pub fn update_or_insert(&mut self, new_attribute: &CssAttribute) {
        // Update source and is_overwritten fields
        if let Some(source) = new_attribute.source {
            self.source = Some(source);
        }
        if let Some(is_overwritten) = new_attribute.is_overwritten {
            self.is_overwritten = Some(is_overwritten);
        }
        
        // Replace the values of the current attribute with the new ones
        self.values = new_attribute.values.clone();
    }
}
