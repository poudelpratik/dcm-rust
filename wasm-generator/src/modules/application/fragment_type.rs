use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize, Clone, Default, Serialize, PartialEq)]
pub enum RustItemType {
    #[serde(rename = "function")]
    #[default]
    Function,
    #[serde(rename = "struct")]
    Struct,
    #[serde(rename = "impl")]
    Impl,
    #[serde(rename = "type")]
    Type,
    #[serde(rename = "static")]
    Static,
    #[serde(rename = "const")]
    Const,
    #[serde(rename = "use")]
    Use,
}

impl fmt::Display for RustItemType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RustItemType::Function => write!(f, "function"),
            RustItemType::Struct => write!(f, "struct"),
            RustItemType::Impl => write!(f, "impl"),
            RustItemType::Type => write!(f, "type"),
            RustItemType::Static => write!(f, "static"),
            RustItemType::Const => write!(f, "const"),
            RustItemType::Use => write!(f, "use"),
        }
    }
}
