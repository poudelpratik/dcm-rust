use figment::{providers::Env, Figment};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Configuration {
    pub app_host: Option<String>,
    pub app_port: Option<u16>,
    pub api_key: String,
    pub fragments_dir: String,
}

impl Configuration {
    pub fn new(
        api_key: String,
        app_port: Option<u16>,
        fragments_dir: String,
        app_host: Option<String>,
    ) -> Self {
        Configuration {
            app_host,
            app_port,
            api_key,
            fragments_dir,
        }
    }

    pub fn from_env() -> Self {
        Figment::new()
            .merge(Env::raw())
            .extract()
            .expect("Failed to load configuration")
    }
}
