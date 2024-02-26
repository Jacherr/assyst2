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
    pub proxy: Vec<String>,
    pub filer: String,
    pub wsi: String,
}

#[derive(Deserialize)]
pub struct Authentication {
    pub discord_token: String,
    pub patreon_token: String,
    pub top_gg_token: String,
    pub top_gg_webhook_token: String,
    pub top_gg_webhook_port: u16,
    pub filer_key: String,
}

#[derive(Deserialize)]
pub struct Database {
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub port: u16,
}
impl Database {
    pub fn to_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }

    pub fn to_url_safe(&self) -> String {
        let mut host = self.host.split(".").take(2).collect::<Vec<_>>();
        host.push("###");
        host.push("###");

        let mut port = self.port.to_string();
        port.replace_range(..3, "...");

        format!(
            "postgres://{}@{}:{}/{}",
            self.username,
            &host.join("."),
            port,
            self.database
        )
    }
}

#[derive(Deserialize)]
pub struct Prefixes {
    pub default: String,
}

#[derive(Deserialize)]
pub struct LoggingWebhooks {
    pub panic: LoggingWebhook,
    pub error: LoggingWebhook,
    pub vote: LoggingWebhook,
    pub enable_webhooks: bool
}

#[derive(Deserialize, Clone)]
pub struct LoggingWebhook {
    pub token: String,
    pub id: u64,
}

#[derive(Deserialize)]
pub struct DevAttributes {
    pub admin_users: Vec<u64>,
    pub prefix_override: Option<String>,
    pub disable_bad_translator_channels: bool,
    pub disable_reminder_check: bool,
    pub disable_bot_list_posting: bool,
    pub dev_guild: u64,
    pub dev_channel: u64,
    pub dev_message: bool,
}
