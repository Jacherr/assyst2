use std::sync::LazyLock;
use std::time::Duration;

pub struct LimitData {
    pub time: Duration,
    pub size: u64,
    pub frames: u64,
    pub video_decode_enabled: bool,
}

pub static LIMITS: LazyLock<Vec<LimitData>> = LazyLock::new(|| {
    vec![
        LimitData {
            time: Duration::from_secs(40),
            size: 768,
            frames: 150,
            video_decode_enabled: false,
        },
        LimitData {
            time: Duration::from_secs(60),
            size: 1024,
            frames: 200,
            video_decode_enabled: true,
        },
        LimitData {
            time: Duration::from_secs(80),
            size: 2048,
            frames: 225,
            video_decode_enabled: true,
        },
        LimitData {
            time: Duration::from_secs(120),
            size: 4096,
            frames: 250,
            video_decode_enabled: true,
        },
    ]
});
