use std::collections::HashMap;
use std::time::{Duration, Instant};

use assyst_common::err;
use tracing::{debug, warn};
use twilight_model::application::interaction::application_command::{
    CommandData as DiscordCommandData, CommandDataOption, CommandOptionValue,
};
use twilight_model::application::interaction::InteractionData;
use twilight_model::gateway::payload::incoming::InteractionCreate;

use super::after_command_execution_success;
use crate::assyst::ThreadSafeAssyst;
use crate::command::registry::find_command_by_name;
use crate::command::source::Source;
use crate::command::{
    CommandCtxt, CommandData, CommandGroupingInteractionInfo, ExecutionTimings, InteractionCommandParseCtxt,
};
use crate::gateway_handler::message_parser::error::{ErrorSeverity, GetErrorSeverity};

fn parse_subcommand_data(data: &DiscordCommandData) -> Option<(String, CommandOptionValue)> {
    if let Some(option_zero) = data.options.first()
        && let CommandOptionValue::SubCommand(_) = option_zero.value
    {
        Some((option_zero.name.clone(), option_zero.value.clone()))
    } else {
        None
    }
}

pub async fn handle(assyst: ThreadSafeAssyst, InteractionCreate(interaction): InteractionCreate) {
    if let Some(InteractionData::ApplicationCommand(command_data)) = interaction.data {
        let command = find_command_by_name(&command_data.name);
        let subcommand_data = parse_subcommand_data(&command_data);

        if let Some(command) = command {
            // we need to re-order the command options to match what assyst expects
            // todo: support both attachment and link for image inputs (when there is only one
            // attachment input)
            let command_interaction_options = match command.interaction_info() {
                CommandGroupingInteractionInfo::Command(x) => x.command_options,
                CommandGroupingInteractionInfo::Group(g) => {
                    let subcommand_name = subcommand_data
                        .clone()
                        .expect("somehow called base command on a subcommand tree?")
                        .0;
                    g.iter()
                        .find(|x| x.0 == subcommand_name)
                        .map(|x| x.1.command_options.clone())
                        .expect("invalid subcommand")
                },
            };

            let incoming_options = if let Some(d) = subcommand_data.clone() {
                match d.1 {
                    CommandOptionValue::SubCommand(s) => s,
                    _ => unreachable!(),
                }
            } else {
                command_data.options
            };

            let mut sorted_incoming_options = Vec::<CommandDataOption>::new();

            for option in command_interaction_options {
                let incoming_match = incoming_options.iter().find(|x| x.name == option.name);
                if let Some(op) = incoming_match {
                    sorted_incoming_options.push(op.clone());
                } else {
                    // default required: false
                    if option.required.unwrap_or(false) {
                        err!(
                            "expected required option {} for command {}, but it was missing",
                            option.name,
                            command.metadata().name
                        );
                        return;
                    }
                }
            }

            let interaction_subcommand = if let Some(d) = subcommand_data.clone() {
                match d.1 {
                    CommandOptionValue::SubCommand(_) => Some(d),
                    _ => unreachable!(),
                }
            } else {
                None
            };

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
                interaction_subcommand,
                channel_id: interaction.channel.unwrap().id,
                guild_id: interaction.guild_id,
                author: interaction.member.and_then(|x| x.user).or(interaction.user).unwrap(),
                interaction_token: Some(interaction.token),
                interaction_id: Some(interaction.id),
                interaction_attachments: command_data.resolved.map(|x| x.attachments).unwrap_or(HashMap::new()),
            };

            let ctxt = InteractionCommandParseCtxt::new(CommandCtxt::new(&data), &sorted_incoming_options);

            if let Err(err) = command.execute_interaction_command(ctxt.clone()).await {
                match err.get_severity() {
                    ErrorSeverity::Low => debug!("{err:?}"),
                    ErrorSeverity::High => {
                        let _ = ctxt.cx.reply(format!(":warning: ``{err:#}``")).await;
                    },
                }
            } else {
                let _ = after_command_execution_success(ctxt.cx, command)
                    .await
                    .map_err(|e| err!("Error handling post-command: {e:#}"));
            }
        } else {
            warn!(
                "Received interaction for non-existent command: {}, ignoring",
                command_data.name
            );
        }
    }
}
