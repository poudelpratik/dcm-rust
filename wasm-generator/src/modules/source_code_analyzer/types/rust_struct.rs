use crate::modules::source_code_analyzer::traits::rust_item::RustItem;
use crate::modules::source_code_analyzer::types::RustItemCommonProperties;
use crate::modules::util;
use quote::ToTokens;
use serde_derive::Serialize;
use syn::spanned::Spanned;
use syn::ItemStruct;

/// This struct represents a Rust struct in Rust syntax tree
#[derive(Debug, Serialize, Clone, Default)]
pub struct RustStruct {
    pub properties: RustItemCommonProperties,
    pub fields: Vec<RustStructField>,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct RustStructField {
    pub name: String,
    pub rust_type: String,
    pub js_type: String,
}

impl From<ItemStruct> for RustStruct {
    fn from(item_struct: ItemStruct) -> Self {
        let struct_name = item_struct.ident.to_string();
        let location = item_struct.span().into();

        let mut fields = Vec::new();

        for field in item_struct.fields.iter() {
            if let Some(ident) = &field.ident {
                let name = ident.to_string();
                let rust_type = field.ty.to_token_stream().to_string().replace(' ', "");
                let js_type = util::get_js_type_from_rust_type(&rust_type);
                fields.push(RustStructField {
                    name,
                    rust_type,
                    js_type,
                });
            }
        }

        let properties = RustItemCommonProperties {
            name: struct_name.clone(),
            position: location,
            ..Default::default()
        };

        Self { properties, fields }
    }
}

impl RustItem for RustStruct {
    fn get_common_properties(&self) -> RustItemCommonProperties {
        self.properties.clone()
    }

    fn get_common_properties_mut(&mut self) -> &mut RustItemCommonProperties {
        &mut self.properties
    }
}
