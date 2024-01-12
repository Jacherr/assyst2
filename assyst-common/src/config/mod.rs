pub mod config;

static CONFIG_LOCATION: &str = "../config.toml";

use lazy_static::lazy_static;
use toml::from_str;
use tracing::info;

use crate::config::config::AssystConfig;

lazy_static! {
    pub static ref CONFIG: AssystConfig = {
        let toml = std::fs::read_to_string(CONFIG_LOCATION).unwrap();
        let config = from_str::<AssystConfig>(&toml).unwrap();
        info!("Loaded config file {}", CONFIG_LOCATION);
        config
    };
}
