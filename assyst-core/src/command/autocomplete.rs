use assyst_common::err;
use twilight_model::application::command::{CommandOptionChoice, CommandOptionChoiceValue};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::{GuildMarker, InteractionMarker};
use twilight_model::id::Id;
use twilight_util::builder::InteractionResponseDataBuilder;

use super::misc::tag::tag_names_autocomplete;
use super::services::cooltext::cooltext_options_autocomplete;
use crate::assyst::ThreadSafeAssyst;

const SUGG_LIMIT: usize = 25;

pub async fn handle_autocomplete(
    assyst: ThreadSafeAssyst,
    interaction_id: Id<InteractionMarker>,
    interaction_token: String,
    guild_id: Option<Id<GuildMarker>>,
    command_full_name: &str,
    option: &str,
    text_to_autocomplete: &str,
) {
    // FIXME: minimise hardcoding strings etc as much as possible
    // future improvement is to use callbacks, but quite a lot of work
    // considering this is only used in a small handful of places
    let opts = match command_full_name {
        "cooltext create" => cooltext_options_autocomplete(),
        // FIXME: this unwrap needs handling properly when tags come to dms etc
        "tag run" => tag_names_autocomplete(assyst.clone(), guild_id.unwrap().get()).await,
        _ => {
            err!("Trying to autocomplete for invalid command: {command_full_name} (arg {option})");
            return;
        },
    };

    let suggestions = get_autocomplete_suggestions(text_to_autocomplete, &opts);

    let b = InteractionResponseDataBuilder::new();
    let b = b.choices(suggestions);
    let r = b.build();
    let r = InteractionResponse {
        kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
        data: Some(r),
    };

    if let Err(e) = assyst
        .interaction_client()
        .create_response(interaction_id, &interaction_token, &r)
        .await
    {
        err!("Failed to send autocomplete options: {e:?}");
    };
}

pub fn get_autocomplete_suggestions(text_to_autocomplete: &str, options: &[String]) -> Vec<CommandOptionChoice> {
    options
        .iter()
        .filter(|x| {
            x.to_ascii_lowercase()
                .starts_with(&text_to_autocomplete.to_ascii_lowercase())
        })
        .take(SUGG_LIMIT)
        .map(|x| CommandOptionChoice {
            name: x.clone(),
            name_localizations: None,
            // FIXME: hardcoded string type
            value: CommandOptionChoiceValue::String(x.clone()),
        })
        .collect::<Vec<_>>()
}
