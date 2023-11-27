use crate::modules::source_code_analyzer::traits::rust_item::RustItem;
use crate::modules::source_code_analyzer::types::RustItemCommonProperties;
use crate::modules::util;
use quote::ToTokens;
use serde_derive::Serialize;
use syn::{spanned::Spanned, FnArg, ImplItemFn, ItemFn, Pat, Signature, Type, Visibility};

/// This struct represents a Rust function in Rust syntax tree
#[derive(Debug, Serialize, Clone, Default)]
pub struct RustFunction {
    pub properties: RustItemCommonProperties,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: ParameterInfo,
    pub is_public: bool,
    pub struct_name: Option<String>,
    pub function_type: RustFunctionType,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct ParameterInfo {
    pub name: String,
    pub rust_type: String,
    pub js_type: String,
    pub is_reference: bool,
    pub is_mutable: bool,
}

#[derive(Debug, Serialize, Clone, Default, PartialEq)]
pub enum RustFunctionType {
    #[default]
    FreeFunction,
    Method,
    AssociatedFunction,
}

fn get_function_parameters(signature: &Signature, is_method: &mut bool) -> Vec<ParameterInfo> {
    let mut params = Vec::new();
    let mut index = 0;

    for param in signature.inputs.iter() {
        let name = match param {
            FnArg::Receiver(_) => {
                *is_method = true;
                continue;
            }
            FnArg::Typed(typed) => match &*typed.pat {
                Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                _ => {
                    let placeholder_name = format!("param{}", index);
                    index += 1;
                    placeholder_name
                }
            },
        };

        if let FnArg::Typed(typed) = param {
            let rust_type = typed
                .ty
                .clone()
                .into_token_stream()
                .to_string()
                .replace(' ', "");
            let json_type = util::get_js_type_from_rust_type(&rust_type);
            params.push(ParameterInfo {
                name,
                rust_type,
                js_type: json_type,
                is_reference: matches!(&*typed.ty, Type::Reference(_)),
                is_mutable: matches!(&*typed.ty, Type::Reference(type_reference) if type_reference.mutability.is_some()),
            });
        }
    }

    params
}

fn get_return_type(signature: &Signature) -> ParameterInfo {
    let (rust_type, is_reference, is_mutable) = match &signature.output {
        syn::ReturnType::Default => ("()".to_string(), false, false),
        syn::ReturnType::Type(_, ty) => match &**ty {
            Type::Reference(type_reference) => {
                let rust_type = type_reference
                    .clone()
                    .elem
                    .into_token_stream()
                    .to_string()
                    .replace(' ', "");
                (rust_type, true, type_reference.mutability.is_some())
            }
            _ => (
                ty.into_token_stream().to_string().replace(' ', ""),
                false,
                false,
            ),
        },
    };

    let json_return_type = util::get_js_type_from_rust_type(&rust_type);

    ParameterInfo {
        name: "return".to_string(),
        rust_type,
        js_type: json_return_type,
        is_reference,
        is_mutable,
    }
}

impl From<ItemFn> for RustFunction {
    fn from(item_fn: ItemFn) -> Self {
        let name = item_fn.sig.ident.to_string();
        let location = item_fn.span().into();
        let mut is_method = false;
        let parameters = get_function_parameters(&item_fn.sig, &mut is_method);
        let is_public = matches!(item_fn.vis, Visibility::Public(_));
        let return_type = get_return_type(&item_fn.sig);

        let properties = RustItemCommonProperties {
            name,
            position: location,
            ..Default::default()
        };

        // Return the struct instance
        Self {
            properties,
            is_public,
            parameters,
            return_type,
            struct_name: None,
            function_type: RustFunctionType::FreeFunction,
        }
    }
}

impl From<ImplItemFn> for RustFunction {
    fn from(item_fn: ImplItemFn) -> Self {
        let name = item_fn.sig.ident.to_string();
        let location = item_fn.span().into();
        let mut is_method = false;
        let parameters = get_function_parameters(&item_fn.sig, &mut is_method);
        let return_type = get_return_type(&item_fn.sig);
        let is_public = matches!(item_fn.vis, Visibility::Public(_));

        let properties = RustItemCommonProperties {
            name,
            position: location,
            ..Default::default()
        };

        // Return the struct instance
        Self {
            properties,
            is_public,
            parameters,
            return_type,
            function_type: if is_method {
                RustFunctionType::Method
            } else {
                RustFunctionType::AssociatedFunction
            },
            ..Default::default()
        }
    }
}

impl RustItem for RustFunction {
    fn get_common_properties(&self) -> RustItemCommonProperties {
        self.properties.clone()
    }

    fn get_common_properties_mut(&mut self) -> &mut RustItemCommonProperties {
        &mut self.properties
    }
}
