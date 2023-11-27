use crate::modules::application::fragment_type::RustItemType;
use crate::modules::application::traits::fragment::Fragment;
use crate::modules::configuration::Configuration;
use crate::modules::dependency_resolver::dependency_locator::DependencyLocator;
use crate::modules::dependency_resolver::rust_item_ast_visitor::RustItemAstVisitor;
use crate::modules::dependency_resolver::{DependencyDefinitionDetail, DependencyUsageDetail};
use crate::modules::language_server_protocol::traits::lsp_client::LspClient;
use crate::modules::source_code_analyzer::rust_file::RustFile;
use crate::modules::source_code_analyzer::types::RustItemCommonProperties;
use crate::modules::util::file_handler;
use std::sync::Arc;
use syn::visit::Visit;

pub struct FragmentDependencyResolver<'a, T>
where
    T: Fragment,
{
    rust_files: Arc<Vec<RustFile>>,
    dependency_locator: DependencyLocator<'a>,
    fragment: &'a mut T,
}

impl<'a, T> FragmentDependencyResolver<'a, T>
where
    T: Fragment,
{
    pub fn new(
        rust_files: Arc<Vec<RustFile>>,
        client: &'a mut Box<dyn LspClient>,
        config: Arc<Configuration>,
        fragment: &'a mut T,
    ) -> Self {
        let dependency_locator = DependencyLocator::new(client, config.clone(), rust_files.clone());
        Self {
            rust_files,
            dependency_locator,
            fragment,
        }
    }

    pub(crate) async fn resolve(&mut self) -> Vec<DependencyDefinitionDetail> {
        let rust_item = self.fragment.get_common_properties();
        // A list of all the rust items that need to be visited.
        // At first, it contains only the mobile impl marked by the user.
        let mut items: Vec<RustItemCommonProperties> = vec![rust_item.clone()];
        // A list of all the rust items that have been visited.
        let mut visited_items: Vec<RustItemCommonProperties> = Vec::new();
        // A list of all the dependencies that need to be located in the source code.
        let mut dependency_usages: Vec<DependencyUsageDetail> = Vec::new();
        // A list of all the dependencies that have been located in the source code.
        let mut located_dependency_definitions: Vec<DependencyDefinitionDetail> = Vec::new();
        // A flag to check if the impl's struct has been found.
        let mut struct_found = false;

        loop {
            for rust_item in &items {
                let mut rust_item_visitor =
                    RustItemAstVisitor::new(rust_item, &mut dependency_usages);
                let source_code = file_handler::read(&rust_item.file_path.get_absolute_path())
                    .expect("Failed to read file");
                let syntax_tree = syn::parse_file(&source_code).expect("Failed to parse file");
                rust_item_visitor.visit_file(&syntax_tree);
            }
            visited_items.append(&mut items);

            if dependency_usages.is_empty() {
                break;
            }

            for dependency_usage_detail in dependency_usages.drain(..) {
                let located_dependency = self
                    .dependency_locator
                    .locate_dependency(&dependency_usage_detail, &visited_items)
                    .await;

                if let Some(dependency) = located_dependency {
                    // The very first dependency that is located should be the struct that is being implemented
                    if rust_item.item_type == RustItemType::Impl
                        && !struct_found
                        && dependency.item_properties.item_type.clone() == RustItemType::Struct
                    {
                        let impl_struct = self
                            .rust_files
                            .iter()
                            .find(|f| {
                                f.absolute_filepath == rust_item.file_path.get_absolute_path()
                            })
                            .unwrap()
                            .structs
                            .iter()
                            .find(|s| s.properties == dependency.item_properties)
                            .unwrap()
                            .clone();
                        self.fragment.set_struct_for_impl(impl_struct);
                        struct_found = true;
                    }
                    items.push(dependency.item_properties.clone());
                    located_dependency_definitions.push(dependency);
                }
            }
        }

        let mut use_statements = self
            .dependency_locator
            .resolve_use_statements(visited_items)
            .await;
        located_dependency_definitions.append(&mut use_statements);
        located_dependency_definitions
    }
}
