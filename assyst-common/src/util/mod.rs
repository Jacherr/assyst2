use std::process::Command;

pub mod discord;
pub mod rate_tracker;

pub mod regexes {
    use lazy_static::lazy_static;
    use regex::Regex;

    lazy_static! {
        pub static ref CUSTOM_EMOJI: Regex = Regex::new(r"<a?:(\w+):(\d{16,20})>").unwrap();
        pub static ref TENOR_GIF: Regex = Regex::new(r"https://\w+\.tenor\.com/[\w\-]+/[^\.]+\.gif").unwrap();
        pub static ref URL: Regex = Regex::new(
            r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)"
        )
        .unwrap();
        pub static ref USER_MENTION: Regex = Regex::new(r"(?:<@!?)?(\d{16,20})>?").unwrap();
        pub static ref TIME_STRING: Regex = Regex::new("(\\d+)([smhd])").unwrap();
        pub static ref COMMAND_FLAG: Regex = Regex::new(r#"\s+-(\w+)(?: *"([^"]+)"| *([^\-\s]+))?"#).unwrap();
    }
}

/// Attempts to extract memory usage for a process by pid
pub fn get_memory_usage_for(pid: &str) -> Option<usize> {
    let field = 1;
    let contents = std::fs::read(&format!("/proc/{pid}/statm")).ok()?;
    let contents = String::from_utf8(contents).ok()?;
    let s = contents.split_whitespace().nth(field)?;
    let npages = s.parse::<usize>().ok()?;
    Some(npages * 4096)
}

/// Attempts to extract memory usage for the current process
pub fn get_own_memory_usage() -> Option<usize> {
    get_memory_usage_for("self")
}

/// Attempts to extract memory usage for the current process
pub fn pid_of(pid: &str) -> Option<usize> {
    let result = exec_sync(&format!("pidof {pid}")).ok()?.stdout;
    Some(result.trim().parse().ok()?)
}

pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
}

/// Executes a bash command
pub fn exec_sync(command: &str) -> Result<CommandOutput, std::io::Error> {
    let mut cmd = Command::new("bash");
    cmd.args(&["-c", command]);

    let output = cmd.output()?;

    Ok(CommandOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

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
    let matches = regexes::TIME_STRING.captures_iter(input);

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
