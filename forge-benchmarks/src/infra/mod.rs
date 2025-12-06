//! Infrastructure Integration Module (Phase 2B)
//!
//! This module provides integration with LLM-Dev-Ops Infra crates for:
//! - Configuration management (llm-config-core)
//! - Prometheus metrics (llm-config-metrics)
//! - Multi-tier caching (llm-config-cache)
//!
//! Enable with feature flags:
//! - `infra-full`: All infrastructure modules
//! - `infra-config`: Configuration management only
//! - `infra-metrics`: Metrics collection only
//! - `infra-cache`: Caching support only
//!
//! ## Usage
//!
//! ```toml
//! [dependencies]
//! forge-benchmarks = { version = "0.1", features = ["infra-full"] }
//! ```

#[cfg(feature = "infra-config")]
pub mod config;

#[cfg(feature = "infra-metrics")]
pub mod metrics;

#[cfg(feature = "infra-cache")]
pub mod cache;

/// Re-export Infra types for convenience when features are enabled
#[cfg(feature = "infra-config")]
pub use llm_config_core;

#[cfg(feature = "infra-metrics")]
pub use llm_config_metrics;

#[cfg(feature = "infra-cache")]
pub use llm_config_cache;

/// Check if any Infra features are enabled
pub fn is_infra_enabled() -> bool {
    cfg!(any(
        feature = "infra-config",
        feature = "infra-metrics",
        feature = "infra-cache"
    ))
}

/// Get a list of enabled Infra features
pub fn enabled_features() -> Vec<&'static str> {
    let mut features = Vec::new();

    #[cfg(feature = "infra-config")]
    features.push("infra-config");

    #[cfg(feature = "infra-metrics")]
    features.push("infra-metrics");

    #[cfg(feature = "infra-cache")]
    features.push("infra-cache");

    features
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enabled_features_returns_list() {
        let features = enabled_features();
        // Features list should be consistent with cargo features
        for feature in &features {
            assert!(
                ["infra-config", "infra-metrics", "infra-cache"].contains(feature),
                "Unknown feature: {}",
                feature
            );
        }
    }
}
