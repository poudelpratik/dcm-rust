# WASM Generator

The WASM Generator is a component of the DCM-RUST (Dynamic Code Migration in Rust) solution. It facilitates the
generation of WebAssembly (WASM) modules as part of the DCM infrastructure. The generator can be run directly from the
source code or deployed as a Docker container.

## Running the WASM Generator

### From a Docker Image

1. Configure the environment variables in `docker-compose.yml` to match your project setup.
2. Run the following command to build and start the container:

    ```bash
    docker-compose up
    ```

### From Source Code

### Prerequisites

Before running the WASM Generator from source code, ensure that you have the following dependencies installed:

- **Rust 1.74.0**: The Rust toolchain, which includes `cargo` and `rustc`.
    - Installation instructions can be found at [Rust Install Page](https://www.rust-lang.org/tools/install).
- **Rust Analyzer**: A tool used to auto-detect the dependencies of specified mobile fragments.
    - Install with `rustup component add rust-analyzer`.
- **rustfmt**: Rust's code formatting tool.
    - Install with `rustup component add rustfmt`.
- **WASM Toolchain**: The target for building WebAssembly with Rust.
    - Install with `rustup target add wasm32-unknown-unknown`.
- **binaryen**: A compilation and optimization tool for WebAssembly.
    - More information and installation instructions are available
      on [Binaryen GitHub](https://github.com/WebAssembly/binaryen).

1. Navigate to the root directory of the WASM Generator.
2. To run in debug mode, execute:

    ```bash
    cargo run
    ```

3. To run in release mode with optimizations, execute:

    ```bash
    cargo run --release
    ```

## Configuration

The WASM Generator can be configured via environment variables, which can be set in the Docker container
through `docker-compose.yml`:

- `host_project`: The path to the project for which WASM modules are to be generated.
- `server_code_distributor`: The path to the server-side code distributor module.
- `client_code_distributor`: The path to the client-side code distributor module within the host project.
- `compilation_enable_release_mode`: Set to "true" to enable release mode compilation.
- `compilation_enable_wasm_optimization`: Set to "true" to enable WebAssembly optimization.
- `compilation_max_thread_pool`: The maximum number of allowed threads in the thread pool for compilation tasks.

## Volumes

The following volumes are mounted in the `docker-compose.yml`:

- `../runtime-code-mobility-demo:/project`: Mounts the host project directory to the container.
- `../code-distributor:/code-distributor`: Mounts the code distributor module to the container.
