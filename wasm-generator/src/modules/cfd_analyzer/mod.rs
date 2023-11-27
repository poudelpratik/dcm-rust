pub mod cfd_visitor;
pub mod traits;

use crate::modules::application::fragment_type::RustItemType;
use crate::modules::application::MobileFragments;
use crate::modules::configuration::Configuration;
use crate::modules::source_code_analyzer::rust_file::RustFile;
use crate::modules::util;
use log::error;
use serde_derive::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CfdAnalyzer {
    pub cfd: CodeFragmentDescriptionContainer,
    pub is_cfd_available: bool,
    pub rust_files: Arc<Vec<RustFile>>,
}

pub fn run(
    config: Arc<Configuration>,
    rust_files: Arc<Vec<RustFile>>,
    mobile_fragments: &mut MobileFragments,
) {
    let cfd_yaml_path = PathBuf::from(&config.host_project).join("CFD.yaml");
    let cfd_yml_path = PathBuf::from(&config.host_project).join("CFD.yml");
    let cfd_container = match cfd_yaml_path.exists() || cfd_yml_path.exists() {
        true => {
            match util::parse_cfd::<CodeFragmentDescriptionContainer>(cfd_yaml_path)
                .or_else(|_| util::parse_cfd::<CodeFragmentDescriptionContainer>(cfd_yml_path))
            {
                Ok(cfd) => Some(cfd),
                Err(_) => None,
            }
        }
        false => None,
    };

    if cfd_container.is_none() {
        return;
    }

    let cfd_container = cfd_container.unwrap();
    let mut cfd_visitor = cfd_visitor::CfdVisitor::new(mobile_fragments, rust_files.clone());

    cfd_visitor::visit_cfd(&mut cfd_visitor, &cfd_container.fragments);

    // If there are any errors, log them and panic.
    if !cfd_visitor.cfd_errors.is_empty() {
        error!("CFD processing failed with the following errors:");
        for error in cfd_visitor.cfd_errors {
            error!("{}", error);
        }
        panic!("CFD processing failed.");
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CodeFragmentDescriptionContainer {
    pub fragments: Vec<CodeFragmentDescription>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CodeFragmentDescription {
    pub id: Option<String>,
    pub name: String,
    pub initial_execution_location: String,
    pub location: CodeFragmentLocation,
    pub crates: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
    pub item_type: Option<RustItemType>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CodeFragmentLocation {
    pub filepath: PathBuf,
    pub module: Option<String>,
}
