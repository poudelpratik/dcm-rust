use figment::providers::Format;
use figment::{
    providers::{Env, Toml},
    Figment,
};
use serde_derive::{Deserialize, Serialize};
use std::io::Write;

/// This struct represents the configuration of the application
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Configuration {
    pub app_host: Option<String>,
    pub app_port: Option<u16>,

    pub code_distributor_api_url: String,
    pub code_distributor_ws_url: String,
    pub code_distributor_api_key: String,
}

impl Default for Configuration {
    fn default() -> Configuration {
        let config: Configuration = Figment::new()
            .merge(Toml::file("Config.toml"))
            .merge(Env::raw())
            .extract()
            .unwrap();
        config
    }
}

impl Configuration {
    pub fn init_logger(&self) {
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
}

#[derive(Debug, Default, Serialize)]
pub struct CodeDistributorConfiguration {
    pub code_distributor_api_url: String,
    pub code_distributor_ws_url: String,
    pub code_distributor_api_key: String,
}

impl From<Configuration> for CodeDistributorConfiguration {
    fn from(config: Configuration) -> Self {
        Self {
            code_distributor_api_url: config.code_distributor_api_url,
            code_distributor_ws_url: config.code_distributor_ws_url,
            code_distributor_api_key: config.code_distributor_api_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::info;

    #[test]
    fn test_logger() {
        let config = Configuration::default();
        config.init_logger();
        info!("Logging works...");
    }
}
