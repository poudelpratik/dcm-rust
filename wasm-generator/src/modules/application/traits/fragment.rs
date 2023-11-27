use crate::modules::application::fragment_type::RustItemType;
use crate::modules::source_code_analyzer::cargo_toml::ProjectCargoToml;
use crate::modules::source_code_analyzer::types::rust_struct::RustStruct;
use crate::modules::source_code_analyzer::types::RustItemCommonProperties;

pub trait Fragment {
    fn get_common_properties(&self) -> RustItemCommonProperties;
    fn get_id(&self) -> String;
    fn set_id(&mut self, id: String);

    fn get_name(&self) -> String;

    fn get_code(&self) -> String;

    fn set_code(&mut self, code: String);

    fn get_type(&self) -> RustItemType;

    fn get_module_hierarchy(&self) -> Vec<String>;

    fn set_struct_for_impl(&mut self, rust_struct: RustStruct);

    fn get_dependency_ids(&self) -> Vec<String>;

    fn set_dependency_ids(&mut self, dependency_ids: Vec<String>);
    fn get_crates(&self) -> Vec<String>;

    fn set_crates(&mut self, crates: Vec<String>);

    fn get_package_name(&self) -> String;

    fn get_wasm_identifier(&self) -> String;

    fn set_cargo_toml(&mut self, cargo_toml: ProjectCargoToml);

    fn get_cargo_toml(&self) -> ProjectCargoToml;
}
