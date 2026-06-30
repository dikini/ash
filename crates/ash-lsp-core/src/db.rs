//! Salsa database for incremental analysis in `ash-lsp-core`.
//!
//! Defines the [`AshLspDatabase`] salsa database and the tracked queries
//! that drive parse, module-graph, type-check, and symbol-index computations.
//!
//! Design note: The Ash AST (`ModuleFile`, `Definition`, etc.) does not implement
//! `Eq` + `Hash` (by design — it contains `f64` via `OrderedFloat`, spans, and
//! other non-hashable data).  Therefore the AST is **not** stored directly in
//! salsa tracked queries.  Instead:
//!
//! 1. `SourceFile` is a salsa input (uri + text + version).
//! 2. `parse_file` returns a lightweight `ParseSummary` that *is* `Eq`+`Hash`.
//! 3. The actual `Arc<ModuleFile>` lives in a `DashMap` side-cache keyed by
//!    `SourceFile` salsa-id.  When salsa detects a change (via `ParseSummary`
//!    inequality) the side-cache entry is refreshed.
//! 4. Symbol-index and diagnostics are salsa tracked queries that depend on
//!    `ParseSummary`, not on the AST itself.

use ash_parser::ParseError;
use ash_parser::surface::ModuleFile;
use dashmap::DashMap;
use std::sync::Arc;

/// Input: a source file tracked by the LSP VFS.
///
/// This is a salsa *input* — mutating it invalidates all queries that
/// depend on it.
#[salsa::input(debug)]
pub struct SourceFile {
    /// The URI of the document.
    pub uri: String,

    /// The full text content of the document.
    pub text: String,

    /// The VFS version (monotonically increasing on each change).
    pub version: i32,
}

/// Lightweight parse summary used as a salsa tracked return value.
///
/// This is cheap to compare and hash.  When it changes, downstream queries
/// re-run, and the AST side-cache is refreshed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParseSummary {
    /// Whether parsing succeeded.
    pub succeeded: bool,
    /// Number of parse errors.
    pub error_count: usize,
    /// Number of top-level definitions.
    pub definition_count: usize,
    /// Number of module declarations.
    pub module_decl_count: usize,
    /// Whether a workflow is present.
    pub has_workflow: bool,
}

/// Side-cache entry for the actual AST.
///
/// Lives outside salsa's tracked graph so that the AST does not need `Eq`+`Hash`.
pub struct ParsedModule {
    /// The parsed AST.
    pub module: Arc<ModuleFile>,
    /// Parse errors, if any.
    pub errors: Vec<ParseError>,
}

/// Tracked query: produce a lightweight parse summary.
///
/// The summary is used by salsa for change-detection.  The actual AST is
/// stored in the side-cache (see [`AshLspDatabase::get_module`]).
#[salsa::tracked]
pub fn parse_summary(db: &dyn salsa::Database, file: SourceFile) -> ParseSummary {
    let text = file.text(db);
    match ash_parser::parse_surface_file(&text) {
        Ok(module) => ParseSummary {
            succeeded: true,
            error_count: 0,
            definition_count: module.definitions.len(),
            module_decl_count: module.module_decls.len(),
            has_workflow: module.workflow.is_some(),
        },
        Err(errors) => ParseSummary {
            succeeded: false,
            error_count: errors.len(),
            definition_count: 0,
            module_decl_count: 0,
            has_workflow: false,
        },
    }
}

/// Symbol index for a single file: maps names to their definition spans.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct SymbolIndex {
    /// Document-level symbols (workflows, functions, types, etc.)
    pub document_symbols: Vec<Symbol>,
    /// Name → definition location mapping for same-file goto-definition.
    pub definitions: Vec<(String, SymbolLocation)>,
}

/// A symbol in the symbol index.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    /// The name of the symbol.
    pub name: String,
    /// The kind of symbol (function, workflow, type, etc.)
    pub kind: SymbolKind,
    /// The line of the symbol's definition (1-indexed).
    pub line: usize,
    /// The column of the symbol's definition (1-indexed).
    pub column: usize,
}

