//! # forge-benchmarks
//!
//! Canonical benchmark crate for LLM-Forge (TypeScript SDK generator).
//! Implements the unified benchmark interface used across all 25 benchmark-target repositories.
//!
//! ## Structure
//!
//! - `benchmarks` module: Core benchmark execution and result types
//! - `adapters` module: TypeScript-to-Rust benchmark adapters for Forge operations
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

pub mod adapters;
pub mod benchmarks;

// Re-export key types for convenience
pub use benchmarks::result::BenchmarkResult;
pub use benchmarks::run_all_benchmarks;
pub use adapters::BenchTarget;
