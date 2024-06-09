use super::errors::{ExecutionError, TagParseError};
use super::{CommandCtxt, TCommand};

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
                    // TODO: use options in metadata instead of things like empty &str

                    static META: crate::command::CommandMetadata = crate::command::CommandMetadata {
                        access: $crate::defaults!(access $($access)?),
                        category: $category,
                        aliases: $crate::defaults!(aliases $(&$aliases)?),
                        cooldown: $crate::defaults!(cooldown $($cooldown)?),
                        description: $description,
                        examples: $crate::defaults!(examples $(&$examples)?),
                        name: stringify!($groupname),
                        age_restricted: $crate::defaults!(age_restricted $($age_restricted)?),
                        usage: $crate::defaults!(usage $($usage)?),
                        send_processing: $crate::defaults!(send_processing $($send_processing)?)
                    };
                    &META
                }

                fn subcommand(&self, sub: &str) -> Option<crate::command::TCommand> {
                    crate::command::group::find_subcommand(sub, Self::SUBCOMMANDS)
                }

                fn as_interaction_command(&self) -> twilight_model::application::command::Command {
                    let meta = self.metadata();

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
                        // todo: handle properly
                        name: "".to_owned(),
                        name_localizations: None,
                        nsfw: Some(meta.age_restricted),
                        // TODO: set options properly
                        options: vec![],
                        version: twilight_model::id::Id::new(1),
                    }
                }

                async fn execute(&self, ctxt: CommandCtxt<'_>) -> Result<(), crate::command::ExecutionError> {
                    #![allow(unreachable_code)]
                    match crate::command::group::execute_subcommand(ctxt.fork(), Self::SUBCOMMANDS).await {
                        Ok(res) => Ok(res),
                        Err(crate::command::ExecutionError::Parse(crate::command::errors::TagParseError::InvalidSubcommand)) => {
                            // No subcommand was found, call either the default if provided, or error out
                            $(
                                return [<$default _command>].execute(ctxt).await;
                            )?
                            return Err(crate::command::ExecutionError::Parse(crate::command::errors::TagParseError::InvalidSubcommand));
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

/// Tries to execute a subcommand, by taking the next word from the arguments and looking for it in
/// `commands`
pub async fn execute_subcommand(
    mut ctxt: CommandCtxt<'_>,
    commands: &[(&str, TCommand)],
) -> Result<(), ExecutionError> {
    let subcommand = ctxt.next_word().map_err(|err| ExecutionError::Parse(err.into()))?;

    let command =
        find_subcommand(subcommand, commands).ok_or(ExecutionError::Parse(TagParseError::InvalidSubcommand))?;

    command.execute(ctxt).await
}
