use crate::util::process::get_processes_mem_usage;
use crate::util::rate_tracker::RateTracker;
use assyst_database::DatabaseHandler;
use prometheus::{register_int_counter, register_int_gauge, register_int_gauge_vec, IntCounter, IntGauge, IntGaugeVec};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::info;

/// Handler for general metrics, including rate trackers, Prometheus metrics, etc.
pub struct MetricsHandler {
    pub cache_sizes: IntGaugeVec,
    pub memory_usage: IntGaugeVec,
    pub guilds: IntGauge,
    pub events: IntCounter,
    pub events_rate_tracker: Mutex<RateTracker>,
    pub commands: IntCounter,
    pub total_commands_rate_tracker: Mutex<RateTracker>,
    pub individual_commands_rate_trackers: Mutex<HashMap<&'static str /* command name */, RateTracker>>,
    pub database_handler: Arc<DatabaseHandler>,
}
impl MetricsHandler {
    pub fn new(database_handler: Arc<DatabaseHandler>) -> anyhow::Result<MetricsHandler> {
        Ok(MetricsHandler {
            cache_sizes: register_int_gauge_vec!("cache_sizes", "Cache sizes", &["cache"])?,
            memory_usage: register_int_gauge_vec!("memory_usage", "Memory usage in MB", &["process"])?,
            guilds: register_int_gauge!("guilds", "Total guilds")?,
            events: register_int_counter!("events", "Total number of events")?,
            events_rate_tracker: Mutex::new(RateTracker::new(Duration::from_secs(1))),
            commands: register_int_counter!("commands", "Total number of commands executed")?,
            total_commands_rate_tracker: Mutex::new(RateTracker::new(Duration::from_secs(60))),
            individual_commands_rate_trackers: Mutex::new(HashMap::new()),
            database_handler,
        })
    }

    pub fn update_cache_size(&self, cache: &str, size: usize) {
        self.cache_sizes.with_label_values(&[cache]).set(size as i64);
    }

    /// Updates some metrics that are not updated as data comes in.
    pub async fn update(&self) {
        info!("Collecting prometheus metrics");

        let database_cache_reader = &self.database_handler;
        let prefixes_cache_size = database_cache_reader.cache.get_prefixes_cache_size();
        self.update_cache_size("prefixes", prefixes_cache_size);

        let memory_usages = get_processes_mem_usage();

        for usage in memory_usages {
            self.memory_usage
                .with_label_values(&[usage.0])
                .set((usage.1 / 1024 / 1024) as i64);
        }
    }

    pub fn add_guilds(&self, guilds: u64) {
        self.guilds.add(guilds as i64);
    }

    pub fn inc_guilds(&self) {
        self.guilds.inc();
    }

    pub fn dec_guilds(&self) {
        self.guilds.dec();
    }

    pub fn add_event(&self) {
        self.events.inc();
        self.events_rate_tracker.lock().unwrap().add_sample();
    }

    pub fn get_events_rate(&self) -> usize {
        self.events_rate_tracker.lock().unwrap().get_rate()
    }

    pub fn add_command(&self) {
        self.commands.inc();
        self.total_commands_rate_tracker.lock().unwrap().add_sample();
    }

    pub fn get_commands_rate(&self) -> usize {
        self.total_commands_rate_tracker.lock().unwrap().get_rate()
    }

    pub fn add_individual_command_usage(&self, command_name: &'static str) {
        let mut lock = self.individual_commands_rate_trackers.lock().unwrap();
        let entry = lock.get_mut(&command_name);
        if let Some(entry) = entry {
            entry.add_sample();
        } else {
            let mut tracker = RateTracker::new(Duration::from_secs(60 * 60));
            tracker.add_sample();
            lock.insert(command_name, tracker);
        }
    }
}
