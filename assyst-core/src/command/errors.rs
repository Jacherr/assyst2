use std::fmt::Display;
use std::num::{ParseFloatError, ParseIntError};
use std::time::Duration;

use assyst_common::util::ParseToMillisError;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::channel::message::sticker::StickerFormatType;

use super::Label;
use crate::downloader::DownloadError;
use crate::gateway_handler::message_parser::error::{ErrorSeverity, GetErrorSeverity};

#[derive(Debug)]
pub enum ExecutionError {
    Parse(TagParseError),
    Command(anyhow::Error),
    MetadataCheck(MetadataCheckError),
}

impl GetErrorSeverity for ExecutionError {
    fn get_severity(&self) -> ErrorSeverity {
        // Simply ignore commands which are dev only
        if let ExecutionError::MetadataCheck(MetadataCheckError::DevOnlyCommand) = self {
            return ErrorSeverity::Low;
        } else if let ExecutionError::MetadataCheck(MetadataCheckError::CommandDisabled) = self {
            return ErrorSeverity::Low;
        }

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
            ExecutionError::MetadataCheck(m) => m.fmt(f),
        }
    }
}
impl std::error::Error for ExecutionError {}

#[derive(Debug)]
pub enum MetadataCheckError {
    CommandOnCooldown(Duration),
    IllegalAgeRestrictedCommand,
    DevOnlyCommand,
    GuildManagerOnlyCommand,
    CommandDisabled,
    GuildOnly,
}
impl Display for MetadataCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataCheckError::CommandOnCooldown(time_left) => write!(
                f,
                "This command is on cooldown for {:.2} seconds.",
                time_left.as_millis() as f64 / 1000.0
            ),
            MetadataCheckError::IllegalAgeRestrictedCommand => {
                f.write_str("This command is only available in age restricted channels.")
            },
            MetadataCheckError::DevOnlyCommand => f.write_str("This command is limited to the Assyst developers only."),
            MetadataCheckError::GuildManagerOnlyCommand => {
                f.write_str("This command is limited to server managers only.")
            },
            MetadataCheckError::CommandDisabled => f.write_str("This command is disabled in this guild."),
            MetadataCheckError::GuildOnly => f.write_str("This command is only available within Discord servers."),
        }
    }
}
impl std::error::Error for MetadataCheckError {}

/// No arguments left
// (name, type), None for arguments we don't know of
#[derive(Clone, Debug)]
pub struct ArgsExhausted(pub Label);

#[derive(Debug)]
pub enum TagParseError {
    ArgsExhausted(ArgsExhausted),
    SubcommandArgsExhausted(String),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    ParseToMillisError(ParseToMillisError),
    // NB: boxed to reduce size -- twilight errors are very large (100+b), which would cause the
    // size of this enum to explode
    // these are very unlikely to occur, so it's okay
    TwilightHttp(Box<twilight_http::Error>),
    TwilightDeserialize(Box<twilight_http::response::DeserializeBodyError>),
    DownloadError(DownloadError),
    UnsupportedSticker(StickerFormatType),
    Reqwest(reqwest::Error),
    NoAttachment,
    NoMention,
    NoUrl,
    NoReply,
    NoEmbed,
    NoEmoji,
    NoSticker,
    NoImageInHistory,
    NoImageFound,
    MediaDownloadFail,
    InvalidSubcommand(String),
    NoInteractionSubcommandProvided,
    InteractionCommandIsBaseSubcommand,
    MismatchedCommandOptionType((String, CommandOptionValue)),
    FlagParseError(anyhow::Error),
    FailedToGetMessageHistory,
    MessageHistoryUnavailableInContext,
}

impl GetErrorSeverity for TagParseError {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            Self::TwilightHttp(..)
            | Self::TwilightDeserialize(..)
            | Self::DownloadError(..)
            | Self::UnsupportedSticker(..)
            | Self::Reqwest(..)
            | Self::FailedToGetMessageHistory
            | Self::MessageHistoryUnavailableInContext
            | Self::NoInteractionSubcommandProvided => ErrorSeverity::High,
            _ => ErrorSeverity::Low,
        }
    }
}

impl From<reqwest::Error> for TagParseError {
    fn from(v: reqwest::Error) -> Self {
        Self::Reqwest(v)
    }
}

