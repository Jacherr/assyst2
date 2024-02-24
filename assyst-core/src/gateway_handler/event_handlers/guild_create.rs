use assyst_common::err;
use tracing::info;
use twilight_model::gateway::payload::incoming::GuildCreate;

use crate::assyst::ThreadSafeAssyst;

pub async fn handle(assyst: ThreadSafeAssyst, event: GuildCreate) {
    let should_handle = match assyst.cache_handler.handle_guild_create_event(event.clone()).await {
        Ok(s) => s,
        Err(e) => {
            err!("assyst-cache failed to handle GUILD_CREATE event: {}", e.to_string());
            return;
        },
    };

    if should_handle {
        info!(
            "Joined guild {}: {} ({} members)",
            event.id.get(),
            event.name,
            event.member_count.unwrap_or(0)
        );
        assyst.prometheus.inc_guilds();
    }
}
