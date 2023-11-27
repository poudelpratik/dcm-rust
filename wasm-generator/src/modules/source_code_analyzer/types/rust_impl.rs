use crate::modules::source_code_analyzer::traits::rust_item::RustItem;
use crate::modules::source_code_analyzer::types::rust_function::RustFunction;
use crate::modules::source_code_analyzer::types::RustItemCommonProperties;
use serde_derive::Serialize;
use syn::spanned::Spanned;
use syn::{ItemImpl, Type};

/// This struct represents a Rust struct in Rust syntax tree
#[derive(Debug, Serialize, Clone, Default)]
pub struct RustImpl {
    pub properties: RustItemCommonProperties,
    pub functions: Vec<RustFunction>,
}

impl From<ItemImpl> for RustImpl {
    fn from(item_impl: ItemImpl) -> Self {
        let struct_name = match &*item_impl.self_ty {
            Type::Path(tp) => tp.path.segments.first().unwrap().ident.to_string(),
            _ => panic!("Failed to get name of impl"),
        };
        let location = item_impl.span().into();
        let functions = item_impl
            .items
            .iter()
            .filter_map(|item| match item {
                syn::ImplItem::Fn(item_fn) => {
                    let mut function: RustFunction = item_fn.clone().into();
                    function.struct_name = Some(struct_name.clone());
                    Some(function)
                }
                _ => None,
            })
            .collect::<Vec<RustFunction>>();

        let properties = RustItemCommonProperties {
            name: struct_name.clone(),
            position: location,
            ..Default::default()
        };

        Self {
            properties,
            functions,
        }
    }
}

impl RustItem for RustImpl {
    fn get_common_properties(&self) -> RustItemCommonProperties {
        self.properties.clone()
    }

    fn get_common_properties_mut(&mut self) -> &mut RustItemCommonProperties {
        &mut self.properties
    }
    // fn get_code(&self) -> String {
    //     self.properties.code.clone()
    // }
    //
    // fn set_item_code_from_source(&mut self, source_code: &str) {
    //     self.properties.code = self.properties.location.get_code_segment(source_code);
    //     self.functions.iter_mut().for_each(|f| {
    //         f.set_item_code_from_source(source_code);
    //     });
    // }
    //
    // fn set_module_hierarchy(&mut self, module_hierarchy: Vec<String>) {
    //     self.properties.module_hierarchy = module_hierarchy;
    //     self.functions.iter_mut().for_each(|f| {
    //         f.set_module_hierarchy(self.properties.module_hierarchy.clone());
    //     });
    // }
}
