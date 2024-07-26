use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref CUSTOM_EMOJI: Regex = Regex::new(r"<a?:(\w+):(\d{16,20})>").unwrap();
    pub static ref TENOR_GIF: Regex = Regex::new(r"https://\w+\.tenor\.com/[\w\-]+/[^\.]+\.gif").unwrap();
    pub static ref URL: Regex = Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap();
    pub static ref USER_MENTION: Regex = Regex::new(r"(?:<@!?)?(\d{16,20})>?").unwrap();
    pub static ref TIME_STRING: Regex = Regex::new("(\\d+)([smhd])").unwrap();
    pub static ref COMMAND_FLAG: Regex = Regex::new(r#"\s+-(\w+)(?: *"([^"]+)"| *([^\-\s]+))?"#).unwrap();
}
