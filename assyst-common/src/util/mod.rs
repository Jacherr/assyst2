use std::time::Duration;

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

        let unit: u64 = unit_to_ms(&current[2])
            .try_into()
            .map_err(|_| ParseToMillisError::Overflow)?;

        let ms = amount.checked_mul(unit).ok_or(ParseToMillisError::Overflow)?;

        total = total.checked_add(ms).ok_or(ParseToMillisError::Overflow)?;
    }

    Ok(total)
}

/// Initialises tracing logging.
pub fn tracing_init() {
    let filter = EnvFilter::from_default_env().add_directive("twilight_gateway=info".parse().unwrap());
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
