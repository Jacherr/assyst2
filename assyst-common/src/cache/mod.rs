use serde::{Deserialize, Serialize};
use tokio::sync::oneshot::Sender;
use twilight_model::gateway::payload::incoming::{GuildCreate, GuildDelete, Ready};
use twilight_model::guild::UnavailableGuild;

/// A cache job, along with a transmitter to send the response back to the calling thread.
pub type CacheJobSend = (Sender<CacheResponseSend>, CacheJob);

/// The different jobs that the cache needs to handle.
#[derive(Serialize, Deserialize, Debug)]
pub enum CacheJob {
    /// Storing data from a GUILD_CREATE event.
    //HandleGuildCreate(GuildCreate),
    /// Storing data from a GUILD_DELETE event.
    //HandleGuildDelete(GuildDelete),
    /// Storing data from a READY event.
    HandleReady(ReadyData),
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
}
