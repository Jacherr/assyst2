use std::time::{Duration, Instant};

use assyst_common::err;
use tracing::{debug, info, warn};
use twilight_model::application::interaction::application_command::{
    CommandData as DiscordCommandData, CommandDataOption, CommandOptionValue,
};
use twilight_model::application::interaction::InteractionData;
use twilight_model::gateway::payload::incoming::InteractionCreate;
use twilight_model::http::interaction::InteractionResponse;
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::assyst::ThreadSafeAssyst;
use crate::command::errors::{ExecutionError, TagParseError};
use crate::command::registry::find_command_by_name;
use crate::command::source::Source;
use crate::command::{CommandCtxt, CommandData, ExecutionTimings, InteractionCommandParseCtxt};
use crate::gateway_handler::message_parser::error::{ErrorSeverity, GetErrorSeverity};

use super::after_command_execution_success;

/// (Full name, is_subcommand)
fn parse_full_command_name_from_interaction_data(data: &DiscordCommandData) -> (String, bool) {
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
        // todo: follow subcommands to find "real" name for commands like tag
        let command = find_command_by_name(&command_data.name);

        if let Some(command) = command {
            // we need to re-order the command options to match what assyst expects
            // todo: support both attachment and link for image inputs (when there is only one attachment input)
            let command_interaction_options = command.interaction_info().command_options;
            let mut sorted_incoming_options = Vec::<CommandDataOption>::new();

            for option in command_interaction_options {
                let incoming_match = command_data.options.iter().find(|x| x.name == option.name);
                if let Some(op) = incoming_match {
                    sorted_incoming_options.push(op.clone());
                } else {
                    // default required: false
                    if !option.required.unwrap_or(false) {
                        err!(
                            "expected required option {} for command {}, but it was missing",
                            option.name,
                            command.metadata().name
                        );
                        return;
                    }
                }
            }

            println!("{sorted_incoming_options:#?}");
            println!("{:#?}", command.interaction_info().command_options);

            let data = CommandData {
                source: Source::Interaction,
                assyst: &assyst,
                execution_timings: ExecutionTimings {
                    parse_total: Duration::from_secs(0),
                    prefix_determiner: Duration::from_secs(0),
                    preprocess_total: Duration::from_secs(0),
                    processing_time_start: Instant::now(),
                    metadata_check_start: Instant::now(),
                },
                calling_prefix: "/".to_owned(),
                message: None,
                interaction_subcommand: None,
                channel_id: interaction.channel.unwrap().id,
                guild_id: interaction.guild_id,
                author: interaction
                    .member
                    .map(|x| x.user)
                    .flatten()
                    .or(interaction.user)
                    .unwrap(),
                interaction_token: Some(interaction.token),
                interaction_id: Some(interaction.id),
            };

            let ctxt = InteractionCommandParseCtxt::new(CommandCtxt::new(&data), &sorted_incoming_options);

            if let Err(err) = command.execute_interaction_command(ctxt.clone()).await {
                match err.get_severity() {
                    ErrorSeverity::Low => debug!("{err:?}"),
                    ErrorSeverity::High => {
                        let _ = ctxt.cx.reply(format!(":warning: ``{err}``")).await;
                    },
                }
            } else {
                after_command_execution_success(ctxt.cx, command);
            }
        } else {
            warn!(
                "Received interaction for non-existent command: {}, ignoring",
                command_data.name
            );
            return;
        }
    }
}
