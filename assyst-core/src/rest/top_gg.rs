use std::sync::LazyLock;

use assyst_common::config::CONFIG;
use serde_json::json;

use crate::assyst::ThreadSafeAssyst;

static ROUTE: LazyLock<String> = LazyLock::new(|| format!("https://top.gg/api/bots/{}/stats", CONFIG.bot_id));

pub async fn post_top_gg_stats(assyst: ThreadSafeAssyst) -> anyhow::Result<()> {
    let guild_count = assyst.metrics_handler.guilds.get();
    let shard_count = assyst.shard_count;

    assyst
        .reqwest_client
        .post(&*ROUTE)
        .header("authorization", &CONFIG.authentication.top_gg_token)
        .json(&json!({ "server_count": guild_count, "shard_count": shard_count, "shards": [] }))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
