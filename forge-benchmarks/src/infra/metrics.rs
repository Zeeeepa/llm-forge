//! Metrics Integration (llm-config-metrics)
//!
//! Provides Prometheus-based metrics collection for benchmark operations
//! using the LLM-Dev-Ops Infra metrics system.

use lazy_static::lazy_static;
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec, Opts, Registry,
};
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during metrics operations
#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Metrics registration failed: {0}")]
    RegistrationError(String),

    #[error("Metrics collection failed: {0}")]
    CollectionError(String),

    #[error("Metrics export failed: {0}")]
    ExportError(String),
}

lazy_static! {
    /// Global metrics registry for benchmark metrics
    pub static ref BENCHMARK_REGISTRY: Arc<Registry> = Arc::new(Registry::new());

    /// Counter for total benchmarks executed
    pub static ref BENCHMARKS_TOTAL: Counter = Counter::new(
        "forge_benchmarks_total",
        "Total number of benchmarks executed"
    ).unwrap();

    /// Counter for successful benchmarks
    pub static ref BENCHMARKS_SUCCESS: Counter = Counter::new(
        "forge_benchmarks_success_total",
        "Total number of successful benchmarks"
    ).unwrap();

    /// Counter for failed benchmarks
    pub static ref BENCHMARKS_FAILED: Counter = Counter::new(
        "forge_benchmarks_failed_total",
        "Total number of failed benchmarks"
    ).unwrap();

    /// Histogram for benchmark duration (seconds)
    pub static ref BENCHMARK_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "forge_benchmark_duration_seconds",
            "Benchmark execution duration in seconds"
        ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0])
    ).unwrap();

    /// Gauge for current benchmark operations per second
    pub static ref BENCHMARK_OPS_PER_SEC: Gauge = Gauge::new(
        "forge_benchmark_ops_per_second",
        "Current benchmark operations per second"
    ).unwrap();

    /// Counter vec for benchmarks by target
    pub static ref BENCHMARKS_BY_TARGET: CounterVec = CounterVec::new(
        Opts::new(
            "forge_benchmarks_by_target_total",
            "Total benchmarks executed by target"
        ),
        &["target_id"]
    ).unwrap();

    /// Histogram vec for benchmark duration by target
    pub static ref BENCHMARK_DURATION_BY_TARGET: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "forge_benchmark_duration_by_target_seconds",
            "Benchmark duration by target in seconds"
        ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]),
        &["target_id"]
    ).unwrap();
}

/// Initialize the metrics system
pub fn init_metrics() -> Result<(), MetricsError> {
    BENCHMARK_REGISTRY
        .register(Box::new(BENCHMARKS_TOTAL.clone()))
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

    BENCHMARK_REGISTRY
        .register(Box::new(BENCHMARKS_SUCCESS.clone()))
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

    BENCHMARK_REGISTRY
        .register(Box::new(BENCHMARKS_FAILED.clone()))
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

    BENCHMARK_REGISTRY
        .register(Box::new(BENCHMARK_DURATION.clone()))
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

    BENCHMARK_REGISTRY
        .register(Box::new(BENCHMARK_OPS_PER_SEC.clone()))
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

    BENCHMARK_REGISTRY
        .register(Box::new(BENCHMARKS_BY_TARGET.clone()))
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

    BENCHMARK_REGISTRY
        .register(Box::new(BENCHMARK_DURATION_BY_TARGET.clone()))
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

    Ok(())
}

/// Record a benchmark execution
pub fn record_benchmark(target_id: &str, duration_secs: f64, success: bool) {
    BENCHMARKS_TOTAL.inc();
    BENCHMARK_DURATION.observe(duration_secs);

    if success {
        BENCHMARKS_SUCCESS.inc();
    } else {
        BENCHMARKS_FAILED.inc();
    }

    BENCHMARKS_BY_TARGET.with_label_values(&[target_id]).inc();
    BENCHMARK_DURATION_BY_TARGET
        .with_label_values(&[target_id])
        .observe(duration_secs);
}

/// Record operations per second for a benchmark
pub fn record_ops_per_sec(ops: f64) {
    BENCHMARK_OPS_PER_SEC.set(ops);
}

/// Export metrics in Prometheus text format
pub fn export_metrics() -> Result<String, MetricsError> {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = BENCHMARK_REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .map_err(|e| MetricsError::ExportError(e.to_string()))?;
    String::from_utf8(buffer).map_err(|e| MetricsError::ExportError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_benchmark() {
        record_benchmark("test-target", 0.1, true);
        // Metrics should be recorded without panic
    }

    #[test]
    fn test_record_ops_per_sec() {
        record_ops_per_sec(1_000_000.0);
        // Should be recorded without panic
    }
}
