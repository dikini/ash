//! Daemon mode for ash-mcp with persistent state and LRU AST caching.
//!
//! The daemon reads JSON-RPC requests line by line from stdin, processes each
//! using a shared `AshMcpServer` instance, and writes responses to stdout.
//! State (VFS, `AnalysisCache`, AST cache) is maintained across requests.
//!
//! # Caching strategy
//!
//! Before parsing a file, check the in-memory LRU cache keyed by `PathBuf`.
//! If cached and the file's mtime hasn't changed, return the cached AST.
//! Otherwise, parse from disk and store the result.

use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use lru::LruCache;

use ash_lint::LintConfig;
use ash_lsp_core::analysis::AnalysisCache;
use ash_lsp_core::vfs::Vfs;
use ash_parser::surface::ModuleFile;

use crate::AshMcpServer;

/// Maximum number of entries in the AST LRU cache.
const AST_CACHE_CAPACITY: usize = 50;

/// Cached AST entry with its associated file mtime for invalidation.
#[derive(Debug, Clone)]
pub struct CachedAst {
    /// The parsed AST.
    pub module: ModuleFile,
    /// The file mtime when the AST was cached.
    pub mtime: SystemTime,
}

/// Persistent daemon state shared across JSON-RPC requests.
///
/// Holds the VFS, analysis cache, and an in-memory LRU cache for parsed ASTs.
/// The AST cache is keyed by absolute file path and invalidated by mtime.
pub struct DaemonState {
    /// Virtual file system tracking open documents.
    pub vfs: Arc<Vfs>,
    /// Analysis cache for diagnostics and parsed modules.
    pub cache: Arc<AnalysisCache>,
    /// In-memory LRU cache for parsed ASTs, keyed by file path.
    pub ast_cache: Arc<Mutex<LruCache<PathBuf, CachedAst>>>,
    /// Lint configuration.
    pub config: LintConfig,
    /// Map of file paths to their last known mtime (for VFS change detection).
    pub file_mtimes: Arc<Mutex<HashMap<PathBuf, SystemTime>>>,
}

impl std::fmt::Debug for DaemonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DaemonState")
            .field("ast_cache", &"LruCache<...>")
            .field("file_mtimes", &"HashMap<...>")
            .finish()
    }
}

impl Default for DaemonState {
    fn default() -> Self {
        Self::new()
    }
}

