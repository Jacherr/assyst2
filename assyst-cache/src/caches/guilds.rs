use std::hash::BuildHasherDefault;

use assyst_common::cache::{GuildCreateData, GuildDeleteData, ReadyData};
use dashmap::DashSet;

/// The cache of all guilds Assyst is part of. When a shard readies up, it receives a list of all
/// guild IDs that shard is responsible for, which are all cached here. In addition, when a shard is
/// ready, it will receive GUILD_CREATE events for every guild that shard is responsible for. This
/// cache allows to differentiate between GUILD_CREATE events sent as part of this procedure (since
/// they were also part of the READY event) and legitimate GUILD_CREATEs fired as a result of Assyst
/// joining a new guild post-ready.
pub struct GuildCache {
    ids: DashSet<u64, BuildHasherDefault<rustc_hash::FxHasher>>,
}
impl GuildCache {
    pub fn new() -> GuildCache {
        GuildCache {
            ids: DashSet::with_hasher(BuildHasherDefault::<rustc_hash::FxHasher>::default()),
        }
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

    /// Handles a GUILD_CREATE. This method returns a bool which states if this guild is new or not.
    /// A new guild is one that was not received during the start-up of the gateway connection.
    pub fn handle_guild_create_event(&mut self, event: GuildCreateData) -> bool {
        self.ids.insert(event.id)
    }

    /// Handles a GUILD_DELETE. This method returns a bool which states if the bot was actually
    /// kicked from this guild.
    pub fn handle_guild_delete_event(&mut self, event: GuildDeleteData) -> bool {
        !event.unavailable && self.ids.remove(&event.id).is_some()
    }

    pub fn size(&self) -> u64 {
        self.ids.len() as u64
    }
}
