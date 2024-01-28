use std::time::Duration;

use tokio::time::Instant;
use tracing::debug;

/// Struct to allow the tracking of how fast a value increases, or how fast a state changes.
///
/// For example, can be used to determine how frequently a command is ran over a time period,
/// or the rate of events being received.
pub struct RateTracker {
    tracking_length: Duration,
    samples: Vec<(usize, Instant)>,
}
impl RateTracker {
    pub fn new(tracking_length: Duration) -> RateTracker {
        RateTracker {
            tracking_length,
            samples: vec![],
        }
    }

    /// Add a sample to the tracker.
    ///
    /// The sample can take a value. When a sample is added, the entire sample list is re-ordered
    /// from largest to smallest value.
    pub fn add_sample(&mut self, value: usize) {
        // add new sample
        self.samples.push((value, Instant::now()));
        let mut to_remove = vec![];
        for (pos, entry) in self.samples.iter().enumerate() {
            // determine which entries are out of range
            if Instant::now().duration_since(entry.1) > self.tracking_length {
                to_remove.push(pos);
            }
        }

        debug!("{} samples to remove (expired)", to_remove.len());

        // remove out of range entries
        for i in to_remove {
            self.samples.remove(i);
        }
        // sort samples oldest to most recent
        self.samples.sort_by(|a, b| a.0.cmp(&b.0));
    }

    /// Fetches the difference between the largest and smallest sample in the tracker.
    pub fn get_rate(&self) -> Option<usize> {
        Some(self.samples.last()?.0 - self.samples.first()?.0)
    }
}
