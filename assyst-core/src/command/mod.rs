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

use std::fmt::Display;
use std::future::Future;
use std::str::SplitAsciiWhitespace;
use std::time::{Duration, Instant};

use super::gateway_handler::reply as gateway_reply;
use crate::assyst::ThreadSafeAssyst;
use async_trait::async_trait;
use twilight_model::channel::message::sticker::MessageSticker;
use twilight_model::channel::message::Embed;
use twilight_model::channel::{Attachment, Message};
use twilight_model::user::User;

use self::errors::{ArgsExhausted, ExecutionError};
use self::messagebuilder::MessageBuilder;
use self::source::Source;

pub mod arguments;
pub mod errors;
pub mod messagebuilder;
pub mod misc;
pub mod registry;
pub mod source;
pub mod wsi;

/// Defines who can use a command in a server.
#[derive(Clone, Copy, Debug)]
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

pub struct CommandMetadata {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub cooldown: Duration,
    pub access: Availability,
    pub category: Category,
    pub examples: &'static [&'static str],
    pub usage: &'static str,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Category {
    Fun,
    Makesweet,
    Wsi,
    Misc,
    None(String),
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Fun => "fun",
                Self::Makesweet => "makesweet",
                Self::Wsi => "wsi",
                Self::Misc => "misc",
                Self::None(t) => &**t,
            }
        )
    }
}

impl From<String> for Category {
    fn from(v: String) -> Category {
        match &*v {
            "fun" => Category::Fun,
            "misc" => Category::Misc,
            "wsi" => Category::Wsi,
            "makesweet" => Category::Makesweet,
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
    fn metadata(&self) -> &'static CommandMetadata;

    /// Parses arguments and executes the command.
    async fn execute(&self, ctxt: CommandCtxt<'_>) -> Result<(), ExecutionError>;
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
    pub message_id: u64,
    pub channel_id: u64,
    /// `None` in a slash command
    pub attachment: Option<&'a Attachment>,
    /// `None` in a slash command
    pub sticker: Option<&'a MessageSticker>,
    pub embed: Option<&'a Embed>,
    pub assyst: &'a ThreadSafeAssyst,
    /// `None` in a slash command, otherwise set if the message is a reply
    pub referenced_message: Option<&'a Message>,
    pub processing_time_start: Instant,
    pub calling_prefix: String,
    pub author: &'a User,
}

#[derive(Clone)]
pub struct CommandCtxt<'a> {
    args: SplitAsciiWhitespace<'a>,
    pub data: &'a CommandData<'a>,
}

impl<'a> CommandCtxt<'a> {
    pub fn new(args: &'a str, data: &'a CommandData<'a>) -> Self {
        Self {
            args: args.split_ascii_whitespace(),
            data,
        }
    }

    pub async fn reply(&self, builder: impl Into<MessageBuilder>) -> anyhow::Result<()> {
        let builder = builder.into();
        match self.data.source {
            Source::Gateway => gateway_reply::reply(self, builder).await,
        }
    }

    pub fn assyst(&self) -> &'a ThreadSafeAssyst {
        self.data.assyst
    }

    /// Cheaply forks this context. Useful for trying different combinations
    /// and throwing the fork away after failing.
    /// Also look at `commit_if_ok`.
    pub fn fork(&self) -> Self {
        // if you change the type of `self.args` and this line starts erroring, check that
        // this is still cheap to clone.
        let _: &SplitAsciiWhitespace<'a> = &self.args;

        Self {
            args: self.args.clone(),
            data: self.data,
        }
    }

    /// Calls the function with a fork of this context (allowing some arbitrary mutations)
    /// and only actually applies the changes made to the fork if it returns `Ok`.
    // Due to a bug in the rust compiler, the fork is passed to the closure by value and should be
    // returned by value (instead of just passing it by `&mut`)
    // https://github.com/rust-lang/rust/issues/70263
    pub async fn commit_if_ok<F, Fut, T, E>(&mut self, f: F) -> Result<T, E>
    where
        Fut: Future<Output = Result<(T, CommandCtxt<'a>), E>>,
        F: FnOnce(CommandCtxt<'a>) -> Fut,
    {
        let fork: CommandCtxt<'a> = self.fork();
        let (res, fork) = f(fork).await?;
        *self = fork;
        Ok(res)
    }

    /// Eagerly takes a word.
    /// If you want to "peek" or you aren't sure if you might want to undo this,
    /// consider using `commit_if_ok` or `fork` to try it in a subcontext.
    pub fn next_word(&mut self) -> Result<&'a str, ArgsExhausted> {
        self.args.next().ok_or(ArgsExhausted)
    }

    /// The rest of the message.
    pub fn rest(&self) -> Result<&'a str, ArgsExhausted> {
        self.args.remainder().ok_or(ArgsExhausted)
    }
}
