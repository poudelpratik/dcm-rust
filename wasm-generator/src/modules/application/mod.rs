use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use derive_new::new;
use serde_derive::{Deserialize, Serialize};

use crate::modules::application::function_fragment::{ExecutionLocation, FunctionFragment};
use crate::modules::application::object_fragment::ObjectFragment;
use crate::modules::configuration::Configuration;
use crate::modules::constants::TEMP_PATH;
use crate::modules::util::file_handler::DirectoryContext;
use crate::modules::{
    cfd_analyzer, dependency_resolver, fragment_compiler, fragment_generator,
    post_compilation_processor, source_code_analyzer, util,
};

pub(crate) mod fragment_type;
pub mod function_fragment;
pub mod object_fragment;
pub mod traits;

pub async fn run() {
    std::env::set_var("RUST_LOG", "debug");

    let config = Arc::new(Configuration::default());
    init_logger();

    // delete the temp directory if it already exists from previous run
    delete_temporary_directory(&config);

    // initialize the mobile fragments container struct
    let mut mobile_fragments = MobileFragments::default();

    // run the source code analyzer to get the syntax tree and project metadata
    // It also updates the mobile fragments container with any located mobile fragments
    let (syntax_tree, project_cargo_toml) =
        source_code_analyzer::run(&mut mobile_fragments, config.clone());
    let syntax_tree = Arc::new(syntax_tree);
    let project_metadata = Arc::new(project_cargo_toml);

    // The cfd analyzer further updates the mobile fragments container with any specified mobile fragments in the cfd
    cfd_analyzer::run(config.clone(), syntax_tree.clone(), &mut mobile_fragments);

    fragment_generator::check_duplicate_and_assign_missing_ids(&mut mobile_fragments);

    // run the dependency resolver to resolve all the dependencies of the mobile fragments
    dependency_resolver::run(
        &mut mobile_fragments,
        syntax_tree,
        project_metadata.as_ref().clone(),
        config.clone(),
    )
    .await;

    // modify and write the extracted mobile fragments to disk
    fragment_generator::generate_wasm_wrapper(&mut mobile_fragments);
    let mut generated_fragments = fragment_generator::generate(&mobile_fragments, config.clone());
    fragment_generator::generate_js_wrappers(&mobile_fragments, config.clone());
    fragment_generator::export_fragments_data(&mobile_fragments, config.clone());

    // run the fragment compiler to compile the generated fragments to wasm
    fragment_compiler::run(&mut generated_fragments, config.clone());

    // delete the deployed fragments if they exist from previous run
    delete_deployed_fragments(&config);
    // deploy the newly generated fragments to the respective directories
    post_compilation_processor::run(&generated_fragments, &config);

    if !config.keep_temp_dir.unwrap_or(false) {
        delete_temporary_directory(&config);
    }
}

pub fn delete_temporary_directory(config: &Configuration) {
    // Delete the temporary directory if already exists
    util::file_handler::delete_directory(&PathBuf::from(&config.project.clone()).join(TEMP_PATH))
        .expect("Failed to delete temporary directory.");
}

pub fn delete_deployed_fragments(config: &Configuration) {
    util::file_handler::delete_directory(
        &PathBuf::from(&config.server_fragments_dir).join("fragments"),
    )
    .expect("Failed to delete server fragments directory.");

    util::file_handler::delete_directory(
        &PathBuf::from(&config.client_code_distributor_dir).join("fragments"),
    )
    .expect("Failed to delete client fragments directory.");
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MobileFragments {
    pub functions: Vec<FunctionFragment>,
    pub impls: Vec<ObjectFragment>,
}

#[derive(Clone, Default, Serialize)]
pub struct FinalFragmentContext {
    pub directory: DirectoryContext,
    pub fragment_identifier: String,
    pub compilation_data: FragmentCompilationMetric,
}

impl FinalFragmentContext {
    pub fn new(directory: DirectoryContext, fragment_identifier: String) -> Self {
        Self {
            directory,
            fragment_identifier,
            ..Default::default()
        }
    }
}

/// This struct is used to represent final fragments with minimal information.
/// It will be later read by the code distributor to get final list of fragments.
#[derive(Debug, new, Clone, Default, Deserialize, PartialEq, Serialize)]
pub struct ExecutableFragmentDataForCodeDistributor {
    pub id: String,
    pub execution_location: ExecutionLocation,
}

impl From<FunctionFragment> for ExecutableFragmentDataForCodeDistributor {
    fn from(final_fragment: FunctionFragment) -> Self {
        Self {
            id: final_fragment.id,
            execution_location: final_fragment.initial_execution_location,
        }
    }
}

impl From<ObjectFragment> for ExecutableFragmentDataForCodeDistributor {
    fn from(final_fragment: ObjectFragment) -> Self {
        Self {
            id: final_fragment.id,
            execution_location: final_fragment.initial_execution_location,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FragmentCompilationMetric {
    pub(crate) fragment_identifier: String,
    pub(crate) release_mode: bool,
    pub(crate) optimization_mode: bool,
    pub(crate) wasm_size: Size,
    pub(crate) compilation_time: Duration,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Size {
    pub mb: f64,
    pub kb: f64,
    pub bytes: f64,
}

pub fn init_logger() {
    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
}
