use std::collections::HashMap;
use std::time::{Duration, Instant};

use assyst_common::config::CONFIG;
use assyst_common::err;
use assyst_database::model::active_guild_premium_entitlement::ActiveGuildPremiumEntitlement;
use tracing::{debug, info, warn};
use twilight_model::application::command::CommandType;
use twilight_model::application::interaction::application_command::{
    CommandData as DiscordCommandData, CommandDataOption, CommandOptionValue,
};
use twilight_model::application::interaction::{InteractionContextType, InteractionData, InteractionType};
use twilight_model::channel::Message;
use twilight_model::gateway::payload::incoming::InteractionCreate;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::user::User;
use twilight_model::util::Timestamp;
use twilight_util::builder::InteractionResponseDataBuilder;

use super::after_command_execution_success;
use crate::assyst::ThreadSafeAssyst;
use crate::command::autocomplete::AutocompleteData;
use crate::command::componentctxt::ComponentInteractionData;
use crate::command::errors::ExecutionError;
use crate::command::registry::find_command_by_name;
use crate::command::source::Source;
use crate::command::{
    CommandCtxt, CommandData, CommandGroupingInteractionInfo, ExecutionTimings, InteractionCommandParseCtxt,
    check_metadata,
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
    // look at entitlements to see if there is anything new - we can cache this if so
    // this usually shouldnt happen except for some edge cases such as a new entitlement was created
    // when the bot was down
    let entitlements = interaction.entitlements.clone();
    let lock = assyst.entitlements.lock().unwrap().clone();
    let mut new = vec![];
    for entitlement in entitlements {
        if let Some(guild_id) = interaction.guild_id
            && let Some(user_id) = entitlement.user_id
            && entitlement.sku_id.get() == CONFIG.entitlements.premium_server_sku_id
            && !lock.contains_key(&(guild_id.get() as i64))
        {
            warn!("New entitlement for guild {}, registering", guild_id.get());
            let active = ActiveGuildPremiumEntitlement {
                entitlement_id: entitlement.id.get() as i64,
                guild_id: guild_id.get() as i64,
                user_id: user_id.get() as i64,
                started_unix_ms: (entitlement
                    .starts_at
                    .unwrap_or(Timestamp::from_micros(0).unwrap())
                    .as_micros()
                    / 1000),
                expiry_unix_ms: (entitlement
                    .ends_at
                    .unwrap_or(Timestamp::from_micros(0).unwrap())
                    .as_micros()
                    / 1000),
            };
            new.push(active);
        }
    }
    for a in new {
        match a.set(&assyst.database_handler).await {
            Err(e) => {
                err!("Error registering new entitlement {}: {e:?}", a.entitlement_id);
            },
            _ => {},
        }
    }

    if interaction.kind == InteractionType::ApplicationCommand
        && let Some(InteractionData::ApplicationCommand(command_data)) = interaction.data
    {
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
                // context menu commands have no options and are handled independently
                } else if command_data.kind != CommandType::Message {
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

            let interaction_attachments = command_data.resolved.clone().map_or(HashMap::new(), |x| x.attachments);

            // resolve messages for context menu message commands
            let mut resolved_messages: Option<Vec<Message>> = None;
            if let Some(ms) = command_data.resolved.as_ref().map(|x| &x.messages)
                && command_data.kind == CommandType::Message
            {
                resolved_messages = Some(ms.values().cloned().collect());
            }

            // resolve users for context menu user commands
            let mut resolved_users: Option<Vec<User>> = None;
            if let Some(ref us) = command_data.resolved.map(|x| x.users)
                && command_data.kind == CommandType::User
            {
                resolved_users = Some(us.values().cloned().collect());
            }

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
                interaction_attachments,
                command_from_install_context: match interaction.context {
                    Some(c) => c == InteractionContextType::PrivateChannel,
                    None => false,
                },
                resolved_messages,
                resolved_users,
            };

            let ctxt = InteractionCommandParseCtxt::new(CommandCtxt::new(&data), &sorted_incoming_options);
            /*
            let ctxt_clone = ctxt.cx.clone();

            let meta_check = check_metadata(command.metadata(), &ctxt_clone).await;
            if let Err(e) = meta_check {
                let _ = ctxt.cx.reply(format!(":warning: ``{e}``")).await;
            }*/

            if let Err(err) = command.execute_interaction_command(ctxt.clone()).await {
                match err.get_severity() {
                    ErrorSeverity::Low => {
                        if let ExecutionError::MetadataCheck(e) = err {
                            let _ = ctxt.cx.reply(format!(":warning: ``{e:#}``")).await;
                        } else {
                            debug!("{err}");
                        }
                    },
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
    } else if let Some(InteractionData::MessageComponent(component)) = interaction.data {
        let ctxt = assyst.component_contexts.get(&component.custom_id);
        let component_data = ComponentInteractionData {
            assyst: assyst.clone(),
            custom_id: component.custom_id.clone(),
            message_interaction_data: Some(component),
            modal_submit_interaction_data: None,
            invocation_guild_id: interaction.guild_id,
            invocation_user_id: interaction
                .member
                .map(|m| m.user.unwrap().id)
                .or(interaction.user.map(|u| u.id))
                .unwrap(),
            interaction_id: interaction.id,
            interaction_token: interaction.token.clone(),
        };

        if let Some(cx) = ctxt {
            match cx.lock().await.handle_component_interaction(&component_data).await {
                Err(e) => {
                    err!("Failed to handle component interaction: {e:?}");
                },
                _ => {},
            }
        } else {
            let response = InteractionResponse {
                kind: InteractionResponseType::DeferredUpdateMessage,
                data: None,
            };

            match assyst
                .interaction_client()
                .create_response(interaction.id, &interaction.token, &response)
                .await
            {
                Err(e) => {
                    err!("Failed to send deferred update message: {e:?}");
                },
                _ => {},
            };
        }
    } else if let Some(InteractionData::ModalSubmit(component)) = interaction.data {
        let ctxt = assyst.component_contexts.get(&component.custom_id);
        let component_data = ComponentInteractionData {
            assyst: assyst.clone(),
            custom_id: component.custom_id.clone(),
            message_interaction_data: None,
            modal_submit_interaction_data: Some(component),
            invocation_guild_id: interaction.guild_id,
            invocation_user_id: interaction
                .member
                .map(|m| m.user.unwrap().id)
                .or(interaction.user.map(|u| u.id))
                .unwrap(),
            interaction_id: interaction.id,
            interaction_token: interaction.token.clone(),
        };

        if let Some(cx) = ctxt {
            match cx.lock().await.handle_component_interaction(&component_data).await {
                Err(e) => {
                    err!("Failed to handle component interaction: {e:?}");
                },
                _ => {},
            }
        } else {
            let response = InteractionResponse {
                kind: InteractionResponseType::DeferredUpdateMessage,
                data: None,
            };

            match assyst
                .interaction_client()
                .create_response(interaction.id, &interaction.token, &response)
                .await
            {
                Err(e) => {
                    err!("Failed to send deferred update message: {e:?}");
                },
                _ => {},
            };
        }
    } else if interaction.kind == InteractionType::ApplicationCommandAutocomplete
        && let Some(InteractionData::ApplicationCommand(command_data)) = interaction.data.clone()
    {
        let command = find_command_by_name(&command_data.name);
        let subcommand_data = parse_subcommand_data(&command_data);

        if let Some(command) = command {
            let incoming_options = if let Some(d) = subcommand_data.clone() {
                match d.1 {
                    CommandOptionValue::SubCommand(s) => s,
                    _ => unreachable!(),
                }
            } else {
                command_data.options
            };

            let interaction_subcommand = if let Some(d) = subcommand_data {
                match d.1 {
                    CommandOptionValue::SubCommand(_) => Some(d),
                    _ => unreachable!(),
                }
            } else {
                None
            };

            let focused_option = incoming_options
                .iter()
                .find(|x| matches!(x.value, CommandOptionValue::Focused(_, _)))
                .expect("no focused option?");

            // we will probably only ever use autocomplete on `Word` arguments
            // FIXME: add support for more arg types here?
            let inner_option = if let CommandOptionValue::Focused(x, y) = focused_option.value.clone() {
                (x, y)
            } else {
                unreachable!()
            };

            let data = AutocompleteData {
                guild_id: interaction.guild_id,
                user: interaction.author().unwrap().clone(),
                subcommand: interaction_subcommand.map(|x| x.0),
            };

            let options = match command
                .arg_autocomplete(assyst.clone(), focused_option.name.clone(), inner_option.0, data)
                .await
            {
                Ok(o) => o,
                Err(e) => {
                    err!("Failed to generate options for option {}: {e:?}", focused_option.name);
                    return;
                },
            };

            let b = InteractionResponseDataBuilder::new();
            let b = b.choices(options);
            let r = b.build();
            let r = InteractionResponse {
                kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
                data: Some(r),
            };

            if let Err(e) = assyst
                .interaction_client()
                .create_response(interaction.id, &interaction.token, &r)
                .await
            {
                err!("Failed to send autocomplete options: {e:?}");
            };
        }
    }
}
