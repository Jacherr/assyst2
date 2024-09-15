use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_model::user::User;

pub struct AutocompleteData {
    pub guild_id: Option<Id<GuildMarker>>,
    pub user: User,
    pub subcommand: Option<String>,
}

pub const SUGG_LIMIT: usize = 25;
