use code_distributor::{configuration::Configuration, init};
use std::io::Write;

#[tokio::main]
async fn main() {
    // set env var enable log and initialize logger
    std::env::set_var("RUST_LOG", "info");
    init_logger();

    let configuration = Configuration::default();

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
