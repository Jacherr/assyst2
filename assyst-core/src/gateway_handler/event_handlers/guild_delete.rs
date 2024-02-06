use assyst_common::err;
use tracing::info;
use twilight_model::gateway::payload::incoming::GuildDelete;

use crate::assyst::ThreadSafeAssyst;

pub async fn handle(assyst: ThreadSafeAssyst, event: GuildDelete) {
    let should_handle = match assyst.cache_handler.handle_guild_delete_event(event.clone()).await {
        Ok(s) => s,
        Err(e) => {
            err!("assyst-cache failed to handle GUILD_DELETE event: {}", e.to_string());
            return;
        },
    };

    if should_handle {
        info!("Removed from guild {}", event.id.get());
        assyst.prometheus.lock().await.dec_guilds();
    }
}
