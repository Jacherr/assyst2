use std::time::Duration;

use moka::sync::Cache;

use crate::model::prefix::Prefix;

/// In-memory cache collection for frequently accessed areas of the database.
pub struct DatabaseCache {
    prefixes: Cache<u64, Prefix>,
}
impl DatabaseCache {
    pub fn new() -> Self {
        DatabaseCache {
            // 10,000 entries max, if prefix not accessed in 5 mins then remove from cache
            prefixes: Cache::builder()
                .max_capacity(10000)
                .time_to_idle(Duration::from_secs(60 * 5))
                .build(),
        }
    }

    pub fn get_prefix(&self, guild_id: u64) -> Option<Prefix> {
        self.prefixes.get(&guild_id)
    }

    pub fn set_prefix(&mut self, guild_id: u64, prefix: Prefix) {
        self.prefixes.insert(guild_id, prefix);
    }

    pub fn get_prefixes_cache_size(&self) -> usize {
        self.prefixes.run_pending_tasks();
        self.prefixes.entry_count() as usize
    }
}
