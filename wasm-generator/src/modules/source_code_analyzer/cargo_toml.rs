use crate::modules::source_code_analyzer::CargoPath;
use crate::modules::util;
use derive_new::new;
use serde_derive::{Deserialize, Serialize};
use toml::Table;

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct ProjectCargoToml {
    pub package: CargoPackageInformation,
    pub dependencies: Option<Table>,
    pub lib: Option<Table>,
}

impl From<CargoPath> for ProjectCargoToml {
    fn from(cargo_path: CargoPath) -> Self {
        let cargo_content = util::file_handler::read(&cargo_path.value).unwrap();
        toml::from_str(&cargo_content).unwrap()
    }
}

#[derive(Debug, Deserialize, Default, Serialize, new, Clone)]
pub struct CargoPackageInformation {
    pub name: String,
    pub version: String,
    pub authors: Option<Vec<String>>,
    pub edition: Option<String>,
}
