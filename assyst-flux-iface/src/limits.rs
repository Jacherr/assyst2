use std::time::Duration;

pub struct LimitData {
    pub time: Duration,
    pub size: u64,
    pub frames: u64,
    pub video_decode_enabled: bool,
}

pub const LIMITS_FREE: LimitData = LimitData {
    time: Duration::from_secs(40),
    size: 768,
    frames: 150,
    video_decode_enabled: false,
};

pub const LIMITS_USER_TIER_1: LimitData = LimitData {
    time: Duration::from_secs(60),
    size: 1024,
    frames: 200,
    video_decode_enabled: true,
};

pub const LIMITS_USER_TIER_2: LimitData = LimitData {
    time: Duration::from_secs(80),
    size: 2048,
    frames: 225,
    video_decode_enabled: true,
};

pub const LIMITS_USER_TIER_3: LimitData = LimitData {
    time: Duration::from_secs(120),
    size: 4096,
    frames: 250,
    video_decode_enabled: true,
};

pub const LIMITS_GUILD_TIER_1: LimitData = LimitData {
    time: Duration::from_secs(60),
    size: 1024,
    frames: 200,
    video_decode_enabled: true,
};

pub fn premium_user_to_limits(tier: u64) -> LimitData {
    match tier {
        0 => LIMITS_FREE,
        1 => LIMITS_GUILD_TIER_1,
        2 => LIMITS_USER_TIER_2,
        3 => LIMITS_USER_TIER_3,
        _ => unreachable!(),
    }
}
