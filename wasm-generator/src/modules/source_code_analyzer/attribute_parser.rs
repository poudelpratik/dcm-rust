use quote::ToTokens;
use regex::Regex;
use std::error::Error;
use syn::{Attribute, Expr, Lit};

mod annotation_constants {
    pub const MAIN_ATTRIBUTE_NAME: &str = "doc";
    pub const MOBILE_ANNOTATION: &str = "@mobile";

    // This is for id with quotes
    pub const ID_PATTERN: &str = r#"id\s*=\s*"([^"]+)"#;

    // This is for id without quotes
    // pub const ID_PATTERN: &str = r#"id\s*=\s*(\d+)"#;
    pub const DEPENDENCIES_PATTERN: &str = r"dependencies\s*=\s*\[(.*?)\]";
    pub const CRATES_PATTERN: &str = r"crates\s*=\s*\[(.*?)\]";
    pub const INITIAL_EXECUTION_LOCATION_PATTERN: &str =
        r#"initial_execution_location\s*=\s*"([^"]+)"#;
}

use crate::modules::application::function_fragment::ExecutionLocation;
pub use annotation_constants::*;

#[derive(Clone, Default, Debug)]
pub struct AttributeParser {
    pub mobile_annotation_exists: bool,
    pub id: Option<String>,
    pub crates: Option<Vec<String>>,
    pub initial_execution_location: Option<ExecutionLocation>,
    pub dependencies: Option<Vec<String>>,
}

impl AttributeParser {
    pub fn new(attributes_list: Vec<Attribute>) -> Self {
        match Self::get_mobile_annotation_content_if_exists(attributes_list) {
            None => Self::default(),
            Some(mobile_annotation_content) => Self {
                mobile_annotation_exists: true,
                id: Self::get_id_from_annotation(&mobile_annotation_content),
                crates: Self::get_crates_from_annotation(&mobile_annotation_content),
                initial_execution_location: Self::get_initial_execution_location_from_annotation(
                    &mobile_annotation_content,
                ),
                dependencies: Self::get_dependencies_from_annotation(&mobile_annotation_content),
            },
        }
    }

    pub fn mobile_annotation_exists(&self) -> bool {
        self.mobile_annotation_exists
    }

    fn get_dependencies_from_annotation(mobile_annotation_content: &str) -> Option<Vec<String>> {
        Self::extract_value_from_mobile_annotation(mobile_annotation_content, DEPENDENCIES_PATTERN)
            .map(|dependencies| {
                dependencies
                    .split(',')
                    .map(|lib| lib.trim().to_string())
                    .collect()
            })
    }

    fn get_initial_execution_location_from_annotation(
        mobile_annotation_content: &str,
    ) -> Option<ExecutionLocation> {
        match Self::extract_value_from_mobile_annotation(
            mobile_annotation_content,
            INITIAL_EXECUTION_LOCATION_PATTERN,
        ) {
            None => None,
            Some(initial_execution_location) => {
                if initial_execution_location == "server" || initial_execution_location == "Server"
                {
                    Some(ExecutionLocation::Server)
                } else {
                    Some(ExecutionLocation::Client)
                }
            }
        }
    }

    fn get_crates_from_annotation(mobile_annotation_content: &str) -> Option<Vec<String>> {
        Self::extract_value_from_mobile_annotation(mobile_annotation_content, CRATES_PATTERN).map(
            |crates| {
                crates
                    .split(',')
                    .map(|lib| lib.trim().to_string())
                    .collect()
            },
        )
    }

    fn get_id_from_annotation(mobile_annotation_content: &str) -> Option<String> {
        Self::extract_value_from_mobile_annotation(mobile_annotation_content, ID_PATTERN)
    }

    /// This function extracts the value from the mobile annotation by matching the given regex pattern.
    pub fn extract_value_from_mobile_annotation(input: &str, pattern: &str) -> Option<String> {
        let re = Regex::new(pattern).unwrap();
        re.captures(input).and_then(|cap| cap.get(1)).map(|m| {
            m.as_str()
                .split(',')
                .map(|s| s.trim_matches('"').trim().replace('\"', ""))
                .collect::<Vec<_>>()
                .join(", ")
        })
    }

    /// This function checks if the parameter attribute contains @mobile annotation.
    /// If so, it returns the content of the @mobile annotation as String, otherwise returns None.
    fn get_mobile_annotation_content(attr: &Attribute) -> Option<String> {
        if !attr
            .to_token_stream()
            .to_string()
            .contains(MAIN_ATTRIBUTE_NAME)
        {
            return None;
        }
        let result: Result<_, Box<dyn Error>> = (|| {
            let meta = attr.meta.require_name_value()?.clone();
            match &meta.value {
                Expr::Lit(c) => match &c.lit {
                    Lit::Str(d) => {
                        if d.value().contains(MOBILE_ANNOTATION) {
                            Ok(Some(d.value()))
                        } else {
                            Ok(None)
                        }
                    }
                    _ => Ok(None),
                },
                _ => Ok(None),
            }
        })();

        match result {
            Ok(value) => value,
            Err(_) => None,
        }
    }

    /// For a given vector of attributes, this function checks for each one if it contains the
    /// mobile annotation and returns the content as String if one of them does.
    fn get_mobile_annotation_content_if_exists(attrs: Vec<Attribute>) -> Option<String> {
        attrs
            .into_iter()
            .find_map(|attr| Self::get_mobile_annotation_content(&attr))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_is_mobile_annotation() {
        let attr: Attribute = parse_quote! {
            /// @mobile(id = "1", dependencies=[1,2,3], initial_execution_location="Client", crates = ["rand", "another crate", "third one", "fourth"])
        };
        let value = AttributeParser::get_mobile_annotation_content(&attr);
        println!("value: {:?}", value);
        assert!(value.is_some());
    }

    #[test]
    fn test_is_empty_mobile_annotation() {
        let attr: Attribute = parse_quote! {
            /// @mobile
        };
        let value = AttributeParser::get_mobile_annotation_content(&attr);
        println!("value: {:?}", value);
        assert!(value.is_some());
    }

    #[test]
    fn test_get_mobile_attribute_if_exists_wth_comments() {
        let input = r#"#[mobile(id = "Hello", dependencies=["Hello1","Hello2","Hello3"], initial_execution_location="Client"), crates = ["rand", "another crate", "third one", "fourth"]]"#;
        let id: Option<String> =
            AttributeParser::extract_value_from_mobile_annotation(input, ID_PATTERN);
        let libs: Option<String> =
            AttributeParser::extract_value_from_mobile_annotation(input, CRATES_PATTERN);
        let execute_on: Option<String> = AttributeParser::extract_value_from_mobile_annotation(
            input,
            INITIAL_EXECUTION_LOCATION_PATTERN,
        );
        let depends_on: Option<String> =
            AttributeParser::extract_value_from_mobile_annotation(input, DEPENDENCIES_PATTERN);
        println!("id: {:?}", id);
        println!("libs: {:?}", libs);
        println!("execute_on: {:?}", execute_on);
        println!("depends_on: {:?}", depends_on);
    }
}
