use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{Deserialize, Serialize};

/* #region CSSMetaData */
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CSSMetaData {
    file_name:     String,
    absolute_path: String,

    #[serde(with = "ts_seconds")]
    last_updated: DateTime<Utc>,

    css_files: Vec<CSSFile>,
    styles:    Vec<Style>,
}

impl CSSMetaData {
    fn new<S: Into<String>>(file_name: S, absolute_path: S, last_updated: i64, css_files: Vec<CSSFile>, styles: Vec<Style>) -> CSSMetaData {
        CSSMetaData {
            file_name:     file_name.into(),
            absolute_path: absolute_path.into(),
            last_updated:  DateTime::from_timestamp(last_updated, 0).unwrap(),
            css_files:     css_files,
            styles:        styles,
        }
    }
}
/* #endregion */

/* #region CSSFile */
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CSSFile {
    id:            u32,
    file_name:     String,
    absolute_path: String,
}

impl CSSFile {
    fn new<S: Into<String>>(id: u32, file_name: S, absolute_path: S) -> CSSFile {
        CSSFile {
            id:            id,
            file_name:     file_name.into(),
            absolute_path: absolute_path.into(),
        }
    }
}
/* #endregion */

/* #region Style */
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Style {
    tag:        String,
    attributes: Vec<Attribute>,
}

impl Style {
    fn new<S: Into<String>>(tag: S, attributes: Vec<Attribute>) -> Style {
        Style {
            tag:        tag.into(),
            attributes: attributes,
        }
    }
}
/* #endregion */

/* #region Attribute */
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Attribute {
    name:     String,
    value:    String,
    owned_by: u32,
}

impl Attribute {
    fn new<S: Into<String>>(name: S, value: S, owned_by: u32) -> Attribute {
        Attribute {
            name:     name.into(),
            value:    value.into(),
            owned_by: owned_by,
        }
    }
}
/* #endregion */

/* #region Unit Tests */
#[cfg(test)]
mod tests {
    use std::fs;

    use super::{Attribute, CSSFile, CSSMetaData, Style};

    #[test]
    fn test() {
        let attr = fs::metadata("D:/programming/web-dev/xd/.bhc/html/test.css").unwrap();

        let attr2 = fs::metadata("D:/programming/web-dev/xd/css/test1.css").unwrap();

        let x = attr.modified().unwrap();

        println!("{:?}", x);

        let z = attr2.modified().unwrap();
        println!("{:?}", z);

        let y = 1;
    }

    #[test]
    fn test2() {
        let attribute1 = Attribute::new("background-color", "red", 1);
        let attribute2 = Attribute::new("background-color", "green", 2);
        let attribute3 = Attribute::new("font-size", "11pt", 2);

        let attributes1: Vec<Attribute> = vec![attribute1];
        let attributes2: Vec<Attribute> = vec![attribute2, attribute3];

        let style1 = Style::new("h1", attributes1);
        let style2 = Style::new("p", attributes2);

        let styles: Vec<Style> = vec![style1, style2];

        let file1 = CSSFile::new(1, "test1.css", "D:/programming/web-dev/xd/css/test1.css");
        let file2 = CSSFile::new(2, "test2.css", "D:/programming/web-dev/xd/css/test2.css");

        let files = vec![file1, file2];

        let metadata = CSSMetaData::new("test.css", "D:/programming/web-dev/xd/.bhc/html/test.css", 1710090300, files, styles);

        let serialized = serde_json::to_string(&metadata).unwrap();

        println!("serialized = {}", serialized);

        let deserialized: CSSMetaData = serde_json::from_str(&serialized).unwrap();

        println!("deserialized = {:?}", deserialized);

        assert_eq!(deserialized, metadata);
    }
}
/* #endregion */
/*
{
    "file_name" : "test.css",
    "absolute_path" : "D:/programming/web-dev/xd/.bhc/html/test.css",
    "last_updated" : "1710090300",
    "css_files" : [
        {
            "id" : 1,
            "file_name" : "test1.css",
            "absolute_path" : "D:/programming/web-dev/xd/css/test1.css"
        },
        {
            "id" : 2,
            "file_name" : "test2.css",
            "absolute_path" : "D:/programming/web-dev/xd/css/test2.css"
        }
    ],
    "styles" : [
        {
            "tag" : "h1",
            "attributes" : [
                {
                    "name" : "background-color",
                    "value" : "red",
                    "owned_by" : 1
                }
            ]
        },
        {
            "tag" : "p",
            "attributes" : [
                {
                    "name" : "background-color",
                    "value" : "green",
                    "owned_by" : 2
                },
                {
                    "name" : "font-size",
                    "value" : "11pt",
                    "owned_by" : 2
                }
            ]
        }
    ]
}
*/

// Need to save metadata for each file
// and overall project metadata

// need to use serde-json

/*
    html_files:
        last_saved
        last_updated

    css_files:
        [html_files that reference them, KVP]
        [styles]
            specific style : specific html file value it belongs to
        last_saved
        last_updated
*/

/*
Imaginary workspace.JSON file

{
    "css_files" : [
        {
            "id" : int,
            "file_name" : string,
            "absolute_path" : string,
            "usage_count" : int
        }
    ],

    "html_files" : [
        {
            "id" : int,
            "file_name" : string,
            "absolute_path": string,
            "last_updated" datetime
            "css_files_referenced" : [
                {
                    "id" : int
                },
            ]
        },
    ]
}

Imaginary concatenated_css.JSON file
{
    "file_name" : string,
    "absolute_path" : string,
    "last_updated" : datetime,
    "css_files" : [
        {
            "id" : int,
            "file_name" : string,
            "absolute_path" : string,
        },
    ],
    "styles" : [
        {
            "tag" : string
            "attributes" : [
                {
                    "attribute" : string,
                    "value" : string,
                    "owned_by" : int
                },
            ]
        },
    ]
}
*/
