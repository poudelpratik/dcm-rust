use crate::modules::error::ApplicationError;
use serde::de::DeserializeOwned;
use std::path::PathBuf;
use syn::File;
use walkdir::DirEntry;

pub mod file_handler;
pub mod id_generator;
pub mod thread_manager;

pub fn parse_cfd<T: DeserializeOwned>(file_path: PathBuf) -> Result<T, ApplicationError> {
    let file_content = file_handler::read(&file_path)?;
    Ok(serde_yaml::from_str(&file_content)?)
}

/// This function is used to get Rust syntax tree from source code using the syn crate
pub fn get_rust_syntax_tree(source_code: &str) -> Result<File, ApplicationError> {
    Ok(syn::parse_file(source_code)?)
}

/// This function is used to check if a DirEntry is a Rust file
pub fn is_rust_file(entry: &DirEntry) -> bool {
    entry
        .path()
        .extension()
        .map(|ext| ext == "rs")
        .unwrap_or(false)
}

/// This function is used to get character position from line and column numbers
pub fn line_col_to_char_pos(source: &str, line: usize, column: usize) -> usize {
    source
        .lines()
        .take(line - 1) // -1 because line numbers start at 1
        .map(|l| l.len() + 1) // +1 for the newline character
        .sum::<usize>()
        + column
}

use regex::Regex;

pub fn get_js_type_from_rust_type(rust_type: &str) -> String {
    let re_collection = Regex::new(r"(Vec|HashSet|BTreeSet)\s*<\s*([^>]+)\s*>").unwrap();
    let re_option = Regex::new(r"Option\s*<\s*([^>]+)\s*>").unwrap();

    match rust_type {
        "()" => "void".to_string(),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" | "f32" | "f64" => "Number".to_string(),
        "bool" => "Boolean".to_string(),
        "String" | "&str" => "String".to_string(),
        parameter if re_collection.is_match(parameter) => {
            let captures = re_collection.captures(parameter).unwrap();
            let inner_type = captures.get(2).unwrap().as_str();
            format!("Array<{}>", get_js_type_from_rust_type(inner_type))
        }
        parameter if re_option.is_match(parameter) => {
            let captures = re_option.captures(parameter).unwrap();
            let inner_type = captures.get(1).unwrap().as_str();
            format!("{} | null", get_js_type_from_rust_type(inner_type))
        }
        _ => "Object".to_string(),
    }
}

pub fn get_default_value_for_js_type(js_type: &str) -> String {
    if js_type.starts_with("Array<") {
        "[]".to_string() // Default for any Array<T> type
    } else if js_type.ends_with(" | null") {
        "null".to_string() // Default for optional types T | null
    } else {
        match js_type {
            "Number" => "0".to_string(),      // Default number
            "Boolean" => "false".to_string(), // Default boolean
            "String" => "\"\"".to_string(),   // Default string (empty string)
            "Object" => "{}".to_string(),     // Default object (empty object)
            _ => "null".to_string(),          // Default for unrecognized types
        }
    }
}

pub fn is_primitive(type_name: &str) -> bool {
    matches!(
        type_name,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "isize"
            | "usize"
            | "f32"
            | "f64"
            | "char"
            | "bool"
            | "str"
            | "String" // You can add more primitive types here as needed
    )
}
