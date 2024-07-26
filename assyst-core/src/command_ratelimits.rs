use std::time::{Duration, Instant};

use moka::sync::Cache;

/// All command ratelimits, in the format <(guild/user id, command name) => time command was
/// ran>
pub struct CommandRatelimits(Cache<(u64, &'static str), Instant>);
impl CommandRatelimits {
    pub fn new() -> Self {
        Self(Cache::builder().max_capacity(1000).time_to_idle(Duration::from_secs(60 * 5)).build())
    }

    pub fn insert(&self, id: u64, command_name: &'static str, value: Instant) {
        self.0.insert((id, command_name), value);
    }

    pub fn get(&self, id: u64, command_name: &'static str) -> Option<Instant> {
        self.0.get(&(id, command_name))
    }
}
