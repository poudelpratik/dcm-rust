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
    pub project: String,
    pub server_fragments_dir: String,
    pub client_code_distributor_dir: String,
    pub release_mode: Option<bool>,
    pub optimize_wasm: Option<bool>,
    pub max_thread_pool: Option<usize>,
    pub benchmarks_dir: Option<String>,
    pub keep_temp_dir: Option<bool>,
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
