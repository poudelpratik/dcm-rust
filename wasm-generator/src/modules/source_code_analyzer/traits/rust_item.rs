use crate::modules::application::fragment_type::RustItemType;
use crate::modules::source_code_analyzer::types::RustItemCommonProperties;
use crate::modules::source_code_analyzer::types::RustItemPosition;
use crate::modules::source_code_analyzer::FilePath;

pub trait RustItem {
    fn get_common_properties(&self) -> RustItemCommonProperties;
    fn get_common_properties_mut(&mut self) -> &mut RustItemCommonProperties;
}

pub trait RustItemCommon {
    fn get_name(&self) -> String;

    fn get_location(&self) -> RustItemPosition;

    fn get_module_hierarchy(&self) -> Vec<String>;

    fn get_file_path(&self) -> FilePath;

    fn get_code(&self) -> String;

    fn get_item_type(&self) -> RustItemType;
}
