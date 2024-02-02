use serde::{Deserialize, Serialize};
use twilight_model::gateway::payload::incoming::{GuildCreate, GuildDelete, Ready};

#[derive(Serialize, Deserialize)]
pub enum CacheJob {
    HandleGuildCreate(GuildCreate),
    HandleGuildDelete(GuildDelete),
    HandleReady(Ready),
}
