use super::errors::{ExecutionError, TagParseError};
use super::{InteractionCommandParseCtxt, RawMessageParseCtxt, TCommand};

// Helper macro that provides defaults
// cfg_attr is needed because of https://github.com/rust-lang/rust/issues/74087
#[cfg_attr(rustfmt, rustfmt::skip)]
#[doc(hidden)] // don't use this anywhere except for inside of the `define_commandgroup` macro
#[macro_export]
macro_rules! defaults {
    (access $x:expr) => { $x };
    (access) => { crate::command::Availability::Public };
    (aliases $x:expr) => { $x };
    (aliases) => { &[] };
    (cooldown $x:expr) => { $x };
    (cooldown) => { std::time::Duration::ZERO };
    (examples $x:expr) => { $x };
    (examples) => { &[] };
    (age_restricted $x:expr) => { $x };
    (age_restricted) => { false };
    (usage $x:expr) => { $x };
    (usage) => { "" };
    (send_processing $x:expr) => { $x };
    (send_processing) => { false };
}

#[macro_export]
macro_rules! define_commandgroup {
    (
        name: $groupname:ident,
        $(access: $access:expr,)?
        category: $category:expr,
        $(aliases: $aliases:expr,)?
        $(cooldown: $cooldown:expr,)?
        description: $description:expr,
        $(examples: $examples:expr,)?
        $(usage: $usage:expr,)?
        $(send_processing: $send_processing:expr,)?
        $(age_restricted: $age_restricted:expr,)?
        commands: [
            $(
                $subcommand:literal => $commandfn:expr
            ),*
        ]
        $(,default_interaction_subcommand: $default_interaction_subcommand:expr)?
        $(,default: $default:expr)?
    ) => {
        paste::paste! {
            #[allow(non_camel_case_types)]
            pub struct [<$groupname _command>];

            impl [<$groupname _command>] {
                const SUBCOMMANDS: &'static [(&'static str, crate::command::TCommand)] = &[
                    $(
                        ($subcommand, &[<$commandfn _command>])
                    ),*
                ];
            }

            #[::async_trait::async_trait]
            impl crate::command::Command for [<$groupname _command>] {
                fn metadata(&self) -> &'static crate::command::CommandMetadata {
                    static META: std::sync::OnceLock<crate::command::CommandMetadata> = std::sync::OnceLock::new();
                    META.get_or_init(|| crate::command::CommandMetadata {
                        access: $crate::defaults!(access $($access)?),
                        category: $category,
                        aliases: $crate::defaults!(aliases $(&$aliases)?),
                        cooldown: $crate::defaults!(cooldown $($cooldown)?),
                        description: $description,
                        examples: $crate::defaults!(examples $(&$examples)?),
                        name: stringify!($groupname),
                        age_restricted: $crate::defaults!(age_restricted $($age_restricted)?),
                        usage: $crate::defaults!(usage $($usage)?),
                        send_processing: $crate::defaults!(send_processing $($send_processing)?),
                        flag_descriptions: std::collections::HashMap::new()
                    })
                }

                fn subcommands(&self) -> Option<&'static [(&'static str, crate::command::TCommand)]> {
                    Some(Self::SUBCOMMANDS)
                }

                fn interaction_info(&self) -> crate::command::CommandGroupingInteractionInfo {
                    let mut subcommands = Vec::<(String, crate::command::CommandInteractionInfo)>::new();
                    for command in Self::SUBCOMMANDS {
                        subcommands.push((command.0.to_owned(), command.1.interaction_info().unwrap_command().clone()));
                    }

                    $(
                        subcommands.push(($default_interaction_subcommand.to_owned(), [<$default _command>].interaction_info().unwrap_command().clone()));
                    )?
                    crate::command::CommandGroupingInteractionInfo::Group(subcommands)
                }

                fn as_interaction_command(&self) -> twilight_model::application::command::Command {
                    let meta = self.metadata();
                    let options = self.interaction_info().group_as_option_tree();

                    twilight_model::application::command::Command {
                        application_id: None,
                        default_member_permissions: None,
                        description: meta.description.to_owned(),
                        description_localizations: None,
                        // TODO: set based on if dms are allowed
                        // TODO: update to `contexts` once this is required
                        // (see https://discord.com/developers/docs/interactions/application-commands#create-global-application-command)
                        dm_permission: Some(false),
                        guild_id: None,
                        id: None,
                        kind: twilight_model::application::command::CommandType::ChatInput,
                        name: meta.name.to_owned(),
                        name_localizations: None,
                        nsfw: Some(meta.age_restricted),
                        options,
                        version: twilight_model::id::Id::new(1),
                    }
                }

                async fn execute_raw_message(&self, ctxt: crate::command::RawMessageParseCtxt<'_>) -> Result<(), crate::command::ExecutionError> {
                    #![allow(unreachable_code)]
                    match crate::command::group::execute_subcommand_raw_message(ctxt.fork(), Self::SUBCOMMANDS).await {
                        Ok(res) => Ok(res),
                        Err(crate::command::ExecutionError::Parse(crate::command::errors::TagParseError::InvalidSubcommand(_))
                        | crate::command::ExecutionError::Parse(crate::command::errors::TagParseError::ArgsExhausted(_))) => {
                            // No subcommand was found, call either the default if provided, or error out
                            $(
                                return [<$default _command>].execute_raw_message(ctxt).await;
                            )?
                            return Err(crate::command::ExecutionError::Parse(crate::command::errors::TagParseError::InvalidSubcommand("unknown".to_owned())));
                        },
                        Err(err) => Err(err)
                    }
                }

                async fn execute_interaction_command(&self, ctxt: crate::command::InteractionCommandParseCtxt<'_>) -> Result<(), crate::command::ExecutionError> {
                    #[allow(unused_mut, unused_assignments)]
                    let mut default = "";
                    $(
                        default = $default_interaction_subcommand;
                    )?

                    #[allow(unreachable_code)]
                    match crate::command::group::execute_subcommand_interaction_command(ctxt.fork(), Self::SUBCOMMANDS, default).await {
                        Ok(res) => Ok(res),
                        Err(crate::command::ExecutionError::Parse(crate::command::errors::TagParseError::InteractionCommandIsBaseSubcommand)) => {
                            // Subcommand was "default" command, call either the default if provided, or error out
                            $(
                                return [<$default _command>].execute_interaction_command(ctxt).await;
                            )?
                            return Err(crate::command::ExecutionError::Parse(crate::command::errors::TagParseError::InvalidSubcommand("unknown".to_owned())));
                        },
                        Err(err) => Err(err)
                    }
                }
            }
        }
    };
}

