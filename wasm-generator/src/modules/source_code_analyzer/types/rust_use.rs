use crate::modules::source_code_analyzer::traits::rust_item::RustItem;
use crate::modules::source_code_analyzer::types::RustItemCommonProperties;
use crate::modules::source_code_analyzer::types::RustItemPosition;
use serde_derive::Serialize;
use syn::spanned::Spanned;
use syn::ItemUse;

/// This struct represents a Import in Rust syntax tree
#[derive(Debug, Serialize, Clone, Default)]
pub struct RustUse {
    pub properties: RustItemCommonProperties,
    pub uses: Vec<Use>,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct Use {
    pub use_string: String,
    pub location: RustItemPosition,
}

impl From<ItemUse> for RustUse {
    fn from(item_use: ItemUse) -> Self {
        let properties = RustItemCommonProperties {
            position: item_use.span().into(),
            ..Default::default()
        };

        let mut uses = Vec::new();

        // A recursive function to handle nested paths and groups.
        fn extract_uses(tree: &syn::UseTree, prefix: Option<String>, uses: &mut Vec<Use>) {
            match tree {
                syn::UseTree::Path(use_path) => {
                    let ident = &use_path.ident;
                    let mut new_prefix = prefix.unwrap_or_default();
                    if !new_prefix.is_empty() {
                        new_prefix.push_str("::");
                    }
                    new_prefix.push_str(&ident.to_string());
                    extract_uses(&use_path.tree, Some(new_prefix), uses);
                }
                syn::UseTree::Name(use_name) => {
                    let mut use_string = use_name.ident.to_string();
                    if let Some(p) = prefix {
                        use_string = format!("{}::{}", p, use_string);
                    }
                    uses.push(Use {
                        use_string,
                        location: use_name.span().into(),
                    });
                }
                syn::UseTree::Rename(use_rename) => {
                    let use_string = if let Some(p) = &prefix {
                        format!("{}::{} as {}", p, use_rename.ident, use_rename.rename)
                    } else {
                        format!("{} as {}", use_rename.ident, use_rename.rename)
                    };
                    uses.push(Use {
                        use_string,
                        location: use_rename.span().into(),
                    });
                }
                syn::UseTree::Group(use_group) => {
                    for item in &use_group.items {
                        extract_uses(item, prefix.clone(), uses);
                    }
                }
                // Handle other variants if necessary.
                _ => {}
            }
        }

        extract_uses(&item_use.tree, None, &mut uses);

        Self { properties, uses }
    }
}

impl RustItem for RustUse {
    fn get_common_properties(&self) -> RustItemCommonProperties {
        self.properties.clone()
    }

    fn get_common_properties_mut(&mut self) -> &mut RustItemCommonProperties {
        &mut self.properties
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Item};

    #[test]
    fn test_rust_use_conversion() {
        let rust_code = r#"
            use std::collections::HashMap;
            use syn::{parse_quote, ItemUse};
            use serde::{Serialize, Deserialize};
            use std::path::{self as std_path, PathBuf};
            use crate::module::{StructA, StructB as RenamedStructB};
        "#;

        // Parse the code to get a list of ItemUse
        let file = syn::parse_file(rust_code).unwrap();
        let item_uses: Vec<&ItemUse> = file
            .items
            .iter()
            .filter_map(|item| {
                if let Item::Use(item_use) = item {
                    Some(item_use)
                } else {
                    None
                }
            })
            .collect();

        // Convert each ItemUse to RustUse and print the results
        for item_use in item_uses {
            let rust_use: RustUse = RustUse::from(item_use.clone());
            println!("RustUse: {:?}", rust_use);

            // Print each use statement captured
            for use_item in rust_use.uses {
                println!("Use: {:?}", use_item.use_string);
            }
        }
    }
}
