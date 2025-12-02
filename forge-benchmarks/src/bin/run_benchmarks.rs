//! CLI binary for running LLM-Forge benchmarks.
//!
//! This binary invokes `run_all_benchmarks()` and writes results to the
//! canonical output directories.
//!
//! # Usage
//!
//! ```bash
//! # Run all benchmarks and write results
//! cargo run --bin run_benchmarks
//!
//! # Run with verbose logging
//! RUST_LOG=info cargo run --bin run_benchmarks
//!
//! # Run with specific output directory
//! cargo run --bin run_benchmarks -- --output ./custom-output
//! ```

use forge_benchmarks::benchmarks::{io, markdown, run_all_benchmarks};
use std::env;
use std::path::PathBuf;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

/// CLI arguments (simple parsing without external crate dependency).
struct Args {
    output_path: PathBuf,
    verbose: bool,
    json_only: bool,
}

impl Args {
    fn parse() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut output_path = PathBuf::from(".");
        let mut verbose = false;
        let mut json_only = false;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--output" | "-o" => {
                    if i + 1 < args.len() {
                        output_path = PathBuf::from(&args[i + 1]);
                        i += 1;
                    }
                }
                "--verbose" | "-v" => verbose = true,
                "--json" => json_only = true,
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {}
            }
            i += 1;
        }

        Self {
            output_path,
            verbose,
            json_only,
        }
    }
}

fn print_help() {
    println!(
        r#"forge-benchmarks - Canonical benchmark runner for LLM-Forge

USAGE:
    run_benchmarks [OPTIONS]

OPTIONS:
    -o, --output <PATH>    Output directory for results (default: current directory)
    -v, --verbose          Enable verbose logging
    --json                 Output JSON results to stdout only (skip file writes)
    -h, --help             Print help information

EXAMPLES:
    # Run benchmarks with default settings
    cargo run --bin run_benchmarks

    # Run with custom output directory
    cargo run --bin run_benchmarks -- --output ./results

    # Run with verbose logging
    RUST_LOG=debug cargo run --bin run_benchmarks -- -v

OUTPUT FILES:
    benchmarks/output/results.json    Combined benchmark results
    benchmarks/output/summary.md      Markdown summary report
    benchmarks/output/raw/*.json      Individual result files
    benchmarks/output/history.jsonl   Historical results log
"#
    );
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Starting LLM-Forge benchmark suite");
    info!("Output directory: {}", args.output_path.display());

    // Ensure output directories exist
    let output_str = args.output_path.to_str().unwrap_or(".");
    if let Err(e) = io::ensure_output_dirs(output_str) {
        error!("Failed to create output directories: {}", e);
        std::process::exit(1);
    }

    // Run all benchmarks
    info!("Executing benchmarks...");
    let results = run_all_benchmarks().await;

    // Report summary
    let total = results.len();
    let failed = results.iter().filter(|r| r.is_failed()).count();
    let passed = total - failed;

    info!("Benchmark suite completed: {}/{} passed", passed, total);

    // Handle JSON-only output mode
    if args.json_only {
        match serde_json::to_string_pretty(&results) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                error!("Failed to serialize results: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Write results to files
    info!("Writing results to {}", output_str);

    if let Err(e) = io::write_results(&results, output_str) {
        error!("Failed to write results: {}", e);
        std::process::exit(1);
    }

    // Append to history
    if let Err(e) = io::append_to_history(&results, output_str) {
        error!("Failed to append to history: {}", e);
        // Non-fatal error, continue
    }

    // Print CI summary to stdout
    let ci_summary = markdown::generate_ci_summary(&results);
    println!("\n{}", ci_summary);

    // Print file locations
    println!("\nResults written to:");
    println!("  - {}/benchmarks/output/results.json", output_str);
    println!("  - {}/benchmarks/output/summary.md", output_str);
    println!("  - {}/benchmarks/output/raw/", output_str);

    // Exit with error code if any benchmarks failed
    if failed > 0 {
        error!("{} benchmark(s) failed", failed);
        std::process::exit(1);
    }

    info!("All benchmarks completed successfully");
}
