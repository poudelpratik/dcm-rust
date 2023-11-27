use crate::modules::source_code_analyzer::traits::rust_item::RustItem;
use crate::modules::source_code_analyzer::types::RustItemCommonProperties;
use serde_derive::Serialize;
use syn::spanned::Spanned;
use syn::ItemConst;

/// This struct represents a Rust global variable in Rust syntax tree
#[derive(Debug, Serialize, Clone, Default)]
pub struct RustConst {
    pub properties: RustItemCommonProperties,
}

impl From<ItemConst> for RustConst {
    fn from(item_const: ItemConst) -> Self {
        let name = item_const.ident.to_string();
        let location = item_const.span().into();
        let properties = RustItemCommonProperties {
            name,
            position: location,
            ..Default::default()
        };
        Self { properties }
    }
}

impl RustItem for RustConst {
    fn get_common_properties(&self) -> RustItemCommonProperties {
        self.properties.clone()
    }

    fn get_common_properties_mut(&mut self) -> &mut RustItemCommonProperties {
        &mut self.properties
    }
}
