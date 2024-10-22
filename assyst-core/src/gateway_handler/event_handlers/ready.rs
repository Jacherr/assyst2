use assyst_common::config::CONFIG;
use assyst_common::err;
use assyst_string_fmt::Ansi;
use tracing::{error, info};
use twilight_model::gateway::payload::incoming::Ready;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::Id;

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
        );
    }

    if event.guilds.iter().any(|x| x.id.get() == CONFIG.dev.dev_guild) && CONFIG.dev.dev_message {
        let channel = Id::<ChannelMarker>::new(CONFIG.dev.dev_channel);
        let _ = assyst
            .http_client
            .create_message(channel)
            .content("Dev shard is READY!")
            .await
            .inspect_err(|e| error!("FAILED to send shard ready message: {}", e.to_string()));
    }

    match assyst.persistent_cache_handler.handle_ready_event(event).await {
        Ok(num) => {
            info!("Adding {num} guilds to prometheus metrics from READY event");
            assyst.metrics_handler.add_guilds(num);
        },
        Err(e) => {
            err!("assyst-cache failed to handle READY event: {}", e.to_string());
        },
    }
}
