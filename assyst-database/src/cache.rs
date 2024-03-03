use moka::sync::Cache;
use std::hash::Hash;
use std::mem::size_of;
use std::time::Duration;

use crate::model::prefix::Prefix;

trait TCacheV = Send + Sync + Clone + 'static;
trait TCacheK = Hash + Send + Sync + Eq + Clone + 'static;

fn default_cache<K: TCacheK, V: TCacheV>() -> Cache<K, V> {
    Cache::builder()
        .max_capacity(1000)
        .time_to_idle(Duration::from_secs(60 * 5))
        .build()
}

/// In-memory cache collection for frequently accessed areas of the database.
pub struct DatabaseCache {
    prefixes: Cache<u64, Prefix>,
    global_blacklist: Cache<u64, bool>,
}
impl DatabaseCache {
    pub fn new() -> Self {
        DatabaseCache {
            prefixes: default_cache(),
            global_blacklist: default_cache(),
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

    pub fn get_user_global_blacklist(&self, user_id: u64) -> Option<bool> {
        self.global_blacklist.get(&user_id)
    }

    pub fn set_user_global_blacklist(&self, user_id: u64, blacklisted: bool) {
        self.global_blacklist.insert(user_id, blacklisted);
    }

    pub fn size_of(&self) -> u64 {
        self.prefixes.run_pending_tasks();
        self.global_blacklist.run_pending_tasks();

        let mut size = 0;

        for prefix in self.prefixes.iter() {
            // add key size
            size += size_of::<u64>() as u64;
            // add value size
            size += prefix.1.size_of();
        }

        size += self.global_blacklist.entry_count() * size_of::<(u64, bool)>() as u64;

        size
    }
}
