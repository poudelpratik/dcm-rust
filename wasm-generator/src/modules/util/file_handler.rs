use crate::modules::error::ApplicationError;
use itertools::Itertools;
use serde_derive::Serialize;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// This function creates a file at the given path.
pub fn create_file_or_directory_recursive(file_path: &PathBuf) -> Result<(), ApplicationError> {
    fs::create_dir_all(file_path)?;
    Ok(())
}

pub fn read(file_path: &PathBuf) -> Result<String, ApplicationError> {
    Ok(fs::read_to_string(file_path)?)
}

pub fn writeln(file_path: &PathBuf, content: String) -> Result<(), ApplicationError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)?;
    writeln!(file, "{}", content)?;
    Ok(())
}

pub fn list_files_in_directory_with_extension_filter(
    dir_path: &PathBuf,
    extension_filter: &str,
) -> Result<Vec<PathBuf>, ApplicationError> {
    let read_dir = fs::read_dir(dir_path)?;
    let file_paths = read_dir
        .filter_map(|entry| {
            let entry = entry.expect("Failed to read directory entry");
            let file_path = entry.path();
            if file_path.is_file()
                && file_path
                    .extension()
                    .map(|ext| ext == extension_filter)
                    .unwrap_or(false)
            {
                Some(file_path)
            } else {
                None
            }
        })
        .collect_vec();
    Ok(file_paths)
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DirectoryContext {
    pub base_path: PathBuf,
}

impl DirectoryContext {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_path_buf();
        create_file_or_directory_recursive(&path).expect("Failed to create directory");
        Self { base_path: path }
    }

    pub fn create_file<P: AsRef<Path>>(&self, relative_path: P) -> io::Result<File> {
        let full_path = self.base_path.join(relative_path);

        // Ensure parent directory exists
        if let Some(parent_dir) = full_path.parent() {
            fs::create_dir_all(parent_dir)?;
        }

        File::create(full_path)
    }
}

pub fn write_to_file<C: AsRef<[u8]>>(mut file: File, content: C) -> io::Result<()> {
    file.write_all(content.as_ref())?;
    Ok(())
}

// This function deletes directory recursively
pub fn delete_directory(dir_path: &PathBuf) -> Result<(), ApplicationError> {
    fs::remove_dir_all(dir_path).ok();
    Ok(())
}

pub fn copy_file(source: &PathBuf, destination: &PathBuf) -> Result<(), ApplicationError> {
    // Create parent directories if they don't exist
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, destination)?;
    Ok(())
}
