use assyst_common::config::CONFIG;
use assyst_common::BOT_ID;
use lazy_static::lazy_static;
use serde_json::json;

use crate::assyst::ThreadSafeAssyst;

lazy_static! {
    pub static ref ROUTE: String = format!("https://top.gg/api/bots/{}/stats", BOT_ID);
}

pub async fn post_top_gg_stats(assyst: ThreadSafeAssyst) -> anyhow::Result<()> {
    let guild_count = assyst.prometheus.lock().await.guilds.get();
    let shard_count = assyst.shard_count;

    assyst
        .reqwest_client
        .post(&*ROUTE)
        .header("authorization", &CONFIG.authentication.top_gg_token)
        .json(&json!({ "server_count": guild_count, "shard_count": shard_count }))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
