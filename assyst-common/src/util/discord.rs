use regex::Regex;
use twilight_http::Client;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_model::user::User;

use super::format_time;
use super::regex::USER_MENTION;

pub const MAX_TIMESTAMP: u64 = 8640000000000000;

/// Attempts to resolve a guild's owner's user ID
pub async fn get_guild_owner(http: &Client, guild_id: u64) -> anyhow::Result<u64> {
    Ok(http
        .guild(Id::<GuildMarker>::new(guild_id))
        .await?
        .model()
        .await?
        .owner_id
        .get())
}

pub fn get_default_avatar_url(user: &User) -> String {
    // Unwrapping discrim parsing is ok, it should never be out of range or non-numeric
    let suffix = if user.discriminator == 0 {
        // Pomelo users
        (user.id.get().wrapping_shr(22) % 6) as u16
    } else {
        // Legacy
        user.discriminator % 5
    };
    format!("https://cdn.discordapp.com/embed/avatars/{suffix}.png?size=1024")
}

pub fn get_avatar_url(user: &User) -> String {
    let avatar = match &user.avatar {
        Some(av) => av,
        None => return get_default_avatar_url(user),
    };

    let ext = if avatar.bytes().starts_with("a_".as_bytes()) {
        "gif"
    } else {
        "png"
    };

    format!(
        "https://cdn.discordapp.com/avatars/{}/{}.{}?size=1024",
        user.id, avatar, ext
    )
}

pub fn id_from_mention(word: &str) -> Option<u64> {
    USER_MENTION
        .captures(word)
        .and_then(|user_id_capture| user_id_capture.get(1))
        .map(|id| id.as_str())
        .and_then(|id| id.parse::<u64>().ok())
}

pub fn format_tag(user: &User) -> String {
    format!("{}#{}", user.name, user.discriminator)
}

/// Generates a message link
pub fn message_link(guild_id: u64, channel_id: u64, message_id: u64) -> String {
    format!("https://discord.com/channels/{guild_id}/{channel_id}/{message_id}")
}

/// Generates a DM message link
pub fn dm_message_link(channel_id: u64, message_id: u64) -> String {
    format!("https://discord.com/channels/@me/{channel_id}/{message_id}")
}

/// Attempts to return the timestamp as a Discord timestamp,
/// and falls back to [`format_time`] if Discord were to render it as "Invalid Date"
pub fn format_discord_timestamp(input: u64) -> String {
    if input <= MAX_TIMESTAMP {
        format!("<t:{}:R>", input / 1000)
    } else {
        format_time(input)
    }
}

pub fn mention_to_id(s: &str) -> Option<u64> {
    let mention: Regex = Regex::new(r"(?:<@!?)?(\d{16,20})>?").unwrap();

    mention
        .captures(s)
        .and_then(|capture| capture.get(1))
        .map(|id| id.as_str())
        .and_then(|id| id.parse::<u64>().ok())
}
