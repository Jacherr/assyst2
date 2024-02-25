use assyst_common::err;
use tracing::info;
use twilight_model::gateway::payload::incoming::GuildCreate;

use crate::assyst::ThreadSafeAssyst;

pub async fn handle(assyst: ThreadSafeAssyst, event: GuildCreate) {
    let id = event.id.get();
    let name = event.name.clone();
    let member_count = event.member_count.unwrap_or(0);

    let should_handle = match assyst.cache_handler.handle_guild_create_event(event).await {
        Ok(s) => s,
        Err(e) => {
            err!("assyst-cache failed to handle GUILD_CREATE event: {}", e.to_string());
            return;
        },
    };

    if should_handle {
        info!("Joined guild {}: {} ({} members)", id, name, member_count);
        assyst.prometheus.inc_guilds();
    }
}
