use std::collections::HashSet;
use std::hash::Hash;
use std::mem::size_of;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use moka::sync::Cache;

use crate::model::prefix::Prefix;

trait TCacheV = Send + Sync + Clone + 'static;
trait TCacheK = Hash + Send + Sync + Eq + Clone + 'static;

fn default_cache<K: TCacheK, V: TCacheV>() -> Cache<K, V> {
    Cache::builder()
        .max_capacity(1000)
        .time_to_idle(Duration::from_secs(60 * 5))
        .build()
}

fn default_cache_sized<K: TCacheK, V: TCacheV>(size: u64) -> Cache<K, V> {
    Cache::builder()
        .max_capacity(size)
        .time_to_idle(Duration::from_secs(60 * 5))
        .build()
}

/// In-memory cache collection for frequently accessed areas of the database.
pub struct DatabaseCache {
    prefixes: Cache<u64, Prefix>,
    global_blacklist: Cache<u64, bool>,
    disabled_commands: Cache<u64, Arc<Mutex<HashSet<String>>>>,
}
impl DatabaseCache {
    pub fn new() -> Self {
        DatabaseCache {
            prefixes: default_cache_sized(100000),
            global_blacklist: default_cache(),
            disabled_commands: default_cache(),
        }
    }

    pub fn get_prefix(&self, guild_id: u64) -> Option<Prefix> {
        self.prefixes.get(&guild_id)
    }

    pub fn set_prefix(&self, guild_id: u64, prefix: Prefix) {
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

    pub fn get_guild_disabled_commands(&self, guild_id: u64) -> Option<Arc<Mutex<HashSet<String>>>> {
        self.disabled_commands.get(&guild_id)
    }

    pub fn get_guild_disabled_commands_size(&self) -> usize {
        self.disabled_commands.run_pending_tasks();
        self.disabled_commands.entry_count() as usize
    }

    pub fn set_command_disabled(&self, guild_id: u64, command: &str) {
        let disabled_commands = self.get_guild_disabled_commands(guild_id);
        if let Some(old) = disabled_commands {
            let mut lock = old.lock().unwrap();
            let cmd = command.to_owned();
            lock.insert(cmd);
        } else {
            self.disabled_commands.insert(
                guild_id,
                Arc::new(Mutex::new(HashSet::from_iter(vec![command.to_owned()]))),
            );
        }
    }

    pub fn set_command_enabled(&self, guild_id: u64, command: &str) {
        let disabled_commands = self.get_guild_disabled_commands(guild_id);
        if let Some(old) = disabled_commands {
            let mut lock = old.lock().unwrap();
            lock.remove(command);
        }
    }

    pub fn reset_disabled_commands_for(&self, guild_id: u64) {
        self.disabled_commands
            .insert(guild_id, Arc::new(Mutex::new(HashSet::new())));
    }

    pub fn size_of(&self) -> u64 {
        self.prefixes.run_pending_tasks();
        self.global_blacklist.run_pending_tasks();
        self.disabled_commands.run_pending_tasks();

        let mut size = 0;

        for prefix in self.prefixes.iter() {
            // add key size
            size += size_of::<u64>() as u64;
            // add value size
            size += prefix.1.size_of();
        }

        for command in self.disabled_commands.iter() {
            // add key size
            size += size_of::<u64>() as u64;
            // add value size - approximate
            size += command
                .1
                .lock()
                .unwrap()
                .iter()
                .cloned()
                .collect::<Vec<String>>()
                .join("")
                .len() as u64;
        }

        size += self.global_blacklist.entry_count() * size_of::<(u64, bool)>() as u64;

        size
    }
}

impl Default for DatabaseCache {
    fn default() -> Self {
        Self::new()
    }
}
