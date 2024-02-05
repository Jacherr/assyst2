use tracing::info;
use twilight_model::gateway::payload::incoming::GuildCreate;

use crate::assyst::ThreadSafeAssyst;

pub async fn handle(assyst: ThreadSafeAssyst, event: GuildCreate) {
    let cache_response = assyst.cache_handler.handle_guild_create_event(event.clone()).await;
    let should_handle = cache_response.unwrap_or(false);
    if should_handle {
        info!(
            "Joined guild {}: {} ({} members)",
            event.id.get(),
            event.name,
            event.member_count.unwrap_or(0)
        );
        assyst.prometheus.lock().await.inc_guilds();
    }
}
