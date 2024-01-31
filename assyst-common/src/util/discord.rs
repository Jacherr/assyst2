use twilight_http::Client;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

/// Attempts to resolve a guild's owner's user ID
pub async fn get_guild_owner(http: &Client, guild_id: u64) -> anyhow::Result<u64> {
    Ok(http
        .guild(Id::<GuildMarker>::new(guild_id))
        .await?
        .model()
        .await
        .unwrap()
        .owner_id
        .get())
}
