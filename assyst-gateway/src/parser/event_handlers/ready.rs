use assyst_common::ansi::Ansi;
use tracing::info;
use twilight_model::gateway::payload::incoming::Ready;

/// Handle a shard sending a READY event.
///
/// READY events are not particularly interesting, but it can be useful to see if any shards are
/// resetting often. In addition, it provides a good gauge as to how much of the bot has started up,
/// after a gateway restart.
pub fn handle(event: Ready) {
    if let Some(shard) = event.shard {
        info!(
            "Shard {} (total {}): {} in {} guilds",
            shard.number(),
            shard.total(),
            "READY".fg_green(),
            event.guilds.len()
        )
    }
}
