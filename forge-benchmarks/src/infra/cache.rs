//! Cache Integration (llm-config-cache)
//!
//! Provides multi-tier caching for benchmark results using the LLM-Dev-Ops
//! Infra caching system with L1 memory cache and optional L2 support.

use crate::benchmarks::result::BenchmarkResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors that can occur during cache operations
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cache initialization failed: {0}")]
    InitError(String),

    #[error("Cache read failed: {0}")]
    ReadError(String),

    #[error("Cache write failed: {0}")]
    WriteError(String),

    #[error("Cache entry expired")]
    Expired,

    #[error("Cache entry not found")]
    NotFound,
}

/// A cached benchmark result with TTL
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedEntry {
    result: BenchmarkResult,
    cached_at: chrono::DateTime<chrono::Utc>,
    ttl_secs: u64,
}

impl CachedEntry {
    fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        let expiry = self.cached_at + chrono::Duration::seconds(self.ttl_secs as i64);
        now > expiry
    }
}

/// In-memory L1 cache for benchmark results
pub struct BenchmarkCache {
    entries: RwLock<HashMap<String, CachedEntry>>,
    default_ttl: Duration,
}

impl BenchmarkCache {
    /// Create a new cache with default TTL
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            default_ttl,
        }
    }

    /// Get a cached benchmark result by target ID
    pub fn get(&self, target_id: &str) -> Result<BenchmarkResult, CacheError> {
        let entries = self
            .entries
            .read()
            .map_err(|e| CacheError::ReadError(e.to_string()))?;

        match entries.get(target_id) {
            Some(entry) if !entry.is_expired() => Ok(entry.result.clone()),
            Some(_) => Err(CacheError::Expired),
            None => Err(CacheError::NotFound),
        }
    }

    /// Cache a benchmark result with default TTL
    pub fn set(&self, result: BenchmarkResult) -> Result<(), CacheError> {
        self.set_with_ttl(result, self.default_ttl)
    }

    /// Cache a benchmark result with custom TTL
    pub fn set_with_ttl(&self, result: BenchmarkResult, ttl: Duration) -> Result<(), CacheError> {
        let mut entries = self
            .entries
            .write()
            .map_err(|e| CacheError::WriteError(e.to_string()))?;

        let entry = CachedEntry {
            result: result.clone(),
            cached_at: chrono::Utc::now(),
            ttl_secs: ttl.as_secs(),
        };

        entries.insert(result.target_id.clone(), entry);
        Ok(())
    }

    /// Remove a cached entry
    pub fn invalidate(&self, target_id: &str) -> Result<(), CacheError> {
        let mut entries = self
            .entries
            .write()
            .map_err(|e| CacheError::WriteError(e.to_string()))?;

        entries.remove(target_id);
        Ok(())
    }

    /// Clear all cached entries
    pub fn clear(&self) -> Result<(), CacheError> {
        let mut entries = self
            .entries
            .write()
            .map_err(|e| CacheError::WriteError(e.to_string()))?;

        entries.clear();
        Ok(())
    }

    /// Get the number of cached entries
    pub fn len(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove expired entries
    pub fn cleanup_expired(&self) -> Result<usize, CacheError> {
        let mut entries = self
            .entries
            .write()
            .map_err(|e| CacheError::WriteError(e.to_string()))?;

        let initial_len = entries.len();
        entries.retain(|_, entry| !entry.is_expired());
        Ok(initial_len - entries.len())
    }
}

impl Default for BenchmarkCache {
    fn default() -> Self {
        Self::new(Duration::from_secs(300)) // 5 minute default TTL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cache_set_get() {
        let cache = BenchmarkCache::default();
        let result = BenchmarkResult::new("test-target".to_string(), json!({"ops_per_sec": 1000}));

        cache.set(result.clone()).unwrap();
        let cached = cache.get("test-target").unwrap();

        assert_eq!(cached.target_id, "test-target");
    }

    #[test]
    fn test_cache_not_found() {
        let cache = BenchmarkCache::default();
        let result = cache.get("nonexistent");

        assert!(matches!(result, Err(CacheError::NotFound)));
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = BenchmarkCache::default();
        let result = BenchmarkResult::new("test-target".to_string(), json!({}));

        cache.set(result).unwrap();
        assert_eq!(cache.len(), 1);

        cache.invalidate("test-target").unwrap();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_clear() {
        let cache = BenchmarkCache::default();

        for i in 0..5 {
            let result = BenchmarkResult::new(format!("target-{}", i), json!({}));
            cache.set(result).unwrap();
        }

        assert_eq!(cache.len(), 5);
        cache.clear().unwrap();
        assert!(cache.is_empty());
    }
}
