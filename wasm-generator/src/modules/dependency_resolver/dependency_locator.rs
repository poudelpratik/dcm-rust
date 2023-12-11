use crate::modules::configuration::Configuration;
use crate::modules::dependency_resolver::{DependencyDefinitionDetail, DependencyUsageDetail};
use crate::modules::language_server_protocol::traits::lsp_client::{
    LspClient, LspFilePath, RustItemLocation,
};
use crate::modules::source_code_analyzer::rust_file::RustFile;
use crate::modules::source_code_analyzer::traits::rust_item::RustItem;
use crate::modules::source_code_analyzer::types::{RustItemCommonProperties, RustItemPosition};
use crate::modules::source_code_analyzer::FilePath;
use std::collections::HashMap;
use std::sync::Arc;

pub struct DependencyLocator<'a> {
    lsp_client: &'a mut Box<dyn LspClient>,
    resolved_item_definitions: Vec<RustItemCommonProperties>,
    config: Arc<Configuration>,
    rust_files: Arc<Vec<RustFile>>,
}

impl<'a> DependencyLocator<'a> {
    pub fn new(
        lsp_client: &'a mut Box<dyn LspClient>,
        config: Arc<Configuration>,
        rust_files: Arc<Vec<RustFile>>,
    ) -> Self {
        Self {
            lsp_client,
            resolved_item_definitions: Vec::new(),
            config,
            rust_files,
        }
    }

    pub async fn locate_dependency(
        &mut self,
        dependency: &DependencyUsageDetail,
        visited_rust_items: &[RustItemCommonProperties],
    ) -> Option<DependencyDefinitionDetail> {
        let file_path: LspFilePath = dependency.file_path.clone().into();
        if let Ok(item_definition_location) = self
            .lsp_client
            .get_definition_location(file_path, dependency.line - 1, dependency.column)
            .await
        {
            // Check if the located dependency has already been processed. If same dependency is used multiple times by the same item, it will appear multiple times in the dependency list but should be processed only once.
            // The second major condition is to check if the dependency is defined within the same item. Example: type definitions inside functions, etc. If so, it should be skipped.
            if lies_within_visited_code(&item_definition_location, visited_rust_items) {
                return None;
            }

            if item_definition_location
                .file_path
                .clone()
                .get_absolute_path()
                .starts_with(self.config.project.as_str())
            {
                return self.process_crate_dependency(dependency, &item_definition_location);
            }
        }
        None
    }

    fn process_crate_dependency(
        &mut self,
        dependency_usage_detail: &DependencyUsageDetail,
        item_definition_location: &RustItemLocation,
    ) -> Option<DependencyDefinitionDetail> {
        let mut dependency: Option<DependencyDefinitionDetail> = None;

        if let Some(rust_file) = self
            .rust_files
            .iter()
            .find(|rf| {
                rf.absolute_filepath
                    .eq(&item_definition_location.file_path.get_absolute_path())
            })
            .cloned()
        {
            // Condition to check if the position of dependency returned by the LSP falls within the start and end lines of the item definition analyzed by syn crate
            let is_within_lines_condition = |item_common_properties: &RustItemCommonProperties| {
                item_common_properties.position.start_line
                    <= item_definition_location.position.start_line
                    && item_common_properties.position.end_line
                        >= item_definition_location.position.start_line
            };

            // check in every item type
            if let Some(item) = rust_file
                .functions
                .iter()
                .find(|item| is_within_lines_condition(&item.get_common_properties()))
            {
                dependency = Some(DependencyDefinitionDetail::new(
                    item.get_common_properties().clone(),
                    dependency_usage_detail.module_hierarchy.clone(),
                ));
            }

            if let Some(item) = rust_file.impls.iter().find(|rust_impl| {
                rust_impl
                    .functions
                    .iter()
                    .any(|function| is_within_lines_condition(&function.get_common_properties()))
            }) {
                let mut module_hierarchy = dependency_usage_detail.module_hierarchy.clone();
                module_hierarchy.pop();
                dependency = Some(DependencyDefinitionDetail::new(
                    item.get_common_properties().clone(),
                    module_hierarchy,
                ));
            }

            if let Some(item) = rust_file
                .structs
                .iter()
                .find(|item| is_within_lines_condition(&item.get_common_properties()))
            {
                dependency = Some(DependencyDefinitionDetail::new(
                    item.get_common_properties().clone(),
                    dependency_usage_detail.module_hierarchy.clone(),
                ));
            }

            if let Some(item) = rust_file
                .consts
                .iter()
                .find(|item| is_within_lines_condition(&item.get_common_properties()))
            {
                dependency = Some(DependencyDefinitionDetail::new(
                    item.get_common_properties().clone(),
                    dependency_usage_detail.module_hierarchy.clone(),
                ));
            }

            if let Some(item) = rust_file
                .statics
                .iter()
                .find(|item| is_within_lines_condition(&item.get_common_properties()))
            {
                dependency = Some(DependencyDefinitionDetail::new(
                    item.get_common_properties().clone(),
                    dependency_usage_detail.module_hierarchy.clone(),
                ));
            }

            if let Some(item) = rust_file
                .type_definitions
                .iter()
                .find(|item| is_within_lines_condition(&item.get_common_properties()))
            {
                dependency = Some(DependencyDefinitionDetail::new(
                    item.get_common_properties().clone(),
                    dependency_usage_detail.module_hierarchy.clone(),
                ));
            }
        }

        if let Some(dependency) = dependency.clone() {
            if self
                .resolved_item_definitions
                .iter()
                .any(|item| item == &dependency.item_properties)
            {
                return None;
            }
            self.resolved_item_definitions
                .push(dependency.item_properties.clone());
        }

        dependency
    }

