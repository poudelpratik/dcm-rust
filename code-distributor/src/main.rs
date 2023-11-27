use code_distributor::{configuration::Configuration, init};

#[tokio::main]
async fn main() {
    // set env var enable log
    std::env::set_var("RUST_LOG", "info");

    // initialize configuration
    let config = Configuration::default();
    config.init_logger();
    init(config).await;
}
