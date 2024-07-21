use std::borrow::Cow;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

use rand::Rng;
use time::format_description;
use tracing::info;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::EnvFilter;

pub mod discord;
pub mod filetype;
pub mod process;
pub mod rate_tracker;
pub mod regex;
pub mod table;

/// Converts a unit string (s, m, h, d) to milliseconds
fn unit_to_ms(u: &str) -> u64 {
    match u {
        "s" => 1000,
        "m" => 1000 * 60,
        "h" => 1000 * 60 * 60,
        "d" => 1000 * 60 * 60 * 24,
        _ => unreachable!(),
    }
}

#[derive(Debug)]
pub enum ParseToMillisError {
    ParseIntError,
    Overflow,
}

impl std::fmt::Display for ParseToMillisError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseToMillisError::ParseIntError => write!(f, "Input string is too large to fit in numeric type"),
            ParseToMillisError::Overflow => write!(f, "Final time is too large to fit in numeric type")
        }
    }
}

impl std::error::Error for ParseToMillisError {}

/// Parses a string such as 2h1m20s to milliseconds
pub fn parse_to_millis(input: &str) -> Result<u64, ParseToMillisError> {
    let matches = regex::TIME_STRING.captures_iter(input);

    let mut total: u64 = 0;

    for current in matches {
        let amount = current[1]
            .parse::<u64>()
            .map_err(|_| ParseToMillisError::ParseIntError)?;

        let unit: u64 = unit_to_ms(&current[2]);

        let ms = amount.checked_mul(unit).ok_or(ParseToMillisError::Overflow)?;

        total = total.checked_add(ms).ok_or(ParseToMillisError::Overflow)?;
    }

    Ok(total)
}

/// Initialises tracing logging.
pub fn tracing_init() {
    let filter = EnvFilter::from_default_env()
        .add_directive("twilight_gateway=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap());
    let description = "[year]-[month]-[day] [hour]:[minute]:[second]";

    tracing_subscriber::fmt()
        .with_timer(UtcTime::new(format_description::parse(description).unwrap()))
        .with_line_number(true)
        .with_env_filter(filter)
        .init();

    info!("Initialised logging");
}

pub fn format_duration(duration: &Duration) -> String {
    if *duration > Duration::SECOND {
        let seconds = duration.as_millis() as f64 / 1000.0;
        format!("{seconds:.2}s")
    } else if *duration > Duration::MILLISECOND {
        let millis = duration.as_micros() as f64 / 1000.0;
        format!("{millis:.2}ms")
    } else {
        let micros = duration.as_nanos() as f64 / 1000.0;
        format!("{micros:.2}μs")
    }
}

mod units {
    pub const SECOND: u64 = 1000;
    pub const MINUTE: u64 = SECOND * 60;
    pub const HOUR: u64 = MINUTE * 60;
    pub const DAY: u64 = HOUR * 24;
}

/// Pluralizes a string
pub fn pluralize<'a>(s: &'a str, adder: &str, count: u64) -> Cow<'a, str> {
    if count == 1 {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(s.to_owned() + adder)
    }
}

/// Converts a timestamp to a humanly readable string
pub fn format_time(input: u64) -> String {
    if input >= units::DAY {
        let amount = input / units::DAY;
        format!("{} {}", amount, pluralize("day", "s", amount))
    } else if input >= units::HOUR {
        let amount = input / units::HOUR;
        format!("{} {}", amount, pluralize("hour", "s", amount))
    } else if input >= units::MINUTE {
        let amount = input / units::MINUTE;
        format!("{} {}", amount, pluralize("minute", "s", amount))
    } else {
        let amount = input / units::SECOND;
        format!("{} {}", amount, pluralize("second", "s", amount))
    }
}

/// Like [`String::from_utf8_lossy`], but takes an owned `Vec<u8>` and is
/// able to reuse the vec's allocation if the bytes are valid UTF-8.
///
/// It is much more efficient for valid UTF-8, but will be
/// much worse than `String::from_utf8` for invalid UTF-8, so
/// only use it if valid UTF-8 is likely!
pub fn string_from_likely_utf8(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).unwrap_or_else(|err| {
        // Unlucky, data was invalid UTF-8, so try again but use lossy decoding this time.
        String::from_utf8_lossy(err.as_bytes()).into_owned()
    })
}

/// Hashes a buffer. Appends a random string.
pub fn hash_buffer(buf: &[u8]) -> String {
    let mut body_hasher = DefaultHasher::new();
    buf.hash(&mut body_hasher);
    let rand = rand::thread_rng().gen::<usize>();
    format!("{:x}{:x}", body_hasher.finish(), rand)
}

pub fn sanitise_filename(name: &str) -> String {
    name.replace("/", "_")
        .replace("<", "_")
        .replace(">", "_")
        .replace(":", "_")
        .replace("\"", "_")
        .replace("|", "_")
        .replace("\\", "_")
        .replace("?", "_")
        .replace("*", "_")
}
