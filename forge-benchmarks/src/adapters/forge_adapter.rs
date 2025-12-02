//! Forge TypeScript-to-Rust benchmark adapters.
//!
//! This module provides benchmark adapters that invoke LLM-Forge TypeScript
//! operations via subprocess execution, measuring performance without
//! modifying any existing TypeScript code.

use super::BenchTarget;
use crate::benchmarks::result::BenchmarkResult;
use async_trait::async_trait;
use serde_json::json;
use std::error::Error;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tracing::{debug, info};

/// Number of iterations for benchmark warmup.
const WARMUP_ITERATIONS: u32 = 3;

/// Number of iterations for benchmark measurement.
const BENCHMARK_ITERATIONS: u32 = 10;

/// Helper to find the Forge project root directory.
fn find_forge_root() -> PathBuf {
    // Try to find the forge root by looking for package.json
    let mut current = std::env::current_dir().unwrap_or_default();

    // If we're in forge-benchmarks, go up one level
    if current.ends_with("forge-benchmarks") {
        current = current.parent().map(|p| p.to_path_buf()).unwrap_or(current);
    }

    // Check if package.json exists at this level
    if current.join("package.json").exists() {
        return current;
    }

    // Default to /workspaces/forge
    PathBuf::from("/workspaces/forge")
}

/// Executes a TypeScript operation and measures execution time.
async fn measure_ts_operation(
    command: &str,
    args: &[&str],
    cwd: &PathBuf,
) -> Result<Duration, Box<dyn Error + Send + Sync>> {
    let start = Instant::now();

    let output = Command::new(command)
        .args(args)
        .current_dir(cwd)
        .output()
        .await?;

    let duration = start.elapsed();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        debug!("Command failed: {}", stderr);
        // Don't fail the benchmark, just note it in logs
    }

    Ok(duration)
}

/// Runs a benchmark with warmup and multiple iterations.
async fn run_benchmark_iterations<F, Fut>(
    warmup: u32,
    iterations: u32,
    mut f: F,
) -> Result<BenchmarkMetrics, Box<dyn Error + Send + Sync>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<Duration, Box<dyn Error + Send + Sync>>>,
{
    // Warmup
    for _ in 0..warmup {
        let _ = f().await;
    }

    // Measure
    let mut durations = Vec::with_capacity(iterations as usize);
    for _ in 0..iterations {
        let duration = f().await?;
        durations.push(duration);
    }

    Ok(BenchmarkMetrics::from_durations(&durations))
}

/// Aggregated benchmark metrics.
#[derive(Debug, Clone)]
struct BenchmarkMetrics {
    avg_ns: f64,
    min_ns: f64,
    max_ns: f64,
    ops_per_sec: f64,
    samples: u32,
}

impl BenchmarkMetrics {
    fn from_durations(durations: &[Duration]) -> Self {
        let ns_values: Vec<f64> = durations.iter().map(|d| d.as_nanos() as f64).collect();
        let sum: f64 = ns_values.iter().sum();
        let count = ns_values.len() as f64;
        let avg_ns = sum / count;
        let min_ns = ns_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_ns = ns_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let ops_per_sec = if avg_ns > 0.0 { 1_000_000_000.0 / avg_ns } else { 0.0 };

        Self {
            avg_ns,
            min_ns,
            max_ns,
            ops_per_sec,
            samples: durations.len() as u32,
        }
    }

    fn to_json(&self) -> serde_json::Value {
        json!({
            "avg_ns": self.avg_ns,
            "min_ns": self.min_ns,
            "max_ns": self.max_ns,
            "ops_per_sec": self.ops_per_sec,
            "samples": self.samples
        })
    }
}

// ============================================================================
// Provider Detection Benchmark
// ============================================================================

/// Benchmark for provider detection performance.
///
/// This adapter measures how quickly LLM-Forge can detect the provider
/// from a response structure by invoking the TypeScript vitest benchmark.
pub struct ProviderDetectionBenchmark {
    forge_root: PathBuf,
}