pub fn find_subcommand(sub: &str, cmds: &[(&str, TCommand)]) -> Option<TCommand> {
    cmds.iter().find(|(k, _)| *k == sub).map(|(_, v)| v).copied()
}

/// Tries to execute a subcommand from a "raw" message, by taking the next word from the arguments
/// and looking for it in `commands`
pub async fn execute_subcommand_raw_message(
    mut ctxt: RawMessageParseCtxt<'_>,
    commands: &[(&str, TCommand)],
) -> Result<(), ExecutionError> {
    // todo: come up with better names for this?
    let subcommand = ctxt.next_word(None).map_err(|err| ExecutionError::Parse(err.into()))?;

    let command = find_subcommand(subcommand, commands).ok_or(ExecutionError::Parse(
        TagParseError::InvalidSubcommand(subcommand.to_owned()),
    ))?;

    command.execute_raw_message(ctxt).await
}

pub fn find_subcommand_interaction_command(sub: &str, cmds: &[(&str, TCommand)]) -> Option<TCommand> {
    cmds.iter().find(|(k, _)| *k == sub).map(|(_, v)| v).copied()
}

/// Tries to execute a subcommand from an interaction command, by seeing if we extracted a
/// subcommand
pub async fn execute_subcommand_interaction_command(
    ctxt: InteractionCommandParseCtxt<'_>,
    commands: &[(&str, TCommand)],
    default_interaction_subcommand: &str,
) -> Result<(), ExecutionError> {
    let subcommand = ctxt
        .cx
        .data
        .interaction_subcommand
        .clone()
        .ok_or(ExecutionError::Parse(TagParseError::NoInteractionSubcommandProvided))?;

    if subcommand.0 == default_interaction_subcommand {
        return Err(ExecutionError::Parse(TagParseError::InteractionCommandIsBaseSubcommand));
    }

    let command = find_subcommand_interaction_command(&subcommand.0, commands)
        .ok_or(ExecutionError::Parse(TagParseError::InvalidSubcommand(subcommand.0)))?;

    command.execute_interaction_command(ctxt).await
}
