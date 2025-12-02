//! Benchmark result types - Standardized result structure.
//!
//! This module defines the canonical `BenchmarkResult` struct used across
//! all 25 benchmark-target repositories for consistent result reporting.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Standardized benchmark result structure.
///
/// This struct contains the exact fields required by the unified benchmark interface:
/// - `target_id`: Unique identifier for the benchmark target
/// - `metrics`: JSON value containing benchmark-specific metrics
/// - `timestamp`: UTC timestamp when the benchmark completed
///
/// # Example
///
/// ```rust
/// use forge_benchmarks::benchmarks::result::BenchmarkResult;
/// use serde_json::json;
///
/// let result = BenchmarkResult::new(
///     "provider-detection".to_string(),
///     json!({
///         "ops_per_sec": 1_000_000,
///         "avg_ns": 1000,
///         "samples": 100
///     }),
/// );
///
/// assert_eq!(result.target_id, "provider-detection");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Unique identifier for the benchmark target.
    ///
    /// This should match the `id()` returned by the corresponding `BenchTarget`.
    pub target_id: String,

    /// JSON value containing benchmark-specific metrics.
    ///
    /// The structure of this field varies by benchmark type but commonly includes:
    /// - `ops_per_sec`: Operations per second
    /// - `avg_ns`: Average nanoseconds per operation
    /// - `min_ns`: Minimum nanoseconds observed
    /// - `max_ns`: Maximum nanoseconds observed
    /// - `samples`: Number of samples collected
    /// - `error`: Error message if the benchmark failed
    pub metrics: Value,

    /// UTC timestamp when the benchmark completed.
    pub timestamp: DateTime<Utc>,
}

impl BenchmarkResult {
    /// Creates a new `BenchmarkResult` with the current UTC timestamp.
    ///
    /// # Arguments
    ///
    /// * `target_id` - Unique identifier for the benchmark target
    /// * `metrics` - JSON value containing benchmark metrics
    ///
    /// # Returns
    ///
    /// A new `BenchmarkResult` instance with the current timestamp.
    pub fn new(target_id: String, metrics: Value) -> Self {
        Self {
            target_id,
            metrics,
            timestamp: Utc::now(),
        }
    }

    /// Creates a new `BenchmarkResult` with a specific timestamp.
    ///
    /// # Arguments
    ///
    /// * `target_id` - Unique identifier for the benchmark target
    /// * `metrics` - JSON value containing benchmark metrics
    /// * `timestamp` - Specific UTC timestamp
    ///
    /// # Returns
    ///
    /// A new `BenchmarkResult` instance with the specified timestamp.
    pub fn with_timestamp(target_id: String, metrics: Value, timestamp: DateTime<Utc>) -> Self {
        Self {
            target_id,
            metrics,
            timestamp,
        }
    }

    /// Creates a failed benchmark result with an error message.
    ///
    /// # Arguments
    ///
    /// * `target_id` - Unique identifier for the benchmark target
    /// * `error` - Error message describing the failure
    ///
    /// # Returns
    ///
    /// A new `BenchmarkResult` with error metrics.
    pub fn failed(target_id: String, error: String) -> Self {
        Self::new(
            target_id,
            serde_json::json!({
                "status": "failed",
                "error": error
            }),
        )
    }

    /// Checks if this benchmark result represents a failure.
    ///
    /// # Returns
    ///
    /// `true` if the metrics contain a "status" field with value "failed".
    pub fn is_failed(&self) -> bool {
        self.metrics
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s == "failed")
            .unwrap_or(false)
    }

    /// Gets the operations per second if available in metrics.
    ///
    /// # Returns
    ///
    /// `Some(ops)` if `ops_per_sec` is present, `None` otherwise.
    pub fn ops_per_sec(&self) -> Option<f64> {
        self.metrics.get("ops_per_sec").and_then(|v| v.as_f64())
    }

    /// Gets the average nanoseconds per operation if available.
    ///
    /// # Returns
    ///
    /// `Some(ns)` if `avg_ns` is present, `None` otherwise.
    pub fn avg_ns(&self) -> Option<f64> {
        self.metrics.get("avg_ns").and_then(|v| v.as_f64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_benchmark_result() {
        let result = BenchmarkResult::new(
            "test-target".to_string(),
            json!({"ops_per_sec": 1000000}),
        );

        assert_eq!(result.target_id, "test-target");
        assert_eq!(result.metrics["ops_per_sec"], 1000000);
        assert!(result.timestamp <= Utc::now());
    }

    #[test]
    fn test_failed_benchmark_result() {
        let result = BenchmarkResult::failed(
            "failing-target".to_string(),
            "Connection timeout".to_string(),
        );

        assert!(result.is_failed());
        assert_eq!(result.metrics["error"], "Connection timeout");
    }

    #[test]
    fn test_ops_per_sec() {
        let result = BenchmarkResult::new(
            "perf-target".to_string(),
            json!({"ops_per_sec": 500000.5, "avg_ns": 2000}),
        );

        assert_eq!(result.ops_per_sec(), Some(500000.5));
        assert_eq!(result.avg_ns(), Some(2000.0));
    }

    #[test]
    fn test_serialization() {
        let result = BenchmarkResult::new(
            "serialize-test".to_string(),
            json!({"samples": 100}),
        );

        let json_str = serde_json::to_string(&result).unwrap();
        let deserialized: BenchmarkResult = serde_json::from_str(&json_str).unwrap();

        assert_eq!(deserialized.target_id, result.target_id);
        assert_eq!(deserialized.metrics, result.metrics);
    }
}
