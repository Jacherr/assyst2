use std::num::NonZeroU64;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Cfg {
    pub token: String,
    pub guild_id: NonZeroU64,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let cfg_file = std::fs::read_to_string("Config.toml").expect("missing Config.toml");
    let cfg: Cfg = toml::from_str(&cfg_file).expect("error parsing TOML");

    let http = twilight_http::Client::new(cfg.token);
    let app = http.current_user_application().await?.model().await?;
    let int = http.interaction(app.id);
    let cmd = int
        .create_guild_command(cfg.guild_id.into())
        .chat_input("ping", "pong")?
        .await?
        .model()
        .await?;
    dbg!(&cmd);
    Ok(())
}
