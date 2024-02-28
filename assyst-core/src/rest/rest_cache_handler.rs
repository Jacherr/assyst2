use moka::sync::Cache;
use std::sync::Arc;
use std::time::Duration;
use twilight_http::Client as HttpClient;
use twilight_model::guild::PremiumTier;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

use super::{
    NORMAL_DISCORD_UPLOAD_LIMIT_BYTES, PREMIUM_TIER2_DISCORD_UPLOAD_LIMIT_BYTES,
    PREMIUM_TIER3_DISCORD_UPLOAD_LIMIT_BYTES,
};

pub struct RestCacheHandler {
    http_client: Arc<HttpClient>,
    guild_upload_limits: Cache<u64, u64>,
}
impl RestCacheHandler {
    pub fn new(client: Arc<HttpClient>) -> RestCacheHandler {
        RestCacheHandler {
            http_client: client,
            guild_upload_limits: Cache::builder()
                .max_capacity(1000)
                .time_to_idle(Duration::from_secs(60 * 5))
                .build(),
        }
    }

    pub fn size_of(&self) -> u64 {
        let mut size = 0;
        self.guild_upload_limits.run_pending_tasks();
        size += self.guild_upload_limits.entry_count() * 16 /* key + value */;
        size
    }

    pub async fn get_guild_upload_limit_bytes(&self, guild_id: u64) -> anyhow::Result<u64> {
        if let Some(amount) = self.guild_upload_limits.get(&guild_id) {
            return Ok(amount);
        };

        let guild = self
            .http_client
            .guild(Id::<GuildMarker>::new(guild_id))
            .await?
            .model()
            .await?;

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
}
