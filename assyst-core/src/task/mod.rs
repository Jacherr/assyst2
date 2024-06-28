use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::spawn;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::assyst::ThreadSafeAssyst;

pub type TaskResult = Pin<Box<dyn Future<Output = ()> + Send>>;
pub type TaskRun = Box<dyn Fn(ThreadSafeAssyst) -> TaskResult + Send + Sync>;

pub mod tasks;

#[macro_export]
macro_rules! function_task_callback {
    ($expression:expr) => {
        Box::new(move |assyst: ThreadSafeAssyst| Box::pin($expression(assyst.clone())))
    };
}

/// A Task is a function which is called repeatedly on a set interval.
///
/// A Task can be created to run on its own thread, and once per interval the provided function will
/// be executed.
pub struct Task {
    _thread: JoinHandle<()>,
}
impl Task {
    pub fn new(assyst: ThreadSafeAssyst, interval: Duration, callback: TaskRun) -> Task {
        let thread = spawn(async move {
            loop {
                callback(assyst.clone()).await;
                sleep(interval).await;
            }
        });

        Task { _thread: thread }
    }

    pub fn new_delayed(assyst: ThreadSafeAssyst, interval: Duration, delay: Duration, callback: TaskRun) -> Task {
        let thread = spawn(async move {
            sleep(delay).await;
            loop {
                callback(assyst.clone()).await;
                sleep(interval).await;
            }
        });

        Task { _thread: thread }
    }
}
