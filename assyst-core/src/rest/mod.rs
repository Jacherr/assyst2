pub mod filer;
pub mod patreon;
pub mod rest_cache_handler;
pub mod top_gg;

pub static NORMAL_DISCORD_UPLOAD_LIMIT_BYTES: u64 = 25_000_000;
pub static PREMIUM_TIER2_DISCORD_UPLOAD_LIMIT_BYTES: u64 = 50_000_000;
pub static PREMIUM_TIER3_DISCORD_UPLOAD_LIMIT_BYTES: u64 = 100_000_000;
