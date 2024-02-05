use assyst_common::cache::ReadyData;
use dashmap::DashSet;

pub struct GuildCache {
    ids: DashSet<u64>,
}
impl GuildCache {
    pub fn new() -> GuildCache {
        GuildCache { ids: DashSet::new() }
    }

    /// Handles a READY event, caching its guilds. Returns the number of newly cached guilds.
    pub fn handle_ready_event(&mut self, event: ReadyData) -> u64 {
        let mut new_guilds = 0;

        for guild in event.guilds {
            if self.ids.insert(guild) {
                new_guilds += 1;
            };
        }

        new_guilds
    }
}
