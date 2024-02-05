use assyst_common::ansi::Ansi;
use tracing::{error, info};
use twilight_model::gateway::payload::incoming::Ready;

use crate::assyst::ThreadSafeAssyst;

/// Handle a shard sending a READY event.
///
/// READY events are not particularly interesting, but it can be useful to see if any shards are
/// resetting often. In addition, it provides a good gauge as to how much of the bot has started up,
/// after a gateway restart.
pub async fn handle(assyst: ThreadSafeAssyst, event: Ready) {
    if let Some(shard) = event.shard {
        info!(
            "Shard {} (total {}): {} in {} guilds",
            shard.number(),
            shard.total(),
            "READY".fg_green(),
            event.guilds.len()
        )
    }

    match assyst.cache_handler.handle_ready_event(event).await {
        Ok(num) => {
            info!("Adding {num} guilds to prometheus metrics from READY event");
            assyst.prometheus.lock().await.add_guilds(num);
        },
        Err(e) => {
            error!("assyst-cache failed to handle READY event: {}", e.to_string());
        },
    }
}
