use assyst_common::err;
use tracing::{info, warn};
use twilight_model::application::interaction::application_command::{CommandData, CommandOptionValue};
use twilight_model::application::interaction::InteractionData;
use twilight_model::gateway::payload::incoming::InteractionCreate;
use twilight_model::http::interaction::InteractionResponse;
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::assyst::ThreadSafeAssyst;
use crate::command::registry::find_command_by_name;

/// (Full name, is_subcommand)
fn parse_full_command_name_from_interaction_data(data: &CommandData) -> (String, bool) {
    let mut is_subcommand = false;
    let mut name = data.name.to_owned();
    if let Some(option_zero) = data.options.get(0)
        && let CommandOptionValue::SubCommand(_) = option_zero.value
    {
        is_subcommand = true;
        name += " ";
        name += &option_zero.name;
    }
    (name, is_subcommand)
}

pub async fn handle(assyst: ThreadSafeAssyst, InteractionCreate(interaction): InteractionCreate) {
    if let Some(InteractionData::ApplicationCommand(command_data)) = interaction.data {
        let (full_name, is_subcommand) = parse_full_command_name_from_interaction_data(&command_data);
        println!("{command_data:?}");

        // todo: follow subcommands to find "real" name for commands like tag
        let command = find_command_by_name(&command_data.name);

        if let Some(command) = command {
        } else {
            warn!(
                "Received interaction for non-existent command: {}, ignoring",
                command_data.name
            );
        }

        /*
        let data = InteractionResponseDataBuilder::new().content("pong!").build();

        let response = InteractionResponse {
            kind: twilight_model::http::interaction::InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        let r = assyst
            .interaction_client()
            .create_response(interaction.id, &interaction.token, &response)
            .await;
        if let Err(r) = r {
            err!("Invalid response: {}", r.to_string());
        }*/
    }
}
