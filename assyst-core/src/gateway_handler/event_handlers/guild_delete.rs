use tracing::info;
use twilight_model::gateway::payload::incoming::GuildDelete;

use crate::assyst::ThreadSafeAssyst;

pub async fn handle(assyst: ThreadSafeAssyst, event: GuildDelete) {
    let cache_response = assyst.cache_handler.handle_guild_delete_event(event.clone()).await;
    let should_handle = cache_response.unwrap_or(false);
    if should_handle {
        info!("Removed from guild {}", event.id.get());
        assyst.prometheus.lock().await.dec_guilds();
    }
}
