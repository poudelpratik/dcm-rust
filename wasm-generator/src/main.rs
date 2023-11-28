pub mod modules;

use wasm_generator::init;

#[tokio::main]
async fn main() {
    init().await;
}