impl Display for TagParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagParseError::ArgsExhausted(ArgsExhausted(Some((name, _)))) => {
                write!(f, "the argument '{name}' is required but was not found")
            },
            TagParseError::ArgsExhausted(ArgsExhausted(None)) => {
                f.write_str("an argument is required but none were found")
            },
            TagParseError::SubcommandArgsExhausted(_) => f.write_str("no valid subcommand was given"),
            TagParseError::ParseIntError(err) => {
                write!(f, "failed to parse an argument as a whole number: {err}")
            },
            TagParseError::ParseFloatError(err) => {
                write!(f, "failed to parse an argument as a decimal number: {err}")
            },
            TagParseError::ParseToMillisError(err) => {
                write!(f, "failed to parse an argument as time: {err}")
            },
            TagParseError::TwilightHttp(err) => {
                write!(f, "failed to send a request to discord: {err}")
            },
            TagParseError::TwilightDeserialize(err) => {
                write!(f, "failed to parse a response from discord: {err}")
            },
            TagParseError::DownloadError(err) => write!(f, "failed to download media: {err}"),
            TagParseError::UnsupportedSticker(sticker) => {
                write!(f, "an unsupported sticker was found: {sticker:?}")
            },
            TagParseError::Reqwest(err) => write!(f, "failed to send a request: {err}"),
            TagParseError::NoAttachment => f.write_str("an attachment was expected but none were found"),
            TagParseError::NoMention => f.write_str("a mention argument was expected but none were found"),
            TagParseError::NoUrl => f.write_str("a URL argument was expected but none were found"),
            TagParseError::NoReply => f.write_str("a reply was expected but none were found"),
            TagParseError::NoEmbed => f.write_str("an embed was expected but none were found"),
            TagParseError::NoEmoji => f.write_str("an emoji argument was expected but none were found"),
            TagParseError::NoSticker => f.write_str("a sticker was expected but none were found"),
            TagParseError::NoImageInHistory => {
                f.write_str("an image was expected in the channel but no image could be found")
            },
            TagParseError::NoImageFound => {
                f.write_str("an image was expected as an argument, but no image could be found")
            },
            TagParseError::MediaDownloadFail => f.write_str("failed to download media content"),
            TagParseError::InvalidSubcommand(name) => {
                write!(f, "no subcommand found for given subcommand name {name}")
            },
            TagParseError::MismatchedCommandOptionType((expected, received)) => {
                write!(
                    f,
                    "Command option mismatch between expected ({expected}) and received ({received:?})"
                )
            },
            TagParseError::NoInteractionSubcommandProvided => {
                f.write_str("Attempted to execute an interaction base command on a command group")
            },
            TagParseError::InteractionCommandIsBaseSubcommand => {
                f.write_str("Interaction subcommand is base subcommand")
            },
            TagParseError::FlagParseError(x) => write!(f, "Error parsing command flags ({x})"),
            TagParseError::FailedToGetMessageHistory => f.write_str(
                "Failed to get message history. Make sure Assyst has permission to do this. Assyst also cannot search for images through a global user install.",
            ),
            TagParseError::MessageHistoryUnavailableInContext => f.write_str(
                "Assyst can't search the channel for images in a user install. Please provide an image to operate on.",
            ),
        }
    }
}
impl std::error::Error for TagParseError {}

impl From<DownloadError> for TagParseError {
    fn from(v: DownloadError) -> Self {
        Self::DownloadError(v)
    }
}

impl From<twilight_http::response::DeserializeBodyError> for TagParseError {
    fn from(v: twilight_http::response::DeserializeBodyError) -> Self {
        Self::TwilightDeserialize(Box::new(v))
    }
}

impl From<twilight_http::Error> for TagParseError {
    fn from(v: twilight_http::Error) -> Self {
        Self::TwilightHttp(Box::new(v))
    }
}

impl From<ParseToMillisError> for TagParseError {
    fn from(v: ParseToMillisError) -> Self {
        Self::ParseToMillisError(v)
    }
}

impl From<ArgsExhausted> for TagParseError {
    fn from(value: ArgsExhausted) -> Self {
        Self::ArgsExhausted(value)
    }
}
impl From<ParseIntError> for TagParseError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}
impl From<ParseFloatError> for TagParseError {
    fn from(value: ParseFloatError) -> Self {
        Self::ParseFloatError(value)
    }
}
