//! The command system.
//!
//! The key things that make up the command system are:
//!
//! - The [`Command`] trait: Defines the `execute` method which executes the actual command.
//!
//!   This is relatively low-level and only gives you a `CommandCtxt`,
//!   from which you manually have to extract args and attachments.
//!
//!   Normally, you don't want or need to implement this trait manually.
//!   Just write the function and annotate it with `#[command]`, which generates a type
//!   that implements this trait (and delegates to the annotated function).
//!   See its documentation for how that works.
//!
//!   This is used as a trait object (`&dyn Command`), because it is stored along with all other
//!   commands in a map, in registry.rs.
//!
//! - The [`arguments::ParseArgument`] trait: Implemented for types that can be parsed from
//!   arguments.
//!
//!   These types also compose well: for example, `Option<T>` implements `ParseArgument` if
//!   `T: ParseArgument`, which allows recovering from low-severity errors in `T`'s parser (e.g. if
//!   the argument is not present, it will be set to `None`).
//!
//! - The registry: registry.rs is responsible for storing a map of `&str -> &dyn Command`. The
//!   entry point (and the only relevant for the outside) is [`registry::find_command_by_name`],
//!   which does the mapping mentioned above.

use std::collections::HashMap;
use std::fmt::Display;
use std::slice;
use std::str::SplitAsciiWhitespace;
use std::time::{Duration, Instant};

use assyst_common::config::CONFIG;
use async_trait::async_trait;
use errors::TagParseError;
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::{CommandDataOption, CommandOptionValue};
use twilight_model::channel::{Attachment, Message};
use twilight_model::http::interaction::InteractionResponse;
use twilight_model::id::marker::{AttachmentMarker, ChannelMarker, GuildMarker, InteractionMarker};
use twilight_model::id::Id;
use twilight_model::user::User;
use twilight_util::builder::command::SubCommandBuilder;

use self::errors::{ArgsExhausted, ExecutionError, MetadataCheckError};
use self::messagebuilder::MessageBuilder;
use self::source::Source;
use super::gateway_handler::reply as gateway_reply;
use crate::assyst::ThreadSafeAssyst;
use crate::flux_handler::FluxHandler;

pub mod arguments;
pub mod errors;
pub mod flags;
pub mod fun;
pub mod group;
pub mod image;
pub mod messagebuilder;
pub mod misc;
pub mod registry;
pub mod services;
pub mod source;

/// Defines who can use a command in a server.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Availability {
    /// Anyone can use this command, subject to blacklisting and whitelisting configuration.
    Public,
    /// Server managers (those with the 'manage server' permission) can use this command.
    ServerManagers,
    /// Only developers, as configured, can use this command.
    Dev,
}

impl Display for Availability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Public => "Public",
                Self::ServerManagers => "Server Managers",
                Self::Dev => "Private",
            }
        )
    }
}

#[derive(Debug)]
pub struct CommandMetadata {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub cooldown: Duration,
    pub access: Availability,
    pub category: Category,
    pub examples: &'static [&'static str],
    pub usage: String, // not &str because usage generation reasons
    /// Whether to send a "Processing..." reply when the command starts executing
    /// or to send a prelim response to an interaction (a.k.a., Assyst is thinking...)
    pub send_processing: bool,
    pub age_restricted: bool,
    pub flag_descriptions: HashMap<&'static str, &'static str>,
}

#[derive(Debug)]
pub enum CommandGroupingInteractionInfo {
    Group(Vec<(String /* subcommand name */, CommandInteractionInfo)>),
    Command(CommandInteractionInfo),
}
impl CommandGroupingInteractionInfo {
    pub fn unwrap_command(&self) -> &CommandInteractionInfo {
        if let Self::Command(x) = self { x } else { unreachable!() }
    }