/// Kinds of symbols tracked in the index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    /// A workflow definition.
    Workflow,
    /// A function definition.
    Function,
    /// A type definition.
    Type,
    /// An interface definition.
    Interface,
    /// An implementation definition.
    Impl,
    /// A capability definition.
    Capability,
    /// A role definition.
    Role,
    /// A policy definition.
    Policy,
    /// A module declaration.
    Module,
    /// A proxy definition.
    Proxy,
    /// A law declaration.
    Law,
    /// A proof declaration.
    Proof,
    /// A builtin function.
    BuiltinFn,
    /// A sealed domain.
    SealedDomain,
    /// A data kind.
    DataKind,
    /// A type function.
    TypeFn,
    /// A proposition predicate.
    Proposition,
    /// A resource type.
    ResourceType,
    /// A capability interface.
    CapabilityInterface,
    /// A capability implementation.
    CapabilityImplementation,
}

/// Location of a symbol definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolLocation {
    /// The URI of the file containing the definition.
    pub uri: String,
    /// The line of the definition (1-indexed).
    pub line: usize,
    /// The column of the definition (1-indexed).
    pub column: usize,
}

/// Concrete Salsa database implementation.
#[salsa::db]
#[derive(Default)]
pub struct AshLspDatabase {
    storage: salsa::Storage<Self>,
    /// Side-cache for parsed ASTs, keyed by salsa `SourceFile` id.
    ///
    /// This is NOT part of salsa's tracked graph — it's a performance
    /// optimization that lets us reuse `Arc<ModuleFile>` without requiring
    /// `Eq`+`Hash` on the AST itself.
    pub ast_cache: DashMap<salsa::Id, ParsedModule>,
}

#[salsa::db]
impl salsa::Database for AshLspDatabase {}

impl std::fmt::Debug for AshLspDatabase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AshLspDatabase").finish()
    }
}

impl AshLspDatabase {
    /// Create a new database with an empty AST cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            storage: salsa::Storage::default(),
            ast_cache: DashMap::new(),
        }
    }

    /// Get or parse a module for the given source file.
    ///
    /// Uses the salsa-tracked `parse_summary` for change detection, but
    /// stores the actual AST in the side-cache.
    pub fn get_module(&self, file: SourceFile) -> Option<Arc<ModuleFile>> {
        let id = salsa::plumbing::AsId::as_id(&file);

        // Fast path: cache hit
        if let Some(entry) = self.ast_cache.get(&id) {
            // Verify the cached entry is still valid by checking the summary
            let summary = parse_summary(self, file);
            let cached_summary = compute_summary_from_cache(&entry);
            if summary == cached_summary {
                return Some(entry.module.clone());
            }
        }

        // Slow path: re-parse and cache
        let text = file.text(self);
        match ash_parser::parse_surface_file(&text) {
            Ok(module) => {
                let module = Arc::new(module);
                let parsed = ParsedModule {
                    module: module.clone(),
                    errors: Vec::new(),
                };
                self.ast_cache.insert(id, parsed);
                Some(module)
            }
            Err(errors) => {
                self.ast_cache.insert(
                    id,
                    ParsedModule {
                        module: Arc::new(ModuleFile::default()),
                        errors,
                    },
                );
                None
            }
        }
    }

    /// Get parse errors for a file (from cache or fresh parse).
    pub fn get_errors(&self, file: SourceFile) -> Vec<ParseError> {
        let id = salsa::plumbing::AsId::as_id(&file);

        if let Some(entry) = self.ast_cache.get(&id) {
            let summary = parse_summary(self, file);
            let cached_summary = compute_summary_from_cache(&entry);
            if summary == cached_summary {
                return entry.errors.clone();
            }
        }

        // Re-parse to get errors
        let text = file.text(self);
        match ash_parser::parse_surface_file(&text) {
            Ok(_) => Vec::new(),
            Err(errors) => errors,
        }
    }
}

/// Build a symbol index for the given file.
///
/// This is a salsa-tracked query that depends on `parse_summary`.
#[salsa::tracked]
pub fn build_symbol_index(db: &dyn salsa::Database, file: SourceFile) -> SymbolIndex {
    let _summary = parse_summary(db, file); // Ensure dependency

    // Get the AST (from cache if available, otherwise parse fresh)
    let text = file.text(db);
    let module = match ash_parser::parse_surface_file(&text) {
        Ok(m) => m,
        Err(_) => return SymbolIndex::default(),
    };

    let mut index = SymbolIndex::default();
    let uri = file.uri(db);

    // Index workflow
    if let Some(workflow) = &module.workflow {
        let name = workflow.name.as_ref().to_string();
        index.document_symbols.push(Symbol {
            name: name.clone(),
            kind: SymbolKind::Workflow,
            line: workflow.span.line,
            column: workflow.span.column,
        });
        index.definitions.push((
            name,
            SymbolLocation {
                uri: uri.clone(),
                line: workflow.span.line,
                column: workflow.span.column,
            },
        ));
    }

    // Index top-level definitions
    for def in &module.definitions {
        index_definition(&mut index, def, &uri);
    }

    // Index module declarations and their definitions
    for module_decl in &module.module_decls {
        let name = module_decl.name.as_ref().to_string();
        index.document_symbols.push(Symbol {
            name: name.clone(),
            kind: SymbolKind::Module,
            line: module_decl.span.line,
            column: module_decl.span.column,
        });
        index.definitions.push((
            name,
            SymbolLocation {
                uri: uri.clone(),
                line: module_decl.span.line,
                column: module_decl.span.column,
            },
        ));

        if let Some(defs) = module_decl.definitions() {
            for def in defs {
                index_definition(&mut index, def, &uri);
            }
        }
    }

    index
}

