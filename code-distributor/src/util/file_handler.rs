use crate::util::error::ApplicationError;
use std::fs;
use std::path::PathBuf;

pub fn read(file_path: &PathBuf) -> Result<String, ApplicationError> {
    Ok(fs::read_to_string(file_path)?)
}

#[derive(Debug, Clone)]
pub struct DirectoryContext {
    pub base_path: PathBuf,
}
