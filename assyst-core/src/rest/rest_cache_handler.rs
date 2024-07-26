use std::hash::Hash;
use std::mem::size_of;
use std::sync::Arc;
use std::time::Duration;

use moka::sync::Cache;
use twilight_http::Client as HttpClient;
use twilight_model::guild::{Permissions, PremiumTier};
use twilight_model::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use twilight_model::id::Id;

use super::{NORMAL_DISCORD_UPLOAD_LIMIT_BYTES, PREMIUM_TIER2_DISCORD_UPLOAD_LIMIT_BYTES, PREMIUM_TIER3_DISCORD_UPLOAD_LIMIT_BYTES};

trait TCacheV = Send + Sync + Clone + 'static;
trait TCacheK = Hash + Send + Sync + Eq + Clone + 'static;

fn default_cache<K: TCacheK, V: TCacheV>() -> Cache<K, V> {
    Cache::builder().max_capacity(1000).time_to_idle(Duration::from_secs(60 * 5)).build()
}

/// Rest cache handler for any common data structures loaded from a network resource.
pub struct RestCacheHandler {
    http_client: Arc<HttpClient>,
    /// Guild ID -> Limit in BYTES
    guild_upload_limits: Cache<u64, u64>,
    /// Guild ID -> true/false
    channel_nsfw_status: Cache<u64, bool>,
    /// Guild ID -> User ID
    guild_owners: Cache<u64, u64>,
    /// List of all download URLs.
    web_download_urls: Cache<String, ()>,
}
impl RestCacheHandler {
    pub fn new(client: Arc<HttpClient>) -> RestCacheHandler {
        RestCacheHandler {
            http_client: client,
            guild_upload_limits: default_cache(),
            channel_nsfw_status: default_cache(),
            guild_owners: default_cache(),
            web_download_urls: Cache::builder().build(),
        }
    }

    pub fn size_of(&self) -> u64 {
        let mut size = 0;
        self.guild_upload_limits.run_pending_tasks();
        self.channel_nsfw_status.run_pending_tasks();
        self.guild_owners.run_pending_tasks();

        size += self.guild_upload_limits.entry_count() * size_of::<(u64, u64)>() as u64;
        size += self.channel_nsfw_status.entry_count() * size_of::<(u64, bool)>() as u64;
        size += self.guild_owners.entry_count() * size_of::<(u64, u64)>() as u64;
        size += self.web_download_urls.iter().fold(0, |acc, x| acc + x.0.as_bytes().len()) as u64;
        size
    }

    pub fn set_guild_upload_limit_bytes(&self, guild_id: u64, tier: PremiumTier) {
        let amount = match tier {
            PremiumTier::None | PremiumTier::Tier1 => NORMAL_DISCORD_UPLOAD_LIMIT_BYTES,
            PremiumTier::Tier2 => PREMIUM_TIER2_DISCORD_UPLOAD_LIMIT_BYTES,
            PremiumTier::Tier3 => PREMIUM_TIER3_DISCORD_UPLOAD_LIMIT_BYTES,
            PremiumTier::Other(_) => NORMAL_DISCORD_UPLOAD_LIMIT_BYTES,
            _ => NORMAL_DISCORD_UPLOAD_LIMIT_BYTES,
        };

        self.guild_upload_limits.insert(guild_id, amount);
    }

    pub async fn get_guild_upload_limit_bytes(&self, guild_id: u64) -> anyhow::Result<u64> {
        if let Some(amount) = self.guild_upload_limits.get(&guild_id) {
            return Ok(amount);
        };

        let guild = self.http_client.guild(Id::<GuildMarker>::new(guild_id)).await?.model().await?;

        let tier = guild.premium_tier;

        let amount = match tier {
            PremiumTier::None | PremiumTier::Tier1 => NORMAL_DISCORD_UPLOAD_LIMIT_BYTES,
            PremiumTier::Tier2 => PREMIUM_TIER2_DISCORD_UPLOAD_LIMIT_BYTES,
            PremiumTier::Tier3 => PREMIUM_TIER3_DISCORD_UPLOAD_LIMIT_BYTES,
            PremiumTier::Other(_) => NORMAL_DISCORD_UPLOAD_LIMIT_BYTES,
            _ => NORMAL_DISCORD_UPLOAD_LIMIT_BYTES,
        };

        self.guild_upload_limits.insert(guild_id, amount);

        Ok(amount)
    }

    pub fn update_channel_age_restricted_status(&self, channel_id: u64, status: bool) {
        self.channel_nsfw_status.insert(channel_id, status);
    }

    pub async fn channel_is_age_restricted(&self, channel_id: u64) -> anyhow::Result<bool> {
        if let Some(nsfw) = self.channel_nsfw_status.get(&channel_id) {
            return Ok(nsfw);
        }

        let nsfw = self.http_client.channel(Id::<ChannelMarker>::new(channel_id)).await?.model().await?.nsfw.unwrap_or(false);

        self.channel_nsfw_status.insert(channel_id, nsfw);

        Ok(nsfw)
    }

    pub fn set_guild_owner(&self, guild_id: u64, owner_id: u64) {
        self.guild_owners.insert(guild_id, owner_id);
    }

    pub async fn get_guild_owner(&self, guild_id: u64) -> anyhow::Result<u64> {
        if let Some(owner) = self.guild_owners.get(&guild_id) {
            return Ok(owner);
        }

        let owner = self.http_client.guild(Id::<GuildMarker>::new(guild_id)).await?.model().await?.owner_id.get();

        self.guild_owners.insert(guild_id, owner);

        Ok(owner)
    }

    /// Checks if a user is a guild manager, i.e., owns the server, has Administrator, or has Manage
    /// Server permissions.
    pub async fn user_is_guild_manager(&self, guild_id: u64, user_id: u64) -> anyhow::Result<bool> {
        // guild owner *or* manage server *or* admin
        // get owner
        let owner = self.get_guild_owner(guild_id).await?;

        // figure out permissions of the user through bitwise operations
        let member = self.http_client.guild_member(Id::<GuildMarker>::new(guild_id), Id::<UserMarker>::new(user_id)).await?.model().await.unwrap();

        let roles = self.http_client.roles(Id::<GuildMarker>::new(guild_id)).await?.models().await.expect("Failed to deserialize body when fetching guild roles");

        let member_roles = roles.iter().filter(|r| member.roles.contains(&r.id)).collect::<Vec<_>>();

        let member_permissions = member_roles.iter().fold(0, |a, r| a | r.permissions.bits());
        let member_is_manager = member_permissions & Permissions::ADMINISTRATOR.bits() == Permissions::ADMINISTRATOR.bits() || member_permissions & Permissions::MANAGE_GUILD.bits() == Permissions::MANAGE_GUILD.bits();

        Ok(owner == user_id || member_is_manager)
    }

    pub fn set_web_download_urls(&self, urls: Vec<String>) {
        for url in urls {
            self.web_download_urls.insert(url, ());
        }
    }

    pub fn get_web_download_urls(&self) -> Vec<Arc<String>> {
        self.web_download_urls.iter().map(|x| x.0).collect::<Vec<_>>()
    }
}
