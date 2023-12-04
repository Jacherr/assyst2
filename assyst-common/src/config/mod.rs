pub mod config;

static CONFIG_LOCATION: &'static str = "./config.toml";

use lazy_static::lazy_static;
use toml::from_str;

use crate::config::config::AssystConfig;

lazy_static! {
    pub static ref CONFIG: AssystConfig = from_str::<AssystConfig>(CONFIG_LOCATION).unwrap();
}
