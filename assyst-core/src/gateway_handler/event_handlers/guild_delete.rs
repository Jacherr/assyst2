use twilight_model::gateway::payload::incoming::GuildDelete;

pub fn handle(event: GuildDelete) {
    // check if we should handle this guild delete based on event.unavailable and
    // what the cache returns
    // check assyst 1 source
}