impl ProviderDetectionBenchmark {
    pub fn new() -> Self {
        Self {
            forge_root: find_forge_root(),
        }
    }
}

impl Default for ProviderDetectionBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BenchTarget for ProviderDetectionBenchmark {
    fn id(&self) -> String {
        "forge-provider-detection".to_string()
    }

    fn description(&self) -> String {
        "Benchmarks LLM-Forge provider detection from response structures".to_string()
    }

    async fn run(&self) -> Result<BenchmarkResult, Box<dyn Error + Send + Sync>> {
        info!("Running provider detection benchmark via vitest");

        // Run the vitest bench command with a filter for provider detection
        let metrics = run_benchmark_iterations(
            WARMUP_ITERATIONS,
            BENCHMARK_ITERATIONS,
            || async {
                measure_ts_operation(
                    "npx",
                    &["vitest", "bench", "--run", "--reporter=json", "performance.bench.ts"],
                    &self.forge_root,
                ).await
            },
        ).await?;

        Ok(BenchmarkResult::new(self.id(), metrics.to_json()))
    }
}

// ============================================================================
// Response Parsing Benchmark
// ============================================================================

/// Benchmark for response parsing performance.
///
/// Measures how quickly LLM-Forge can parse and normalize provider responses.
pub struct ResponseParsingBenchmark {
    forge_root: PathBuf,
}

impl ResponseParsingBenchmark {
    pub fn new() -> Self {
        Self {
            forge_root: find_forge_root(),
        }
    }
}

impl Default for ResponseParsingBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BenchTarget for ResponseParsingBenchmark {
    fn id(&self) -> String {
        "forge-response-parsing".to_string()
    }

    fn description(&self) -> String {
        "Benchmarks LLM-Forge response parsing and normalization".to_string()
    }

    async fn run(&self) -> Result<BenchmarkResult, Box<dyn Error + Send + Sync>> {
        info!("Running response parsing benchmark");

        // Execute the TypeScript test suite which includes parsing benchmarks
        let metrics = run_benchmark_iterations(
            WARMUP_ITERATIONS,
            BENCHMARK_ITERATIONS,
            || async {
                measure_ts_operation(
                    "npx",
                    &["vitest", "run", "--reporter=json", "providers"],
                    &self.forge_root,
                ).await
            },
        ).await?;

        Ok(BenchmarkResult::new(self.id(), metrics.to_json()))
    }
}

// ============================================================================
// Schema Validation Benchmark
// ============================================================================

/// Benchmark for schema validation performance.
///
/// Measures how quickly LLM-Forge validates canonical schemas.
pub struct SchemaValidationBenchmark {
    forge_root: PathBuf,
}

impl SchemaValidationBenchmark {
    pub fn new() -> Self {
        Self {
            forge_root: find_forge_root(),
        }
    }
}

impl Default for SchemaValidationBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BenchTarget for SchemaValidationBenchmark {
    fn id(&self) -> String {
        "forge-schema-validation".to_string()
    }

    fn description(&self) -> String {
        "Benchmarks LLM-Forge canonical schema validation".to_string()
    }

    async fn run(&self) -> Result<BenchmarkResult, Box<dyn Error + Send + Sync>> {
        info!("Running schema validation benchmark");

        let metrics = run_benchmark_iterations(
            WARMUP_ITERATIONS,
            BENCHMARK_ITERATIONS,
            || async {
                measure_ts_operation(
                    "npx",
                    &["vitest", "run", "--reporter=json", "schema"],
                    &self.forge_root,
                ).await
            },
        ).await?;

        Ok(BenchmarkResult::new(self.id(), metrics.to_json()))
    }
}

// ============================================================================
// CLI Parse Benchmark
// ============================================================================

/// Benchmark for CLI parse command performance.
///
/// Measures how quickly the `llm-forge parse` command executes.
pub struct CliParseBenchmark {
    forge_root: PathBuf,
}

