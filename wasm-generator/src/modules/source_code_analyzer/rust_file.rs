use crate::modules::source_code_analyzer::types::rust_const::RustConst;
use crate::modules::source_code_analyzer::types::rust_function::RustFunction;
use crate::modules::source_code_analyzer::types::rust_impl::RustImpl;
use crate::modules::source_code_analyzer::types::rust_static::RustStatic;
use crate::modules::source_code_analyzer::types::rust_struct::RustStruct;
use crate::modules::source_code_analyzer::types::rust_type_definition::RustTypeDefinition;
use crate::modules::source_code_analyzer::types::rust_use::RustUse;
use crate::modules::source_code_analyzer::FilePath;
use serde_derive::Serialize;
use std::path::PathBuf;

/// This struct represents a Rust file in Rust syntax tree
#[derive(Debug, Default, Serialize, Clone)]
pub struct RustFile {
    pub absolute_filepath: PathBuf,
    pub relative_filepath: PathBuf,
    pub uses: Vec<RustUse>,
    pub functions: Vec<RustFunction>,
    pub consts: Vec<RustConst>,
    pub statics: Vec<RustStatic>,
    pub type_definitions: Vec<RustTypeDefinition>,
    pub structs: Vec<RustStruct>,
    pub impls: Vec<RustImpl>,

    // This property is here to keep track of the index of this Rust file in the vector.
    // So later, while resolving dependencies, we do not have to search the file by matching file_path again.
    pub index_in_vector: usize,
}

impl From<FilePath> for RustFile {
    fn from(file_path: FilePath) -> Self {
        Self {
            absolute_filepath: file_path.absolute,
            relative_filepath: file_path.relative,
            ..Default::default()
        }
    }
}
