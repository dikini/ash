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
use ash_parser::module::ModuleDecl;
use ash_parser::surface::{Definition, Expr, MacroTypeSignatureSummary, ModuleFile, Type};
use dashmap::DashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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
    /// Number of macro declarations visible in this parsed surface.
    pub macro_count: usize,
    /// Lightweight syntax-phase macro keys that affect LSP cache validity.
    pub macro_summary_keys: Vec<MacroSummaryKey>,
}

/// Lightweight syntax-phase macro key used for LSP cache invalidation.
///
/// This intentionally stores strings and compact hashes instead of full AST
/// nodes so the salsa-tracked parse summary remains `Eq + Hash`. The key is
/// syntax-phase metadata only; it does not carry callable authority, rows,
/// contracts, providers, or runtime effects.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MacroSummaryKey {
    /// Macro declaration name.
    pub name: String,
    /// Compact syntax-phase macro identity key.
    pub identity_key: SymbolIdentityKey,
    /// Public/private visibility spelling.
    pub visibility: String,
    /// Number of macro parameters.
    pub param_count: usize,
    /// Macro parameter names in declaration order.
    pub param_names: Vec<String>,
    /// Compact typed-signature shape, when present.
    pub typed_signature: Option<String>,
    /// Stable-enough template fingerprint for cache invalidation.
    pub template_hash: u64,
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
        Ok(module) => parse_summary_from_module(&module),
        Err(errors) => ParseSummary {
            succeeded: false,
            error_count: errors.len(),
            definition_count: 0,
            module_decl_count: 0,
            macro_count: 0,
            macro_summary_keys: Vec::new(),
        },
    }
}

/// Symbol index for a single file: maps names to their definition spans.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct SymbolIndex {
    /// Document-level symbols (functions, types, modules, etc.)
    pub document_symbols: Vec<Symbol>,
    /// Name → definition location mapping for same-file goto-definition.
    pub definitions: Vec<(String, SymbolLocation)>,
}

/// A symbol in the symbol index.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    /// The name of the symbol.
    pub name: String,
    /// The kind of symbol (function, type, module, etc.)
    pub kind: SymbolKind,
    /// The line of the symbol's definition (1-indexed).
    pub line: usize,
    /// The column of the symbol's definition (1-indexed).
    pub column: usize,
    /// Compact semantic identity key when this symbol participates in resolved
    /// same-file navigation.
    pub identity_key: Option<SymbolIdentityKey>,
}

/// Compact symbol identity key suitable for LSP summaries and indexes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolIdentityKey {
    /// Identity class.
    pub kind: SymbolIdentityKind,
    /// Name visible at the current file/use site.
    pub local_name: String,
    /// Origin module path for imported summaries, when known.
    pub origin_module: Option<String>,
    /// Exported origin name before aliasing, when known.
    pub origin_name: String,
    /// Declaration line used to distinguish same-file same-name declarations.
    pub origin_line: usize,
    /// Declaration column used to distinguish same-file same-name declarations.
    pub origin_column: usize,
    /// Declaration arity.
    pub param_count: usize,
}

/// Semantic class for a compact symbol identity key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SymbolIdentityKind {
    /// Syntax-phase macro identity.
    Macro,
    /// Runtime-callable ordinary function identity.
    Function,
    /// Runtime-callable builtin function identity.
    BuiltinFn,
}

/// Kinds of symbols tracked in the index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    /// A function definition.
    Function,
    /// A syntax-phase macro definition.
    Macro,
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
            identity_key: None,
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
    let mut summary = parse_summary_from_module(&parsed.module);
    summary.succeeded = parsed.errors.is_empty();
    summary.error_count = parsed.errors.len();
    summary
}

fn parse_summary_from_module(module: &ModuleFile) -> ParseSummary {
    let macro_summary_keys = macro_summary_keys(module);
    ParseSummary {
        succeeded: true,
        error_count: 0,
        definition_count: module.definitions.len(),
        module_decl_count: module.module_decls.len(),
        macro_count: macro_summary_keys.len(),
        macro_summary_keys,
    }
}

