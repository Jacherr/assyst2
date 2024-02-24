use crate::util::process::get_processes_mem_usage;
use crate::util::rate_tracker::RateTracker;
use assyst_database::DatabaseHandler;
use prometheus::{register_int_counter, register_int_gauge, register_int_gauge_vec, IntCounter, IntGauge, IntGaugeVec};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::info;

/// Gauges for all metrics tracked by Prometheus.
pub struct Prometheus {
    pub cache_sizes: IntGaugeVec,
    pub memory_usage: IntGaugeVec,
    pub guilds: IntGauge,
    pub events: IntCounter,
    pub events_rate_tracker: RateTracker,
    pub commands: IntCounter,
    pub commands_rate_tracker: RateTracker,
    pub database_handler: Arc<RwLock<DatabaseHandler>>,
}
impl Prometheus {
    pub fn new(database_handler: Arc<RwLock<DatabaseHandler>>) -> anyhow::Result<Prometheus> {
        Ok(Prometheus {
            cache_sizes: register_int_gauge_vec!("cache_sizes", "Cache sizes", &["cache"])?,
            memory_usage: register_int_gauge_vec!("memory_usage", "Memory usage in MB", &["process"])?,
            guilds: register_int_gauge!("guilds", "Total guilds")?,
            events: register_int_counter!("events", "Total number of events")?,
            events_rate_tracker: RateTracker::new(Duration::from_secs(1)),
            commands: register_int_counter!("commands", "Total number of commands executed")?,
            commands_rate_tracker: RateTracker::new(Duration::from_secs(60)),
            database_handler,
        })
    }

    pub fn update_cache_size(&self, cache: &str, size: usize) {
        self.cache_sizes.with_label_values(&[cache]).set(size as i64);
    }

    /// Updates some metrics that are not updated as data comes in.
    pub async fn update(&mut self) {
        info!("Collecting prometheus metrics");

        let database_cache_reader = self.database_handler.read().await;
        let prefixes_cache_size = database_cache_reader.cache.get_prefixes_cache_size();
        self.update_cache_size("prefixes", prefixes_cache_size);

        let memory_usages = get_processes_mem_usage();

        for usage in memory_usages {
            self.memory_usage
                .with_label_values(&[usage.0])
                .set((usage.1 / 1024 / 1024) as i64);
        }
    }

    pub fn add_guilds(&mut self, guilds: u64) {
        self.guilds.add(guilds as i64);
    }

    pub fn inc_guilds(&mut self) {
        self.guilds.inc();
    }

    pub fn dec_guilds(&mut self) {
        self.guilds.dec();
    }

    pub fn add_event(&mut self) {
        self.events.inc();
        self.events_rate_tracker.add_sample(self.events.get() as _);
    }

    pub fn get_events_rate(&mut self) -> Option<isize> {
        self.events_rate_tracker.get_rate()
    }

    pub fn add_command(&mut self) {
        self.commands.inc();
        self.commands_rate_tracker.add_sample(self.commands.get() as _);
    }

    pub fn get_commands_rate(&mut self) -> Option<isize> {
        self.commands_rate_tracker.get_rate()
    }
}
