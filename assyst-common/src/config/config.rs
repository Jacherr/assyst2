// See config.toml for information on the variables here.

use serde::Deserialize;

#[derive(Deserialize)]
pub struct AssystConfig {
    pub urls: Urls,
    pub authentication: Authentication,
    pub database: Database,
    pub prefix: Prefixes,
    pub logging_webhooks: LoggingWebhooks,
    pub dev: DevAttributes,
}

#[derive(Deserialize)]
pub struct Urls {
    
}

#[derive(Deserialize)]
pub struct Authentication {
    pub discord_token: String
}

#[derive(Deserialize)]
pub struct Database {
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub port: u16
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
    pub admin_users: Vec<u64>,
    pub prefix_override: Option<String>,
    pub disable_bad_translator_channels: bool,
    pub disable_reminder_check: bool,
    pub db_logs: bool
}