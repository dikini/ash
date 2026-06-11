//! Salsa-backed analysis cache for `ash-lsp-core`.
//!
//! This module provides [`SalsaAnalysisCache`], an incremental analysis cache
//! built on top of the Salsa database in [`crate::db`].  It mirrors the API of
//! the existing [`crate::analysis::AnalysisCache`] so that callers can migrate
//! incrementally.
//!
//! The cache maps LSP document URIs to salsa [`SourceFile`] inputs.  When the
//! VFS reports a change, the corresponding `SourceFile` is updated, which
//! automatically invalidates dependent salsa queries (`parse_summary`,
//! `build_symbol_index`, and future typecheck diagnostics).

use crate::db::{AshLspDatabase, SourceFile};
use crate::diagnostics::compute_diagnostics;
use crate::vfs::Vfs;
use ash_lint::LintConfig;
use ash_parser::surface::ModuleFile;
use lsp_types::{Diagnostic, Uri};
use salsa::Setter;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Salsa-backed analysis cache keyed by document URI.
///
/// Holds a single [`AshLspDatabase`] and a mapping from LSP URIs to salsa
/// [`SourceFile`] inputs.  The database itself tracks versions and
/// invalidation, so this cache only needs to keep the URI→input mapping.
#[derive(Debug, Clone)]
pub struct SalsaAnalysisCache {
    inner: Arc<Mutex<SalsaAnalysisCacheInner>>,
}

#[derive(Debug)]
struct SalsaAnalysisCacheInner {
    /// The Salsa database that owns all inputs and tracked queries.
    db: AshLspDatabase,
    /// Maps LSP URIs to their salsa input handles.
    inputs: HashMap<Uri, SourceFile>,
}

impl Default for SalsaAnalysisCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SalsaAnalysisCache {
    /// Creates a new, empty salsa-backed analysis cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SalsaAnalysisCacheInner {
                db: AshLspDatabase::new(),
                inputs: HashMap::new(),
            })),
        }
    }

    /// Analyzes the document at `uri`, using the VFS for content and `config`
    /// for lint settings.
    ///
    /// Returns the current diagnostics.  The first call for a URI parses and
    /// lints the file; subsequent calls reuse salsa-memoized results until the
    /// VFS version changes.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn analyze(&self, uri: &Uri, vfs: &Vfs, config: &LintConfig) -> Vec<Diagnostic> {
        let entry = vfs.get(uri);
        let Some(entry) = entry else {
            debug!(uri = uri.as_str(), "document not in VFS");
            return Vec::new();
        };

        let mut inner = self.inner.lock().expect("lock poisoned");
        let file = inner.get_or_create_source_file(uri, &entry.content, entry.version);

        info!(
            uri = uri.as_str(),
            version = entry.version,
            "salsa re-analyzing"
        );

        // Force salsa to observe the input so that dependent queries are
        // invalidated on subsequent changes.
        let _summary = crate::db::parse_summary(&inner.db, file);

        // Get the AST via the side-cache (or fresh parse).
        let module = inner.db.get_module(file);

        // Compute diagnostics from source + lint config.
        let mut diagnostics = compute_diagnostics(&entry.content, config);

        // Add parse errors from the salsa side-cache.
        if module.is_none() {
            for err in inner.db.get_errors(file) {
                diagnostics.push(parse_error_to_diagnostic(&err));
            }
        }

        diagnostics
    }

    /// Returns the cached parsed module for `uri`, if any.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn get_module(&self, uri: &Uri) -> Option<Arc<ModuleFile>> {
        let inner = self.inner.lock().expect("lock poisoned");
        let file = *inner.inputs.get(uri)?;
        inner.db.get_module(file)
    }

    /// Invalidates the cached result for `uri`.
    ///
    /// This removes the URI→input mapping and the AST side-cache entry.  The
    /// next `analyze` call will create a fresh input.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn invalidate(&self, uri: &Uri) {
        let mut inner = self.inner.lock().expect("lock poisoned");
        if let Some(file) = inner.inputs.remove(uri) {
            let id = salsa::plumbing::AsId::as_id(&file);
            inner.db.ast_cache.remove(&id);
        }
    }
}

impl SalsaAnalysisCacheInner {
    /// Creates or updates the salsa input for `uri`.
    fn get_or_create_source_file(&mut self, uri: &Uri, content: &str, version: i32) -> SourceFile {
        if let Some(file) = self.inputs.get(uri) {
            // Input exists: update if version changed.
            let current_version = file.version(&self.db);
            if current_version != version {
                file.set_version(&mut self.db).to(version);
                file.set_text(&mut self.db).to(content.to_string());
            } else if file.text(&self.db) != content {
                file.set_text(&mut self.db).to(content.to_string());
            }
            *file
        } else {
            let file = SourceFile::new(&self.db, uri.to_string(), content.to_string(), version);
            self.inputs.insert(uri.clone(), file);
            file
        }
    }
}

fn parse_error_to_diagnostic(err: &ash_parser::ParseError) -> Diagnostic {
    use lsp_types::{DiagnosticSeverity, Position, Range};

    Diagnostic {
        range: Range {
            start: Position {
                line: err.span.line.saturating_sub(1) as u32,
                character: err.span.column.saturating_sub(1) as u32,
            },
            end: Position {
                line: err.span.line.saturating_sub(1) as u32,
                character: err.span.column.saturating_sub(1) as u32,
            },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: None,
        code_description: None,
        source: Some("ash".to_string()),
        message: err.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::Vfs;

    fn test_uri() -> Uri {
        "file:///test.ash".parse().unwrap()
    }

    #[test]
    fn test_salsa_cache_basic() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "workflow main { done }".to_string());

        let cache = SalsaAnalysisCache::new();
        let config = LintConfig::default();
        let diags = cache.analyze(&uri, &vfs, &config);

        // Should not panic and should return some result.
        let _ = diags;
        assert!(cache.get_module(&uri).is_some());
    }

    #[test]
    fn test_salsa_cache_invalidation() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(
            uri.clone(),
            1,
            "fn add(a: Int, b: Int) -> Int { a + b }".to_string(),
        );

        let cache = SalsaAnalysisCache::new();
        let config = LintConfig::default();
        let _ = cache.analyze(&uri, &vfs, &config);

        let module1 = cache.get_module(&uri).unwrap();
        assert_eq!(module1.definitions.len(), 1);

        // Simulate a VFS change.
        vfs.open(
            uri.clone(),
            2,
            "fn add(a: Int, b: Int) -> Int { a + b }\nfn sub(a: Int, b: Int) -> Int { a - b }"
                .to_string(),
        );
        let _ = cache.analyze(&uri, &vfs, &config);

        let module2 = cache.get_module(&uri).unwrap();
        assert_eq!(module2.definitions.len(), 2);
    }

    #[test]
    fn test_salsa_cache_invalidate_method() {
        let vfs = Vfs::new();
        let uri = test_uri();
        vfs.open(uri.clone(), 1, "workflow main { done }".to_string());

        let cache = SalsaAnalysisCache::new();
        let config = LintConfig::default();
        let _ = cache.analyze(&uri, &vfs, &config);
        assert!(cache.get_module(&uri).is_some());

        cache.invalidate(&uri);
        assert!(cache.get_module(&uri).is_none());
    }
}
