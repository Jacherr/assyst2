use moka::sync::Cache;

use crate::model::prefix::Prefix;

/// In-memory cache collection for frequently accessed areas of the database.
pub struct DatabaseCache {
    prefixes: Cache<u64, Prefix>,
}
impl DatabaseCache {
    pub fn new() -> Self {
        DatabaseCache {
            prefixes: Cache::new(1000),
        }
    }

    pub fn get_prefix(&self, guild_id: u64) -> Option<Prefix> {
        self.prefixes.get(&guild_id)
    }

    pub fn set_prefix(&mut self, guild_id: u64, prefix: Prefix) {
        self.prefixes.insert(guild_id, prefix);
    }
}