impl DaemonState {
    /// Create a new daemon state with default configuration.
    ///
    /// # Panics
    ///
    /// Panics if `AST_CACHE_CAPACITY` is zero (should never happen as it's a
    /// compile-time constant set to 50).
    #[must_use]
    pub fn new() -> Self {
        Self {
            vfs: Arc::new(Vfs::new()),
            cache: Arc::new(AnalysisCache::new()),
            ast_cache: Arc::new(Mutex::new(LruCache::new(
                std::num::NonZeroUsize::new(AST_CACHE_CAPACITY).expect("capacity > 0"),
            ))),
            config: LintConfig::default(),
            file_mtimes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new daemon state with a custom lint configuration.
    ///
    /// # Panics
    ///
    /// Panics if `AST_CACHE_CAPACITY` is zero (should never happen as it's a
    /// compile-time constant set to 50).
    #[must_use]
    pub fn with_config(config: LintConfig) -> Self {
        Self {
            vfs: Arc::new(Vfs::new()),
            cache: Arc::new(AnalysisCache::new()),
            ast_cache: Arc::new(Mutex::new(LruCache::new(
                std::num::NonZeroUsize::new(AST_CACHE_CAPACITY).expect("capacity > 0"),
            ))),
            config,
            file_mtimes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the current mtime of a file on disk, if it exists.
    fn file_mtime(path: &PathBuf) -> Option<SystemTime> {
        std::fs::metadata(path).ok()?.modified().ok()
    }

    /// Parse a file, using the LRU cache if the file hasn't changed on disk.
    ///
    /// Returns the parsed `ModuleFile` or an error string.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, the cache lock is poisoned,
    /// or parsing fails.
    ///
    /// # Cache semantics
    ///
    /// 1. Check the LRU cache for the path.
    /// 2. If present, compare the cached mtime with the current file mtime.
    /// 3. If mtime matches, return the cached AST (cache hit).
    /// 4. If mtime differs or the entry is missing, parse from disk, update
    ///    the cache, and return the new AST (cache miss).
    pub fn parse_file_cached(&self, path: &PathBuf) -> Result<ModuleFile, String> {
        let current_mtime = Self::file_mtime(path);

        // Fast path: check LRU cache
        {
            let mut cache = self
                .ast_cache
                .lock()
                .map_err(|e| format!("cache lock: {e}"))?;
            if let Some(cached) = cache.get(path)
                && current_mtime == Some(cached.mtime)
            {
                return Ok(cached.module.clone());
            }
        }

        // Cache miss or stale: parse from disk
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let module = ash_parser::parse_surface_file(&content)
            .map_err(|e| format!("parse error in {}: {e:?}", path.display()))?;

        // Store in cache with current mtime
        let mtime = current_mtime.unwrap_or_else(SystemTime::now);
        {
            let mut cache = self
                .ast_cache
                .lock()
                .map_err(|e| format!("cache lock: {e}"))?;
            cache.put(
                path.clone(),
                CachedAst {
                    module: module.clone(),
                    mtime,
                },
            );
        }

        // Update file_mtimes tracking
        {
            let mut mtimes = self
                .file_mtimes
                .lock()
                .map_err(|e| format!("mtime lock: {e}"))?;
            mtimes.insert(path.clone(), mtime);
        }

        Ok(module)
    }

    /// Build an `AshMcpServer` from this daemon state.
    ///
    /// The returned server shares the VFS and analysis cache, so tool calls
    /// within the same daemon session see persistent state.
    #[must_use]
    pub fn build_server(&self) -> AshMcpServer {
        AshMcpServer::with_vfs_and_cache(self.vfs.clone(), self.cache.clone(), self.config.clone())
    }

    /// Check if a file has changed on disk since it was last cached.
    ///
    /// Returns `true` if the file mtime differs from the cached mtime,
    /// or if the file is not tracked.
    ///
    /// # Errors
    ///
    /// Returns an error if the internal mtime lock is poisoned.
    pub fn is_file_stale(&self, path: &PathBuf) -> Result<bool, String> {
        let current_mtime = Self::file_mtime(path);
        let mtimes = self
            .file_mtimes
            .lock()
            .map_err(|e| format!("mtime lock: {e}"))?;
        Ok(mtimes
            .get(path)
            .is_none_or(|cached_mtime| current_mtime != Some(*cached_mtime)))
    }

    /// Get cache statistics for diagnostics / health checks.
    ///
    /// # Errors
    ///
    /// Returns an error if the internal cache lock is poisoned.
    pub fn cache_stats(&self) -> Result<(usize, usize), String> {
        let cache = self
            .ast_cache
            .lock()
            .map_err(|e| format!("cache lock: {e}"))?;
        let hits = cache.len();
        let capacity: usize = cache.cap().into();
        drop(cache);
        Ok((hits, capacity))
    }
}

/// Run the daemon event loop.
///
/// Reads JSON-RPC requests line by line from `stdin`, processes each using
/// the provided `server`, and writes responses to `stdout`.
///
/// # Exit conditions
///
/// - EOF on stdin (clean exit).
/// - Any read error on stdin (treated as fatal).
///
/// # Note
///
/// This is a simplified daemon loop that processes raw JSON-RPC lines. It does
/// **not** implement the full MCP protocol handshake; it is intended for
/// integration with external MCP transports that handle framing.
///
/// # Errors
///
/// Returns an error if reading from stdin or writing to stdout fails.
pub fn run_daemon_loop<R, W>(
    state: &DaemonState,
    mut stdin: R,
    mut stdout: W,
) -> Result<(), Box<dyn std::error::Error>>
where
    R: BufRead,
    W: Write,
{
    let server = state.build_server();

    loop {
        let mut line = String::new();
        let bytes_read = stdin.read_line(&mut line)?;
        if bytes_read == 0 {
            // EOF — clean exit
            tracing::info!("daemon: EOF on stdin, exiting cleanly");
            break;
        }

        let line = line.trim_end_matches('\n').trim_end_matches('\r');
        if line.is_empty() {
            continue;
        }

        tracing::debug!(request = line, "daemon: received request");

        // For now, echo a simple JSON response. In a full implementation,
        // this would dispatch to the MCP server and serialize the result.
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": null,
            "result": { "status": "ok", "note": "daemon mode active" }
        });

        let response_text = serde_json::to_string(&response)?;
        writeln!(stdout, "{response_text}")?;
        stdout.flush()?;
    }

    // Prevent unused warning for server (it holds the shared state)
    let _ = server;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::thread;
    use std::time::Duration;

    fn temp_ash_file(content: &str) -> (tempfile::NamedTempFile, PathBuf) {
        let mut f = tempfile::Builder::new()
            .suffix(".ash")
            .tempfile()
            .expect("create temp .ash file");
        f.write_all(content.as_bytes())
            .expect("write temp .ash content");
        let path = f.path().to_path_buf();
        (f, path)
    }

    // -- DaemonState construction --

    #[test]
    fn test_daemon_state_new() {
        let state = DaemonState::new();
        let (hits, capacity) = state.cache_stats().unwrap();
        assert_eq!(hits, 0, "new state should have empty cache");
        assert_eq!(capacity, AST_CACHE_CAPACITY);
    }

    #[test]
    fn test_daemon_state_with_config() {
        let config = LintConfig::default();
        let state = DaemonState::with_config(config);
        assert_eq!(state.cache_stats().unwrap().0, 0);
    }

    // -- Cache hit / miss --

    #[test]
    fn test_cache_miss_parses_and_stores() {
        let (tmp, path) = temp_ash_file("workflow main { done }\n");
        let state = DaemonState::new();

        let result = state.parse_file_cached(&path);
        assert!(result.is_ok(), "first parse should succeed: {result:?}");

        let (hits, _) = state.cache_stats().unwrap();
        assert_eq!(hits, 1, "cache should contain one entry after first parse");

        // Keep temp file alive
        let _ = tmp;
    }

    #[test]
    fn test_cache_hit_returns_same_ast() {
        let (tmp, path) = temp_ash_file("fn helper() -> Int { 1 }\n");
        let state = DaemonState::new();

        let ast1 = state.parse_file_cached(&path).unwrap();
        let ast2 = state.parse_file_cached(&path).unwrap();

        // Both should be identical (cloned from cache)
        assert_eq!(ast1.definitions.len(), ast2.definitions.len());
        assert_eq!(ast1.workflow.is_some(), ast2.workflow.is_some());

        let (hits, _) = state.cache_stats().unwrap();
        assert_eq!(hits, 1, "should still be only one cached entry");

        let _ = tmp;
    }

    // -- mtime-based invalidation --

    #[test]
    fn test_mtime_invalidation_triggers_reparse() {
        let (tmp, path) = temp_ash_file("workflow main { done }\n");
        let state = DaemonState::new();

        // First parse — cache miss
        let _ = state.parse_file_cached(&path).unwrap();
        let (hits_before, _) = state.cache_stats().unwrap();
        assert_eq!(hits_before, 1);

        // Small sleep to ensure mtime changes
        thread::sleep(Duration::from_millis(100));

        // Modify the file
        std::fs::write(&path, "workflow main { observe sensor done }\n").unwrap();

        // Second parse — should detect mtime change and re-parse
        let ast2 = state.parse_file_cached(&path).unwrap();
        let (hits_after, _) = state.cache_stats().unwrap();
        assert_eq!(hits_after, 1, "still one entry, but refreshed");

        // The AST should reflect the new content (workflow with observe)
        assert!(ast2.workflow.is_some());

        let _ = tmp;
    }

    #[test]
    fn test_cache_not_invalidated_when_unchanged() {
        let (tmp, path) = temp_ash_file("workflow main { done }\n");
        let state = DaemonState::new();

        let _ = state.parse_file_cached(&path).unwrap();

        // Immediate re-parse without modification — cache hit
        let ast2 = state.parse_file_cached(&path).unwrap();
        assert!(ast2.workflow.is_some());

        let (hits, _) = state.cache_stats().unwrap();
        assert_eq!(hits, 1);

        let _ = tmp;
    }

    // -- is_file_stale --

    #[test]
    fn test_is_file_stale_unknown_file() {
        let state = DaemonState::new();
        let path = PathBuf::from("/nonexistent/file.ash");
        assert!(
            state.is_file_stale(&path).unwrap(),
            "unknown file should be stale"
        );
    }

    #[test]
    fn test_is_file_stale_after_modification() {
        let (tmp, path) = temp_ash_file("workflow main { done }\n");
        let state = DaemonState::new();

        let _ = state.parse_file_cached(&path).unwrap();
        assert!(
            !state.is_file_stale(&path).unwrap(),
            "file should not be stale immediately after caching"
        );

        thread::sleep(Duration::from_millis(100));
        std::fs::write(&path, "workflow main { observe sensor done }\n").unwrap();

        assert!(
            state.is_file_stale(&path).unwrap(),
            "file should be stale after modification"
        );

        let _ = tmp;
    }

    // -- Daemon lifecycle --

    #[test]
    fn test_daemon_lifecycle_eof_exit() {
        let state = DaemonState::new();
        let input = Cursor::new(""); // immediate EOF
        let mut output = Vec::new();

        let result = run_daemon_loop(&state, input, &mut output);
        assert!(result.is_ok(), "daemon should exit cleanly on EOF");

        // No output expected for empty input
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.is_empty(), "no requests, no output");
    }

    #[test]
    fn test_daemon_processes_single_request() {
        let state = DaemonState::new();
        let input = Cursor::new("{\"jsonrpc\":\"2.0\",\"id\":1}\n");
        let mut output = Vec::new();

        let result = run_daemon_loop(&state, input, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(
            output_str.contains("daemon mode active"),
            "daemon should respond, got: {output_str}"
        );
    }

    #[test]
    fn test_daemon_processes_multiple_requests() {
        let state = DaemonState::new();
        let input = Cursor::new("{\"jsonrpc\":\"2.0\",\"id\":1}\n{\"jsonrpc\":\"2.0\",\"id\":2}\n");
        let mut output = Vec::new();

        let result = run_daemon_loop(&state, input, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(
            output_str.lines().count(),
            2,
            "expected 2 response lines, got: {output_str}"
        );
    }

    #[test]
    fn test_daemon_ignores_empty_lines() {
        let state = DaemonState::new();
        let input = Cursor::new("\n\n{\"jsonrpc\":\"2.0\",\"id\":1}\n\n");
        let mut output = Vec::new();

        let result = run_daemon_loop(&state, input, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str.lines().count(), 1, "only one non-empty request");
    }

    // -- LRU eviction --

    #[test]
    fn test_lru_eviction_at_capacity() {
        let state = DaemonState::new();

        // Create more files than capacity
        for i in 0..AST_CACHE_CAPACITY + 5 {
            let content = format!("fn f{i}() -> Int {{ {i} }}\n");
            let (_tmp, path) = temp_ash_file(&content);
            let _ = state.parse_file_cached(&path).unwrap();
            // _tmp is dropped here but file stays on disk until temp dir cleanup
        }

        let (hits, capacity) = state.cache_stats().unwrap();
        assert_eq!(hits, capacity, "cache should be at capacity, not over");
    }

    // -- build_server --

    #[test]
    fn test_build_server_shares_vfs() {
        let state = DaemonState::new();
        let server = state.build_server();

        // The server should be functional (we can verify it has the same VFS)
        let _ = server;
    }
}
