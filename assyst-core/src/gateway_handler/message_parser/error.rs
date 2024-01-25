use std::fmt::Display;

trait GetErrorSeverity {
    fn get_severity(&self) -> ErrorSeverity;
}

#[derive(Debug)]
/// An error when pre-processing the message.
pub enum PreParseError {
    /// Invocating user is globally blacklisted from using the bot.
    UserGloballyBlacklisted,
    /// Other unknown failure. Unexpected error with high severity.
    Failure(String)
}
impl Display for PreParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserGloballyBlacklisted => {
                write!(f, "User is globally blacklisted")
            },
            Self::Failure(message) => {
                write!(f, "Preprocessor failure: {}", message)
            },
        }
    }
}
impl GetErrorSeverity for PreParseError {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            PreParseError::Failure(_) => ErrorSeverity::High,
            _ => ErrorSeverity::Low
        }
    }
}
impl std::error::Error for PreParseError {}

#[derive(Debug)]
pub enum ParseError {
    PreParseFail(PreParseError),
}
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreParseFail(message) => {
                write!(f, "Pre-parse failed: {}", message)
            },
        }
    }
}
impl std::error::Error for ParseError {}
impl GetErrorSeverity for ParseError {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            ParseError::PreParseFail(e) => e.get_severity(),
            _ => ErrorSeverity::Low
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
    High
}

pub fn get_parser_error_severity<T: GetErrorSeverity>(error: &T) -> ErrorSeverity {
    error.get_severity()
}