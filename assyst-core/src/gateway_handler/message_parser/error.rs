use std::fmt::Display;

use twilight_model::channel::message::MessageType;

pub trait GetErrorSeverity {
    fn get_severity(&self) -> ErrorSeverity;
}

#[derive(Debug)]
/// An error when pre-processing the message.
pub enum PreParseError {
    /// Message does not start with the correct prefix.
    MessageNotPrefixed(String),
    /// Invocating user is globally blacklisted from using the bot.
    UserGloballyBlacklisted(u64),
    /// Invocating user is a bot or webhook.
    UserIsBotOrWebhook(Option<u64>),
    /// The kind of message is not supported, e.g., a user join system message.
    UnsupportedMessageKind(MessageType),
    /// A MESSAGE_UPDATE was received, but it had no edited timestamp.
    EditedMessageWithNoTimestamp,
    /// Other unknown failure. Unexpected error with high severity.
    Failure(String),
}
impl Display for PreParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MessageNotPrefixed(prefix) => {
                write!(f, "Message does not start with correct prefix ({prefix})")
            },
            Self::UserGloballyBlacklisted(id) => {
                write!(f, "User {id} is globally blacklisted")
            },
            Self::UserIsBotOrWebhook(id) => {
                write!(f, "User is a bot or webhook ({})", id.unwrap_or(0))
            },
            Self::UnsupportedMessageKind(kind) => {
                write!(f, "Unsupported message kind ({kind:?})")
            },
            Self::EditedMessageWithNoTimestamp => f.write_str("The message was updated, but not edited."),
            Self::Failure(message) => {
                write!(f, "Preprocessor failure: {message}")
            },
        }
    }
}
impl GetErrorSeverity for PreParseError {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            PreParseError::Failure(_) => ErrorSeverity::High,
            _ => ErrorSeverity::Low,
        }
    }
}
impl std::error::Error for PreParseError {}

#[derive(Debug)]
pub enum MetadataCheckInvalidated {}
impl GetErrorSeverity for MetadataCheckInvalidated {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            _ => todo!(),
        }
    }
}
impl Display for MetadataCheckInvalidated {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    /// Failure with preprocessing of the message.
    PreParseFail(PreParseError),
}
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreParseFail(message) => {
                write!(f, "Pre-parse failed: {message}")
            },
        }
    }
}
impl std::error::Error for ParseError {}
impl GetErrorSeverity for ParseError {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            ParseError::PreParseFail(e) => e.get_severity(),
        }
    }
}
impl From<PreParseError> for ParseError {
    fn from(value: PreParseError) -> Self {
        ParseError::PreParseFail(value)
    }
}

#[derive(PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    High,
}
