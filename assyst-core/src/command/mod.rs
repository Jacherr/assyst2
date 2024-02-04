use std::fmt::Display;
use std::future::Future;
use std::str::SplitAsciiWhitespace;
use std::time::Duration;

use assyst_common::assyst::ThreadSafeAssyst;
use async_trait::async_trait;
use twilight_model::channel::message::sticker::MessageSticker;
use twilight_model::channel::message::Embed;
use twilight_model::channel::{Attachment, Message};

use crate::gateway_handler::message_parser::error::{ErrorSeverity, GetErrorSeverity};

use self::errors::{ArgsExhausted, TagParseError};

pub mod arguments;
pub mod errors;
pub mod misc;
pub mod registry;

/// Defines who can use a command in a server.
pub enum Availability {
    /// Anyone can use this command, subject to blacklisting and whitelisting configuration.
    Public,
    /// Server managers (those with the 'manage server' permission) can use this command.
    ServerManagers,
    /// Only developers, as configured, can use this command.
    Dev,
}

pub struct CommandMetadata {
    name: &'static str,
    aliases: &'static [&'static str],
    description: &'static str,
    cooldown: Duration,
    access: Availability,
}

#[derive(Debug)]
pub enum ExecutionError {
    Parse(TagParseError),
    Command(anyhow::Error),
}

impl GetErrorSeverity for ExecutionError {
    fn get_severity(&self) -> ErrorSeverity {
        // Even though tag parse errors can define themselves if they're high or low severity,
        // at the end of execution (here) we always want to report errors back if they got here,
        // so treat them as high severity
        ErrorSeverity::High
    }
}
impl Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::Parse(p) => p.fmt(f),
            ExecutionError::Command(c) => c.fmt(f),
        }
    }
}
impl std::error::Error for ExecutionError {}

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
pub struct CommandData<'a> {
    pub channel_id: u64,
    /// `None` in a slash command
    pub attachment: Option<&'a Attachment>,
    /// `None` in a slash command
    pub sticker: Option<&'a MessageSticker>,
    pub embed: Option<&'a Embed>,
    pub assyst: &'a ThreadSafeAssyst,
    /// `None` in a slash command, otherwise set if the message is a reply
    pub referenced_message: Option<&'a Message>,
}

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