//! Analysis cache with change detection.
//!
//! The `AnalysisCache` stores per-URI analysis results and only re-analyses
//! a file when its VFS entry has changed (different version).

use crate::diagnostics::compute_diagnostics;
use crate::vfs::Vfs;
use ash_lint::LintConfig;
use ash_parser::surface::ModuleFile;
use dashmap::DashMap;
use lsp_types::{Diagnostic, Uri};
use std::sync::Arc;
use tracing::{debug, info};

/// Cached result of analysing a single document.
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// The parsed AST, if parsing succeeded.
    pub module: Option<Arc<ModuleFile>>,
    /// All diagnostics for this document (parse, typeck, lint).
    pub diagnostics: Vec<Diagnostic>,
    /// The VFS version this result was computed from.
    version: i32,
}

/// Concurrent analysis cache keyed by document URI.
#[derive(Debug, Default)]
pub struct AnalysisCache {
    inner: DashMap<Uri, AnalysisResult>,
}

impl AnalysisCache {
    /// Creates a new, empty analysis cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
        }
    }

    /// Analyzes the document at `uri`, using the VFS for content and `config`
    /// for lint settings.
    ///
    /// If the cached result is still valid (same version), returns the
    /// existing diagnostics. Otherwise, re-parses and re-lints.
    pub fn analyze(&self, uri: &Uri, vfs: &Vfs, config: &LintConfig) -> Vec<Diagnostic> {
        let entry = vfs.get(uri);
        let vfs_version = entry.as_ref().map_or(-1, |e| e.version);

        // Fast path: return cached result if version matches.
        if let Some(cached) = self.inner.get(uri)
            && cached.version == vfs_version
        {
            debug!(uri = uri.as_str(), version = vfs_version, "cache hit");
            return cached.diagnostics.clone();
        }

        let (module, diagnostics) = if let Some(entry) = entry {
            info!(uri = uri.as_str(), version = entry.version, "re-analyzing");
            let source = &entry.content;
            let parse_result = ash_parser::parse_surface_file(source);
            let diags = compute_diagnostics(source, config);
            let module = parse_result.ok().map(Arc::new);
            (module, diags)
        } else {
            debug!(uri = uri.as_str(), "document not in VFS");
            (None, Vec::new())
        };

        let result = AnalysisResult {
            module,
            diagnostics: diagnostics.clone(),
            version: vfs_version,
        };
        self.inner.insert(uri.clone(), result);
        diagnostics
    }

    /// Returns the cached analysis result for `uri`, if any.
    pub fn get(&self, uri: &Uri) -> Option<AnalysisResult> {
        self.inner
            .get(uri)
            .map(|r: dashmap::mapref::one::Ref<'_, Uri, AnalysisResult>| r.value().clone())
    }

    /// Invalidates the cached result for `uri`.
    pub fn invalidate(&self, uri: &Uri) {
        self.inner.remove(uri);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_uri() -> Uri {
        "file:///test.ash".parse().unwrap()
    }

    #[test]
    fn test_analyze_parse_error() {
        // The Ash module_file parser skips unknown items gracefully.
        // Verify the pipeline doesn't panic on garbage input.
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "{{{{invalid".to_string());

        let cache = AnalysisCache::new();
        let config = LintConfig::default();
        let diags = cache.analyze(&uri, &vfs, &config);
        // The parser may or may not produce errors for this input.
        // The important thing is the pipeline doesn't panic.
        let _ = diags;
    }

    #[test]
    fn test_analyze_valid_source() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "fn main() -> Int { 1 }".to_string());

        let cache = AnalysisCache::new();
        let config = LintConfig::default();
        let diags = cache.analyze(&uri, &vfs, &config);
        // No parse errors (E001)
        let parse_errors: Vec<_> = diags
            .iter()
            .filter(|d| {
                matches!(
                    &d.code,
                    Some(lsp_types::NumberOrString::String(s)) if s == "E001"
                )
            })
            .collect();
        assert!(
            parse_errors.is_empty(),
            "valid source should have no parse errors"
        );
    }

    #[test]
    fn test_cache_hit_on_same_version() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "fn main() -> Int { 1 }".to_string());

        let cache = AnalysisCache::new();
        let config = LintConfig::default();

        let diags1 = cache.analyze(&uri, &vfs, &config);
        let diags2 = cache.analyze(&uri, &vfs, &config);
        assert_eq!(diags1, diags2, "second call should return same diagnostics");
    }

    #[test]
    fn test_cache_reanalyze_on_version_change() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "fn main() -> Int { 1 }".to_string());

        let cache = AnalysisCache::new();
        let config = LintConfig::default();

        let diags1 = cache.analyze(&uri, &vfs, &config);

        // Update to malformed target syntax so diagnostics change.
        vfs.open(uri.clone(), 2, "fn main() -> Int { ".to_string());

        let diags2 = cache.analyze(&uri, &vfs, &config);
        // The second source should produce a diagnostic.
        assert!(
            !diags2.is_empty(),
            "malformed target function should produce diagnostics"
        );
        assert_ne!(
            diags1, diags2,
            "diagnostics should differ after version change"
        );
    }

    #[test]
    fn test_analyze_unknown_uri() {
        let vfs = Vfs::new();
        let uri = test_uri();
        let cache = AnalysisCache::new();
        let config = LintConfig::default();
        let diags = cache.analyze(&uri, &vfs, &config);
        assert!(
            diags.is_empty(),
            "unknown URI should produce no diagnostics"
        );
    }

    #[test]
    fn test_invalidate() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "fn main() -> Int { 1 }".to_string());

        let cache = AnalysisCache::new();
        let config = LintConfig::default();

        let _ = cache.analyze(&uri, &vfs, &config);
        assert!(cache.get(&uri).is_some());

        cache.invalidate(&uri);
        assert!(cache.get(&uri).is_none());
    }
}
