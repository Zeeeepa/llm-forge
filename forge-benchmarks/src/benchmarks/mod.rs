//! Benchmarks module - Core benchmark execution infrastructure.
//!
//! This module provides the canonical benchmark interface for the LLM-Forge project,
//! implementing the unified benchmark structure used across all 25 benchmark-target repositories.

pub mod io;
pub mod markdown;
pub mod result;

use crate::adapters::{all_targets, BenchTarget};
use result::BenchmarkResult;
use tracing::{info, warn};

/// Runs all registered benchmark targets and returns their results.
///
/// This is the main entrypoint for the benchmark suite, executing each
/// registered `BenchTarget` and collecting `BenchmarkResult` instances.
///
/// # Returns
///
/// A `Vec<BenchmarkResult>` containing the results from all benchmark targets.
///
/// # Example
///
/// ```rust,no_run
/// use forge_benchmarks::benchmarks::run_all_benchmarks;
///
/// #[tokio::main]
/// async fn main() {
///     let results = run_all_benchmarks().await;
///     println!("Completed {} benchmarks", results.len());
/// }
/// ```
pub async fn run_all_benchmarks() -> Vec<BenchmarkResult> {
    let targets = all_targets();
    let mut results = Vec::with_capacity(targets.len());

    info!("Starting benchmark suite with {} targets", targets.len());

    for target in targets {
        let target_id = target.id();
        info!("Running benchmark: {}", target_id);

        match target.run().await {
            Ok(result) => {
                info!(
                    "Benchmark {} completed successfully",
                    result.target_id
                );
                results.push(result);
            }
            Err(e) => {
                warn!("Benchmark {} failed: {}", target_id, e);
                // Create a failed result entry
                results.push(BenchmarkResult::failed(target_id, e.to_string()));
            }
        }
    }

    info!("Benchmark suite completed: {} results", results.len());
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_all_benchmarks() {
        let results = run_all_benchmarks().await;
        // Should have at least the registered targets
        assert!(!results.is_empty(), "Should have benchmark results");

        for result in &results {
            assert!(!result.target_id.is_empty(), "Target ID should not be empty");
            assert!(result.timestamp <= chrono::Utc::now(), "Timestamp should be in the past");
        }
    }
}
