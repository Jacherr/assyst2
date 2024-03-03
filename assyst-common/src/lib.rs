#![feature(duration_constants, trait_alias)]

pub mod ansi;
pub mod cache;
pub mod config;
pub mod macros;
pub mod markdown;
pub mod pipe;
pub mod prometheus;
pub mod util;

pub static BOT_ID: u64 = 571661221854707713;

#[cfg(test)]
mod tests {
    use std::thread::sleep;
    use std::time::Duration;

    use self::util::rate_tracker::RateTracker;

    use super::*;

    #[test]
    fn rate_tracker_seq() {
        let mut tracker = RateTracker::new(Duration::from_secs_f32(0.3));
        tracker.add_sample(1);
        sleep(Duration::from_millis(350));
        tracker.add_sample(3);
        tracker.add_sample(5);
        assert_eq!(tracker.get_rate(), Some(2));
    }

    #[test]
    fn rate_tracker_neg() {
        let mut tracker = RateTracker::new(Duration::from_secs_f32(0.3));
        tracker.add_sample(5);
        sleep(Duration::from_millis(350));
        tracker.add_sample(1);
        tracker.add_sample(-3);
        assert_eq!(tracker.get_rate(), Some(-4));
    }

    #[test]
    fn rate_tracker_single() {
        let mut tracker = RateTracker::new(Duration::from_secs_f32(0.3));
        tracker.add_sample(1);
        assert_eq!(tracker.get_rate(), Some(0));
    }

    #[test]
    fn rate_tracker_none() {
        let mut tracker = RateTracker::new(Duration::from_secs_f32(0.3));
        assert_eq!(tracker.get_rate(), None);
    }
}
