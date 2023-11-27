mod helpers_generator;
pub mod javascript_wrappers_generator;
pub mod wasm_wrapper_generator;

use crate::modules::application::traits::fragment::Fragment;
use crate::modules::application::ExecutableFragmentDataForCodeDistributor;
use crate::modules::application::{FinalFragmentContext, MobileFragments};
use crate::modules::configuration::Configuration;
use crate::modules::constants::{FRAGMENTS_LOCATION, TEMP_PATH};
use crate::modules::util::file_handler::DirectoryContext;
use crate::modules::util::id_generator::FragmentIdGenerator;
use crate::modules::util::{file_handler, id_generator};
use std::path::PathBuf;
use std::sync::Arc;

pub fn export_fragments_data(mobile_fragments: &MobileFragments, config: Arc<Configuration>) {
    // export minified executable_fragments to executable_fragments.json for use by the codedistributor
    let final_function_fragments = mobile_fragments
        .functions
        .iter()
        .map(|f| f.clone().into())
        .collect::<Vec<ExecutableFragmentDataForCodeDistributor>>();
    let final_impl_fragments = mobile_fragments
        .impls
        .iter()
        .map(|f| f.clone().into())
        .collect::<Vec<ExecutableFragmentDataForCodeDistributor>>();
    let final_fragments_combined = final_function_fragments
        .into_iter()
        .chain(final_impl_fragments)
        .collect::<Vec<ExecutableFragmentDataForCodeDistributor>>();
    let final_fragments_minimal_json =
        serde_json::to_string_pretty(&final_fragments_combined).unwrap();
    file_handler::writeln(
        &PathBuf::from(&config.host_project)
            .join(TEMP_PATH)
            .join("executable_fragments.json"),
        final_fragments_minimal_json,
    )
    .expect("Failed to write to executable_fragments.json");
}

fn create_fragment_path(identifier: &String, project_root: &String, path: &str) -> PathBuf {
    let fragments_path = PathBuf::from(project_root).join(path);
    fragments_path.join(identifier)
}

pub fn check_duplicate_and_assign_missing_ids(mobile_fragments: &mut MobileFragments) {
    id_generator::check_duplicates(&mut mobile_fragments.functions);
    id_generator::check_duplicates(&mut mobile_fragments.impls);

    let mut fragment_id_generator = FragmentIdGenerator::default();
    id_generator::assign_missing_ids(&mut mobile_fragments.functions, &mut fragment_id_generator);
    id_generator::assign_missing_ids(&mut mobile_fragments.impls, &mut fragment_id_generator);
}

pub fn generate_wasm_wrapper(mobile_fragments: &mut MobileFragments) {
    wasm_wrapper_generator::generate_wrapper_for_free_functions(&mut mobile_fragments.functions);
    wasm_wrapper_generator::generate_wrapper_for_impls(&mut mobile_fragments.impls);
    helpers_generator::generate_helper(mobile_fragments);
}

pub fn generate_js_wrappers(mobile_fragments: &MobileFragments, config: Arc<Configuration>) {
    let js_wrappers = javascript_wrappers_generator::run(mobile_fragments);
    file_handler::writeln(
        &PathBuf::from(&config.host_project)
            .join(TEMP_PATH)
            .join("js_wrappers.js"),
        js_wrappers,
    )
    .expect("Failed to write to js_wrappers.js");
}

pub fn generate(
    mobile_fragments: &MobileFragments,
    config: Arc<Configuration>,
) -> Vec<FinalFragmentContext> {
    let mut generated_fragments: Vec<FinalFragmentContext> = Vec::new();
    generate_fragments(
        &mobile_fragments.functions,
        config.clone(),
        &mut generated_fragments,
    );
    generate_fragments(
        &mobile_fragments.impls,
        config.clone(),
        &mut generated_fragments,
    );
    generated_fragments
}

fn generate_fragments(
    executable_fragments: &[impl Fragment],
    config: Arc<Configuration>,
    generated_fragments: &mut Vec<FinalFragmentContext>,
) {
    for executable_fragment in executable_fragments.iter() {
        let fragment_path = create_fragment_path(
            &executable_fragment.get_package_name(),
            &config.host_project,
            FRAGMENTS_LOCATION,
        );

        let fragment_context = FinalFragmentContext::new(
            DirectoryContext::new(&fragment_path),
            executable_fragment.get_wasm_identifier(),
        );

        // Create Cargo.toml
        let cargo_file = fragment_context
            .directory
            .create_file(fragment_path.join("Cargo.toml"))
            .expect("Failed to create Cargo.toml");
        let toml_content = toml::to_string(&executable_fragment.get_cargo_toml()).unwrap();
        file_handler::write_to_file(cargo_file, toml_content)
            .expect("Failed to write to Cargo.toml");

        // Create lib.rs
        let lib_file = fragment_context
            .directory
            .create_file(fragment_path.join("src/lib.rs"))
            .expect("Failed to create lib.rs");
        file_handler::write_to_file(lib_file, executable_fragment.get_code().clone())
            .expect("Failed to write to lib.rs");
        generated_fragments.push(fragment_context);
    }
}
