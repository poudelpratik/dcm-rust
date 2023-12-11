use crate::modules::application::function_fragment::ExecutionLocation;
use crate::modules::application::FinalFragmentContext;
use crate::modules::configuration::Configuration;
use crate::modules::constants::TEMP_PATH;
use crate::modules::util::file_handler;
use log::info;
use std::path::{Path, PathBuf};

pub fn run(fragments: &Vec<FinalFragmentContext>, config: &Configuration) {
    move_fragments(fragments, ExecutionLocation::Client, config);
    move_fragments(fragments, ExecutionLocation::Server, config);

    move_fragments_data(ExecutionLocation::Client, &config);
    move_fragments_data(ExecutionLocation::Server, &config);

    move_js_wrappers(config);
}

fn get_wasm_module_source_path(
    base_path: &Path,
    fragment_identifier: &String,
    config: &Configuration,
) -> PathBuf {
    let mut path = base_path.join("target").join("wasm32-unknown-unknown");
    match config.compilation_enable_release_mode.unwrap_or(false) {
        true => path.push("release"),
        false => path.push("debug"),
    };
    path.join(fragment_identifier)
}

fn get_wasm_module_destination_path(
    execution_location: &ExecutionLocation,
    fragment_identifier: &str,
    config: &Configuration,
) -> PathBuf {
    let path = match execution_location {
        ExecutionLocation::Client => {
            PathBuf::from(&config.client_code_distributor_dir).join("fragments")
        }
        ExecutionLocation::Server => PathBuf::from(&config.server_fragments_dir),
    };
    path.join(fragment_identifier)
}

pub fn move_fragments(
    final_fragments: &Vec<FinalFragmentContext>,
    execution_location: ExecutionLocation,
    config: &Configuration,
) {
    for final_fragment in final_fragments {
        let source = get_wasm_module_source_path(
            &final_fragment.directory.base_path,
            &final_fragment.fragment_identifier,
            config,
        );
        let destination = get_wasm_module_destination_path(
            &execution_location,
            &final_fragment.fragment_identifier,
            config,
        );
        info!(
            "Moving wasm module from {:?} to {:?}",
            &source, &destination
        );
        file_handler::copy_file(&source, &destination).expect("Failed to move wasm module");
    }
}

fn move_fragments_data(execution_location: ExecutionLocation, config: &&Configuration) {
    // Move executable_fragments.json to client and server code distributors
    let source = PathBuf::from(&config.project)
        .join(TEMP_PATH)
        .join("executable_fragments.json");
    let mut destination = match execution_location {
        ExecutionLocation::Client => {
            PathBuf::from(&config.client_code_distributor_dir).join("fragments")
        }
        ExecutionLocation::Server => PathBuf::from(&config.server_fragments_dir),
    };
    destination = destination.join("executable_fragments.json");
    info!(
        "Moving executable_fragments.json from {:?} to {:?}",
        &source, &destination
    );
    file_handler::copy_file(&source, &destination)
        .expect("Failed to move executable_fragments.json");
}

fn move_js_wrappers(config: &Configuration) {
    // Move js_wrappers.js to client side code distributor fragments directory
    let source = PathBuf::from(&config.project)
        .join(TEMP_PATH)
        .join("js_wrappers.js");
    let destination = PathBuf::from(&config.client_code_distributor_dir).join("exports.js");
    info!("Moving js_glue.js from {:?} to {:?}", &source, &destination);
    file_handler::copy_file(&source, &destination).expect("Failed to move js_wrappers.js");
}
