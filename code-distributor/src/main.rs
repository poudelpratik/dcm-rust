use code_distributor::{configuration::Configuration, init};
use std::io::Write;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    // set env var enable log and initialize logger
    std::env::set_var("RUST_LOG", "info");
    init_logger();

    // initialize configuration
    let fragment_dir = PathBuf::from("fragments")
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    std::env::set_var("fragments_dir", fragment_dir.clone());
    std::env::set_var("api_key", "tZwqxgVXSEaqYQZ");
    let configuration = Configuration::from_env();

    // initialize code distributor
    init(configuration).await;
}

fn init_logger() {
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
