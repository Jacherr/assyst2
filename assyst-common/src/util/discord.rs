use twilight_http::Client;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_model::user::User;

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

pub fn get_default_avatar_url(user: &User) -> String {
    // Unwrapping discrim parsing is ok, it should never be out of range or non-numeric
    let suffix = if user.discriminator == 0 {
        // Pomelo users
        (user.id.get().wrapping_shr(22) % 6) as u16
    } else {
        // Legacy
        user.discriminator % 5
    };
    format!("https://cdn.discordapp.com/embed/avatars/{}.png", suffix)
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
    format!("https://cdn.discordapp.com/avatars/{}/{}.{}", user.id, avatar, ext)
}
