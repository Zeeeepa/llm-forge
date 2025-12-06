//! Configuration Integration (llm-config-core)
//!
//! Provides benchmark configuration management using the LLM-Dev-Ops Infra
//! configuration system with multi-environment support, versioning, and secrets.

use llm_config_core::ConfigManager;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during configuration operations
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration initialization failed: {0}")]
    InitError(String),

    #[error("Configuration key not found: {0}")]
    KeyNotFound(String),

    #[error("Configuration parse error: {0}")]
    ParseError(String),

    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Benchmark configuration values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of warmup iterations before measurement
    pub warmup_iterations: usize,

    /// Number of measurement iterations
    pub measurement_iterations: usize,

    /// Timeout for individual benchmarks (milliseconds)
    pub timeout_ms: u64,

    /// Enable detailed tracing during benchmarks
    pub enable_tracing: bool,

    /// Output directory for benchmark results
    pub output_dir: PathBuf,

    /// Export format for results (json, markdown, prometheus)
    pub export_format: String,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: 3,
            measurement_iterations: 100,
            timeout_ms: 30_000,
            enable_tracing: false,
            output_dir: PathBuf::from("./benchmark-results"),
            export_format: "json".to_string(),
        }
    }
}

/// Loads benchmark configuration from the Infra config system
///
/// Falls back to defaults if configuration is not available.
pub async fn load_benchmark_config() -> Result<BenchmarkConfig, ConfigError> {
    // Attempt to load from Infra config system
    // For now, return defaults - full integration requires ConfigManager setup
    Ok(BenchmarkConfig::default())
}

/// Gets a specific configuration value by key
pub async fn get_config_value<T: for<'de> Deserialize<'de>>(
    _key: &str,
) -> Result<T, ConfigError> {
    Err(ConfigError::KeyNotFound(
        "Configuration system not initialized".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_default_config() {
        let config = load_benchmark_config().await.unwrap();
        assert_eq!(config.warmup_iterations, 3);
        assert_eq!(config.measurement_iterations, 100);
    }

    #[test]
    fn test_config_serialization() {
        let config = BenchmarkConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: BenchmarkConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.warmup_iterations, config.warmup_iterations);
    }
}