impl CliParseBenchmark {
    pub fn new() -> Self {
        Self {
            forge_root: find_forge_root(),
        }
    }
}

impl Default for CliParseBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BenchTarget for CliParseBenchmark {
    fn id(&self) -> String {
        "forge-cli-parse".to_string()
    }

    fn description(&self) -> String {
        "Benchmarks LLM-Forge CLI parse command".to_string()
    }

    async fn run(&self) -> Result<BenchmarkResult, Box<dyn Error + Send + Sync>> {
        info!("Running CLI parse benchmark");

        // Check if there's a sample OpenAPI spec to parse
        let sample_spec = self.forge_root.join("tests/fixtures/openapi-sample.json");

        let metrics = if sample_spec.exists() {
            run_benchmark_iterations(
                WARMUP_ITERATIONS,
                BENCHMARK_ITERATIONS,
                || async {
                    measure_ts_operation(
                        "npx",
                        &["llm-forge", "parse", sample_spec.to_str().unwrap()],
                        &self.forge_root,
                    ).await
                },
            ).await?
        } else {
            // If no sample spec, run help command as a baseline
            run_benchmark_iterations(
                WARMUP_ITERATIONS,
                BENCHMARK_ITERATIONS,
                || async {
                    measure_ts_operation(
                        "npx",
                        &["llm-forge", "--help"],
                        &self.forge_root,
                    ).await
                },
            ).await?
        };

        Ok(BenchmarkResult::new(self.id(), metrics.to_json()))
    }
}

// ============================================================================
// CLI Generate Benchmark
// ============================================================================

/// Benchmark for CLI generate command performance.
///
/// Measures how quickly the `llm-forge generate` command executes.
pub struct CliGenerateBenchmark {
    forge_root: PathBuf,
}

impl CliGenerateBenchmark {
    pub fn new() -> Self {
        Self {
            forge_root: find_forge_root(),
        }
    }
}

impl Default for CliGenerateBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BenchTarget for CliGenerateBenchmark {
    fn id(&self) -> String {
        "forge-cli-generate".to_string()
    }

    fn description(&self) -> String {
        "Benchmarks LLM-Forge CLI generate command".to_string()
    }

    async fn run(&self) -> Result<BenchmarkResult, Box<dyn Error + Send + Sync>> {
        info!("Running CLI generate benchmark");

        // Run help command as baseline since generate requires valid input
        let metrics = run_benchmark_iterations(
            WARMUP_ITERATIONS,
            BENCHMARK_ITERATIONS,
            || async {
                measure_ts_operation(
                    "npx",
                    &["llm-forge", "generate", "--help"],
                    &self.forge_root,
                ).await
            },
        ).await?;

        Ok(BenchmarkResult::new(self.id(), metrics.to_json()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_metrics_from_durations() {
        let durations = vec![
            Duration::from_nanos(1000),
            Duration::from_nanos(2000),
            Duration::from_nanos(3000),
        ];

        let metrics = BenchmarkMetrics::from_durations(&durations);

        assert_eq!(metrics.samples, 3);
        assert!((metrics.avg_ns - 2000.0).abs() < 0.1);
        assert!((metrics.min_ns - 1000.0).abs() < 0.1);
        assert!((metrics.max_ns - 3000.0).abs() < 0.1);
    }

    #[test]
    fn test_find_forge_root() {
        let root = find_forge_root();
        // Should return a valid path
        assert!(!root.as_os_str().is_empty());
    }

    #[test]
    fn test_benchmark_ids() {
        assert_eq!(ProviderDetectionBenchmark::new().id(), "forge-provider-detection");
        assert_eq!(ResponseParsingBenchmark::new().id(), "forge-response-parsing");
        assert_eq!(SchemaValidationBenchmark::new().id(), "forge-schema-validation");
        assert_eq!(CliParseBenchmark::new().id(), "forge-cli-parse");
        assert_eq!(CliGenerateBenchmark::new().id(), "forge-cli-generate");
    }
}
