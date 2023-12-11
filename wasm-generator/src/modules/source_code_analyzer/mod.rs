use crate::modules::configuration::Configuration;
use crate::modules::constants::SOURCE_CODE_DIR;

use crate::modules::source_code_analyzer::cargo_toml::ProjectCargoToml;
use crate::modules::source_code_analyzer::rust_file::RustFile;
use crate::modules::util::{file_handler, is_rust_file};
use derive_new::new;
use itertools::Itertools;
use log::info;

use crate::modules::application::MobileFragments;
use crate::modules::language_server_protocol::traits::lsp_client::LspFilePath;
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use walkdir::WalkDir;

pub mod ast_visitor;
pub mod attribute_parser;
pub mod cargo_toml;
pub mod rust_file;

pub mod traits;
pub mod types;

pub fn run(
    mobile_fragments: &mut MobileFragments,
    config: Arc<Configuration>,
) -> (Vec<RustFile>, ProjectCargoToml) {
    let mut rust_files: Vec<RustFile> = Vec::new();

    let project_root = PathBuf::from(&config.project.clone());
    let source_code_dir = &project_root.join(SOURCE_CODE_DIR);

    let entries = WalkDir::new(source_code_dir)
        .into_iter()
        .filter_ok(is_rust_file)
        .filter_map(|e| e.ok());

    // Iterate over each Rust file and build custom syntax tree
    for entry in entries {
        info!("Analyzing file: {:?}", entry.path());
        let absolute_path = entry.path().to_path_buf();
        let relative_path = absolute_path
            .strip_prefix(&config.project)
            .unwrap()
            .to_path_buf();
        let file_path = FilePath {
            absolute: absolute_path.clone(),
            relative: relative_path.clone(),
        };
        let source_code = file_handler::read(&file_path.absolute).expect("Failed to read file");
        let syntax_tree = syn::parse_file(&source_code).expect("Failed to parse file");
        let mut ast_visitor =
            ast_visitor::AstVisitor::new(file_path, source_code, mobile_fragments);
        syn::visit::visit_file(&mut ast_visitor, &syntax_tree);
        rust_files.push(ast_visitor.rust_file);
    }

    // Parse Cargo.toml and derive CargoToml struct
    let cargo_toml: ProjectCargoToml = CargoPath::new(project_root.join("Cargo.toml")).into();

    (rust_files, cargo_toml)
    // export self.rust_files to rust_files.json for debugging purposes
    // let rust_files_json = serde_json::to_string_pretty(&self.rust_files).unwrap();
    // file_handler::writeln(&PathBuf::from("rust_files.json"), rust_files_json)
    //     .expect("Failed to write to rust_files.json");
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
pub struct FilePath {
    absolute: PathBuf,
    relative: PathBuf,
}

impl FilePath {
    pub fn get_absolute_path_as_string(&self) -> String {
        self.absolute.to_str().unwrap().to_string()
    }

    pub fn get_absolute_path(&self) -> PathBuf {
        self.absolute.clone()
    }
}

impl From<String> for FilePath {
    fn from(path: String) -> Self {
        Self {
            absolute: PathBuf::from(path.clone()),
            relative: PathBuf::new(),
        }
    }
}

impl From<LspFilePath> for FilePath {
    fn from(lsp_file_path: LspFilePath) -> Self {
        Self {
            absolute: lsp_file_path.get_as_string().replace("file://", "").into(),
            relative: PathBuf::new(),
        }
    }
}

#[derive(Debug, Deserialize, new)]
pub struct CargoPath {
    value: PathBuf,
}
