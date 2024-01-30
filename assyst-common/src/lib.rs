pub mod ansi;
pub mod assyst;
pub mod command;
pub mod config;
pub mod macros;
pub mod pipe;
pub mod task;
pub mod util;

pub static BOT_ID: u64 = 571661221854707713;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::Duration;

    use tokio::sync::Mutex;

    use self::assyst::Assyst;
    use self::task::Task;
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

    async fn task_fn(assyst: Arc<Mutex<Assyst>>) {}

    #[tokio::test]
    async fn task_create() {
        let assyst = Arc::new(Mutex::new(Assyst::new().await.unwrap()));
        let task = Task::new(
            assyst.clone(),
            Duration::from_secs(10),
            Box::new(move |assyst| Box::pin(task_fn(assyst))),
        );
        assert_eq!(task.is_running(), true);
    }

    #[tokio::test]
    async fn task_terminate() {
        let assyst = Arc::new(Mutex::new(Assyst::new().await.unwrap()));
        let task = Task::new(
            assyst.clone(),
            Duration::from_secs_f32(0.3),
            Box::new(move |assyst| Box::pin(task_fn(assyst))),
        );
        assert_eq!(task.terminate().await, true);
    }
}
