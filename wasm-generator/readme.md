# The WASM Generator

This is the WASM generator infrastructure of the DCM-RUST solution.

## Getting Started

It can be run in two ways:

- From a Docker image
- From source code

## Dependencies

- Rust 1.74.0
    - Installation documentation: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
- Rust Analyzer Server
    - Installation command: "rustup component add rust-analyzer"
- rustfmt tool
    - Installation command: "rustup component add rustfmt"
- rust-wasm toolchain
    - Installation command: "rustup target add wasm32-unknown-unknown"
- binaryen tool
    - Installation documentation: [https://github.com/WebAssembly/binaryen](https://github.com/WebAssembly/binaryen)

## Executing program

- ### From a Docker image
    - Run "docker compose up".

- ### From source code
    - Run "cargo run" to run in debug mode.
    - Run "cargo run --release" to run in release mode.