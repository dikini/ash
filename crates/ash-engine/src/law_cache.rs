//! Persistent cache for law-derived synthetic test results.
//!
//! The cache is intentionally separate from `ash.lock`: lockfiles describe
//! dependency/source resolution, while `.ash/law-cache.toml` records local law
//! test outcomes tied to a source hash.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Relative path of the law cache file under a project root.
pub const LAW_CACHE_RELATIVE_PATH: &str = ".ash/law-cache.toml";

/// Result state recorded for a law test cache entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LawCacheResult {
    /// The law was verified as valid by the available checker/test mode.
    Valid,
    /// The law was tested successfully, without claiming stronger proof validity.
    Tested,
    /// The law test produced a counterexample or other broken result.
    Broken,
    /// The law was seen but not executed or not supported by the current runner.
    Untested,
}

/// One cached law result keyed by declared law name and source hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LawCacheEntry {
    /// Declared law name.
    pub law_name: String,
    /// Stable hash of the source/summary used to produce the result.
    pub source_hash: String,
    /// Cached result state.
    pub result: LawCacheResult,
    /// Seed used for reproducible synthetic execution, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// Unix timestamp in seconds when this result was recorded.
    pub timestamp_unix_secs: u64,
}

/// In-memory representation of `.ash/law-cache.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LawCache {
    entries: Vec<LawCacheEntry>,
}

impl LawCache {
    /// Create an empty law cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return all cached entries in storage order.
    #[must_use]
    pub fn entries(&self) -> &[LawCacheEntry] {
        &self.entries
    }

    /// Load `.ash/law-cache.toml` from a project root.
    ///
    /// Missing cache files load as an empty cache.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache file exists but cannot be read or parsed.
    pub fn load_from_project_root(root: impl AsRef<Path>) -> Result<Self, LawCacheError> {
        let path = cache_path(root.as_ref());
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = std::fs::read_to_string(&path).map_err(|source| LawCacheError::Read {
            path: path.clone(),
            source,
        })?;
        toml::from_str(&content).map_err(|source| LawCacheError::Parse { path, source })
    }

    /// Save this cache to `.ash/law-cache.toml` under a project root.
    ///
    /// # Errors
    ///
    /// Returns an error if the `.ash` directory or cache file cannot be written,
    /// or if TOML serialization fails.
    pub fn save_to_project_root(&self, root: impl AsRef<Path>) -> Result<(), LawCacheError> {
        let root = root.as_ref();
        let path = cache_path(root);
        let directory = cache_dir(root);
        std::fs::create_dir_all(&directory).map_err(|source| LawCacheError::CreateDir {
            path: directory,
            source,
        })?;
        let content = toml::to_string_pretty(self).map_err(LawCacheError::Serialize)?;
        std::fs::write(&path, content).map_err(|source| LawCacheError::Write { path, source })
    }

    /// Record or replace a law result for the declared law name.
    pub fn record_result(
        &mut self,
        law_name: impl Into<String>,
        source_hash: impl Into<String>,
        result: LawCacheResult,
        seed: Option<u64>,
    ) {
        let law_name = law_name.into();
        let entry = LawCacheEntry {
            law_name: law_name.clone(),
            source_hash: source_hash.into(),
            result,
            seed,
            timestamp_unix_secs: current_unix_secs(),
        };

        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|candidate| candidate.law_name == law_name)
        {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
    }

    /// Look up a cache entry only if the source hash still matches.
    #[must_use]
    pub fn lookup_current(&self, law_name: &str, source_hash: &str) -> Option<&LawCacheEntry> {
        self.entries
            .iter()
            .find(|entry| entry.law_name == law_name && entry.source_hash == source_hash)
    }

    /// Remove stale entries for `law_name` when their source hash differs.
    ///
    /// Returns `true` when one or more stale entries were removed.
    pub fn invalidate_if_source_changed(
        &mut self,
        law_name: &str,
        current_source_hash: &str,
    ) -> bool {
        let original_len = self.entries.len();
        self.entries
            .retain(|entry| entry.law_name != law_name || entry.source_hash == current_source_hash);
        self.entries.len() != original_len
    }
}

/// Errors produced while loading or saving a law cache.
#[derive(Debug, Error)]
pub enum LawCacheError {
    /// Cache file could not be read.
    #[error("failed to read law cache at {path}: {source}")]
    Read {
        /// Cache file path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Cache file could not be parsed as TOML.
    #[error("failed to parse law cache at {path}: {source}")]
    Parse {
        /// Cache file path.
        path: PathBuf,
        /// Underlying TOML parse error.
        #[source]
        source: toml::de::Error,
    },
    /// Cache data could not be serialized as TOML.
    #[error("failed to serialize law cache: {0}")]
    Serialize(#[source] toml::ser::Error),
    /// Cache directory could not be created.
    #[error("failed to create law cache directory at {path}: {source}")]
    CreateDir {
        /// Cache directory path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Cache file could not be written.
    #[error("failed to write law cache at {path}: {source}")]
    Write {
        /// Cache file path.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

fn cache_path(root: &Path) -> PathBuf {
    cache_dir(root).join("law-cache.toml")
}

fn cache_dir(root: &Path) -> PathBuf {
    root.join(".ash")
}

fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}
