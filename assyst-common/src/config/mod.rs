pub mod config;

static CONFIG_LOCATION: &str = "../config.toml";

use lazy_static::lazy_static;
use toml::from_str;

use crate::config::config::AssystConfig;

lazy_static! {
    pub static ref CONFIG: AssystConfig = {
        let toml = std::fs::read_to_string(CONFIG_LOCATION).unwrap();
        from_str::<AssystConfig>(&toml).unwrap()
    };
}
