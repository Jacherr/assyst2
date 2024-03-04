use std::fmt::Display;
use std::num::{ParseFloatError, ParseIntError};

use assyst_common::util::ParseToMillisError;
use twilight_model::channel::message::sticker::StickerFormatType;

use crate::downloader::DownloadError;
use crate::gateway_handler::message_parser::error::{ErrorSeverity, GetErrorSeverity};

use super::arguments::DesiredCmpTy;

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

/// No arguments left
pub struct ArgsExhausted;

#[derive(Debug)]
pub enum TagParseError {
    ArgsExhausted,
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    ParseToMillisError(ParseToMillisError),
    // NB: boxed to reduce size -- twilight errors are very large (100+b), which would cause the size of this enum to
    // explode
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
    IllegalAgeRestrictedCommand,
    InvalidSubcommand,
    // given, cmp_to, adjective
    ComparisonError(DesiredCmpTy, DesiredCmpTy, String),
    // given, min, max
    RangeError(DesiredCmpTy, DesiredCmpTy, DesiredCmpTy),
}

impl GetErrorSeverity for TagParseError {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            Self::TwilightHttp(..)
            | Self::TwilightDeserialize(..)
            | Self::DownloadError(..)
            | Self::UnsupportedSticker(..)
            | Self::Reqwest(..) => ErrorSeverity::High,
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
            TagParseError::ArgsExhausted => f.write_str("an argument is required but none were found"),
            TagParseError::ParseIntError(err) => write!(f, "failed to parse an argument as a number: {err}"),
            TagParseError::ParseFloatError(err) => write!(f, "failed to parse an argument as a number: {err}"),
            TagParseError::ParseToMillisError(err) => write!(f, "failed to parse an argument as time: {err}"),
            TagParseError::TwilightHttp(_) => f.write_str("failed to send a request to discord"),
            TagParseError::TwilightDeserialize(_) => f.write_str("failed to parse a response from discord"),
            TagParseError::DownloadError(_) => f.write_str("failed to download media"),
            TagParseError::UnsupportedSticker(sticker) => write!(f, "an unsupported sticker was found: {sticker:?}"),
            TagParseError::Reqwest(_) => f.write_str("failed to send a request"),
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
            TagParseError::IllegalAgeRestrictedCommand => {
                f.write_str("this command is only available in age restricted channels")
            },
            TagParseError::InvalidSubcommand => f.write_str("no subcommand found for given name"),
            TagParseError::ComparisonError(given, cmp_to, adjective) => {
                write!(f, "expected a value {adjective} {cmp_to}, got {given}")
            },
            TagParseError::RangeError(given, min, max) => {
                write!(f, "expected a value between {min}-{max}, got {given}")
            },
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
    fn from(_: ArgsExhausted) -> Self {
        Self::ArgsExhausted
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