    pub fn group_as_option_tree(&self, subcommands: &'static [(&'static str, TCommand)]) -> Vec<CommandOption> {
        let group = if let Self::Group(x) = self { x } else { unreachable!() };
        let mut options = Vec::new();

        for member in group {
            let subcommand_description = subcommands
                .iter()
                .find_map(|x| {
                    if x.0 == member.0 {
                        Some(x.1.metadata().description.to_owned())
                    } else {
                        None
                    }
                })
                .unwrap_or(format!("{} subcommand", member.0));

            let mut subcommand = SubCommandBuilder::new(member.0.clone(), subcommand_description);

            for option in member.1.command_options.clone() {
                subcommand = subcommand.option(option);
            }

            options.push(subcommand.build());
        }

        options
    }
}

#[derive(Debug, Clone)]
pub struct CommandInteractionInfo {
    pub command_options: Vec<CommandOption>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Category {
    Audio,
    Fun,
    Makesweet,
    Image,
    Misc,
    Services,
    None(String),
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Audio => "audio",
                Self::Fun => "fun",
                Self::Makesweet => "makesweet",
                Self::Image => "image",
                Self::Misc => "misc",
                Self::Services => "services",
                Self::None(t) => &**t,
            }
        )
    }
}

impl From<String> for Category {
    fn from(v: String) -> Category {
        match &*v {
            "audio" => Category::Audio,
            "fun" => Category::Fun,
            "misc" => Category::Misc,
            "image" => Category::Image,
            "makesweet" => Category::Makesweet,
            "services" => Category::Services,
            t => Category::None(t.to_string()),
        }
    }
}

/// A command that can be executed.
///
/// You usually don't want to or need to implement this manually -- write the function that handles
/// the command and apply the `#[command]` proc macro. It will generate a struct that implements
/// this.
/// See the proc macro's documentation too for more details.
// This trait is used as a trait object and AFIT makes traits not object safe, so we still need
// #[async_trait] here :(
#[async_trait]
pub trait Command {
    /// Returns the **direct** metadata.
    ///
    /// It's important to note that this might not return what you're looking for! In particular, if
    /// this is a command group, then this method will (maybe unsurprisingly) return the metadata of
    /// the command group only, not a specific subcommand!
    ///
    /// If you do want metadata of a subcommand, you may want to follow a chain of `subcommand`
    /// calls.
    fn metadata(&self) -> &'static CommandMetadata;

    /// Tries to find a subcommand given a name, provided that `self` is a command group
    fn subcommands(&self) -> Option<&'static [(&'static str, TCommand)]>;

    /// Creates an interaction command for subitting for Discord on startup
    fn as_interaction_command(&self) -> twilight_model::application::command::Command;

    /// Loads all interaction-specific info for sending to Discord
    fn interaction_info(&self) -> CommandGroupingInteractionInfo;

    /// Parses arguments and executes the command, when the source is a "raw" prefixed message.
    async fn execute_raw_message(&self, ctxt: RawMessageParseCtxt<'_>) -> Result<(), ExecutionError>;

    /// Parses arguments and executes the command, when the source is an interaction command.
    async fn execute_interaction_command(&self, ctxt: InteractionCommandParseCtxt<'_>) -> Result<(), ExecutionError>;
}

/// A set of timings used to diagnose slow areas of parsing for commands.
#[derive(Clone)]
pub struct ExecutionTimings {
    /// Total time spent on the preprocessing phase.
    pub preprocess_total: Duration,
    /// Total time spent determining the correct prefix.
    pub prefix_determiner: Duration,
    /// Total time spent on the parsing phase.
    pub parse_total: Duration,
    /// Instant checking command metadata started.
    pub metadata_check_start: Instant,
    /// Instant full command processing started.
    pub processing_time_start: Instant,
}

/// Just a type alias for a command as a trait object with other necessary bounds.
/// See [Command] for more documentation.
pub type TCommand = &'static (dyn Command + Send + Sync);

/// Other static data that can be shared and does not need to be cloned between
/// subcontexts
#[derive(Clone)]
pub struct CommandData<'a> {
    /// The source of this command invocation
    pub source: Source,
    pub assyst: &'a ThreadSafeAssyst,
    pub execution_timings: ExecutionTimings,
    pub calling_prefix: String,
    pub channel_id: Id<ChannelMarker>,
    pub guild_id: Option<Id<GuildMarker>>,
    pub author: User,
    pub interaction_subcommand: Option<(String /* name */, CommandOptionValue)>,
    pub message: Option<&'a Message>,
    pub interaction_token: Option<String>,
    pub interaction_id: Option<Id<InteractionMarker>>,
    pub interaction_attachments: HashMap<Id<AttachmentMarker>, Attachment>,
}

