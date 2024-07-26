use tracing::debug;
use twilight_model::gateway::payload::incoming::GuildUpdate;

use crate::assyst::ThreadSafeAssyst;

pub fn handle(assyst: ThreadSafeAssyst, event: GuildUpdate) {
    assyst
        .rest_cache_handler
        .set_guild_owner(event.id.get(), event.owner_id.get());

    assyst
        .rest_cache_handler
        .set_guild_upload_limit_bytes(event.id.get(), event.premium_tier);

    debug!("Updated guild {} cache info", event.id.get());
}
