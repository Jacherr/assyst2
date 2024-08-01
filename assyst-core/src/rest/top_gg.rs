use std::sync::LazyLock;

use assyst_common::config::CONFIG;
use reqwest::Client;
use serde_json::json;

static ROUTE: LazyLock<String> = LazyLock::new(|| format!("https://top.gg/api/bots/{}/stats", CONFIG.bot_id));

pub async fn post_top_gg_stats(client: &Client, guild_count: u64) -> anyhow::Result<()> {
    client
        .post(&*ROUTE)
        .header("authorization", &CONFIG.authentication.top_gg_token)
        .json(&json!({ "server_count": guild_count }))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