fn macro_summary_keys(module: &ModuleFile) -> Vec<MacroSummaryKey> {
    let mut keys = Vec::new();
    collect_macro_summary_keys(&module.definitions, &mut keys);
    collect_module_macro_summary_keys(&module.module_decls, &mut keys);
    keys.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.identity_key.cmp(&right.identity_key))
            .then_with(|| left.param_count.cmp(&right.param_count))
            .then_with(|| left.param_names.cmp(&right.param_names))
            .then_with(|| left.visibility.cmp(&right.visibility))
            .then_with(|| left.typed_signature.cmp(&right.typed_signature))
            .then_with(|| left.template_hash.cmp(&right.template_hash))
    });
    keys
}

fn collect_module_macro_summary_keys(modules: &[ModuleDecl], keys: &mut Vec<MacroSummaryKey>) {
    for module in modules {
        if let Some(definitions) = module.definitions() {
            collect_macro_summary_keys(definitions, keys);
        }
    }
}

fn collect_macro_summary_keys(definitions: &[Definition], keys: &mut Vec<MacroSummaryKey>) {
    for definition in definitions {
        if let Definition::Macro(decl) = definition {
            keys.push(MacroSummaryKey {
                name: decl.name.to_string(),
                identity_key: SymbolIdentityKey {
                    kind: SymbolIdentityKind::Macro,
                    local_name: decl.name.to_string(),
                    origin_module: None,
                    origin_name: decl.name.to_string(),
                    origin_line: decl.span.line,
                    origin_column: decl.span.column,
                    param_count: decl.params.len(),
                },
                visibility: format!("{:?}", decl.visibility),
                param_count: decl.params.len(),
                param_names: decl.params.iter().map(ToString::to_string).collect(),
                typed_signature: decl.typed_signature.as_ref().map(format_macro_signature),
                template_hash: hash_expr_template(&decl.body),
            });
        }
    }
}

