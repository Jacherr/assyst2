// See config.toml for information on the variables here.

use serde::Deserialize;
use toml::from_str;

#[derive(Deserialize)]
pub struct AssystConfig {
    urls: Urls,
    authentication: Authentication,
    database: Database,
    prefix: Prefixes,
    logging_webhooks: LoggingWebhooks,
    dev: DevAttributes,
}

#[derive(Deserialize)]
pub struct Urls {
    
}

#[derive(Deserialize)]
pub struct Authentication {
    discord_token: String
}

#[derive(Deserialize)]
pub struct Database {
    host: String,
    username: String,
    password: String,
    database: String,
    port: u16
}

#[derive(Deserialize)]
pub struct Prefixes {
    pub default: String
}

#[derive(Deserialize)]
pub struct LoggingWebhooks {

}

#[derive(Deserialize)]
pub struct DevAttributes {
    admin_users: Vec<u64>,
    prefix_override: Option<String>,
    disable_bad_translator_channels: bool,
    disable_reminder_check: bool,
    db_logs: bool
}