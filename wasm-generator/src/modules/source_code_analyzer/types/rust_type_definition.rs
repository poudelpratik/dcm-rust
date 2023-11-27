use crate::modules::source_code_analyzer::traits::rust_item::RustItem;
use crate::modules::source_code_analyzer::types::RustItemCommonProperties;
use serde_derive::Serialize;
use syn::spanned::Spanned;
use syn::ItemType;

#[derive(Debug, Serialize, Clone, Default)]
pub struct RustTypeDefinition {
    pub properties: RustItemCommonProperties,
}

impl From<ItemType> for RustTypeDefinition {
    fn from(item_type: ItemType) -> Self {
        let location = item_type.span().into();
        let properties = RustItemCommonProperties {
            name: item_type.ident.to_string(),
            position: location,
            ..Default::default()
        };
        Self { properties }
    }
}

impl RustItem for RustTypeDefinition {
    fn get_common_properties(&self) -> RustItemCommonProperties {
        self.properties.clone()
    }

    fn get_common_properties_mut(&mut self) -> &mut RustItemCommonProperties {
        &mut self.properties
    }
}
