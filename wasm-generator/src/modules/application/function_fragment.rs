use crate::modules::application::fragment_type::RustItemType;
use crate::modules::application::traits::fragment::Fragment;
use crate::modules::cfd_analyzer::CodeFragmentDescription;
use crate::modules::source_code_analyzer::attribute_parser::AttributeParser;
use crate::modules::source_code_analyzer::cargo_toml::ProjectCargoToml;
use crate::modules::source_code_analyzer::types::rust_function::RustFunction;
use crate::modules::source_code_analyzer::types::rust_struct::RustStruct;
use crate::modules::source_code_analyzer::types::RustItemCommonProperties;
use derive_new::new;
use serde_derive::{Deserialize, Serialize};
use syn::Attribute;

#[derive(Debug, Clone, Serialize)]
pub struct FunctionFragment {
    pub id: String,
    pub cargo_toml_content: ProjectCargoToml,
    pub initial_execution_location: ExecutionLocation,
    pub crates: Vec<String>,
    pub dependencies: Vec<String>,
    pub fragment_type: RustItemType,
    pub rust_function: RustFunction,
    pub module_hierarchy: Vec<String>,
}

impl FunctionFragment {
    pub fn try_create_from_attributes(
        attrs: Vec<Attribute>,
        rust_function: RustFunction,
        module_hierarchy: Vec<String>,
    ) -> Option<Self> {
        let attribute_parser = AttributeParser::new(attrs);
        match attribute_parser.mobile_annotation_exists() {
            false => None,
            true => {
                let _self = Self {
                    id: attribute_parser.id.unwrap_or_default(),
                    cargo_toml_content: ProjectCargoToml::default(),
                    initial_execution_location: attribute_parser
                        .initial_execution_location
                        .unwrap_or(ExecutionLocation::Client),
                    crates: attribute_parser.crates.unwrap_or_default(),
                    dependencies: attribute_parser.dependencies.clone().unwrap_or_default(),
                    module_hierarchy,
                    fragment_type: RustItemType::Function,
                    rust_function: rust_function,
                };
                Some(_self)
            }
        }
    }

    pub fn create_from_cfd(
        rust_function: RustFunction,
        cfd: &CodeFragmentDescription,
        module_hierarchy: Vec<String>,
    ) -> Self {
        let execute_on = match cfd.initial_execution_location.as_str() {
            "server" | "Server" => ExecutionLocation::Server,
            _ => ExecutionLocation::Client,
        };

        Self {
            id: cfd.id.clone().unwrap_or_default(),
            cargo_toml_content: ProjectCargoToml::default(),
            initial_execution_location: execute_on,
            crates: cfd.crates.clone().unwrap_or_default(),
            dependencies: cfd
                .dependencies
                .clone()
                .unwrap_or_default()
                .into_iter()
                .collect(),
            rust_function: rust_function,
            module_hierarchy,
            fragment_type: RustItemType::Function,
        }
    }
}

#[derive(Debug, new, Clone, Default, Deserialize, PartialEq, Serialize)]
pub enum ExecutionLocation {
    #[default]
    Client,
    Server,
}

impl Fragment for FunctionFragment {
    fn get_common_properties(&self) -> RustItemCommonProperties {
        self.rust_function.properties.clone()
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    fn get_name(&self) -> String {
        self.rust_function.properties.name.clone()
    }

    fn get_code(&self) -> String {
        self.rust_function.properties.code.clone()
    }

    fn set_code(&mut self, code: String) {
        self.rust_function.properties.code = code;
    }

    fn get_type(&self) -> RustItemType {
        self.fragment_type.clone()
    }

    fn get_module_hierarchy(&self) -> Vec<String> {
        self.module_hierarchy.clone()
    }
    fn set_struct_for_impl(&mut self, _: RustStruct) {
        // do nothing
    }
    fn get_dependency_ids(&self) -> Vec<String> {
        self.dependencies.clone()
    }

    fn set_dependency_ids(&mut self, dependency_ids: Vec<String>) {
        self.dependencies = dependency_ids;
    }

    fn get_crates(&self) -> Vec<String> {
        self.crates.clone()
    }

    fn set_crates(&mut self, crates: Vec<String>) {
        self.crates = crates;
    }

    fn get_package_name(&self) -> String {
        self.id.clone()
    }

    fn get_wasm_identifier(&self) -> String {
        format!("{}.wasm", &self.id)
    }

    fn set_cargo_toml(&mut self, cargo_toml: ProjectCargoToml) {
        self.cargo_toml_content = cargo_toml;
    }

    fn get_cargo_toml(&self) -> ProjectCargoToml {
        self.cargo_toml_content.clone()
    }
}
