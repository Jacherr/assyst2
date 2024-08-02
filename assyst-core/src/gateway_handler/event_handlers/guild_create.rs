use assyst_common::err;
use tracing::info;
use twilight_model::gateway::payload::incoming::GuildCreate;

use crate::assyst::ThreadSafeAssyst;

pub async fn handle(assyst: ThreadSafeAssyst, event: GuildCreate) {
    let id = event.id().get();
    let (name, member_count) = match &event {
        GuildCreate::Available(g) => (g.name.clone(), g.member_count.unwrap_or(0)),
        GuildCreate::Unavailable(_) => (String::new(), 0),
    };

    let should_handle = match assyst.persistent_cache_handler.handle_guild_create_event(event).await {
        Ok(s) => s,
        Err(e) => {
            err!("assyst-cache failed to handle GUILD_CREATE event: {}", e);
            return;
        },
    };

    if should_handle {
        info!("Joined guild {}: {} ({} members)", id, name, member_count);
        assyst.metrics_handler.inc_guilds();
    }
}
