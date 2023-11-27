use figment::providers::Format;
use figment::{
    providers::{Env, Toml},
    Figment,
};
use serde_derive::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Configuration {
    pub app_host: Option<String>,
    pub app_port: Option<u16>,
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
