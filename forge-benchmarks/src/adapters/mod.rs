//! Adapters module - TypeScript-to-Rust benchmark adapters.
//!
//! This module provides the `BenchTarget` trait and adapter implementations
//! for benchmarking LLM-Forge TypeScript operations from Rust.

mod forge_adapter;

use crate::benchmarks::result::BenchmarkResult;
use async_trait::async_trait;
use std::error::Error;

// Re-export adapters
pub use forge_adapter::*;

/// Trait for benchmark targets.
///
/// Implement this trait to create a new benchmark target that can be
/// registered with `all_targets()` and executed via `run_all_benchmarks()`.
///
/// # Example
///
/// ```rust
/// use forge_benchmarks::adapters::BenchTarget;
/// use forge_benchmarks::benchmarks::result::BenchmarkResult;
/// use async_trait::async_trait;
/// use serde_json::json;
///
/// struct MyBenchmark;
///
/// #[async_trait]
/// impl BenchTarget for MyBenchmark {
///     fn id(&self) -> String {
///         "my-benchmark".to_string()
///     }
///
///     async fn run(&self) -> Result<BenchmarkResult, Box<dyn std::error::Error + Send + Sync>> {
///         // Perform benchmark...
///         Ok(BenchmarkResult::new(
///             self.id(),
///             json!({"ops_per_sec": 1000000}),
///         ))
///     }
/// }
/// ```
#[async_trait]
pub trait BenchTarget: Send + Sync {
    /// Returns the unique identifier for this benchmark target.
    ///
    /// This ID is used in result files and reports to identify the benchmark.
    fn id(&self) -> String;

    /// Executes the benchmark and returns the result.
    ///
    /// # Returns
    ///
    /// A `BenchmarkResult` containing the metrics, or an error if the
    /// benchmark could not be executed.
    async fn run(&self) -> Result<BenchmarkResult, Box<dyn Error + Send + Sync>>;

    /// Returns a human-readable description of the benchmark.
    ///
    /// Default implementation returns the ID.
    fn description(&self) -> String {
        self.id()
    }
}

/// Returns all registered benchmark targets.
///
/// This function provides the canonical registry of all benchmark targets
/// that will be executed by `run_all_benchmarks()`.
///
/// # Returns
///
/// A vector of boxed `BenchTarget` implementations.
///
/// # Example
///
/// ```rust
/// use forge_benchmarks::adapters::all_targets;
///
/// let targets = all_targets();
/// for target in &targets {
///     println!("Registered: {} - {}", target.id(), target.description());
/// }
/// ```
pub fn all_targets() -> Vec<Box<dyn BenchTarget>> {
    vec![
        // TypeScript-to-Rust benchmark adapters for Forge operations
        Box::new(forge_adapter::ProviderDetectionBenchmark::new()),
        Box::new(forge_adapter::ResponseParsingBenchmark::new()),
        Box::new(forge_adapter::SchemaValidationBenchmark::new()),
        Box::new(forge_adapter::CliParseBenchmark::new()),
        Box::new(forge_adapter::CliGenerateBenchmark::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_targets_not_empty() {
        let targets = all_targets();
        assert!(!targets.is_empty(), "Should have registered targets");
    }

    #[test]
    fn test_all_targets_unique_ids() {
        let targets = all_targets();
        let mut ids: Vec<_> = targets.iter().map(|t| t.id()).collect();
        let original_len = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(
            ids.len(),
            original_len,
            "All target IDs should be unique"
        );
    }

    #[test]
    fn test_target_descriptions() {
        let targets = all_targets();
        for target in &targets {
            let desc = target.description();
            assert!(!desc.is_empty(), "Description should not be empty");
        }
    }
}
