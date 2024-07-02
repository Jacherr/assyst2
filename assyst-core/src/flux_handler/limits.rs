use std::time::Duration;

pub struct LimitData {
    pub time: Duration,
    pub size: u64,
    pub frames: u64,
}

lazy_static::lazy_static! {
    pub static ref LIMITS: Vec<LimitData> = {
        let limits: Vec<LimitData> = vec![
            LimitData {
                time: Duration::from_secs(40),
                size: 768,
                frames: 150,
            },
            LimitData {
                time: Duration::from_secs(60),
                size: 1024,
                frames: 200
            },
            LimitData {
                time: Duration::from_secs(80),
                size: 2048,
                frames: 225,
            },
            LimitData {
                time: Duration::from_secs(120),
                size: 4096,
                frames: 250,
            }
        ];
        limits
    };
}