fn compute_summary_from_cache(parsed: &ParsedModule) -> ParseSummary {
    ParseSummary {
        succeeded: parsed.errors.is_empty(),
        error_count: parsed.errors.len(),
        definition_count: parsed.module.definitions.len(),
        module_decl_count: parsed.module.module_decls.len(),
        has_workflow: parsed.module.workflow.is_some(),
    }
}

#[allow(clippy::too_many_lines)]
fn index_definition(index: &mut SymbolIndex, def: &ash_parser::surface::Definition, uri: &str) {
    use ash_parser::surface::Definition;

    let (name, kind, line, column) = match def {
        Definition::Notation(n) => (
            n.pattern.raw.as_ref().to_string(),
            SymbolKind::Function,
            n.span.line,
            n.span.column,
        ),
        Definition::Macro(m) => (
            m.name.as_ref().to_string(),
            SymbolKind::Function,
            m.span.line,
            m.span.column,
        ),
        Definition::Function(f) => (
            f.name.as_ref().to_string(),
            SymbolKind::Function,
            f.span.line,
            f.span.column,
        ),
        Definition::Type(t) => (
            t.name.as_ref().to_string(),
            SymbolKind::Type,
            t.span.line,
            t.span.column,
        ),
        Definition::Interface(i) => (
            i.name.as_ref().to_string(),
            SymbolKind::Interface,
            i.span.line,
            i.span.column,
        ),
        Definition::Impl(i) => (
            i.interface.as_ref().to_string(),
            SymbolKind::Impl,
            i.span.line,
            i.span.column,
        ),
        Definition::Capability(c) => (
            c.name.as_ref().to_string(),
            SymbolKind::Capability,
            c.span.line,
            c.span.column,
        ),
        Definition::Role(r) => (
            r.name.as_ref().to_string(),
            SymbolKind::Role,
            r.span.line,
            r.span.column,
        ),
        Definition::Policy(p) => (
            p.name.as_ref().to_string(),
            SymbolKind::Policy,
            p.span.line,
            p.span.column,
        ),
        Definition::Proxy(p) => (
            p.name.as_ref().to_string(),
            SymbolKind::Proxy,
            p.span.line,
            p.span.column,
        ),
        Definition::Law(l) => (
            l.name.as_ref().to_string(),
            SymbolKind::Law,
            l.span.line,
            l.span.column,
        ),
        Definition::Proof(p) => (
            p.name.as_ref().to_string(),
            SymbolKind::Proof,
            p.span.line,
            p.span.column,
        ),
        Definition::BuiltinFn(b) => (
            b.name.as_ref().to_string(),
            SymbolKind::BuiltinFn,
            b.span.line,
            b.span.column,
        ),
        Definition::SealedDomain(s) => (
            s.name.as_ref().to_string(),
            SymbolKind::SealedDomain,
            s.span.line,
            s.span.column,
        ),
        Definition::DataKind(d) => (
            d.name.as_ref().to_string(),
            SymbolKind::DataKind,
            d.span.line,
            d.span.column,
        ),
        Definition::TypeFn(t) => (
            t.name.as_ref().to_string(),
            SymbolKind::TypeFn,
            t.span.line,
            t.span.column,
        ),
        Definition::PropositionPredicate(p) => (
            p.name.as_ref().to_string(),
            SymbolKind::Proposition,
            p.span.line,
            p.span.column,
        ),
        Definition::ResourceType(r) => (
            r.name.as_ref().to_string(),
            SymbolKind::ResourceType,
            r.span.line,
            r.span.column,
        ),
        Definition::CapabilityInterface(c) => (
            c.name.as_ref().to_string(),
            SymbolKind::CapabilityInterface,
            c.span.line,
            c.span.column,
        ),
        Definition::CapabilityImplementation(c) => (
            c.name.as_ref().to_string(),
            SymbolKind::CapabilityImplementation,
            c.span.line,
            c.span.column,
        ),
    };

    index.document_symbols.push(Symbol {
        name: name.clone(),
        kind,
        line,
        column,
    });
    index.definitions.push((
        name,
        SymbolLocation {
            uri: uri.to_string(),
            line,
            column,
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use salsa::Setter;

    #[test]
    fn test_parse_summary_basic() {
        let db = AshLspDatabase::new();
        let file = SourceFile::new(
            &db,
            "file:///test.ash".to_string(),
            "fn add(a: Int, b: Int) -> Int { a + b }".to_string(),
            1,
        );

        let summary = parse_summary(&db, file);
        assert!(summary.succeeded);
        assert_eq!(summary.error_count, 0);
        assert_eq!(summary.definition_count, 1);
        assert!(!summary.has_workflow);
    }

    #[test]
    fn test_parse_summary_updates_on_change() {
        let mut db = AshLspDatabase::new();
        let file = SourceFile::new(
            &db,
            "file:///test.ash".to_string(),
            "fn add(a: Int, b: Int) -> Int { a + b }".to_string(),
            1,
        );

        let summary1 = parse_summary(&db, file);
        assert_eq!(summary1.definition_count, 1);

        file.set_text(&mut db).to(
            "fn add(a: Int, b: Int) -> Int { a + b }\nfn sub(a: Int, b: Int) -> Int { a - b }"
                .to_string(),
        );

        let summary2 = parse_summary(&db, file);
        assert_eq!(summary2.definition_count, 2);
    }

    #[test]
    fn test_get_module_caches_ast() {
        let db = AshLspDatabase::new();
        let file = SourceFile::new(
            &db,
            "file:///test.ash".to_string(),
            "workflow main() { let x = 42 }".to_string(),
            1,
        );

        let module = db.get_module(file).expect("should parse");
        assert!(module.workflow.is_some());
        assert_eq!(db.ast_cache.len(), 1);

        // Second call should use cache
        let module2 = db.get_module(file).expect("should parse");
        assert!(Arc::ptr_eq(&module, &module2));
    }

    #[test]
    fn test_get_module_invalidates_on_change() {
        let mut db = AshLspDatabase::new();
        let file = SourceFile::new(
            &db,
            "file:///test.ash".to_string(),
            "fn add(a: Int, b: Int) -> Int { a + b }".to_string(),
            1,
        );

        let module1 = db.get_module(file).expect("should parse");
        assert_eq!(module1.definitions.len(), 1);

        file.set_text(&mut db).to(
            "fn add(a: Int, b: Int) -> Int { a + b }\nfn sub(a: Int, b: Int) -> Int { a - b }"
                .to_string(),
        );

        let module2 = db.get_module(file).expect("should parse");
        assert_eq!(module2.definitions.len(), 2);
        assert!(!Arc::ptr_eq(&module1, &module2));
    }

    #[test]
    fn test_get_errors_for_invalid_code() {
        let db = AshLspDatabase::new();
        let file = SourceFile::new(
            &db,
            "file:///test.ash".to_string(),
            "fn add(a: Int, b: Int) -> { a + b }".to_string(),
            1,
        );

        let errors = db.get_errors(file);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_build_symbol_index() {
        let db = AshLspDatabase::new();
        let file = SourceFile::new(
            &db,
            "file:///test.ash".to_string(),
            "fn add(a: Int, b: Int) -> Int { a + b }".to_string(),
            1,
        );

        let index = build_symbol_index(&db, file);
        assert_eq!(index.document_symbols.len(), 1);
        assert_eq!(index.document_symbols[0].name, "add");
        assert_eq!(index.definitions.len(), 1);
    }

    #[test]
    fn test_build_symbol_index_workflow() {
        let db = AshLspDatabase::new();
        let file = SourceFile::new(
            &db,
            "file:///test.ash".to_string(),
            "workflow main() { let x = 42 }".to_string(),
            1,
        );

        let index = build_symbol_index(&db, file);
        assert_eq!(index.document_symbols.len(), 1);
        assert_eq!(index.document_symbols[0].name, "main");
        assert_eq!(index.document_symbols[0].kind, SymbolKind::Workflow);
    }
}
