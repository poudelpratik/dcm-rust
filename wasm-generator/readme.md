# WASM Generator

The WASM Generator is a component of the DCM-RUST (Dynamic Code Migration in Rust) solution. It facilitates the
generation of WebAssembly (WASM) modules as part of the DCM infrastructure. The generator can be run directly from the
source code or deployed as a Docker container.

## Installation and Usage

### Via Docker

#### Building the Image

Run the following command in wasm-generator's root directory to build the image:

```bash
docker build -t wasm-generator .
```

A build argument can be passed to the Docker build command to specify whether to build the WASM Generator in debug or
release mode.
Below is an example of building the WASM Generator in release mode.

```bash
docker build -t wasm-generator --build-arg BUILD_MODE=release .
```

The default is debug mode if 'BUILD_MODE' argument is not specified.

#### Running the Image

Either execute:

```bash
docker run wasm-generator
```

and manually supply environment variables in command line.
Or, create a 'docker-compose.yml', define the environment variables there, and execute:

```bash
docker compose up
```

## Volumes

Volumes have to be mounted to the container to supply the wasm-generator with the necessary directories.

- `/path/to/project:/project`: Mounts the project directory to the container.
- `path/to/server-side-code-distributor:/code-distributor`: Mounts the server-side code distributor module to the
  container.

### Via Source Code

#### Prerequisites

Before building the wasm-generator from source code, ensure that you have the following dependencies installed:

- **Rust 1.74.0**: The Rust toolchain, which includes `cargo` and `rustc`.
    - Installation instructions can be found at [Rust Install Page](https://www.rust-lang.org/tools/install).
- **Rust Analyzer**: A tool used to auto-detect the dependencies of specified mobile fragments.
    - Install with `rustup component add rust-analyzer`.
- **rustfmt**: Rust's code formatting tool. It is used to format the generated code as to allow the user to understand
  its contents easily.
    - Install with `rustup component add rustfmt`.
- **WASM Toolchain**: The target for building WebAssembly with Rust.
    - Install with `rustup target add wasm32-unknown-unknown`.
- **binaryen**: A compilation and optimization tool for WebAssembly.
    - More information and installation instructions are available
      on [Binaryen GitHub](https://github.com/WebAssembly/binaryen).

#### Building the WASM Generator

In root directory of the WASM Generator.
execute:

```bash
cargo build
```

To build in debug mode. Or, execute:

```bash
cargo build --release
```

To build in release mode.
Above commands will produce a executable binary in the `target/debug` or `target/release` directory respectively.
In Linux systems, it can be executed directly from the command line.

#### Running the WASM Generator

Assuming it was built in release mode, execute from the wasm-generator's root directory:

```bash 
target/release/wasm-generator
```

Environment variables can be supplied by creating a Config.toml file in the current directory.
An example of Config.toml file is provided in the root directory of the wasm-generator.

If Config.toml file is not present in the current directory, then use '-c' parameter to specify it:

```bash
target/release/wasm-generator -c path/to/Config.toml
```

## Supplying Configuration

Following is a list of configuration variables that can be supplied to the WASM Generator:

- `project`:
    - The path to the project for which WASM modules are to be generated.
    - Must be an absolute path.
- `server_fragments_dir`:
    - The path to the server-side fragments directory.
    - Can be either an absolute path or a relative path from the current working directory.
- `client_code_distributor_dir`:
    - The path to the client-side code distributor.
    - Can be either an absolute path or a relative path from the current working directory.
- `release_mode`:
    - Set to "true" to enable release mode compilation of WASM modules.
    - Default is "false".
- `optimize_wasm`:
    - Set to "true" to enable optimization of WebAssembly optimization using the `binaryen` toolchain.
    - Default is "false".
- `max_thread_pool`:
    - Set this variable to any number to limit the maximum CPU threads that can be used by the WASM Generator.
    - Default is to use all CPU threads available.
- `benchmarks_dir`:
    - Set this to some directory where you want to store the benchmark results like compilation time and size of the
      generated WASM modules.
    - If not specified, benchmarks will not be saved.
- `keep_temp_dir`:
    - Set to "true" to keep the temporary directory created by the wasm-generator.
    - It contains the generated code for all the fragments. These are the ones that are compiled to WASM modules.
    - Default is "false".