pub type RawMessageArgsIter<'a> = SplitAsciiWhitespace<'a>;
pub type InteractionMessageArgsIter<'a> = slice::Iter<'a, CommandDataOption>;

/// A parsing context. Parsing contexts can either be for raw message commands or interaction
/// commands, and the parsing method differs for each.
#[derive(Clone)]
pub struct ParseCtxt<'a, T> {
    pub cx: CommandCtxt<'a>,
    args: T,
}
impl<'a, T: Clone> ParseCtxt<'a, T> {
    /// Cheaply forks this context. Useful for trying different combinations
    /// and throwing the fork away after failing.
    /// Also look at `commit_if_ok`.
    pub fn fork(&self) -> Self {
        // if you change either type and these lines starts erroring, check that
        // these are still cheap to clone.
        let _: &T = &self.args;
        let _: &CommandCtxt<'a> = &self.cx;

        Self {
            cx: self.cx.clone(),
            args: self.args.clone(),
        }
    }
}

/// Calls the function with a fork of this context (allowing some arbitrary mutations)
/// and only actually applies the changes made to the fork if it returns `Ok`.
///
/// This used to be a function, however due to compiler bugs and the inability to properly express
/// this pattern with bounds, this was ultimately just made into a macro where no such bounds need
/// to be specified.
#[allow(clippy::crate_in_macro_def)]
#[macro_export]
macro_rules! commit_if_ok {
    ($ctxt:expr, $f:expr, $label:expr) => {{
        let ctxt: &mut crate::command::ParseCtxt<'_, _> = $ctxt;
        let mut fork = ctxt.fork();
        // label should be cheaply cloneable?
        let res = ($f)(&mut fork, $label.clone()).await;
        if res.is_ok() {
            *ctxt = fork;
        }
        res
    }};
}

/// A label for a command argument.
pub type Label = Option<(String, String)>;

impl<'a> ParseCtxt<'a, RawMessageArgsIter<'a>> {
    pub fn new(ctxt: CommandCtxt<'a>, args: &'a str) -> Self {
        Self {
            args: args.split_ascii_whitespace(),
            cx: ctxt,
        }
    }

    /// Eagerly takes a word.
    /// If you want to "peek" or you aren't sure if you might want to undo this,
    /// consider using `commit_if_ok` or `fork` to try it in a subcontext.
    pub fn next_word(&mut self, label: Label) -> Result<&'a str, ArgsExhausted> {
        self.args.next().ok_or(ArgsExhausted(label))
    }

    /// The rest of the message, excluding flags.
    pub fn rest(&mut self, label: Label) -> Result<String, TagParseError> {
        let raw = self
            .args
            .remainder()
            .ok_or(TagParseError::ArgsExhausted(ArgsExhausted(label.clone())))?;

        let (args, flags) = if let Some(idx) = raw.find("--") {
            (&raw[..idx], &raw[idx..])
        } else {
            (raw, "")
        };

        if args.is_empty() {
            return Err(TagParseError::ArgsExhausted(ArgsExhausted(label)));
        }

        self.args = flags.split_ascii_whitespace();

        Ok(args.to_owned())
    }

    pub fn rest_all(&self, _: Label) -> String {
        self.args.remainder().map(|x| x.to_owned()).unwrap_or_default()
    }
}

impl<'a> ParseCtxt<'a, InteractionMessageArgsIter<'a>> {
    pub fn new(ctxt: CommandCtxt<'a>, args: &'a [CommandDataOption]) -> Self {
        Self {
            args: args.iter(),
            cx: ctxt,
        }
    }

    /// Eagerly takes an option.
    /// If you want to "peek" or you aren't sure if you might want to undo this,
    /// consider using `commit_if_ok` or `fork` to try it in a subcontext.
    pub fn next_option(&mut self) -> Result<&'a CommandDataOption, ArgsExhausted> {
        self.args.next().ok_or(ArgsExhausted(None))
    }
}

