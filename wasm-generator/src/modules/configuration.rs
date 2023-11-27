use figment::providers::Format;
use figment::{
    providers::{Env, Toml},
    Figment,
};
use serde_derive::Deserialize;
use std::io::Write;

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

impl Default for Configuration {
    fn default() -> Self {
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
