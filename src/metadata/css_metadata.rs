use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CssMetaData {
    pub file_name: String,
    pub absolute_path: String,

    #[serde(with = "ts_seconds")]
    pub last_updated: DateTime<Utc>,
    pub imported_sheets: Vec<CssFile>, // imported files from .bhc/.shared/
    pub styles: Vec<Style>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CssFile {
    pub id: u32,
    pub file_name: String,
    pub absolute_path: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Style {
    pub tag: String,
    pub attributes: Vec<Attribute>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<u32>, // 0 if inline style in HTML, id otherwise. If it is missing, then it is an original from the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_overwritten: Option<bool>, // will be None if it's not an imported style, true if the value is different to that of the source file, and false if it's the same.
}

impl CssMetaData {
    fn new<S: Into<String>>(
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
    fn new<S: Into<String>>(tag: S, attributes: Vec<Attribute>) -> Style {
        Style {
            tag: tag.into(),
            attributes: attributes,
        }
    }
}

impl Attribute {
    fn new<S: Into<String>>(name: S, value: S, source: Option<u32>, is_overwritten: Option<bool>) -> Attribute {
        Attribute {
            name: name.into(),
            value: value.into(),
            source: source,
            is_overwritten: is_overwritten,
        }
    }
}

/* #region Unit Tests */
#[cfg(test)]
mod tests {
    use std::{fs, time::UNIX_EPOCH};

    use super::{Attribute, CssFile, CssMetaData, Style};

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
        let attribute1 = Attribute::new("background-color", "red", None, None);
        let attribute2 = Attribute::new("background-color", "green", Some(2), Some(false));
        let attribute3 = Attribute::new("font-size", "11pt", Some(2), Some(true));

        let attributes1: Vec<Attribute> = vec![attribute1];
        let attributes2: Vec<Attribute> = vec![attribute2, attribute3];

        let style1 = Style::new("h1", attributes1);
        let style2 = Style::new("p", attributes2);

        let styles: Vec<Style> = vec![style1, style2];

        let file1 = CssFile::new(2, "base.css", "D:/programming/web-dev/xd/.bhc/.shared/base.css");

        let files = vec![file1];

        let metadata = CssMetaData::new("test.css", "D:/programming/web-dev/xd/css/test.css", 1710090300, styles, files);

        let serialized = serde_json::to_string(&metadata).unwrap();

        println!("serialized = {}", serialized);

        let deserialized: CssMetaData = serde_json::from_str(&serialized).unwrap();

        println!("deserialized = {:?}", deserialized);

        assert_eq!(deserialized, metadata);
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
