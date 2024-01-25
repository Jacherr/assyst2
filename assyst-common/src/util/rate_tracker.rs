use std::time::Duration;

use tokio::time::Instant;

pub struct RateTracker {
    tracking_length: Duration,
    samples: Vec<(usize, Instant)>,
}
impl RateTracker {
    pub fn new(trackling_length: Duration) -> RateTracker {
        RateTracker {
            tracking_length,
            samples: vec![],
        }
    }

    pub fn add_sample(&mut self, value: usize) {
        self.samples.push((i.uses as usize, Instant::now()));
        let mut to_remove = vec![];
        for (pos, entry) in self.samples.iter().enumerate() {
            // remove old entries
            if Instant::now().duration_since(entry.1).as_secs() > self.tracking_length {
                to_remove.push(pos);
            }
        }
        for i in to_remove {
            self.samples.remove(i);
        }
        self.samples.sort_by(|a, b| a.0.cmp(&b.0));
    }

    pub fn get_rate(&self) -> Option<usize> {
        Some(self.samples.last()?.0 - self.samples.first()?.0)
    }
}
