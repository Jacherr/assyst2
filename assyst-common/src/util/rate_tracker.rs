use std::time::Duration;

use tokio::time::Instant;

/// Struct to allow the tracking of how fast a value increases, or how fast a state changes.
///
/// For example, can be used to determine how frequently a command is ran over a time period,
/// or the rate of events being received.
pub struct RateTracker {
    tracking_length: Duration,
    samples: Vec<Instant>,
}
impl RateTracker {
    pub fn new(tracking_length: Duration) -> RateTracker {
        RateTracker {
            tracking_length,
            samples: vec![],
        }
    }

    /// Removes all samples from this tracker which are older than the tracking length.
    pub fn remove_expired_samples(&mut self) {
        self.samples
            .retain(|x| Instant::now().duration_since(*x) <= self.tracking_length);
    }

    /// Add a sample to the tracker.
    pub fn add_sample(&mut self) {
        self.samples.push(Instant::now());
        self.remove_expired_samples();
    }

    /// Remove the oldest sample from the tracker.
    pub fn remove_sample(&mut self) {
        if !self.samples.is_empty() {
            self.samples.remove(0);
        }
        self.remove_expired_samples();
    }

    /// Fetches the amount of current non-expired samples.
    pub fn get_rate(&mut self) -> usize {
        self.remove_expired_samples();
        self.samples.len()
    }
}
