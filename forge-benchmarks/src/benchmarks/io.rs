//! I/O utilities for benchmark results.
//!
//! This module provides functions for reading and writing benchmark results
//! to the canonical output directories.

use super::result::BenchmarkResult;
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

/// Default output directory for benchmark results.
pub const OUTPUT_DIR: &str = "benchmarks/output";

/// Default raw output directory for individual result files.
pub const RAW_OUTPUT_DIR: &str = "benchmarks/output/raw";

/// Default summary file name.
pub const SUMMARY_FILE: &str = "summary.md";

/// Errors that can occur during benchmark I/O operations.
#[derive(Error, Debug)]
pub enum IoError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Directory does not exist: {0}")]
    DirectoryNotFound(String),
}

/// Writes benchmark results to the canonical output directories.
///
/// This function:
/// 1. Writes individual JSON files to `benchmarks/output/raw/`
/// 2. Writes a combined JSON file to `benchmarks/output/results.json`
/// 3. Writes a Markdown summary to `benchmarks/output/summary.md`
///
/// # Arguments
///
/// * `results` - The benchmark results to write
/// * `base_path` - Base path for output (defaults to crate root)
///
/// # Returns
///
/// `Ok(())` on success, or an `IoError` on failure.
///
/// # Example
///
/// ```rust,no_run
/// use forge_benchmarks::benchmarks::io::write_results;
/// use forge_benchmarks::benchmarks::result::BenchmarkResult;
/// use serde_json::json;
///
/// let results = vec![
///     BenchmarkResult::new("test".to_string(), json!({})),
/// ];
///
/// write_results(&results, ".").unwrap();
/// ```
pub fn write_results(results: &[BenchmarkResult], base_path: &str) -> Result<(), IoError> {
    let output_dir = Path::new(base_path).join(OUTPUT_DIR);
    let raw_dir = Path::new(base_path).join(RAW_OUTPUT_DIR);

    // Ensure directories exist
    fs::create_dir_all(&output_dir)?;
    fs::create_dir_all(&raw_dir)?;

    // Write individual raw result files
    for result in results {
        let filename = format!("{}.json", sanitize_filename(&result.target_id));
        let filepath = raw_dir.join(&filename);
        let json = serde_json::to_string_pretty(result)?;
        fs::write(&filepath, json)?;
    }

    // Write combined results file
    let combined_path = output_dir.join("results.json");
    let combined_json = serde_json::to_string_pretty(results)?;
    fs::write(&combined_path, combined_json)?;

    // Write markdown summary
    let summary = super::markdown::generate_summary(results);
    let summary_path = output_dir.join(SUMMARY_FILE);
    fs::write(&summary_path, summary)?;

    Ok(())
}

/// Reads benchmark results from the canonical output directory.
///
/// # Arguments
///
/// * `base_path` - Base path where results are stored
///
/// # Returns
///
/// A vector of `BenchmarkResult` or an `IoError`.
pub fn read_results(base_path: &str) -> Result<Vec<BenchmarkResult>, IoError> {
    let results_path = Path::new(base_path).join(OUTPUT_DIR).join("results.json");

    if !results_path.exists() {
        return Err(IoError::DirectoryNotFound(
            results_path.to_string_lossy().to_string(),
        ));
    }

    let content = fs::read_to_string(&results_path)?;
    let results: Vec<BenchmarkResult> = serde_json::from_str(&content)?;

    Ok(results)
}

/// Reads a single benchmark result by target ID.
///
/// # Arguments
///
/// * `base_path` - Base path where results are stored
/// * `target_id` - The target ID to look up
///
/// # Returns
///
/// The `BenchmarkResult` or an `IoError`.
pub fn read_result_by_id(base_path: &str, target_id: &str) -> Result<BenchmarkResult, IoError> {
    let filename = format!("{}.json", sanitize_filename(target_id));
    let filepath = Path::new(base_path).join(RAW_OUTPUT_DIR).join(&filename);

    if !filepath.exists() {
        return Err(IoError::DirectoryNotFound(
            filepath.to_string_lossy().to_string(),
        ));
    }

    let content = fs::read_to_string(&filepath)?;
    let result: BenchmarkResult = serde_json::from_str(&content)?;

    Ok(result)
}

/// Appends results to the historical log file.
///
/// Results are appended as JSONL (one JSON object per line) for easy parsing
/// and historical analysis.
///
/// # Arguments
///
/// * `results` - The benchmark results to append
/// * `base_path` - Base path for output
///
/// # Returns
///
/// `Ok(())` on success, or an `IoError` on failure.
pub fn append_to_history(results: &[BenchmarkResult], base_path: &str) -> Result<(), IoError> {
    let history_path = Path::new(base_path).join(OUTPUT_DIR).join("history.jsonl");

    let mut history_content = String::new();
    for result in results {
        let json_line = serde_json::to_string(result)?;
        history_content.push_str(&json_line);
        history_content.push('\n');
    }

    // Append to existing file or create new one
    use std::fs::OpenOptions;
    use std::io::Write;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&history_path)?;

    file.write_all(history_content.as_bytes())?;

    Ok(())
}

/// Sanitizes a string for use as a filename.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

/// Ensures the canonical output directory structure exists.
///
/// Creates:
/// - `benchmarks/output/`
/// - `benchmarks/output/raw/`
///
/// # Arguments
///
/// * `base_path` - Base path for output directories
///
/// # Returns
///
/// `Ok(())` on success, or an `IoError` on failure.
pub fn ensure_output_dirs(base_path: &str) -> Result<(), IoError> {
    let output_dir = Path::new(base_path).join(OUTPUT_DIR);
    let raw_dir = Path::new(base_path).join(RAW_OUTPUT_DIR);

    fs::create_dir_all(&output_dir)?;
    fs::create_dir_all(&raw_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_write_and_read_results() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        let results = vec![
            BenchmarkResult::new("test-1".to_string(), json!({"ops_per_sec": 1000})),
            BenchmarkResult::new("test-2".to_string(), json!({"ops_per_sec": 2000})),
        ];

        // Write results
        write_results(&results, base_path).unwrap();

        // Read back combined results
        let read_results = read_results(base_path).unwrap();
        assert_eq!(read_results.len(), 2);

        // Read individual result
        let single = read_result_by_id(base_path, "test-1").unwrap();
        assert_eq!(single.target_id, "test-1");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal-name"), "normal-name");
        assert_eq!(sanitize_filename("path/with/slashes"), "path_with_slashes");
        assert_eq!(sanitize_filename("file:name"), "file_name");
    }

    #[test]
    fn test_ensure_output_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        ensure_output_dirs(base_path).unwrap();

        assert!(Path::new(base_path).join(OUTPUT_DIR).exists());
        assert!(Path::new(base_path).join(RAW_OUTPUT_DIR).exists());
    }

    #[test]
    fn test_append_to_history() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        ensure_output_dirs(base_path).unwrap();

        let results = vec![BenchmarkResult::new("hist-test".to_string(), json!({}))];

        append_to_history(&results, base_path).unwrap();
        append_to_history(&results, base_path).unwrap();

        let history_path = Path::new(base_path).join(OUTPUT_DIR).join("history.jsonl");
        let content = fs::read_to_string(&history_path).unwrap();
        let lines: Vec<_> = content.lines().collect();

        assert_eq!(lines.len(), 2);
    }
}
