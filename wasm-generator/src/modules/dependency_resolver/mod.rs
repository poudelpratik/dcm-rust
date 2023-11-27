use crate::modules::application::fragment_type::RustItemType;
use crate::modules::application::traits::fragment::Fragment;
use crate::modules::application::MobileFragments;
use crate::modules::configuration::Configuration;
use crate::modules::dependency_resolver::code_appender::CodeAppender;
use crate::modules::dependency_resolver::crates_resolver::set_cargo_toml;
use crate::modules::dependency_resolver::fragment_dependency_resolver::FragmentDependencyResolver;
use crate::modules::language_server_protocol::rust_analyzer::RustAnalyzerClient;
use crate::modules::language_server_protocol::traits::lsp_client::{LspClient, LspFilePath};
use crate::modules::source_code_analyzer::cargo_toml::ProjectCargoToml;
use crate::modules::source_code_analyzer::rust_file::RustFile;
use crate::modules::source_code_analyzer::types::RustItemCommonProperties;
use crate::modules::source_code_analyzer::FilePath;
use derive_new::new;
use log::info;
use std::sync::Arc;

pub mod code_appender;
pub mod crates_resolver;
pub mod dependency_locator;
pub mod fragment_dependency_resolver;
pub mod rust_item_ast_visitor;

pub async fn run(
    mobile_fragments: &mut MobileFragments,
    rust_files: Arc<Vec<RustFile>>,
    project_cargo_toml: ProjectCargoToml,
    config: Arc<Configuration>,
) {
    let mut automatic_dependency_resolver = FragmentsDependencyResolver::new(
        rust_files.clone(),
        project_cargo_toml.clone(),
        config.clone(),
    )
    .await;
    automatic_dependency_resolver
        .resolve_dependencies(&mut mobile_fragments.functions)
        .await;
    automatic_dependency_resolver
        .resolve_dependencies(&mut mobile_fragments.impls)
        .await;
}

pub struct FragmentsDependencyResolver {
    rust_files: Arc<Vec<RustFile>>,
    project_cargo_toml: ProjectCargoToml,
    config: Arc<Configuration>,
    client: Box<dyn LspClient>,
}

impl FragmentsDependencyResolver {
    pub async fn new(
        rust_files: Arc<Vec<RustFile>>,
        project_cargo_toml: ProjectCargoToml,
        config: Arc<Configuration>,
    ) -> Self {
        let project_root: FilePath = config.host_project.clone().into();
        let project_root_url: LspFilePath = project_root.into();
        info!("Initializing LSP client");
        let mut client: Box<dyn LspClient> = Box::new(RustAnalyzerClient::new().await.unwrap());
        client.initialize(project_root_url).await.unwrap();
        Self {
            rust_files,
            project_cargo_toml,
            config,
            client,
        }
    }

    pub async fn resolve_dependencies(&mut self, fragments: &mut [impl Fragment]) {
        for fragment in fragments.iter_mut() {
            info!(
                "Resolving dependencies for fragment: {}",
                fragment.get_package_name()
            );
            let mut resolver = FragmentDependencyResolver::new(
                self.rust_files.clone(),
                &mut self.client,
                self.config.clone(),
                fragment,
            );
            let mut code_appender = CodeAppender::default();
            let mut final_dependencies = resolver.resolve().await;
            // append dependencies to the code
            for dependency in &mut final_dependencies {
                let mut code = dependency.item_properties.code.clone();
                if dependency.item_properties.item_type == RustItemType::Struct {
                    code = format!("#[derive(Serialize, Deserialize)]\n{}", code);
                }
                code_appender.insert(&dependency.module_hierarchy, &code);
            }
            fragment.set_code(format!(
                "{}\n\n{}",
                fragment.get_code(),
                code_appender.generate_code()
            ));
            set_cargo_toml(fragment, &self.project_cargo_toml);
        }
    }
}

/// This struct is used to store the information about a dependency usage.
#[derive(Debug, Clone)]
pub struct DependencyUsageDetail {
    module_hierarchy: Vec<String>,
    file_path: FilePath,
    line: u32,
    column: u32,
}

/// This struct is used to store the information about the definition of a dependency usage.
#[derive(Debug, Clone, new)]
pub struct DependencyDefinitionDetail {
    pub item_properties: RustItemCommonProperties,
    pub module_hierarchy: Vec<String>,
}
