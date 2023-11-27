use crate::modules::application::fragment_type::RustItemType;
use crate::modules::source_code_analyzer::traits::rust_item::RustItemCommon;
use crate::modules::source_code_analyzer::FilePath;
use crate::modules::util;
use proc_macro2::Span;
use serde_derive::{Deserialize, Serialize};

pub mod rust_function;

pub mod rust_impl;

pub mod rust_struct;

pub mod rust_type_definition;

pub mod rust_use;

pub mod rust_static;

pub mod rust_const;

#[derive(Debug, Serialize, Clone, Default, PartialEq)]
pub struct RustItemCommonProperties {
    pub name: String,
    pub code: String,
    pub position: RustItemPosition,
    pub module_hierarchy: Vec<String>,
    pub file_path: FilePath,
    pub item_type: RustItemType,
}

impl RustItemCommonProperties {
    pub fn new(
        name: String,
        code: String,
        location: RustItemPosition,
        module_hierarchy: Vec<String>,
        file_path: FilePath,
        item_type: RustItemType,
    ) -> Self {
        Self {
            name,
            code,
            position: location,
            module_hierarchy,
            file_path,
            item_type,
        }
    }

    pub fn set_item_code_from_source(&mut self, source_code: &str) {
        self.code = self.position.get_code_segment(source_code);
    }

    pub fn set_module_hierarchy(&mut self, module_hierarchy: Vec<String>) {
        self.module_hierarchy = module_hierarchy;
    }

    pub fn set_file_path(&mut self, file_path: FilePath) {
        self.file_path = file_path;
    }
}

impl RustItemCommon for RustItemCommonProperties {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_location(&self) -> RustItemPosition {
        self.position.clone()
    }

    fn get_module_hierarchy(&self) -> Vec<String> {
        self.module_hierarchy.clone()
    }

    fn get_file_path(&self) -> FilePath {
        self.file_path.clone()
    }

    fn get_code(&self) -> String {
        self.code.clone()
    }

    fn get_item_type(&self) -> RustItemType {
        self.item_type.clone()
    }
}

/// This struct represents start and end position of a node in the source code.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct RustItemPosition {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl RustItemPosition {
    pub fn get_code_segment(&self, source_code: &str) -> String {
        let start_byte =
            util::line_col_to_char_pos(source_code, self.start_line, self.start_column);
        let end_byte = util::line_col_to_char_pos(source_code, self.end_line, self.end_column);
        source_code[start_byte..end_byte].to_string()
    }
}

impl From<Span> for RustItemPosition {
    fn from(span: Span) -> Self {
        Self {
            start_line: span.start().line,
            start_column: span.start().column,
            end_line: span.end().line,
            end_column: span.end().column,
        }
    }
}
