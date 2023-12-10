use clap::Parser;
use figment::providers::Format;
use figment::{
    providers::{Env, Toml},
    Figment,
};
use serde_derive::Deserialize;

/// This struct represents the configuration of the application
#[derive(Debug, Deserialize, Clone)]
pub struct Configuration {
    pub host_project: String,
    pub server_code_distributor: String,
    pub client_code_distributor: String,
    pub compilation_enable_release_mode: Option<bool>,
    pub compilation_enable_wasm_optimization: Option<bool>,
    pub compilation_max_thread_pool: Option<usize>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "Config.toml")]
    config: String,
}

impl Default for Configuration {
    fn default() -> Self {
        let args = Args::parse();
        let config: Configuration = Figment::new()
            .merge(Toml::file(args.config))
            .merge(Env::raw())
            .extract()
            .unwrap();
        config
    }
}