pub type RawMessageParseCtxt<'a> = ParseCtxt<'a, RawMessageArgsIter<'a>>;
pub type InteractionCommandParseCtxt<'a> = ParseCtxt<'a, InteractionMessageArgsIter<'a>>;

#[derive(Clone)]
pub struct CommandCtxt<'a> {
    pub data: &'a CommandData<'a>,
}

impl<'a> CommandCtxt<'a> {
    pub fn new(data: &'a CommandData<'a>) -> Self {
        Self { data }
    }

    pub async fn reply(&self, builder: impl Into<MessageBuilder>) -> anyhow::Result<()> {
        let builder = builder.into();
        match self.data.source {
            Source::RawMessage => gateway_reply::reply_raw_message(self, builder).await,
            Source::Interaction => gateway_reply::reply_interaction_command(self, builder).await,
        }
    }

    pub fn assyst(&self) -> &'a ThreadSafeAssyst {
        self.data.assyst
    }

    pub fn flux_handler(&self) -> &'a FluxHandler {
        &self.data.assyst.flux_handler
    }
}

pub async fn check_metadata(
    metadata: &'static CommandMetadata,
    ctxt: &mut CommandCtxt<'_>,
) -> Result<(), ExecutionError> {
    if metadata.age_restricted {
        let channel_age_restricted = ctxt
            .assyst()
            .rest_cache_handler
            .channel_is_age_restricted(ctxt.data.channel_id.get())
            .await
            .unwrap_or(false);

        if !channel_age_restricted {
            return Err(ExecutionError::MetadataCheck(
                MetadataCheckError::IllegalAgeRestrictedCommand,
            ));
        };
    };

    // command availability check
    match metadata.access {
        Availability::Dev => {
            if !CONFIG.dev.admin_users.contains(&ctxt.data.author.id.get()) {
                return Err(ExecutionError::MetadataCheck(MetadataCheckError::DevOnlyCommand));
            }
        },
        Availability::ServerManagers => {
            if let Some(guild_id) = ctxt.data.guild_id {
                if !ctxt
                    .assyst()
                    .rest_cache_handler
                    .user_is_guild_manager(guild_id.get(), ctxt.data.author.id.get())
                    .await
                    .unwrap_or(false)
                {
                    return Err(ExecutionError::MetadataCheck(
                        MetadataCheckError::GuildManagerOnlyCommand,
                    ));
                }
            }
        },
        _ => {},
    }

    if !CONFIG.dev.admin_users.contains(&ctxt.data.author.id.get()) {
        // ratelimit check
        let id = ctxt
            .data
            .guild_id
            .map_or_else(|| ctxt.data.author.id.get(), |id| id.get());
        let last_command_invoked = ctxt.assyst().command_ratelimits.get(id, metadata.name);
        if let Some(invocation_time) = last_command_invoked {
            let elapsed = invocation_time.elapsed();
            if elapsed < metadata.cooldown {
                return Err(ExecutionError::MetadataCheck(MetadataCheckError::CommandOnCooldown(
                    metadata.cooldown - elapsed,
                )));
            }
        }

        // update/set new last invocation time
        ctxt.assyst()
            .command_ratelimits
            .insert(id, metadata.name, Instant::now());
    }

    if metadata.send_processing && ctxt.data.source == Source::RawMessage {
        if let Err(e) = ctxt.reply("Processing...").await {
            return Err(ExecutionError::Command(e));
        }
    } else if metadata.send_processing && ctxt.data.source == Source::Interaction {
        let response = InteractionResponse {
            kind: twilight_model::http::interaction::InteractionResponseType::DeferredChannelMessageWithSource,
            data: None,
        };

        ctxt.assyst()
            .interaction_client()
            .create_response(
                ctxt.data.interaction_id.unwrap(),
                &ctxt.data.interaction_token.clone().unwrap(),
                &response,
            )
            .await
            .map_err(|e| ExecutionError::Parse(errors::TagParseError::TwilightHttp(Box::new(e))))?;

        ctxt.assyst()
            .replies
            .insert_interaction_command(ctxt.data.interaction_id.unwrap().get());
    }
    Ok(())
}