    /// This function looks at the resolved dependencies and tries to resolve the use statements required for them.
    pub async fn resolve_use_statements(
        &mut self,
        visited_rust_items: Vec<RustItemCommonProperties>,
    ) -> Vec<DependencyDefinitionDetail> {
        let mut use_statements: Vec<DependencyDefinitionDetail> = Vec::new();
        let mut grouped_visited_items: HashMap<FilePath, Vec<RustItemCommonProperties>> =
            HashMap::new();

        for item in visited_rust_items.clone() {
            grouped_visited_items
                .entry(item.file_path.clone())
                .or_default()
                .push(item);
        }

        for (file_path, visited_rust_items) in grouped_visited_items {
            let use_statements_in_file = self
                .rust_files
                .iter()
                .find(|rust_file| {
                    rust_file
                        .absolute_filepath
                        .eq(&file_path.get_absolute_path())
                })
                .map(|rust_file| rust_file.uses.clone())
                .unwrap();
            for use_item in use_statements_in_file {
                for current_use in &use_item.uses {
                    if let Ok(item_definition_location) = self
                        .lsp_client
                        .get_definition_location(
                            LspFilePath::from(file_path.clone()),
                            (current_use.location.start_line - 1) as u32,
                            (current_use.location.start_column) as u32,
                        )
                        .await
                    {
                        if item_definition_location
                            .file_path
                            .clone()
                            .get_absolute_path()
                            .starts_with(self.config.project.as_str())
                        {
                            continue;
                        }
                        if let Ok(document_highlight_positions) = self
                            .lsp_client
                            .get_document_highlight_positions(
                                LspFilePath::from(file_path.clone()),
                                (current_use.location.start_line - 1) as u32,
                                (current_use.location.start_column) as u32,
                            )
                            .await
                        {
                            if is_any_highlighted_position_inside_used_code(
                                &document_highlight_positions,
                                &visited_rust_items,
                            ) && !use_statements
                                .iter()
                                .any(|dep| dep.item_properties == use_item.properties)
                            {
                                use_statements.push(DependencyDefinitionDetail::new(
                                    use_item.properties.clone(),
                                    vec![],
                                ));
                            }
                        }
                    }
                }
            }
        }

        use_statements
    }
}

fn is_any_highlighted_position_inside_used_code(
    highlighted_positions: &[RustItemPosition],
    rust_items_in_file: &[RustItemCommonProperties],
) -> bool {
    highlighted_positions.iter().any(|highlight| {
        rust_items_in_file.iter().any(|item| {
            highlight.start_line >= item.position.start_line
                && highlight.end_line <= item.position.end_line
        })
    })
}

fn lies_within_visited_code(
    item_definition_location: &RustItemLocation,
    visited_rust_items: &[RustItemCommonProperties],
) -> bool {
    visited_rust_items.iter().any(|visited_item| {
        item_definition_location
            .file_path
            .eq(&visited_item.file_path)
            && item_definition_location.position.start_line >= visited_item.position.start_line
            && item_definition_location.position.start_line <= visited_item.position.end_line
    })
    // || (item_definition_location.position.start_line >= rust_item.location.start_line
    // && item_definition_location.position.start_line <= rust_item.location.end_line
    // && item_definition_location.file_path.eq(&rust_item.file_path))
}
