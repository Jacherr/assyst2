use crate::util::process::get_processes_mem_usage;
use assyst_database::DatabaseHandler;
use prometheus::{register_int_gauge, register_int_gauge_vec, IntGauge, IntGaugeVec};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Gauges for all metrics tracked by Prometheus.
pub struct Prometheus {
    pub cache_sizes: IntGaugeVec,
    pub memory_usage: IntGaugeVec,
    pub guilds: IntGauge,
    pub database_handler: Arc<RwLock<DatabaseHandler>>,
}
impl Prometheus {
    pub fn new(database_handler: Arc<RwLock<DatabaseHandler>>) -> anyhow::Result<Prometheus> {
        Ok(Prometheus {
            cache_sizes: register_int_gauge_vec!("cache_sizes", "Cache sizes", &["cache"])?,
            memory_usage: register_int_gauge_vec!("memory_usage", "Memory usage in MB", &["process"])?,
            guilds: register_int_gauge!("guilds", "Total guilds")?,
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
}
