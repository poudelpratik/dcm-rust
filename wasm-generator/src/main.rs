pub mod modules;

// use modules::application;
use wasm_generator::init;

#[tokio::main]
async fn main() {
    init().await;
    // application::run().await;
}