fn format_macro_signature(signature: &MacroTypeSignatureSummary) -> String {
    let params = signature
        .param_types
        .iter()
        .map(|ty| ty.as_ref().map_or_else(|| "_".to_string(), format_type))
        .collect::<Vec<_>>()
        .join(", ");
    let ret = signature
        .return_type
        .as_ref()
        .map_or_else(|| "_".to_string(), format_type);
    format!("({params}) -> {ret}")
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::Name(name) => name.to_string(),
        Type::Hole { .. } => "_".to_string(),
        Type::List(inner) => format!("[{}]", format_type(inner)),
        Type::Tuple(items) => format!(
            "({})",
            items.iter().map(format_type).collect::<Vec<_>>().join(", ")
        ),
        Type::Record(fields) => format!(
            "{{{}}}",
            fields
                .iter()
                .map(|(name, ty)| format!("{name}: {}", format_type(ty)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::Capability(name) => format!("Capability<{name}>"),
        Type::Constructor { name, args } => format!(
            "{}<{}>",
            name,
            args.iter().map(format_type).collect::<Vec<_>>().join(", ")
        ),
        Type::Associated { base, name } => format!("{}::{name}", format_type(base)),
        Type::AssociatedFamilyProjection {
            interface,
            args,
            member,
            ..
        } => format!(
            "<{}<{}>>::{}",
            interface,
            args.iter().map(format_type).collect::<Vec<_>>().join(", "),
            member
        ),
        Type::Fn(params, _row, ret) => {
            let params = params
                .iter()
                .map(format_type)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({params}) -> {}", format_type(ret))
        }
    }
}

fn hash_expr_template(expr: &Expr) -> u64 {
    let mut hasher = DefaultHasher::new();
    format!("{expr:?}").hash(&mut hasher);
    hasher.finish()
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
            SymbolKind::Macro,
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
    };

    let identity_key = match def {
        Definition::Macro(m) => Some(SymbolIdentityKey {
            kind: SymbolIdentityKind::Macro,
            local_name: m.name.to_string(),
            origin_module: None,
            origin_name: m.name.to_string(),
            origin_line: m.span.line,
            origin_column: m.span.column,
            param_count: m.params.len(),
        }),
        Definition::Function(f) => Some(SymbolIdentityKey {
            kind: SymbolIdentityKind::Function,
            local_name: f.name.to_string(),
            origin_module: None,
            origin_name: f.name.to_string(),
            origin_line: f.span.line,
            origin_column: f.span.column,
            param_count: f.params.len(),
        }),
        Definition::BuiltinFn(b) => Some(SymbolIdentityKey {
            kind: SymbolIdentityKind::BuiltinFn,
            local_name: b.name.to_string(),
            origin_module: None,
            origin_name: b.name.to_string(),
            origin_line: b.span.line,
            origin_column: b.span.column,
            param_count: b.params.len(),
        }),
        _ => None,
    };

    index.document_symbols.push(Symbol {
        name: name.clone(),
        kind,
        line,
        column,
        identity_key,
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
    fn test_parse_summary_tracks_macro_signature_and_template_shape() {
        let mut db = AshLspDatabase::new();
        let file = SourceFile::new(
            &db,
            "file:///test.ash".to_string(),
            "pub macro id(x: Int) => x;".to_string(),
            1,
        );

        let summary1 = parse_summary(&db, file);
        assert_eq!(summary1.definition_count, 1);
        assert_eq!(summary1.macro_count, 1);
        assert_eq!(
            summary1.macro_summary_keys[0].typed_signature.as_deref(),
            Some("(Int) -> _")
        );
        assert_eq!(
            summary1.macro_summary_keys[0].param_names,
            vec!["x".to_string()]
        );
        assert_eq!(
            summary1.macro_summary_keys[0].identity_key.kind,
            SymbolIdentityKind::Macro
        );
        assert_eq!(summary1.macro_summary_keys[0].identity_key.local_name, "id");

        file.set_text(&mut db)
            .to("pub macro id(x: Bool) => !x;".to_string());

        let summary2 = parse_summary(&db, file);
        assert_eq!(summary2.definition_count, 1);
        assert_eq!(summary2.macro_count, 1);
        assert_ne!(
            summary1.macro_summary_keys, summary2.macro_summary_keys,
            "same-count macro signature/template edits must invalidate LSP parse summaries"
        );

        file.set_text(&mut db)
            .to("pub macro id(y: Int) => y;".to_string());

        let summary3 = parse_summary(&db, file);
        assert_ne!(
            summary1.macro_summary_keys, summary3.macro_summary_keys,
            "same-count macro parameter-name edits must invalidate LSP parse summaries"
        );
    }

    #[test]
    fn macro_summary_renders_function_types_with_target_callable_syntax() {
        let db = AshLspDatabase::new();
        let file = SourceFile::new(
            &db,
            "file:///test.ash".to_string(),
            "pub macro apply(f: (Int) -> Bool) -> (String) -> Bool => f;".to_string(),
            1,
        );

        let summary = parse_summary(&db, file);

        assert!(summary.succeeded);
        assert_eq!(
            summary.macro_summary_keys[0].typed_signature.as_deref(),
            Some("((Int) -> Bool) -> (String) -> Bool")
        );
    }

    #[test]
    fn test_get_module_caches_ast() {
        let db = AshLspDatabase::new();
        let file = SourceFile::new(
            &db,
            "file:///test.ash".to_string(),
            "fn main() -> Int { 42 }".to_string(),
            1,
        );

        let module = db.get_module(file).expect("should parse");
        assert!(module.definitions.iter().any(|def| matches!(
            def,
            ash_parser::Definition::Function(function) if function.name.as_ref() == "main"
        )));
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
    fn test_build_symbol_index_entry_function() {
        let db = AshLspDatabase::new();
        let file = SourceFile::new(
            &db,
            "file:///test.ash".to_string(),
            "fn main() -> Int { 42 }".to_string(),
            1,
        );

        let index = build_symbol_index(&db, file);
        assert_eq!(index.document_symbols.len(), 1);
        assert_eq!(index.document_symbols[0].name, "main");
        assert_eq!(index.document_symbols[0].kind, SymbolKind::Function);
    }

    #[test]
    fn test_build_symbol_index_marks_macros_as_macros() {
        let db = AshLspDatabase::new();
        let file = SourceFile::new(
            &db,
            "file:///test.ash".to_string(),
            "macro id(x) => x;".to_string(),
            1,
        );

        let index = build_symbol_index(&db, file);
        assert_eq!(index.document_symbols.len(), 1);
        assert_eq!(index.document_symbols[0].name, "id");
        assert_eq!(index.document_symbols[0].kind, SymbolKind::Macro);
    }
}
