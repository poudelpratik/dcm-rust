use crate::modules::source_code_analyzer::attribute_parser::MobileAttributeKeys;
use quote::ToTokens;
use std::error::Error;
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, Lit, Meta, Token};

pub fn get_attribute_value(attr: &Attribute, attr_key: MobileAttributeKeys) -> Option<String> {
    if !attr.to_token_stream().to_string().contains("@mobile") {
        return None;
    }

    let result: Result<_, Box<dyn Error>> = (|| {
        let list = attr.meta.require_list()?;
        let nested_meta = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        let attr_key_str = attr_key.to_string();

        for meta in nested_meta {
            match meta {
                Meta::Path(_) | Meta::List(_) => {} // probably not useful to us
                Meta::NameValue(meta) => {
                    if meta.path.is_ident(&attr_key_str) {
                        return match &meta.value {
                            Expr::Lit(c) => match &c.lit {
                                // Currently we only support string and integer literals in the attribute, add more if needed
                                Lit::Str(d) => Ok(Some(d.value())),
                                Lit::Int(d) => Ok(Some(d.base10_digits().to_string())),
                                _ => Ok(None),
                            },
                            _ => Ok(None),
                        };
                    }
                }
            }
        }
        Ok(None)
    })();

    match result {
        Ok(value) => value,
        Err(_) => None,
    }
}

pub fn get_mobile_attribute_if_exists(attrs: Vec<Attribute>) -> Option<Attribute> {
    attrs
        .into_iter()
        .find(|attr| attr.path().is_ident("mobile"))
}

mod test {

    #[test]
    fn test_get_attribute_value() {
        let attr: Attribute = parse_quote! {
            #[mobile(id = 1, libs = "lib1, lib2", run_on = "server")]
        };
        let value = get_attribute_value(&attr, MobileAttributeKeys::Id);
        assert_eq!(value, Some("1".to_string()));
        let value = get_attribute_value(&attr, MobileAttributeKeys::Libs);
        assert_eq!(value, Some("lib1, lib2".to_string()));
        let value = get_attribute_value(&attr, MobileAttributeKeys::ExecuteOn);
        assert_eq!(value, Some("server".to_string()));
    }

    #[test]
    fn test_get_attribute_value_no_attr() {
        let attr: Attribute = parse_quote! {
            #[serde(rename = "id")]
        };
        let value = get_attribute_value(&attr, MobileAttributeKeys::Id);
        assert_eq!(value, None);
    }

    #[test]
    fn test_get_attribute_value_no_value() {
        let attr: Attribute = parse_quote! {
            #[mobile(id)]
        };
        let value = get_attribute_value(&attr, MobileAttributeKeys::Id);
        assert_eq!(value, None);
    }

    #[test]
    fn test_get_mobile_attribute_if_exists() {
        let attr: Attribute = parse_quote! {
            #[mobile(id = 1, libs = "lib1, lib2", run_on = "server")]
        };
        let attrs = vec![attr];
        let value = get_mobile_attribute_if_exists(attrs);
        assert_eq!(value.is_some(), true);
    }
}
