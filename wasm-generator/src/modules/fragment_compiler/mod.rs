use crate::modules::configuration::Configuration;
use crate::modules::error::ApplicationError;
use log::{error, info};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

use crate::modules::application::FragmentCompilationMetric;
use crate::modules::application::{FinalFragmentContext, Size};
use crate::modules::util::thread_manager::rayon::RayonThreadManager;
use crate::modules::util::thread_manager::ThreadManager;

pub fn run(fragments: &mut Vec<FinalFragmentContext>, config: Arc<Configuration>) {
    // Create an instance of the thread manager
    let mut thread_manager = RayonThreadManager::new();
    if let Some(max_thread_pool) = config.compilation_max_thread_pool {
        thread_manager.set_max_threads(max_thread_pool);
    }

    let operation = get_compile_operation(&config);
    thread_manager.process_mut(fragments, operation);

    if config.benchmarks_dir.is_some() {
        // write compilation data to file
        export_compilation_metrics(fragments, &config);
    }
}

fn get_compile_operation<'a>(config: &Configuration) -> impl Fn(&mut FinalFragmentContext) + 'a {
    let config = config.clone();
    move |fragment: &mut FinalFragmentContext| {
        let result = format_code(&fragment.directory.base_path);
        if let Err(e) = result {
            error!("Error formatting fragment code: {:?}", e);
            std::process::exit(1);
        }
        let result = compile(fragment, &config);
        if let Err(e) = result {
            error!("Error compiling fragment: {:?}", e);
            std::process::exit(1);
        }
        if config.compilation_enable_wasm_optimization.unwrap_or(false) {
            let result = optimize(fragment, &config);
            if let Err(e) = result {
                error!("Error optimizing fragment: {:?}", e);
                std::process::exit(1);
            }
        }
    }
}

/// This function builds the command name and arguments to compile the fragment to wasm, and passes it to run_command function for execution.
fn compile(
    fragment: &mut FinalFragmentContext,
    config: &Configuration,
) -> Result<(), ApplicationError> {
    let fragment_path = &fragment.directory.base_path;
    let mut args = vec!["build", "--target", "wasm32-unknown-unknown"];
    if config.compilation_enable_release_mode.unwrap_or(false) {
        args.push("--release");
    }
    info!("Compiling fragment: {:?}", &fragment_path.display());
    let start_time = Instant::now();
    let compilation_result = run_command("cargo", args, fragment_path);
    let duration = start_time.elapsed();
    fragment.compilation_data = FragmentCompilationMetric {
        fragment_identifier: fragment_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        release_mode: config.compilation_enable_release_mode.unwrap_or(false),
        optimization_mode: config.compilation_enable_wasm_optimization.unwrap_or(false),
        compilation_time: duration,
        ..Default::default()
    };

    compilation_result
}

fn format_code(fragment_path: &Path) -> Result<(), ApplicationError> {
    let args = vec!["fmt"];
    run_command("cargo", args, fragment_path)
}

/// This function builds the command name and arguments to optimize the fragment to wasm, and passes it to run_command function for execution.
fn optimize(
    fragment: &mut FinalFragmentContext,
    config: &Configuration,
) -> Result<(), ApplicationError> {
    info!(
        "Optimizing fragment: {:?}",
        &fragment.directory.base_path.display()
    );
    let mut fragment_path = fragment
        .directory
        .base_path
        .join("target")
        .join("wasm32-unknown-unknown");
    match config.compilation_enable_release_mode.unwrap_or(false) {
        true => fragment_path.push("release"),
        false => fragment_path.push("debug"),
    };

    let fragment_identifier = fragment.fragment_identifier.clone();
    let start_time = Instant::now();
    let optimization_result = run_command(
        "wasm-opt",
        vec![
            "--strip",
            "--vacuum",
            "-Oz",
            "-o",
            fragment_identifier.as_str(),
            fragment_identifier.as_str(),
        ],
        &fragment_path,
    );
    fragment.compilation_data.compilation_time += start_time.elapsed();
    optimization_result
}

fn run_command(
    command_name: &str,
    args: Vec<&str>,
    fragment_path: &Path,
) -> Result<(), ApplicationError> {
    let result = Command::new(command_name)
        .args(&args)
        .current_dir(fragment_path)
        .output()?;
    let command_str = format!(
        "cd {} && {} {}",
        fragment_path.display(),
        command_name,
        args.join(" ")
    );
    if result.status.success() {
        info!("Successfully executed command: {}", command_str);
        Ok(())
    } else {
        error!(
            "Error executing command: {} - {}",
            command_str,
            String::from_utf8_lossy(&result.stderr)
        );
        Err(ApplicationError::CommandExecutionError {
            command: command_str,
            error_message: String::from_utf8_lossy(&result.stderr).to_string(),
        })
    }
}

fn export_compilation_metrics(fragments: &mut [FinalFragmentContext], config: &Configuration) {
    let benchmarks_dir = PathBuf::from(&config.benchmarks_dir.clone().unwrap_or_default());
    if !benchmarks_dir.exists() {
        fs::create_dir_all(&benchmarks_dir).expect("Unable to create benchmarks directory");
    }
    let file_name = format!(
        "compilation_data_{}.json",
        chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
    );
    let file = File::create(benchmarks_dir.join(file_name)).expect("Unable to create file");

    // Iterate over the fragments to calculate WASM size and collect compilation data
    let compilation_data: Vec<FragmentCompilationMetric> = fragments
        .iter_mut()
        .map(|fragment| {
            // Construct the path to the compiled WASM file
            let mut fragment_path = fragment
                .directory
                .base_path
                .join("target")
                .join("wasm32-unknown-unknown");
            if config.compilation_enable_release_mode.unwrap_or(false) {
                fragment_path.push("release");
            } else {
                fragment_path.push("debug");
            }
            fragment_path.push(&fragment.fragment_identifier); // Assuming this is the WASM file name

            // Get the size of the WASM file
            let wasm_size_in_bytes = fs::metadata(&fragment_path).unwrap().len() as f64;

            // Update the wasm_size in the fragment's compilation data
            fragment.compilation_data.wasm_size = Size {
                mb: format!("{:.2}", wasm_size_in_bytes / 1_048_576.0)
                    .parse::<f64>()
                    .unwrap(),
                kb: format!("{:.2}", wasm_size_in_bytes / 1_024.0)
                    .parse::<f64>()
                    .unwrap(),
                bytes: format!("{:.2}", wasm_size_in_bytes).parse::<f64>().unwrap(),
            };

            fragment.compilation_data.clone()
        })
        .collect();

    serde_json::to_writer_pretty(file, &compilation_data).expect("Unable to write data");
}
