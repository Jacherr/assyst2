use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot::Sender;
use twilight_model::gateway::payload::incoming::{GuildCreate, GuildDelete, Ready};

/// A cache job, along with a transmitter to send the response back to the calling thread.
pub type CacheJobSend = (Sender<CacheResponseSend>, CacheJob);

/// The different jobs that the cache needs to handle.
#[derive(Serialize, Deserialize, Debug)]
pub enum CacheJob {
    /// Storing data from a GUILD_CREATE event.
    HandleGuildCreate(GuildCreateData),
    /// Storing data from a GUILD_DELETE event.
    HandleGuildDelete(GuildDeleteData),
    /// Storing data from a READY event.
    HandleReady(ReadyData),
    GetGuildCount,
}
impl Display for CacheJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HandleReady(x) => write!(f, "HandleReady ({} guilds)", x.guilds.len()),
            Self::HandleGuildCreate(x) => write!(f, "HandleGuildCreate (ID: {})", x.id),
            Self::HandleGuildDelete(x) => write!(f, "HandleGuildDelete (ID: {})", x.id),
            Self::GetGuildCount => f.write_str("GetGuildCount"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadyData {
    pub guilds: Vec<u64>,
}
impl From<Ready> for ReadyData {
    fn from(value: Ready) -> Self {
        ReadyData {
            guilds: value.guilds.iter().map(|x| x.id.get()).collect::<Vec<_>>(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildCreateData {
    pub id: u64,
    pub name: String,
    pub members: Option<u64>,
}
impl From<GuildCreate> for GuildCreateData {
    fn from(value: GuildCreate) -> Self {
        GuildCreateData {
            id: value.id.get(),
            name: value.name.clone(),
            members: value.member_count,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildDeleteData {
    pub id: u64,
    pub unavailable: bool,
}
impl From<GuildDelete> for GuildDeleteData {
    fn from(value: GuildDelete) -> Self {
        GuildDeleteData {
            id: value.id.get(),
            unavailable: value.unavailable,
        }
    }
}

pub type CacheResponseSend = anyhow::Result<CacheResponse>;

/// All the responses the cache can send back. Usually it is a 1-1 relation between a CacheJob
/// variant and CacheResponse variant.
#[derive(Serialize, Deserialize, Debug)]
pub enum CacheResponse {
    /// Whether Assyst should handle a GUILD_CREATE event. False if this guild is coming back from
    /// unavailable, or if this guild has already been cached.
    ShouldHandleGuildCreate(bool),
    /// Whether Assyst should handle a GUILD_DELETE event. False if this guild went unavailable, or
    /// if it was not in the cache.
    ShouldHandleGuildDelete(bool),
    /// The amount of new guilds Assyst receives when a shard enters a READY state. Some guilds may
    /// be duplicated, which is why this number may differ from the length of the guilds array in
    /// this event.
    NewGuildsFromReady(u64),
    /// Total number of cached guilds.
    TotalGuilds(u64),
}
