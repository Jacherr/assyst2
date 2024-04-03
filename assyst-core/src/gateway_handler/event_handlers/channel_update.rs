use tracing::debug;
use twilight_model::gateway::payload::incoming::ChannelUpdate;

use crate::assyst::ThreadSafeAssyst;

pub fn handle(assyst: ThreadSafeAssyst, event: ChannelUpdate) {
    if let Some(nsfw) = event.nsfw {
        assyst
            .rest_cache_handler
            .update_channel_age_restricted_status(event.id.get(), nsfw);

        debug!("Updated channel {} age restricted status to {nsfw}", event.id.get());
    };
}
