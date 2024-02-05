use prometheus::{register_int_gauge, register_int_gauge_vec, IntGauge, IntGaugeVec};
use tracing::info;

use crate::assyst::ThreadSafeAssyst;
use assyst_common::util::process::{get_memory_usage_for, get_own_memory_usage, pid_of};

/// Gauges for all metrics tracked by Prometheus.
pub struct Prometheus {
    pub cache_sizes: IntGaugeVec,
    pub memory_usage: IntGaugeVec,
    pub guilds: IntGauge,
}
impl Prometheus {
    pub fn new() -> anyhow::Result<Prometheus> {
        Ok(Prometheus {
            cache_sizes: register_int_gauge_vec!("cache_sizes", "Cache sizes", &["cache"])?,
            memory_usage: register_int_gauge_vec!("memory_usage", "Memory usage in MB", &["process"])?,
            guilds: register_int_gauge!("guilds", "Total guilds")?,
        })
    }

    pub fn update_cache_size(&self, cache: &str, size: usize) {
        self.cache_sizes.with_label_values(&[cache]).set(size as i64);
    }

    /// Updates some metrics that are not updated as data comes in.
    pub async fn update(&mut self, assyst: ThreadSafeAssyst) {
        info!("Collecting prometheus metrics");

        let database_cache_reader = assyst.database_handler.read().await;
        let prefixes_cache_size = database_cache_reader.cache.get_prefixes_cache_size();
        self.update_cache_size("prefixes", prefixes_cache_size);

        let core_memory_usage = get_own_memory_usage().unwrap_or(0) / 1024 / 1024;
        let gateway_pid = pid_of("assyst-gateway").unwrap_or(0).to_string();
        let gateway_memory_usage = get_memory_usage_for(&gateway_pid).unwrap_or(0) / 1024 / 1024;
        self.memory_usage
            .with_label_values(&["assyst-core"])
            .set(core_memory_usage as i64);
        self.memory_usage
            .with_label_values(&["assyst-gateway"])
            .set(gateway_memory_usage as i64);
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
