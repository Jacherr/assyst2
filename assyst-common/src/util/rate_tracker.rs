use std::time::Duration;

use tokio::time::Instant;
use tracing::debug;

/// Struct to allow the tracking of how fast a value increases, or how fast a state changes.
///
/// For example, can be used to determine how frequently a command is ran over a time period,
/// or the rate of events being received.
pub struct RateTracker {
    tracking_length: Duration,
    samples: Vec<(isize, Instant)>,
}
impl RateTracker {
    pub fn new(tracking_length: Duration) -> RateTracker {
        RateTracker {
            tracking_length,
            samples: vec![],
        }
    }

    pub fn remove_expired_samples(&mut self) {
        self.samples
            .retain(|x| Instant::now().duration_since(x.1) <= self.tracking_length);
    }

    /// Add a sample to the tracker.
    ///
    /// The sample can take a value.
    pub fn add_sample(&mut self, value: isize) {
        // add new sample
        self.samples.push((value, Instant::now()));
        self.remove_expired_samples();
    }

    /// Fetches the difference between the largest and smallest sample in the tracker.
    pub fn get_rate(&mut self) -> Option<isize> {
        self.remove_expired_samples();
        Some(self.samples.last()?.0 - self.samples.first()?.0)
    }
}
