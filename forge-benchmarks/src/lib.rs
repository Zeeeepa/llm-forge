//! # forge-benchmarks
//!
//! Canonical benchmark crate for LLM-Forge (TypeScript SDK generator).
//! Implements the unified benchmark interface used across all 25 benchmark-target repositories.
//!
//! ## Structure
//!
//! - `benchmarks` module: Core benchmark execution and result types
//! - `adapters` module: TypeScript-to-Rust benchmark adapters for Forge operations
//! - `infra` module: LLM-Dev-Ops Infra integration (Phase 2B) for config, metrics, caching
//!
//! ## Usage
//!
//! ```rust,no_run
//! use forge_benchmarks::benchmarks::run_all_benchmarks;
//!
//! #[tokio::main]
//! async fn main() {
//!     let results = run_all_benchmarks().await;
//!     for result in results {
//!         println!("{}: {:?}", result.target_id, result.metrics);
//!     }
//! }
//! ```
//!
//! ## Infra Integration (Phase 2B)
//!
//! Enable infrastructure features for enterprise-grade capabilities:
//!
//! ```toml
//! [dependencies]
//! forge-benchmarks = { version = "0.1", features = ["infra-full"] }
//! ```
//!
//! Available features:
//! - `infra-full`: All infrastructure modules
//! - `infra-config`: Configuration management (llm-config-core)
//! - `infra-metrics`: Prometheus metrics (llm-config-metrics)
//! - `infra-cache`: Multi-tier caching (llm-config-cache)

pub mod adapters;
pub mod benchmarks;

// Infrastructure integration module (Phase 2B)
// Enabled via feature flags: infra-config, infra-metrics, infra-cache, infra-full
#[cfg(any(
    feature = "infra-config",
    feature = "infra-metrics",
    feature = "infra-cache"
))]
pub mod infra;

// Re-export key types for convenience
pub use adapters::BenchTarget;
pub use benchmarks::result::BenchmarkResult;
pub use benchmarks::run_all_benchmarks;

/// Version of the forge-benchmarks crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Phase 2B Infra integration status
pub fn infra_status() -> &'static str {
    #[cfg(feature = "infra-full")]
    return "Phase 2B: Full Infra integration enabled";

    #[cfg(all(
        any(
            feature = "infra-config",
            feature = "infra-metrics",
            feature = "infra-cache"
        ),
        not(feature = "infra-full")
    ))]
    return "Phase 2B: Partial Infra integration enabled";

    #[cfg(not(any(
        feature = "infra-config",
        feature = "infra-metrics",
        feature = "infra-cache"
    )))]
    return "Phase 2B: Infra integration available (enable with features)";
}
