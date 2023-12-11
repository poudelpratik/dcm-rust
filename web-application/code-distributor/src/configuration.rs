use clap::Parser;
use figment::providers::{Format, Toml};
use figment::{providers::Env, Figment};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Configuration {
    pub app_host: Option<String>,
    pub app_port: Option<u16>,
    pub api_key: String,
    pub fragments_dir: String,
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
