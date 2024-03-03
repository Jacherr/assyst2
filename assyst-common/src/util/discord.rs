use twilight_model::user::User;

pub fn get_default_avatar_url(user: &User) -> String {
    // Unwrapping discrim parsing is ok, it should never be out of range or non-numeric
    let suffix = if user.discriminator == 0 {
        // Pomelo users
        (user.id.get().wrapping_shr(22) % 6) as u16
    } else {
        // Legacy
        user.discriminator % 5
    };
    format!("https://cdn.discordapp.com/embed/avatars/{}.png?size=1024", suffix)
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
