//! Ordinary file loader for import-backed execution.
//!
//! This loader supports a constrained executable subset:
//! - contiguous leading `use` imports on ordinary source files
//! - module resolution from the source tree, `ASH_LIBRARY_PATH`, and the built-in stdlib
//! - imported `pub type` definitions from resolved modules
//! - imported callable bodies from local source modules and stdlib `pub fn` / `pub use`

use crate::EngineError;
use ash_core::ast::{
    TypeBody as CoreTypeBody, TypeDef as CoreTypeDef, TypeExpr as CoreTypeExpr,
    VariantDef as CoreVariantDef, VariantPayload as CoreVariantPayload,
    Visibility as CoreVisibility,
};
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    AssociatedFamilySummary, AssociatedMemberIdentityId, ConstructorSummary,
    InterfaceEvidenceConstraintSummary, InterfaceIdentityId, InterfaceIdentitySummary,
    ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin, PromotedConstructorId,
    PromotedDataKindId, RepresentationExposure, SealedDomainId, SummaryVersion, TypeDeclId,
    TypeDeclSummary, TypeFunctionSummary, TypeRepresentationSummary,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, TypeComputationHeadId, TypeFunctionPattern, TypeFunctionPatternConstraint,
    TypeFunctionResultConstraint, TypeFunctionResultExpr, TypeProposition, TypePropositionTerm,
};
use ash_parser::Spanned;
use ash_parser::input::new_input;
use ash_parser::parse_module::{parse_builtin_fn_definition, parse_fn_definition};
use ash_parser::parse_use::parse_use;
use ash_parser::surface::{
    Definition, Expr, InterfaceDef, LocalMacroEntry, MacroDeclarationIdentity, MacroIdentityOrigin,
    MacroSummary, Type,
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use winnow::prelude::Parser;

type TypeFunctionNameSet = HashSet<String>;

const fn type_env_error_span(error: &ash_typeck::error::TypeEnvError) -> ash_parser::token::Span {
    error.span()
}

/// Ordinary-file loading output after import masking and dependency collection.
#[derive(Debug, Clone)]
pub struct LoadedOrdinaryFile {
    /// Ordinary entry/module source with the leading `use` prelude replaced by
    /// whitespace, preserving the original source coordinates.
    pub ordinary_source: String,
    /// Imported public type definitions collected from resolved modules.
    pub imported_type_defs: Vec<CoreTypeDef>,
    /// Imported semantic summaries collected from resolved modules.
    pub imported_semantic_summaries: Vec<ModuleSemanticSummary>,
    /// Source-visible imported type-function names keyed to canonical heads.
    ///
    /// Selected/glob imports populate this list; dependency-closure helper heads
    /// are transported in `imported_semantic_summaries` but deliberately omitted.
    pub imported_type_function_heads: Vec<(String, TypeComputationHeadId)>,
    /// Imported callable bodies keyed by the imported name.
    pub imported_callables: HashMap<String, InlineCallable>,
    /// Imported syntax-phase macro summaries. These do not activate macros yet.
    pub imported_macro_summaries: Vec<MacroSummary>,
}

/// Check whether a source file is a valid importable module surface.
///
/// # Errors
///
/// Returns [`EngineError`] if module exports cannot be collected.
pub fn check_importable_module_file(path: &Path) -> Result<(), EngineError> {
    let mut cache = HashMap::new();
    let mut visiting = HashSet::new();
    collect_module_exports(path, &mut cache, &mut visiting).map_err(|error| {
        EngineError::Parse(format!(
            "in '{}': failed to collect module exports: {error}",
            path.display()
        ))
    })?;
    Ok(())
}

pub(crate) fn validate_expanded_surface_module_file(
    path: &Path,
    source: &str,
) -> Result<(), EngineError> {
    expand_surface_module_file(path, source).map(|_| ())
}

pub(crate) fn expand_surface_module_file(
    path: &Path,
    source: &str,
) -> Result<ash_parser::surface::ExpandedSurfaceModule, EngineError> {
    let metadata_source = strip_module_metadata_non_definition_lines(source);
    let module = parse_module_file_for_type_metadata(path, &metadata_source)?;
    let imported_macros = collect_imported_macro_entries(path, source)?;
    ash_parser::surface::expand_surface_module_with_imported_macros(module, imported_macros)
        .map_err(|error| {
            EngineError::Parse(format!(
                "in '{}': expanded-surface validation failed: {error}",
                path.display()
            ))
        })
}

fn collect_imported_macro_entries(
    path: &Path,
    source: &str,
) -> Result<Vec<LocalMacroEntry>, EngineError> {
    let mut module_cache = HashMap::new();
    let mut visiting = HashSet::new();
    visiting.insert(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
    collect_imported_macro_entries_with_state(path, source, &mut module_cache, &mut visiting)
}

fn collect_imported_macro_entries_with_state(
    path: &Path,
    source: &str,
    module_cache: &mut HashMap<PathBuf, ModuleExports>,
    visiting: &mut HashSet<PathBuf>,
) -> Result<Vec<LocalMacroEntry>, EngineError> {
    let entry_root = path.parent().ok_or_else(|| {
        EngineError::Configuration(format!("module path '{}' has no parent", path.display()))
    })?;
    let crate_root = discover_crate_root(entry_root);
    let mut imported_macros = Vec::new();
    for import in parse_module_imports(source)? {
        let (module_segments, search_roots) =
            import_resolution_roots(&import.module_segments, entry_root, crate_root.as_deref())?;
        let module_path =
            resolve_module_path(&module_segments, &search_roots)?.ok_or_else(|| {
                EngineError::Parse(format!(
                    "module '{}' not found",
                    import.module_segments.join("::")
                ))
            })?;
        let exports = collect_module_exports(module_path.as_path(), module_cache, visiting)?;
        for selection in import.selections {
            match selection {
                ImportSelection::Glob => {
                    for summary in exports.macro_summaries.values() {
                        let template = exports
                            .macro_templates
                            .get(summary.name.as_ref())
                            .ok_or_else(|| {
                                EngineError::Parse(format!(
                                    "macro summary '{}' has no expansion template",
                                    summary.name
                                ))
                            })?;
                        validate_macro_summary_template(summary, template)?;
                        let mut entry = template.clone();
                        entry.identity =
                            MacroDeclarationIdentity::imported(summary, summary.name.clone());
                        imported_macros.push(entry);
                    }
                }
                ImportSelection::Named { name, alias } => {
                    if let Some(summary) = exports.macro_summaries.get(&name) {
                        let mut entry = exports
                            .macro_templates
                            .get(&name)
                            .ok_or_else(|| {
                                EngineError::Parse(format!(
                                    "macro summary '{name}' has no expansion template"
                                ))
                            })?
                            .clone();
                        validate_macro_summary_template(summary, &entry)?;
                        let local_name = alias.map_or_else(|| summary.name.clone(), Into::into);
                        entry.name = local_name.clone();
                        entry.identity = MacroDeclarationIdentity::imported(summary, local_name);
                        imported_macros.push(entry);
                    }
                }
            }
        }
    }
    Ok(imported_macros)
}

/// Whether a callable carries an Ash-level body or is bodyless (builtin).
#[derive(Debug, Clone)]
pub enum CallableKind {
    /// User-defined callable with an Ash expression body.
    User {
        /// The Ash expression constituting the callable body.
        body: Expr,
    },
    /// Bodyless builtin function resolved at link time.
    Builtin {
        /// Module path joined by `::` (e.g. `"string"`, `"record"`).
        ///
        /// # Invariant
        ///
        /// This field is `String::new()` inside raw module exports — it is
        /// populated by [`load_ordinary_file`] from the import path before the
        /// callable is inserted into `imported_callables`. Any code reading this
        /// field from a raw module-export value (i.e., outside of
        /// `load_ordinary_file`) will observe an empty string.
        module: String,
    },
}

/// Imported callable body and parameter list.
#[derive(Debug, Clone)]
pub struct InlineCallable {
    /// Name the callable was imported under.
    pub exported_name: String,
    /// Parameter names in call order.
    pub params: Vec<String>,
    /// Unqualified function names from the exporting module that should be
    /// treated as effectful when lowering imported callable bodies.
    pub effectful_names: HashSet<String>,
    /// Whether this callable has an Ash body or is a bodyless builtin.
    pub kind: CallableKind,
    /// Full declared type signature for imported callables.
    pub signature: Option<CallableSignature>,
    /// Explicit callable row requirement metadata parsed from the source
    /// signature. This is requirement metadata only; it does not install
    /// provider, admission, handler, or runtime authority.
    pub row_requirement: Option<CallableRowRequirementSummary>,
    /// Modules that have exported or re-exported this callable.
    ///
    /// This lets alias-rewrite passes update callables from the module whose
    /// type aliases changed without rewriting unrelated local callables that
    /// happen to use the same surface type name.
    pub exporting_modules: HashSet<ModuleIdentity>,
    /// Module-local user callables needed when this callable constructs a closure
    /// that refers to sibling helpers. These are runtime-only dependencies and
    /// are not inserted into the caller's source-visible import environment.
    pub module_runtime_callables: HashMap<String, Box<Self>>,
}

/// Source location category for an explicit callable row requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallableRowRequirementSource {
    /// Inline callable row after the function arrow: `-> { ... } T`.
    InlineReturn,
    /// Expanded callable row from `where row { ... }`.
    WhereRow,
}

/// Explicit row requirement carried with an imported/exported callable.
#[derive(Debug, Clone, PartialEq)]
pub struct CallableRowRequirementSummary {
    /// Whether the row came from inline syntax or an expanded `where row` block.
    pub source: CallableRowRequirementSource,
    /// Source-preserving row payload.
    pub row: ash_parser::surface::ComputationRow,
}

pub(crate) fn callable_row_requirement_from_fn_def(
    function: &ash_parser::surface::FnDef,
) -> Option<CallableRowRequirementSummary> {
    callable_inline_return_row(function.return_type.as_ref())
        .map(|row| CallableRowRequirementSummary {
            source: CallableRowRequirementSource::InlineReturn,
            row: row.clone(),
        })
        .or_else(|| {
            function
                .proposition_tail
                .as_ref()
                .and_then(|tail| tail.row.as_ref())
                .map(|row| CallableRowRequirementSummary {
                    source: CallableRowRequirementSource::WhereRow,
                    row: row.row.clone(),
                })
        })
}

pub(crate) fn callable_row_requirement_from_builtin(
    builtin: &ash_parser::surface::BuiltinFnDef,
) -> Option<CallableRowRequirementSummary> {
    callable_inline_return_row(Some(&builtin.return_type))
        .map(|row| CallableRowRequirementSummary {
            source: CallableRowRequirementSource::InlineReturn,
            row: row.clone(),
        })
        .or_else(|| {
            builtin
                .proposition_tail
                .as_ref()
                .and_then(|tail| tail.row.as_ref())
                .map(|row| CallableRowRequirementSummary {
                    source: CallableRowRequirementSource::WhereRow,
                    row: row.row.clone(),
                })
        })
}

pub(crate) const fn callable_inline_return_row(
    return_type: Option<&ash_parser::surface::Type>,
) -> Option<&ash_parser::surface::ComputationRow> {
    match return_type {
        Some(ash_parser::surface::Type::Fn(params, Some(row), _)) if params.is_empty() => Some(row),
        _ => None,
    }
}

/// Declared signature preserved for imported callables.
#[derive(Debug, Clone)]
pub enum CallableSignature {
    /// Signature from an ordinary `pub fn` definition.
    Function(ash_parser::surface::FnDef),
    /// Signature from a `pub builtin fn` definition.
    Builtin(ash_parser::surface::BuiltinFnDef),
}

#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ImportSpec {
    #[allow(missing_docs)]
    pub module_segments: Vec<String>,
    #[allow(missing_docs)]
    pub selections: Vec<ImportSelection>,
}

#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum ImportSelection {
    #[allow(missing_docs)]
    Named { name: String, alias: Option<String> },
    #[allow(missing_docs)]
    Glob,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ModuleExports {
    /// Digest of the source that produced this cache entry.  Module summaries
    /// are source-derived contracts, so a memory-cache hit is valid only while
    /// the source remains identical and its summary still validates.
    source_fingerprint: String,
    /// Transitive cache-validation fingerprints for public module dependencies.
    /// This is engine-private cache state, never a semantic-summary payload.
    public_dependency_fingerprints: HashMap<PathBuf, String>,
    /// Digest of the exported V7 provider/binding facts, including their
    /// sanitizer closure metadata.  A cache hit recomputes this from the
    /// candidate summary before it can expose a forged non-empty digest.
    effect_row_contract_fingerprint: Option<String>,
    pub(crate) type_defs: HashMap<String, CoreTypeDef>,
    pub(crate) constructor_defs: HashMap<String, CoreTypeDef>,
    pub(crate) callables: HashMap<String, InlineCallable>,
    /// Syntax-phase macro summaries keyed by exported public macro name.
    pub(crate) macro_summaries: HashMap<String, MacroSummary>,
    /// Syntax-phase macro templates keyed by exported public macro name.
    pub(crate) macro_templates: HashMap<String, LocalMacroEntry>,
    /// Core-owned ordinary type semantic summary lowered from the parsed `ModuleFile`.
    pub(crate) semantic_summary: Option<ModuleSemanticSummary>,
    /// Source-visible public type-function summary heads keyed by exported name.
    ///
    /// The summaries themselves are core-owned; this engine-private map is only
    /// an import-selection index. Dependency-closure helpers are transported in
    /// `semantic_summary.exported_type_functions` and are not inserted here
    /// unless the source module exported the head publicly.
    pub(crate) type_function_summaries: HashMap<String, TypeFunctionSummary>,
    /// Source-visible public associated-family summary heads keyed by exported member name.
    ///
    /// Dependency helper families remain transported in
    /// `semantic_summary.exported_associated_families` and are not inserted here
    /// unless the provider publicly exports the family head.
    pub(crate) associated_family_summaries: HashMap<String, AssociatedFamilySummary>,
    /// Child module exports loaded via `pub mod <name>;` declarations.
    ///
    /// Populated by TASK-540 but not yet consumed by `merge_use_exports` --
    /// `pub use` resolution currently goes through filesystem path resolution
    /// via `resolve_use_target`. This field is infrastructure for future
    /// qualified module path access (`llm::types::Role`).
    pub(crate) child_modules: HashMap<String, Self>,
}

/// Parse target-Ash source containing function definitions with `fn main` as
/// the entry computation.
///
/// # Errors
///
/// Returns a string describing the parse error if the source is invalid.
///
#[derive(Debug, Clone)]
pub(crate) struct ParsedProgram {
    pub program: ash_parser::surface::Program,
    /// Optional module path retained by the parser for source-sidecar provenance.
    pub source_path: Option<Box<str>>,
    pub expansion_origins: Vec<ash_parser::surface::ExpandedSurfaceOrigin>,
    /// Parser-validated expanded-surface identifier metadata retained only for
    /// the entry diagnostic/audit sidecar.
    pub identifier_hygiene: Vec<ash_parser::surface::IdentifierHygieneMetadata>,
}

pub(crate) fn parse_program_with_functions(
    source: &str,
    source_path: Option<&Path>,
) -> Result<ParsedProgram, String> {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use ash_parser::parse_utils::skip_whitespace_and_comments;
    use winnow::Parser;

    if source_contains_named_do_target(source) {
        return Err(
            "generic do target annotations are removed; use ambient `do { ... }` with row requirements"
                .to_string(),
        );
    }

    if let Ok(module) = ash_parser::parse_surface_file_with_path(source, source_path) {
        let expanded = ash_parser::surface::expand_surface_module(module)
            .map_err(|error| format!("expanded-surface validation failed: {error}"))?;
        let ash_parser::surface::ExpandedSurfaceModule {
            module,
            origins,
            hygiene,
            ..
        } = expanded;
        let source_path = module.path.clone();
        if let Some(program) = program_from_module_file(module) {
            return Ok(ParsedProgram {
                program,
                source_path,
                expansion_origins: origins,
                identifier_hygiene: hygiene,
            });
        }
    }

    let mut input = new_input(source);
    skip_whitespace_and_comments(&mut input);

    let mut definitions = Vec::new();
    loop {
        let snapshot = input.clone();
        if let Ok(definition) = parse_fn_definition.parse_next(&mut input) {
            definitions.push(definition);
            skip_whitespace_and_comments(&mut input);
        } else {
            input = snapshot;
            break;
        }
    }

    let (entry_function, entry_span) = definitions
        .iter()
        .find_map(|definition| match definition {
            ash_parser::surface::Definition::Function(function)
                if function.name.as_ref() == "main" =>
            {
                Some((function.name.clone(), function.span))
            }
            _ => None,
        })
        .ok_or_else(|| "expected fn main entry".to_string())?;
    if !input.input.is_empty() {
        return Err("unexpected trailing input after function definitions".to_string());
    }

    Ok(ParsedProgram {
        program: ash_parser::surface::Program {
            definitions,
            entry: ash_parser::surface::ProgramEntry {
                function: entry_function,
                span: entry_span,
            },
        },
        source_path: source_path.map(|path| path.to_string_lossy().into_owned().into()),
        expansion_origins: Vec::new(),
        identifier_hygiene: Vec::new(),
    })
}

/// Detect a target-form `do:<name>` before the entry parser can route it into
/// the historical generalized-do substrate.
///
/// Target Ash retains only ambient `do { ... }`; this narrow lexical guard is
/// intentionally placed at the engine's source-entry boundary so rejected
/// syntax never reaches typechecking or generic-do lowering. Strings and
/// single-line comments are skipped to avoid treating source text as syntax.
fn source_contains_named_do_target(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                index += 1;
                while index < bytes.len() {
                    if bytes[index] == b'\\' {
                        index = index.saturating_add(2);
                    } else if bytes[index] == b'"' {
                        index += 1;
                        break;
                    } else {
                        index += 1;
                    }
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'-' if bytes.get(index + 1) == Some(&b'-') => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'd' if bytes.get(index + 1) == Some(&b'o')
                && index
                    .checked_sub(1)
                    .and_then(|previous| bytes.get(previous))
                    .is_none_or(|previous| {
                        !previous.is_ascii_alphanumeric() && *previous != b'_'
                    }) =>
            {
                let mut after_keyword = index + 2;
                while bytes
                    .get(after_keyword)
                    .is_some_and(u8::is_ascii_whitespace)
                {
                    after_keyword += 1;
                }
                if bytes.get(after_keyword) == Some(&b':') {
                    let mut target = after_keyword + 1;
                    while bytes.get(target).is_some_and(u8::is_ascii_whitespace) {
                        target += 1;
                    }
                    if bytes.get(target).is_some_and(|character| {
                        character.is_ascii_alphabetic() || *character == b'_'
                    }) {
                        return true;
                    }
                }
                index += 2;
            }
            _ => index += 1,
        }
    }
    false
}

fn program_from_module_file(
    module: ash_parser::surface::ModuleFile,
) -> Option<ash_parser::surface::Program> {
    let (entry_function, entry_span) =
        module
            .definitions
            .iter()
            .find_map(|definition| match definition {
                ash_parser::surface::Definition::Function(function)
                    if function.name.as_ref() == "main" =>
                {
                    Some((function.name.clone(), function.span))
                }
                _ => None,
            })?;

    Some(ash_parser::surface::Program {
        definitions: module.definitions,
        entry: ash_parser::surface::ProgramEntry {
            function: entry_function,
            span: entry_span,
        },
    })
}

/// Load an ordinary source file together with its imported metadata.
///
/// # Errors
///
/// Returns [`EngineError`] if the source file cannot be read, an import
/// cannot be resolved, or an imported module cannot be parsed into the
/// supported type/callable subset.
#[allow(clippy::too_many_lines)]
pub fn load_ordinary_file(path: &Path) -> Result<LoadedOrdinaryFile, EngineError> {
    let source = std::fs::read_to_string(path)?;
    load_ordinary_source(path, &source)
}

/// Load an ordinary source snapshot using `path` only as import and
/// module-identity context.
///
/// This is for admitted-artifact execution paths that have already read and
/// hashed the source bytes and must not read the entry file again before
/// execution.
///
/// # Errors
///
/// Returns [`EngineError`] if an import cannot be resolved, or an imported
/// module cannot be parsed into the supported type/callable subset.
#[allow(clippy::too_many_lines)]
pub fn load_ordinary_source(path: &Path, source: &str) -> Result<LoadedOrdinaryFile, EngineError> {
    let canonical_entry = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let entry_root = path.parent().ok_or_else(|| {
        EngineError::Configuration(format!("source path '{}' has no parent", path.display()))
    })?;

    if let Some(error) = ash_parser::reserved_callable_arrow_diagnostic(source) {
        return Err(EngineError::Parse(error.to_string()));
    }

    let mut module_cache = HashMap::new();
    let mut visiting = HashSet::new();
    visiting.insert(canonical_entry);

    let mut imports = Vec::new();
    let mut ordinary_source = String::with_capacity(source.len());
    let mut seen_non_import = false;

    let mut lines = source.split_inclusive('\n');
    while let Some(line_with_ending) = lines.next() {
        let line = line_with_ending.trim_end_matches(['\r', '\n']);
        let trimmed = line.trim();
        if !seen_non_import && is_skippable_prelude_line(trimmed) {
            ordinary_source.push_str(line_with_ending);
            continue;
        }

        if !seen_non_import && (trimmed.starts_with("use ") || trimmed.starts_with("pub use ")) {
            let mut snippet = line.to_string();
            let mut consumed_import_source = line_with_ending.to_string();
            while import_needs_more_lines(&snippet) {
                let Some(next_line_with_ending) = lines.next() else {
                    break;
                };
                let next_line = next_line_with_ending.trim_end_matches(['\r', '\n']);
                snippet.push('\n');
                snippet.push_str(next_line);
                consumed_import_source.push_str(next_line_with_ending);
            }
            imports.push(parse_ordinary_import(snippet.trim())?);
            ordinary_source.push_str(&mask_import_source(&consumed_import_source));
            continue;
        }

        seen_non_import = true;
        ordinary_source.push_str(line_with_ending);
    }

    let mut imported_type_defs = Vec::new();
    let mut imported_type_names = HashSet::new();
    let mut imported_semantic_summaries = Vec::new();
    let mut imported_summary_keys = HashSet::new();
    let mut imported_type_function_heads = Vec::new();
    let mut imported_callables = HashMap::new();
    let mut imported_macro_summaries = Vec::new();

    let crate_root = discover_crate_root(entry_root);
    for import in imports {
        let (module_segments, search_roots) =
            import_resolution_roots(&import.module_segments, entry_root, crate_root.as_deref())?;
        let module_path =
            resolve_module_path(&module_segments, &search_roots)?.ok_or_else(|| {
                EngineError::Parse(format!(
                    "module '{}' not found",
                    import.module_segments.join("::")
                ))
            })?;
        let exports = collect_module_exports(&module_path, &mut module_cache, &mut visiting)?;

        for selection in import.selections {
            match selection {
                ImportSelection::Glob => {
                    for (name, type_def) in &exports.type_defs {
                        let imported_type = type_def_with_visible_name(type_def, name);
                        if imported_type_names.insert(imported_type.name.clone()) {
                            imported_type_defs.push(imported_type);
                        }
                    }
                    if let Some(summary) = exports.semantic_summary.as_ref() {
                        for row in &summary.exported_effect_rows {
                            push_selected_effect_row_semantic_summary(
                                &mut imported_semantic_summaries,
                                &mut imported_summary_keys,
                                Some(summary),
                                &row.exported_name,
                                &row.exported_name,
                                ash_core::semantic_summary::EffectRowBindingExposure::GlobImport,
                            )?;
                        }
                        // Keep non-row summary facts on the existing glob path.
                        let mut non_row_summary = summary.clone();
                        non_row_summary.exported_effect_rows.clear();
                        let key = imported_summary_key(&non_row_summary);
                        if imported_summary_keys.insert(key) {
                            imported_semantic_summaries.push(non_row_summary);
                        }
                    }
                    let mut type_function_exports =
                        exports.type_function_summaries.iter().collect::<Vec<_>>();
                    type_function_exports.sort_by_key(|(name, _)| *name);
                    for (name, summary) in type_function_exports {
                        push_imported_type_function_head(
                            &mut imported_type_function_heads,
                            name,
                            &summary.head,
                        );
                    }
                    for (k, mut v) in exports.callables.clone() {
                        if let CallableKind::Builtin { ref mut module } = v.kind
                            && module.is_empty()
                        {
                            *module = module_segments.join("::");
                        }
                        imported_callables.insert(k, v);
                    }
                    imported_macro_summaries.extend(exports.macro_summaries.values().cloned());
                }
                ImportSelection::Named { name, alias } => {
                    let exported_name = alias.as_ref().map_or_else(|| name.clone(), Clone::clone);
                    let selected_effect_row = push_selected_effect_row_semantic_summary(
                        &mut imported_semantic_summaries,
                        &mut imported_summary_keys,
                        exports.semantic_summary.as_ref(),
                        &name,
                        &exported_name,
                        ash_core::semantic_summary::EffectRowBindingExposure::NamedImport,
                    )?;
                    let selected_value = push_selected_value_semantic_summary(
                        &mut imported_semantic_summaries,
                        &mut imported_summary_keys,
                        exports.semantic_summary.as_ref(),
                        &name,
                        &exported_name,
                    );
                    if selected_effect_row || selected_value {
                        continue;
                    }
                    if let Some(type_def) = exports.type_defs.get(&name) {
                        let imported_type = selected_type_def_with_import_visibility(
                            type_def,
                            exports.semantic_summary.as_ref(),
                            &name,
                            &exported_name,
                        );
                        if imported_type_names.insert(imported_type.name.clone()) {
                            imported_type_defs.push(imported_type);
                        }
                        push_selected_semantic_summary(
                            &mut imported_semantic_summaries,
                            &mut imported_summary_keys,
                            exports.semantic_summary.as_ref(),
                            &name,
                            &exported_name,
                        );
                    } else if let Some(type_def) = exports.constructor_defs.get(&name) {
                        if alias.is_some() {
                            return Err(constructor_alias_error(&name));
                        }
                        let imported_type = selected_type_def_with_import_visibility(
                            type_def,
                            exports.semantic_summary.as_ref(),
                            &type_def.name,
                            &type_def.name,
                        );
                        if imported_type_names.insert(imported_type.name.clone()) {
                            imported_type_defs.push(imported_type);
                        }
                        push_selected_constructor_semantic_summary(
                            &mut imported_semantic_summaries,
                            &mut imported_summary_keys,
                            exports.semantic_summary.as_ref(),
                            &name,
                        );
                    } else if let Some(callable) = exports.callables.get(&name) {
                        push_signature_semantic_summaries(
                            &mut imported_semantic_summaries,
                            &mut imported_summary_keys,
                            exports.semantic_summary.as_ref(),
                            callable,
                        );
                        let mut callable = callable.clone();
                        callable.exported_name.clone_from(&exported_name);
                        if let CallableKind::Builtin { ref mut module } = callable.kind
                            && module.is_empty()
                        {
                            *module = module_segments.join("::");
                        }
                        imported_callables.insert(exported_name, callable);
                    } else if let Some(summary) = exports.macro_summaries.get(&name) {
                        let mut summary = summary.clone();
                        summary.name = exported_name.clone().into();
                        imported_macro_summaries.push(summary);
                    } else if let Some(summary) = exports.type_function_summaries.get(&name) {
                        push_selected_type_function_semantic_summary(
                            &mut imported_semantic_summaries,
                            &mut imported_summary_keys,
                            exports.semantic_summary.as_ref(),
                            &name,
                            &exported_name,
                        );
                        push_imported_type_function_head(
                            &mut imported_type_function_heads,
                            &exported_name,
                            &summary.head,
                        );
                    } else if exports.associated_family_summaries.contains_key(&name) {
                        push_selected_associated_family_semantic_summary(
                            &mut imported_semantic_summaries,
                            &mut imported_summary_keys,
                            exports.semantic_summary.as_ref(),
                            &name,
                            &exported_name,
                        );
                    } else if semantic_summary_has_interface(
                        exports.semantic_summary.as_ref(),
                        &name,
                    ) {
                        push_selected_interface_semantic_summary(
                            &mut imported_semantic_summaries,
                            &mut imported_summary_keys,
                            exports.semantic_summary.as_ref(),
                            &name,
                            &exported_name,
                        );
                    } else {
                        return Err(EngineError::Parse(format!(
                            "item '{name}' not found in module '{}'",
                            import.module_segments.join("::")
                        )));
                    }
                }
            }
        }
    }

    Ok(LoadedOrdinaryFile {
        ordinary_source,
        imported_type_defs,
        imported_semantic_summaries,
        imported_type_function_heads,
        imported_callables,
        imported_macro_summaries,
    })
}

/// Replace an import prelude with parse-neutral whitespace while retaining
/// original byte positions and line endings for later source-sidecar spans.
fn mask_import_source(source: &str) -> String {
    let mut masked = String::with_capacity(source.len());
    for character in source.chars() {
        if matches!(character, '\r' | '\n') {
            masked.push(character);
        } else {
            masked.extend(std::iter::repeat_n(' ', character.len_utf8()));
        }
    }
    masked
}

fn push_imported_type_function_head(
    heads: &mut Vec<(String, TypeComputationHeadId)>,
    visible_name: &str,
    head: &TypeComputationHeadId,
) {
    if !heads.iter().any(|(existing_name, existing_head)| {
        existing_name == visible_name && existing_head == head
    }) {
        heads.push((visible_name.to_string(), head.clone()));
    }
}

type ImportedSummaryKey = Vec<String>;

/// Create a type def with the given visible name.
#[must_use]
pub fn type_def_with_visible_name(type_def: &CoreTypeDef, visible_name: &str) -> CoreTypeDef {
    let alias_map = HashMap::from([(type_def.name.clone(), visible_name.to_string())]);
    type_def_with_visible_name_and_aliases(type_def, visible_name, &alias_map)
}

/// Create a type def with import visibility.
#[must_use]
pub fn selected_type_def_with_import_visibility(
    type_def: &CoreTypeDef,
    summary: Option<&ModuleSemanticSummary>,
    source_name: &str,
    imported_name: &str,
) -> CoreTypeDef {
    let mut alias_map = HashMap::from([(source_name.to_string(), imported_name.to_string())]);
    if let Some(summary) = summary {
        alias_map.extend(representation_dependency_metadata_aliases(
            summary,
            source_name,
            &alias_map,
        ));
    }
    type_def_with_visible_name_and_aliases(type_def, imported_name, &alias_map)
}

fn representation_dependency_metadata_aliases(
    summary: &ModuleSemanticSummary,
    selected_name: &str,
    visible_aliases: &HashMap<String, String>,
) -> HashMap<String, String> {
    let Some(selected) = summary
        .exported_types
        .iter()
        .find(|ty| ty.exported_name == selected_name)
    else {
        return HashMap::new();
    };
    let dependencies = transitive_representation_dependency_summaries(summary, selected);
    dependencies
        .into_iter()
        .map(|dependency| {
            let visible_name = visible_aliases
                .get(&dependency.exported_name)
                .cloned()
                .unwrap_or_else(|| dependency.exported_name.clone());
            (
                dependency.exported_name.clone(),
                dependency_metadata_name(&visible_name),
            )
        })
        .collect()
}

fn dependency_metadata_name(visible_name: &str) -> String {
    const DEPENDENCY_METADATA_PREFIX: &str = "$ash_dependency$";
    if visible_name.starts_with(DEPENDENCY_METADATA_PREFIX) {
        visible_name.to_string()
    } else {
        format!("{DEPENDENCY_METADATA_PREFIX}{visible_name}")
    }
}

fn is_dependency_metadata_name(visible_name: &str) -> bool {
    visible_name.starts_with("$ash_dependency$")
}

fn transitive_representation_dependency_summaries<'a>(
    summary: &'a ModuleSemanticSummary,
    selected: &TypeDeclSummary,
) -> Vec<&'a TypeDeclSummary> {
    let mut dependencies = Vec::new();
    let mut included_ids = HashSet::from([selected.id.clone()]);
    let mut pending = representation_dependency_names(selected);

    while let Some(name) = pending.pop() {
        let Some(dependency) = summary
            .exported_types
            .iter()
            .find(|ty| ty.exported_name == name)
        else {
            continue;
        };
        if !included_ids.insert(dependency.id.clone()) {
            continue;
        }
        pending.extend(representation_dependency_names(dependency));
        dependencies.push(dependency);
    }

    dependencies
}

/// Create a type def with visible name and aliases.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn type_def_with_visible_name_and_aliases(
    type_def: &CoreTypeDef,
    visible_name: &str,
    alias_map: &HashMap<String, String>,
) -> CoreTypeDef {
    let mut renamed = type_def.clone();
    rewrite_core_type_body_aliases(&mut renamed.body, alias_map);
    renamed.name = visible_name.to_string();
    renamed
}

fn rewrite_core_type_body_aliases(body: &mut CoreTypeBody, alias_map: &HashMap<String, String>) {
    match body {
        CoreTypeBody::Struct(fields) => {
            for (_, field_ty) in fields {
                rewrite_core_type_expr_aliases(field_ty, alias_map);
            }
        }
        CoreTypeBody::Enum(variants) => {
            for variant in variants {
                for (_, field_ty) in &mut variant.fields {
                    rewrite_core_type_expr_aliases(field_ty, alias_map);
                }
                rewrite_variant_payload_aliases(&mut variant.payload, alias_map);
            }
        }
        CoreTypeBody::Alias(target) => {
            rewrite_core_type_expr_aliases(target, alias_map);
        }
    }
}

fn rewrite_variant_payload_aliases(
    payload: &mut CoreVariantPayload,
    alias_map: &HashMap<String, String>,
) {
    match payload {
        CoreVariantPayload::Unit => {}
        CoreVariantPayload::Record(fields) => {
            for (_, field_ty) in fields {
                rewrite_core_type_expr_aliases(field_ty, alias_map);
            }
        }
        CoreVariantPayload::Tuple(items) => {
            for item in items {
                rewrite_core_type_expr_aliases(item, alias_map);
            }
        }
    }
}

fn rewrite_type_representation_aliases(
    representation: &mut TypeRepresentationSummary,
    alias_map: &HashMap<String, String>,
) {
    if let TypeRepresentationSummary::Exposed(body) = representation {
        rewrite_core_type_body_aliases(body, alias_map);
    }
}

fn rewrite_core_type_expr_aliases(expr: &mut CoreTypeExpr, alias_map: &HashMap<String, String>) {
    match expr {
        CoreTypeExpr::Named(name) => {
            if let Some(alias) = alias_map.get(name) {
                *name = alias.clone();
            }
        }
        CoreTypeExpr::Constructor { name, args } => {
            if let Some(alias) = alias_map.get(name) {
                *name = alias.clone();
            }
            for arg in args {
                rewrite_core_type_expr_aliases(arg, alias_map);
            }
        }
        CoreTypeExpr::Tuple(items) => {
            for item in items {
                rewrite_core_type_expr_aliases(item, alias_map);
            }
        }
        CoreTypeExpr::Record(fields) => {
            for (_, field_ty) in fields {
                rewrite_core_type_expr_aliases(field_ty, alias_map);
            }
        }
        CoreTypeExpr::Associated { base, .. } => {
            rewrite_core_type_expr_aliases(base, alias_map);
        }
    }
}

fn constructor_alias_error(name: &str) -> EngineError {
    EngineError::Parse(format!(
        "constructor alias for '{name}' is not supported; import or re-export the constructor by its original name"
    ))
}

fn push_selected_semantic_summary(
    imported_semantic_summaries: &mut Vec<ModuleSemanticSummary>,
    imported_summary_keys: &mut HashSet<ImportedSummaryKey>,
    summary: Option<&ModuleSemanticSummary>,
    source_name: &str,
    imported_name: &str,
) {
    let Some(summary) = summary else {
        return;
    };
    let Some(selected) = selected_import_type_semantic_summary(summary, source_name, imported_name)
    else {
        return;
    };
    merge_or_push_imported_semantic_summary(
        imported_semantic_summaries,
        imported_summary_keys,
        selected,
    );
}

fn merge_or_push_imported_semantic_summary(
    imported_semantic_summaries: &mut Vec<ModuleSemanticSummary>,
    imported_summary_keys: &mut HashSet<ImportedSummaryKey>,
    selected: ModuleSemanticSummary,
) {
    if let Some(existing) = imported_semantic_summaries
        .iter_mut()
        .find(|existing| imported_summary_type_set_matches(existing, &selected))
    {
        merge_imported_summary_payloads(existing, selected);
        imported_summary_keys.insert(imported_summary_key(existing));
        return;
    }

    if imported_type_functions_already_present(imported_semantic_summaries, &selected)
        || imported_associated_families_already_present(imported_semantic_summaries, &selected)
    {
        return;
    }

    let key = imported_summary_key(&selected);
    if imported_summary_keys.insert(key) {
        imported_semantic_summaries.push(selected);
    }
}

fn merge_imported_summary_payloads(
    existing: &mut ModuleSemanticSummary,
    selected: ModuleSemanticSummary,
) {
    for row in selected.exported_effect_rows {
        let exists = existing
            .exported_effect_rows
            .iter()
            .any(|existing| existing.id == row.id && existing.exported_name == row.exported_name);
        if !exists {
            existing.exported_effect_rows.push(row);
        }
    }
    for value in selected.exported_values {
        let exists = existing.exported_values.iter().any(|existing| {
            existing.exported_name == value.exported_name && existing.kind == value.kind
        });
        if !exists {
            existing.exported_values.push(value);
        }
    }
    for constructor in selected.exported_constructors {
        let exists = existing.exported_constructors.iter().any(|existing| {
            existing.id == constructor.id && existing.exported_name == constructor.exported_name
        });
        if !exists {
            existing.exported_constructors.push(constructor);
        }
    }
    for domain in selected.exported_sealed_domains {
        let exists = existing.exported_sealed_domains.iter().any(|existing| {
            existing.id == domain.id && existing.exported_name == domain.exported_name
        });
        if !exists {
            existing.exported_sealed_domains.push(domain);
        }
    }
    for data_kind in selected.exported_promoted_data_kinds {
        if !existing
            .exported_promoted_data_kinds
            .iter()
            .any(|existing| existing.id == data_kind.id)
        {
            existing.exported_promoted_data_kinds.push(data_kind);
        }
    }
    for type_function in selected.exported_type_functions {
        if !existing_has_type_function_summary(existing, &type_function) {
            existing.exported_type_functions.push(type_function);
        }
    }
    for family in selected.exported_associated_families {
        if !existing
            .exported_associated_families
            .iter()
            .any(|existing| existing.head == family.head)
        {
            existing.exported_associated_families.push(family);
        }
    }
    for predicate in selected.exported_proposition_predicates {
        if !existing
            .exported_proposition_predicates
            .iter()
            .any(|existing| existing.id == predicate.id)
        {
            existing.exported_proposition_predicates.push(predicate);
        }
    }
    for fact in selected.exported_proposition_facts {
        if !existing
            .exported_proposition_facts
            .iter()
            .any(|existing| existing == &fact)
        {
            existing.exported_proposition_facts.push(fact);
        }
    }
    for identity in selected.interface_identities {
        if !existing
            .interface_identities
            .iter()
            .any(|existing| existing.id == identity.id)
        {
            existing.interface_identities.push(identity);
        }
    }
    for identity in selected.associated_member_identities {
        if !existing
            .associated_member_identities
            .iter()
            .any(|existing| existing.id == identity.id)
        {
            existing.associated_member_identities.push(identity);
        }
    }
}

fn existing_has_type_function_summary(
    summary: &ModuleSemanticSummary,
    selected: &TypeFunctionSummary,
) -> bool {
    summary.exported_type_functions.iter().any(|existing| {
        existing.head == selected.head
            && (existing.exported_name == selected.exported_name
                || is_dependency_metadata_name(&existing.exported_name)
                || is_dependency_metadata_name(&selected.exported_name))
    })
}

fn imported_type_functions_already_present(
    imported_semantic_summaries: &[ModuleSemanticSummary],
    selected: &ModuleSemanticSummary,
) -> bool {
    !selected.exported_type_functions.is_empty()
        && selected.exported_type_functions.iter().all(|selected| {
            imported_semantic_summaries
                .iter()
                .any(|summary| existing_has_type_function_summary(summary, selected))
        })
}

fn imported_associated_families_already_present(
    imported_semantic_summaries: &[ModuleSemanticSummary],
    selected: &ModuleSemanticSummary,
) -> bool {
    !selected.exported_associated_families.is_empty()
        && selected
            .exported_associated_families
            .iter()
            .all(|selected| {
                imported_semantic_summaries.iter().any(|summary| {
                    summary
                        .exported_associated_families
                        .iter()
                        .any(|existing| existing.head == selected.head && existing == selected)
                })
            })
}

fn imported_summary_type_set_matches(
    left: &ModuleSemanticSummary,
    right: &ModuleSemanticSummary,
) -> bool {
    if left.module != right.module || left.version != right.version {
        return false;
    }
    if left.imported_summary_refs != right.imported_summary_refs {
        return false;
    }
    if !left.exported_type_functions.is_empty()
        || !right.exported_type_functions.is_empty()
        || !left.exported_associated_families.is_empty()
        || !right.exported_associated_families.is_empty()
        || !left.exported_proposition_predicates.is_empty()
        || !right.exported_proposition_predicates.is_empty()
        || !left.exported_proposition_facts.is_empty()
        || !right.exported_proposition_facts.is_empty()
        || !left.exported_promoted_data_kinds.is_empty()
        || !right.exported_promoted_data_kinds.is_empty()
    {
        return selected_summary_identity_facts_are_compatible(left, right);
    }
    if left.exported_sealed_domains != right.exported_sealed_domains {
        return false;
    }
    let mut left_types = left
        .exported_types
        .iter()
        .map(|ty| (ty.id.clone(), ty.exported_name.clone()))
        .collect::<Vec<_>>();
    let mut right_types = right
        .exported_types
        .iter()
        .map(|ty| (ty.id.clone(), ty.exported_name.clone()))
        .collect::<Vec<_>>();
    left_types.sort_unstable_by_key(|item| format!("{item:?}"));
    right_types.sort_unstable_by_key(|item| format!("{item:?}"));
    left_types == right_types
}

fn selected_summary_identity_facts_are_compatible(
    left: &ModuleSemanticSummary,
    right: &ModuleSemanticSummary,
) -> bool {
    left.exported_type_functions.len() == right.exported_type_functions.len()
        && left.exported_associated_families.len() == right.exported_associated_families.len()
        && left
            .exported_proposition_predicates
            .iter()
            .all(|left_predicate| {
                right
                    .exported_proposition_predicates
                    .iter()
                    .find(|right_predicate| right_predicate.id == left_predicate.id)
                    .is_none_or(|right_predicate| right_predicate == left_predicate)
            })
        && left.exported_proposition_facts.iter().all(|left_fact| {
            right
                .exported_proposition_facts
                .iter()
                .any(|right_fact| right_fact == left_fact)
        })
        && left
            .exported_type_functions
            .iter()
            .all(|left_type_function| {
                right
                    .exported_type_functions
                    .iter()
                    .find(|right_type_function| right_type_function.head == left_type_function.head)
                    .is_some_and(|right_type_function| right_type_function == left_type_function)
            })
        && left.exported_associated_families.iter().all(|left_family| {
            right
                .exported_associated_families
                .iter()
                .find(|right_family| right_family.head == left_family.head)
                .is_some_and(|right_family| right_family == left_family)
        })
        && left.exported_sealed_domains.iter().all(|left_domain| {
            right
                .exported_sealed_domains
                .iter()
                .find(|right_domain| right_domain.id == left_domain.id)
                .is_none_or(|right_domain| right_domain == left_domain)
        })
        && left
            .exported_promoted_data_kinds
            .iter()
            .all(|left_data_kind| {
                right
                    .exported_promoted_data_kinds
                    .iter()
                    .find(|right_data_kind| right_data_kind.id == left_data_kind.id)
                    .is_none_or(|right_data_kind| right_data_kind == left_data_kind)
            })
        && left.exported_types.iter().all(|left_type| {
            right
                .exported_types
                .iter()
                .find(|right_type| right_type.id == left_type.id)
                .is_none_or(|right_type| right_type == left_type)
        })
        && left.exported_constructors.iter().all(|left_constructor| {
            right
                .exported_constructors
                .iter()
                .find(|right_constructor| right_constructor.id == left_constructor.id)
                .is_none_or(|right_constructor| right_constructor == left_constructor)
        })
        && left.interface_identities.iter().all(|left_interface| {
            right
                .interface_identities
                .iter()
                .find(|right_interface| right_interface.id == left_interface.id)
                .is_none_or(|right_interface| right_interface == left_interface)
        })
        && left.associated_member_identities.iter().all(|left_member| {
            right
                .associated_member_identities
                .iter()
                .find(|right_member| right_member.id == left_member.id)
                .is_none_or(|right_member| right_member == left_member)
        })
}

fn push_selected_constructor_semantic_summary(
    imported_semantic_summaries: &mut Vec<ModuleSemanticSummary>,
    imported_summary_keys: &mut HashSet<ImportedSummaryKey>,
    summary: Option<&ModuleSemanticSummary>,
    constructor_name: &str,
) {
    let Some(summary) = summary else {
        return;
    };
    let Some(selected) = selected_import_constructor_semantic_summary(summary, constructor_name)
    else {
        return;
    };
    merge_or_push_imported_semantic_summary(
        imported_semantic_summaries,
        imported_summary_keys,
        selected,
    );
}

fn push_selected_effect_row_semantic_summary(
    imported_semantic_summaries: &mut Vec<ModuleSemanticSummary>,
    imported_summary_keys: &mut HashSet<ImportedSummaryKey>,
    summary: Option<&ModuleSemanticSummary>,
    row_name: &str,
    imported_name: &str,
    exposure: ash_core::semantic_summary::EffectRowBindingExposure,
) -> Result<bool, EngineError> {
    let Some(summary) = summary else {
        return Ok(false);
    };
    let Some(mut selected) =
        sanitized_effect_row_semantic_summary(summary, row_name, imported_name, exposure)?
    else {
        return Ok(false);
    };
    validate_imported_effect_row_visible_bindings(imported_semantic_summaries, &selected)?;
    retain_unpublished_effect_row_visible_bindings(imported_semantic_summaries, &mut selected);
    merge_or_push_imported_semantic_summary(
        imported_semantic_summaries,
        imported_summary_keys,
        selected,
    );
    Ok(true)
}

/// Drop compatible bindings that an earlier import already published.  The
/// surrounding summaries may have distinct facade identities, but those are
/// not part of the caller-visible provider/binding contract.
fn retain_unpublished_effect_row_visible_bindings(
    imported_semantic_summaries: &[ModuleSemanticSummary],
    selected: &mut ModuleSemanticSummary,
) {
    selected.exported_effect_rows.retain(|incoming| {
        !imported_semantic_summaries
            .iter()
            .flat_map(|summary| &summary.exported_effect_rows)
            .any(|existing| {
                existing.binding.visible_name == incoming.binding.visible_name
                    && effect_row_visible_bindings_are_compatible(existing, incoming)
            })
    });
}

/// Reject a caller-visible effect-row binding before it can be published into
/// the import summary collection.  Provider identity is immutable, while the
/// visible name is a caller binding; accepting two providers for one name
/// would make ordinary import order observable.
fn validate_imported_effect_row_visible_bindings(
    imported_semantic_summaries: &[ModuleSemanticSummary],
    selected: &ModuleSemanticSummary,
) -> Result<(), EngineError> {
    // A single selected closure can contain both the selected provider and a
    // dependency under the same caller-visible name (for example importing
    // `Audit as Dependency` when `Audit = { group Dependency }`).  Reject it
    // before either row can enter the caller summary or cache.
    validate_effect_row_visible_binding_contracts(&[], &selected.exported_effect_rows)?;
    for incoming in &selected.exported_effect_rows {
        for existing in imported_semantic_summaries
            .iter()
            .flat_map(|summary| &summary.exported_effect_rows)
        {
            if existing.binding.visible_name == incoming.binding.visible_name
                && !effect_row_visible_bindings_are_compatible(existing, incoming)
            {
                // Do not include either provider identity or closure contents:
                // the classification must be stable when the import order is
                // reversed and must not disclose imported implementation data.
                return Err(EngineError::Parse(
                    "import-order-conflict: effect-row visible binding has conflicting provider or sanitized closure"
                        .to_string(),
                ));
            }
        }
    }
    Ok(())
}

/// Compare the public binding contract without treating the enclosing facade
/// summary identity as provider identity.  A facade rehomes `id.module`, but
/// that must not turn an otherwise identical provider binding into a conflict.
fn effect_row_visible_bindings_are_compatible(
    left: &ash_core::semantic_summary::EffectRowExportSummary,
    right: &ash_core::semantic_summary::EffectRowExportSummary,
) -> bool {
    left.binding.visible_name == right.binding.visible_name
        && left.exported_name == right.exported_name
        && left.provider == right.provider
        && left.binding.provider == right.binding.provider
        && left.binding.exposure == right.binding.exposure
        && left.binding.closure_status == right.binding.closure_status
        && left.visibility == right.visibility
        && left.classification == right.classification
        && left.authority == right.authority
        && left.row_items == right.row_items
        && left.closure_metadata == right.closure_metadata
}

/// Validate existing and incoming visible bindings, including collisions
/// internal to a single sanitized closure, before publication occurs.
fn validate_effect_row_visible_binding_contracts(
    existing_rows: &[ash_core::semantic_summary::EffectRowExportSummary],
    incoming_rows: &[ash_core::semantic_summary::EffectRowExportSummary],
) -> Result<(), EngineError> {
    for (index, incoming) in incoming_rows.iter().enumerate() {
        let conflicts = existing_rows
            .iter()
            .chain(incoming_rows[..index].iter())
            .any(|existing| {
                existing.binding.visible_name == incoming.binding.visible_name
                    && !effect_row_visible_bindings_are_compatible(existing, incoming)
            });
        if conflicts {
            return Err(EngineError::Parse(
                "import-order-conflict: effect-row visible binding has conflicting provider or sanitized closure"
                    .to_string(),
            ));
        }
    }
    Ok(())
}

fn push_selected_value_semantic_summary(
    imported_semantic_summaries: &mut Vec<ModuleSemanticSummary>,
    imported_summary_keys: &mut HashSet<ImportedSummaryKey>,
    summary: Option<&ModuleSemanticSummary>,
    value_name: &str,
    imported_name: &str,
) -> bool {
    let Some(summary) = summary else {
        return false;
    };
    let Some(selected) = selected_value_semantic_summary(summary, value_name, imported_name) else {
        return false;
    };
    merge_or_push_imported_semantic_summary(
        imported_semantic_summaries,
        imported_summary_keys,
        selected,
    );
    true
}

fn push_selected_type_function_semantic_summary(
    imported_semantic_summaries: &mut Vec<ModuleSemanticSummary>,
    imported_summary_keys: &mut HashSet<ImportedSummaryKey>,
    summary: Option<&ModuleSemanticSummary>,
    type_function_name: &str,
    imported_name: &str,
) {
    let Some(summary) = summary else {
        return;
    };
    let Some(selected) =
        selected_type_function_semantic_summary(summary, type_function_name, imported_name)
    else {
        return;
    };
    merge_or_push_imported_semantic_summary(
        imported_semantic_summaries,
        imported_summary_keys,
        selected,
    );
}

fn push_selected_associated_family_semantic_summary(
    imported_semantic_summaries: &mut Vec<ModuleSemanticSummary>,
    imported_summary_keys: &mut HashSet<ImportedSummaryKey>,
    summary: Option<&ModuleSemanticSummary>,
    family_name: &str,
    imported_name: &str,
) {
    let Some(summary) = summary else {
        return;
    };
    let Some(selected) =
        selected_associated_family_semantic_summary(summary, family_name, imported_name)
    else {
        return;
    };
    merge_or_push_imported_semantic_summary(
        imported_semantic_summaries,
        imported_summary_keys,
        selected,
    );
}

fn push_selected_interface_semantic_summary(
    imported_semantic_summaries: &mut Vec<ModuleSemanticSummary>,
    imported_summary_keys: &mut HashSet<ImportedSummaryKey>,
    summary: Option<&ModuleSemanticSummary>,
    interface_name: &str,
    imported_name: &str,
) {
    let Some(summary) = summary else {
        return;
    };
    let Some(selected) =
        selected_interface_semantic_summary(summary, interface_name, imported_name)
    else {
        return;
    };
    merge_or_push_imported_semantic_summary(
        imported_semantic_summaries,
        imported_summary_keys,
        selected,
    );
}

fn push_signature_semantic_summaries(
    imported_semantic_summaries: &mut Vec<ModuleSemanticSummary>,
    imported_summary_keys: &mut HashSet<ImportedSummaryKey>,
    summary: Option<&ModuleSemanticSummary>,
    callable: &InlineCallable,
) {
    if callable_signature_has_proposition_requirements(callable)
        && let Some(summary) = selected_proposition_semantic_summary(summary)
    {
        merge_or_push_imported_semantic_summary(
            imported_semantic_summaries,
            imported_summary_keys,
            summary,
        );
    }
    let mut names = callable_signature_type_names(callable);
    names.sort_unstable();
    names.dedup();
    for name in names {
        push_selected_semantic_summary(
            imported_semantic_summaries,
            imported_summary_keys,
            summary,
            &name,
            &name,
        );
    }
}

const fn callable_signature_has_proposition_requirements(callable: &InlineCallable) -> bool {
    match callable.signature.as_ref() {
        Some(CallableSignature::Function(function)) => function.proposition_tail.is_some(),
        Some(CallableSignature::Builtin(builtin)) => builtin.proposition_tail.is_some(),
        None => false,
    }
}

fn selected_proposition_semantic_summary(
    summary: Option<&ModuleSemanticSummary>,
) -> Option<ModuleSemanticSummary> {
    let summary = summary?;
    if summary.exported_proposition_predicates.is_empty()
        && summary.exported_proposition_facts.is_empty()
    {
        return None;
    }
    let mut selected = ModuleSemanticSummary::new(summary.module.clone());
    selected.version = summary.version;
    selected.exported_types.clone_from(&summary.exported_types);
    selected
        .exported_constructors
        .clone_from(&summary.exported_constructors);
    selected
        .exported_sealed_domains
        .clone_from(&summary.exported_sealed_domains);
    let mut dependencies = proposition_summary_dependencies(summary);
    expand_promoted_data_kind_dependency_closure(summary, &mut dependencies);
    selected.exported_promoted_data_kinds =
        hidden_promoted_data_kind_dependencies(summary, &dependencies.promoted_data_kinds);
    selected
        .interface_identities
        .clone_from(&summary.interface_identities);
    selected
        .associated_member_identities
        .clone_from(&summary.associated_member_identities);
    selected
        .exported_type_functions
        .clone_from(&summary.exported_type_functions);
    selected
        .exported_associated_families
        .clone_from(&summary.exported_associated_families);
    selected
        .exported_proposition_predicates
        .clone_from(&summary.exported_proposition_predicates);
    selected
        .exported_proposition_facts
        .clone_from(&summary.exported_proposition_facts);
    Some(selected)
}

fn callable_signature_type_names(callable: &InlineCallable) -> Vec<String> {
    let mut names = Vec::new();
    match callable.signature.as_ref() {
        Some(CallableSignature::Function(function)) => {
            for param in &function.params {
                collect_surface_type_names(&param.ty, &mut names);
            }
            if let Some(return_type) = function.return_type.as_ref() {
                collect_surface_type_names(return_type, &mut names);
            }
            names.retain(|name| {
                !function
                    .type_params
                    .iter()
                    .any(|param| param.as_ref() == name)
            });
        }
        Some(CallableSignature::Builtin(builtin)) => {
            for param in &builtin.params {
                collect_surface_type_names(&param.ty, &mut names);
            }
            collect_surface_type_names(&builtin.return_type, &mut names);
            names.retain(|name| {
                !builtin
                    .type_params
                    .iter()
                    .any(|param| param.as_ref() == name)
            });
        }
        None => {}
    }
    names
}

fn collect_surface_type_names(ty: &Type, names: &mut Vec<String>) {
    match ty {
        Type::Name(name) => names.push(name.to_string()),
        Type::Hole { .. } | Type::Capability(_) => {}
        Type::List(inner) | Type::Associated { base: inner, .. } => {
            collect_surface_type_names(inner, names);
        }
        Type::Tuple(items) => {
            for item in items {
                collect_surface_type_names(item, names);
            }
        }
        Type::Record(fields) => {
            for (_, field_ty) in fields {
                collect_surface_type_names(field_ty, names);
            }
        }
        Type::Constructor { name, args } => {
            names.push(name.to_string());
            for arg in args {
                collect_surface_type_names(arg, names);
            }
        }
        Type::AssociatedFamilyProjection {
            interface, args, ..
        } => {
            names.push(interface.to_string());
            for arg in args {
                collect_surface_type_names(arg, names);
            }
        }
        Type::Fn(params, _row, ret) => {
            for param in params {
                collect_surface_type_names(param, names);
            }
            collect_surface_type_names(ret, names);
        }
    }
}

fn rewrite_callable_signature_aliases(
    callable: &mut InlineCallable,
    source_summary: Option<&ModuleSemanticSummary>,
    exported_summary: Option<&ModuleSemanticSummary>,
) {
    let Some(source_summary) = source_summary else {
        return;
    };
    let Some(exported_summary) = exported_summary else {
        return;
    };

    let aliases = source_summary
        .exported_types
        .iter()
        .filter_map(|source_ty| {
            let exported_ty = exported_summary
                .exported_types
                .iter()
                .find(|exported_ty| exported_ty.id == source_ty.id)?;
            if source_ty.exported_name == exported_ty.exported_name {
                return None;
            }
            Some((
                source_ty.exported_name.clone(),
                exported_ty.exported_name.clone(),
            ))
        })
        .collect::<HashMap<_, _>>();
    if aliases.is_empty() {
        return;
    }

    match callable.signature.as_mut() {
        Some(CallableSignature::Function(function)) => {
            for param in &mut function.params {
                rewrite_surface_type_aliases(&mut param.ty, &aliases);
            }
            if let Some(return_type) = function.return_type.as_mut() {
                rewrite_surface_type_aliases(return_type, &aliases);
            }
        }
        Some(CallableSignature::Builtin(builtin)) => {
            for param in &mut builtin.params {
                rewrite_surface_type_aliases(&mut param.ty, &aliases);
            }
            rewrite_surface_type_aliases(&mut builtin.return_type, &aliases);
        }
        None => {}
    }
}

fn rewrite_exported_callable_signature_aliases(
    exports: &mut ModuleExports,
    source_summary: Option<&ModuleSemanticSummary>,
) {
    let Some(source_summary) = source_summary else {
        return;
    };
    let exported_summary = exports.semantic_summary.clone();
    for callable in exports
        .callables
        .values_mut()
        .filter(|callable| callable.exporting_modules.contains(&source_summary.module))
    {
        rewrite_callable_signature_aliases(
            callable,
            Some(source_summary),
            exported_summary.as_ref(),
        );
    }
}

fn stamp_callable_export_module(
    callable: &mut InlineCallable,
    summary: Option<&ModuleSemanticSummary>,
) {
    if let Some(summary) = summary {
        callable.exporting_modules.insert(summary.module.clone());
    }
}

fn stamp_builtin_callable_modules(exports: &mut ModuleExports, module_name: &str) {
    for callable in exports.callables.values_mut() {
        if let CallableKind::Builtin { module } = &mut callable.kind
            && module.is_empty()
        {
            *module = module_name.to_string();
        }
    }
}

fn rewrite_surface_type_aliases(ty: &mut Type, aliases: &HashMap<String, String>) {
    match ty {
        Type::Hole { .. } => {}
        Type::Name(name) | Type::Capability(name) => {
            if let Some(alias) = aliases.get(name.as_ref()) {
                *name = alias.as_str().into();
            }
        }
        Type::List(inner) | Type::Associated { base: inner, .. } => {
            rewrite_surface_type_aliases(inner, aliases);
        }
        Type::Tuple(items) => {
            for item in items {
                rewrite_surface_type_aliases(item, aliases);
            }
        }
        Type::Record(fields) => {
            for (_, field_ty) in fields {
                rewrite_surface_type_aliases(field_ty, aliases);
            }
        }
        Type::Constructor { name, args } => {
            if let Some(alias) = aliases.get(name.as_ref()) {
                *name = alias.as_str().into();
            }
            for arg in args {
                rewrite_surface_type_aliases(arg, aliases);
            }
        }
        Type::AssociatedFamilyProjection {
            interface, args, ..
        } => {
            if let Some(alias) = aliases.get(interface.as_ref()) {
                *interface = alias.as_str().into();
            }
            for arg in args {
                rewrite_surface_type_aliases(arg, aliases);
            }
        }
        Type::Fn(params, _row, ret) => {
            for param in params {
                rewrite_surface_type_aliases(param, aliases);
            }
            rewrite_surface_type_aliases(ret, aliases);
        }
    }
}

fn append_callable_signature_visibility_errors(
    callable: &InlineCallable,
    private_ordinary_types: &HashSet<String>,
    errors: &mut Vec<String>,
) {
    let mut leaked = callable_signature_type_names(callable)
        .into_iter()
        .filter(|name| private_ordinary_types.contains(name))
        .collect::<Vec<_>>();
    leaked.sort_unstable();
    leaked.dedup();
    for name in leaked {
        errors.push(format!(
            "public callable '{}' exposes private ordinary type '{}' in its signature",
            callable.exported_name, name
        ));
    }
}

pub(crate) fn public_callable_signature_resolution_errors(
    path: &Path,
    source: &str,
    type_defs: &[CoreTypeDef],
) -> Vec<String> {
    let module_root = path.parent().unwrap_or_else(|| Path::new("."));
    let crate_root = discover_crate_root(module_root);
    let import_info = collect_import_visibility_info(source, module_root, crate_root.as_deref());
    let mut known_types = builtin_public_signature_type_names();
    known_types.extend(type_defs.iter().map(|type_def| type_def.name.clone()));
    known_types.extend(import_info.known);
    known_types.extend(import_info.private);
    let metadata_source = strip_module_metadata_non_definition_lines(source);
    if let Ok(module) = parse_module_file_for_type_metadata(path, &metadata_source) {
        known_types.extend(local_public_interface_names(&module));
    }
    if let Ok(imported_interfaces) = directly_visible_imported_interface_names(path, source) {
        known_types.extend(imported_interfaces);
    }
    if let Ok(metadata) = collect_module_type_metadata_from_module_file(path, source) {
        known_types.extend(
            metadata
                .summary
                .interface_identities
                .iter()
                .map(|identity| identity.name.clone()),
        );
        let pub_use_exports =
            collect_public_import_visibility_exports(path, source, &metadata, &mut HashSet::new());
        known_types.extend(pub_use_exports.type_names);
    }

    let local_type_function_names = collect_module_type_metadata_from_module_file(path, source)
        .map_or_else(
            |_| local_type_function_names_from_source(source),
            |metadata| {
                metadata
                    .type_function_defs
                    .iter()
                    .map(|type_fn| type_fn.name.to_string())
                    .collect()
            },
        );
    let mut errors = Vec::new();
    for callable in public_callable_signatures(source) {
        append_callable_signature_type_function_leaks(
            &callable,
            &local_type_function_names,
            &mut errors,
        );
        let mut missing = callable_signature_type_names(&callable)
            .into_iter()
            .filter(|name| !known_types.contains(name) && !local_type_function_names.contains(name))
            .collect::<Vec<_>>();
        missing.sort_unstable();
        missing.dedup();
        for name in missing {
            if import_info.unresolved.contains(&name) {
                errors.push(format!(
                    "public callable '{}' references unresolved imported ordinary type '{}' in its signature",
                    callable.exported_name, name
                ));
            } else {
                let help = public_type_import_hint(module_root, crate_root.as_deref(), &name)
                    .map_or_else(String::new, |hint| format!(" Help: add `{hint}`."));
                errors.push(format!(
                    "public callable '{}' references unresolved ordinary type '{}' in its signature.{help}",
                    callable.exported_name, name
                ));
            }
        }
    }
    errors
}

fn public_type_import_hint(
    module_root: &Path,
    crate_root: Option<&Path>,
    name: &str,
) -> Option<String> {
    let mut roots = Vec::new();
    roots.push(module_root.to_path_buf());
    if let Some(crate_root) = crate_root
        && crate_root != module_root
    {
        roots.push(crate_root.to_path_buf());
    }

    for root in roots {
        let Ok(entries) = std::fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("ash") {
                continue;
            }
            let Ok(source) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(metadata) = collect_module_type_metadata_from_module_file(&path, &source) else {
                continue;
            };
            if metadata.type_defs.iter().any(|type_def| {
                type_def.name == name && matches!(type_def.visibility, CoreVisibility::Public)
            }) {
                let module = path.file_stem()?.to_str()?;
                return Some(format!("use {module}::{{{name}}}"));
            }
        }
    }
    None
}

fn append_callable_signature_type_function_leaks(
    callable: &InlineCallable,
    local_type_function_names: &TypeFunctionNameSet,
    errors: &mut Vec<String>,
) {
    let mut leaked = callable_signature_type_names(callable)
        .into_iter()
        .filter(|name| local_type_function_names.contains(name))
        .collect::<Vec<_>>();
    leaked.sort_unstable();
    leaked.dedup();
    for name in leaked {
        errors.push(format!(
            "public callable '{}' exposes local type function '{}' in its signature before SPEC-F",
            callable.exported_name, name
        ));
    }
}

fn public_callable_signatures(source: &str) -> Vec<InlineCallable> {
    let mut callables = Vec::new();
    for snippet in extract_braced_snippets(source, |trimmed| trimmed.starts_with("pub fn ")) {
        if let Ok(Some(callable)) = parse_supported_pub_fn_callable(&snippet) {
            callables.push(callable.callable);
        }
    }
    for snippet in
        extract_semicolon_snippets(source, |trimmed| trimmed.starts_with("pub builtin fn "))
    {
        if let Ok(Some(callable)) = parse_builtin_fn_callable(&snippet, String::new()) {
            callables.push(callable.callable);
        }
    }
    callables
}

fn builtin_public_signature_type_names() -> HashSet<String> {
    [
        "Int", "String", "Bool", "Float", "Null", "Unit", "Time", "Ref", "Record", "Bytes", "List",
        "Option", "Result", "Map", "Stream", "P", "Fn",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

pub(crate) fn public_representation_visibility_errors(
    path: &Path,
    source: &str,
    type_defs: &[CoreTypeDef],
) -> Vec<String> {
    let module_root = path.parent().unwrap_or_else(|| Path::new("."));
    let crate_root = discover_crate_root(module_root);
    let import_info = collect_import_visibility_info(source, module_root, crate_root.as_deref());

    let mut known_types = builtin_public_signature_type_names();
    known_types.extend(type_defs.iter().map(|type_def| type_def.name.clone()));
    known_types.extend(import_info.known);
    known_types.extend(import_info.private);
    let metadata_source = strip_module_metadata_non_definition_lines(source);
    if let Ok(module) = parse_module_file_for_type_metadata(path, &metadata_source) {
        known_types.extend(local_public_interface_names(&module));
    }
    if let Ok(imported_interfaces) = directly_visible_imported_interface_names(path, source) {
        known_types.extend(imported_interfaces);
    }
    if let Ok(metadata) = collect_module_type_metadata_from_module_file(path, source) {
        known_types.extend(
            metadata
                .summary
                .interface_identities
                .iter()
                .map(|identity| identity.name.clone()),
        );
        let pub_use_exports =
            collect_public_import_visibility_exports(path, source, &metadata, &mut HashSet::new());
        known_types.extend(pub_use_exports.type_names);
    }

    let private_ordinary_types = private_ordinary_type_names(type_defs);
    let mut errors = Vec::new();
    for type_def in type_defs
        .iter()
        .filter(|type_def| matches!(type_def.visibility, CoreVisibility::Public))
    {
        let mut leaked = Vec::new();
        collect_core_type_body_names(&type_def.body, &mut leaked);
        leaked.retain(|name| {
            private_ordinary_types.contains(name)
                && !type_def.params.iter().any(|param| param == name)
        });
        leaked.sort_unstable();
        leaked.dedup();
        for name in leaked {
            errors.push(format!(
                "public type '{}' exposes private ordinary type '{}' in its representation",
                type_def.name, name
            ));
        }

        // Check for unresolved types in type definitions (imported types should be resolvable)
        let mut unresolved = Vec::new();
        collect_core_type_body_names(&type_def.body, &mut unresolved);
        unresolved.retain(|name| {
            !known_types.contains(name) && !type_def.params.iter().any(|param| param == name)
        });
        unresolved.sort_unstable();
        unresolved.dedup();
        for name in unresolved {
            if import_info.unresolved.contains(&name) {
                errors.push(format!(
                    "public type '{}' references unresolved imported ordinary type '{}' in its representation",
                    type_def.name, name
                ));
            } else {
                errors.push(format!(
                    "public type '{}' references unresolved ordinary type '{}' in its representation",
                    type_def.name, name
                ));
            }
        }
    }
    errors
}

pub(crate) fn public_representation_type_function_leak_errors(
    type_defs: &[CoreTypeDef],
    local_type_function_names: &TypeFunctionNameSet,
) -> Vec<String> {
    if local_type_function_names.is_empty() {
        return Vec::new();
    }

    let mut errors = Vec::new();
    for type_def in type_defs
        .iter()
        .filter(|type_def| matches!(type_def.visibility, CoreVisibility::Public))
    {
        let mut leaked = Vec::new();
        collect_core_type_body_names(&type_def.body, &mut leaked);
        leaked.retain(|name| local_type_function_names.contains(name));
        leaked.sort_unstable();
        leaked.dedup();
        for name in leaked {
            errors.push(format!(
                "public type '{}' exposes local type function '{}' in its representation before SPEC-F",
                type_def.name, name
            ));
        }
    }
    errors
}

fn private_ordinary_type_names(type_defs: &[CoreTypeDef]) -> HashSet<String> {
    type_defs
        .iter()
        .filter(|type_def| {
            !matches!(type_def.visibility, CoreVisibility::Public) && !type_def.builtin
        })
        .map(|type_def| type_def.name.clone())
        .collect()
}

/// Collect names from a core type body.
pub fn collect_core_type_body_names(body: &CoreTypeBody, names: &mut Vec<String>) {
    match body {
        CoreTypeBody::Struct(fields) => {
            for (_, field_ty) in fields {
                collect_core_type_expr_names(field_ty, names);
            }
        }
        CoreTypeBody::Enum(variants) => {
            for variant in variants {
                for (_, field_ty) in &variant.fields {
                    collect_core_type_expr_names(field_ty, names);
                }
                match &variant.payload {
                    CoreVariantPayload::Unit => {}
                    CoreVariantPayload::Record(fields) => {
                        for (_, field_ty) in fields {
                            collect_core_type_expr_names(field_ty, names);
                        }
                    }
                    CoreVariantPayload::Tuple(items) => {
                        for item in items {
                            collect_core_type_expr_names(item, names);
                        }
                    }
                }
            }
        }
        CoreTypeBody::Alias(target) => collect_core_type_expr_names(target, names),
    }
}

fn collect_core_type_expr_names(expr: &CoreTypeExpr, names: &mut Vec<String>) {
    match expr {
        CoreTypeExpr::Named(name) => names.push(name.clone()),
        CoreTypeExpr::Constructor { name, args } => {
            names.push(name.clone());
            for arg in args {
                collect_core_type_expr_names(arg, names);
            }
        }
        CoreTypeExpr::Tuple(items) => {
            for item in items {
                collect_core_type_expr_names(item, names);
            }
        }
        CoreTypeExpr::Record(fields) => {
            for (_, field_ty) in fields {
                collect_core_type_expr_names(field_ty, names);
            }
        }
        CoreTypeExpr::Associated { base, .. } => collect_core_type_expr_names(base, names),
    }
}

fn public_api_visibility_errors(
    path: &Path,
    source: &str,
    type_defs: &[CoreTypeDef],
) -> Vec<String> {
    // Phase 154: a private ordinary type mentioned by a public callable is
    // exported as an opaque, publicly nameable type identity. Its constructors
    // and representation remain hidden, so representation visibility checks
    // still reject public data shapes that expose private types.
    let mut errors = public_representation_visibility_errors(path, source, type_defs);
    errors.extend(public_representation_type_function_leak_errors(
        type_defs,
        &local_type_function_names_from_source(source),
    ));
    errors.extend(public_interface_constraint_visibility_errors(path, source));
    errors
}

pub(crate) fn public_interface_constraint_visibility_errors(
    path: &Path,
    source: &str,
) -> Vec<String> {
    let Ok(module) = parse_module_file_for_type_metadata(path, source) else {
        return Vec::new();
    };
    let local_public_interface_names = local_public_interface_names(&module);
    let Ok(directly_visible_imported_interfaces) =
        directly_visible_imported_interface_names(path, source)
    else {
        return Vec::new();
    };
    validate_local_interface_constraint_visibility(
        path,
        &module,
        &local_public_interface_names,
        &directly_visible_imported_interfaces,
    )
    .map_or_else(|error| vec![error.to_string()], |()| Vec::new())
}

fn local_type_function_names_from_source(source: &str) -> TypeFunctionNameSet {
    ash_parser::parse_surface_file(source)
        .map(|module| {
            module
                .definitions
                .iter()
                .filter_map(|definition| match definition {
                    Definition::TypeFn(type_fn) => Some(type_fn.name.to_string()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Debug, Default)]
struct ImportVisibilityInfo {
    private: HashSet<String>,
    known: HashSet<String>,
    unresolved: HashSet<String>,
}

#[derive(Debug, Default)]
struct PublicImportVisibilityExports {
    type_names: HashSet<String>,
    constructor_parent_names: HashMap<String, String>,
}

pub(crate) fn public_imported_type_visibility_errors(path: &Path, source: &str) -> Vec<String> {
    let module_root = path.parent().unwrap_or_else(|| Path::new("."));
    let crate_root = discover_crate_root(module_root);
    let import_info = collect_import_visibility_info(source, module_root, crate_root.as_deref());
    let imported_private_types = import_info.private;

    if imported_private_types.is_empty() {
        return Vec::new();
    }

    let mut errors = Vec::new();
    for callable in public_callable_signatures(source) {
        append_callable_signature_visibility_errors(
            &callable,
            &imported_private_types,
            &mut errors,
        );
    }
    let local_type_defs = type_defs_from_source_for_visibility(source);
    errors.extend(public_representation_import_visibility_errors(
        &local_type_defs,
        &imported_private_types,
    ));
    errors
}

pub(crate) fn public_opaque_import_constructor_errors(path: &Path, source: &str) -> Vec<String> {
    let module_root = path.parent().unwrap_or_else(|| Path::new("."));
    let crate_root = discover_crate_root(module_root);
    let opaque_names =
        imported_callable_signature_private_type_names(source, module_root, crate_root.as_deref());
    if opaque_names.is_empty() {
        return Vec::new();
    }

    let Ok(module) = parse_module_file_for_type_metadata(path, source) else {
        return Vec::new();
    };
    let mut errors = Vec::new();
    for definition in &module.definitions {
        let Definition::Function(function) = definition else {
            continue;
        };
        if !matches!(function.visibility, ash_parser::surface::Visibility::Public) {
            continue;
        }
        let mut constructed = Vec::new();
        collect_expr_constructor_names(&function.body, &mut constructed);
        constructed.sort_unstable();
        constructed.dedup();
        for name in constructed {
            if opaque_names.contains(&name) {
                errors.push(format!(
                    "public callable '{}' constructs opaque imported ordinary type '{}' without an exported constructor",
                    function.name, name
                ));
            }
        }
    }
    errors
}

fn imported_callable_signature_private_type_names(
    source: &str,
    module_root: &Path,
    crate_root: Option<&Path>,
) -> HashSet<String> {
    let mut names = HashSet::new();
    for snippet in extract_import_snippets(source) {
        let Ok(import_spec) = parse_ordinary_import(snippet.trim()) else {
            continue;
        };
        let Ok((module_segments, search_roots)) =
            import_resolution_roots(&import_spec.module_segments, module_root, crate_root)
        else {
            continue;
        };
        let Ok(Some(target_path)) = resolve_module_path(&module_segments, &search_roots) else {
            continue;
        };
        let Ok(target_source) = std::fs::read_to_string(&target_path) else {
            continue;
        };
        let Ok(target_metadata) =
            collect_module_type_metadata_from_module_file(&target_path, &target_source)
        else {
            continue;
        };
        let private_target_types = private_ordinary_type_names(&target_metadata.type_defs);
        let target_callables = public_callable_signatures(&target_source);
        for selection in import_spec.selections {
            match selection {
                ImportSelection::Glob => {
                    for callable in &target_callables {
                        names.extend(
                            callable_signature_type_names(callable)
                                .into_iter()
                                .filter(|name| private_target_types.contains(name)),
                        );
                    }
                }
                ImportSelection::Named { name, .. } => {
                    if let Some(callable) = target_callables
                        .iter()
                        .find(|callable| callable.exported_name == name)
                    {
                        names.extend(
                            callable_signature_type_names(callable)
                                .into_iter()
                                .filter(|name| private_target_types.contains(name)),
                        );
                    }
                }
            }
        }
    }
    names
}

#[allow(clippy::too_many_lines)]
fn collect_expr_constructor_names(expr: &Expr, names: &mut Vec<String>) {
    match expr {
        Expr::Constructor {
            name,
            fields,
            payload,
            ..
        } => {
            names.push(name.to_string());
            for (_, expr) in fields {
                collect_expr_constructor_names(expr, names);
            }
            match payload {
                ash_parser::surface::ConstructorPayload::Unit => {}
                ash_parser::surface::ConstructorPayload::Record(fields) => {
                    for (_, expr) in fields {
                        collect_expr_constructor_names(expr, names);
                    }
                }
                ash_parser::surface::ConstructorPayload::Tuple(items) => {
                    for expr in items {
                        collect_expr_constructor_names(expr, names);
                    }
                }
            }
        }
        Expr::Record { fields, .. } => {
            for (_, expr) in fields {
                collect_expr_constructor_names(expr, names);
            }
        }
        Expr::FieldAccess { base, .. } | Expr::Unary { operand: base, .. } => {
            collect_expr_constructor_names(base, names);
        }
        Expr::IndexAccess { base, index, .. } => {
            collect_expr_constructor_names(base, names);
            collect_expr_constructor_names(index, names);
        }
        Expr::Binary { left, right, .. } => {
            collect_expr_constructor_names(left, names);
            collect_expr_constructor_names(right, names);
        }
        Expr::Call { args, .. } | Expr::List { items: args, .. } => {
            for arg in args {
                collect_expr_constructor_names(arg, names);
            }
        }
        Expr::FnApply { func, args, .. } => {
            collect_expr_constructor_names(func, names);
            for arg in args {
                collect_expr_constructor_names(arg, names);
            }
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            collect_expr_constructor_names(scrutinee, names);
            for arm in arms {
                collect_expr_constructor_names(&arm.body, names);
            }
        }
        Expr::IfLet {
            expr,
            then_branch,
            else_branch,
            ..
        } => {
            collect_expr_constructor_names(expr, names);
            collect_expr_constructor_names(then_branch, names);
            collect_expr_constructor_names(else_branch, names);
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_expr_constructor_names(condition, names);
            collect_expr_constructor_names(then_branch, names);
            if let Some(else_branch) = else_branch {
                collect_expr_constructor_names(else_branch, names);
            }
        }
        Expr::Fail { payload, .. } => {
            collect_expr_constructor_names(payload, names);
        }
        Expr::WithError { body, arms, .. } => {
            collect_expr_constructor_names(body, names);
            for arm in arms {
                collect_expr_constructor_names(&arm.body, names);
            }
        }
        Expr::On {
            computation,
            clauses,
            ..
        } => {
            collect_expr_constructor_names(computation, names);
            for clause in clauses {
                let body = match clause {
                    ash_parser::surface::HandlerClause::Operation { body, .. }
                    | ash_parser::surface::HandlerClause::Done { body, .. } => body,
                };
                collect_expr_constructor_names(body, names);
            }
        }
        Expr::HandleWith { expression, .. } => collect_expr_constructor_names(expression, names),
        Expr::Block {
            statements,
            tail_expr,
            ..
        } => {
            for statement in statements {
                match statement {
                    ash_parser::surface::BlockStmt::Let { expr, .. }
                    | ash_parser::surface::BlockStmt::Expr { expr, .. } => {
                        collect_expr_constructor_names(expr, names);
                    }
                }
            }
            if let Some(tail_expr) = tail_expr {
                collect_expr_constructor_names(tail_expr, names);
            }
        }
        Expr::FnDef { body, .. } => collect_expr_constructor_names(body, names),
        Expr::Policy(policy) => collect_policy_expr_constructor_names(policy, names),
        Expr::Literal(_)
        | Expr::Variable { .. }
        | Expr::OperatorSection { .. }
        | Expr::MacroInvocation { .. }
        | Expr::CheckObligation { .. }
        | Expr::Panic { .. }
        | Expr::DoBlock { .. }
        | Expr::Comprehension { .. } => {}
    }
}

fn collect_policy_expr_constructor_names(
    policy: &ash_parser::surface::PolicyExpr,
    names: &mut Vec<String>,
) {
    match policy {
        ash_parser::surface::PolicyExpr::ForAll { items, body, .. }
        | ash_parser::surface::PolicyExpr::Exists { items, body, .. } => {
            collect_expr_constructor_names(items, names);
            collect_policy_expr_constructor_names(body, names);
        }
        ash_parser::surface::PolicyExpr::MethodCall { receiver, args, .. } => {
            collect_policy_expr_constructor_names(receiver, names);
            for arg in args {
                collect_expr_constructor_names(arg, names);
            }
        }
        ash_parser::surface::PolicyExpr::Call { args, .. } => {
            for arg in args {
                collect_expr_constructor_names(arg, names);
            }
        }
        ash_parser::surface::PolicyExpr::And(items)
        | ash_parser::surface::PolicyExpr::Or(items)
        | ash_parser::surface::PolicyExpr::Sequential(items)
        | ash_parser::surface::PolicyExpr::Concurrent(items) => {
            for item in items {
                collect_policy_expr_constructor_names(item, names);
            }
        }
        ash_parser::surface::PolicyExpr::Not(inner) => {
            collect_policy_expr_constructor_names(inner, names);
        }
        ash_parser::surface::PolicyExpr::Implies(left, right) => {
            collect_policy_expr_constructor_names(left, names);
            collect_policy_expr_constructor_names(right, names);
        }
        ash_parser::surface::PolicyExpr::Var { .. } => {}
    }
}

fn collect_import_visibility_info(
    source: &str,
    module_root: &Path,
    crate_root: Option<&Path>,
) -> ImportVisibilityInfo {
    let mut info = ImportVisibilityInfo::default();

    for snippet in extract_import_snippets(source) {
        let trimmed = snippet.trim();
        let Ok(import_spec) = parse_ordinary_import(trimmed) else {
            continue;
        };
        let Ok((module_segments, search_roots)) =
            import_resolution_roots(&import_spec.module_segments, module_root, crate_root)
        else {
            add_unresolved_import_selections(&mut info, import_spec.selections);
            continue;
        };
        let Ok(Some(target_path)) = resolve_module_path(&module_segments, &search_roots) else {
            add_unresolved_import_selections(&mut info, import_spec.selections);
            continue;
        };
        let Ok(target_source) = std::fs::read_to_string(&target_path) else {
            add_unresolved_import_selections(&mut info, import_spec.selections);
            continue;
        };
        let Ok(target_metadata) =
            collect_module_type_metadata_from_module_file(&target_path, &target_source)
        else {
            add_unresolved_import_selections(&mut info, import_spec.selections);
            continue;
        };
        let target_exports = collect_public_import_visibility_exports(
            &target_path,
            &target_source,
            &target_metadata,
            &mut HashSet::new(),
        );
        add_resolved_import_selection_visibility(
            &mut info,
            &target_source,
            &target_metadata,
            &target_exports,
            import_spec.selections,
        );
    }

    info
}

fn add_unresolved_import_selections(
    info: &mut ImportVisibilityInfo,
    selections: Vec<ImportSelection>,
) {
    for selection in selections {
        match selection {
            ImportSelection::Named { name, alias } => {
                info.unresolved.insert(alias.unwrap_or(name));
            }
            ImportSelection::Glob => {}
        }
    }
}

fn collect_public_import_visibility_exports(
    path: &Path,
    source: &str,
    metadata: &ash_parser::lower::LoweredTypeMetadata,
    visited: &mut HashSet<PathBuf>,
) -> PublicImportVisibilityExports {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canonical) {
        return PublicImportVisibilityExports::default();
    }

    let mut exports = PublicImportVisibilityExports::default();
    for type_def in metadata
        .type_defs
        .iter()
        .filter(|type_def| matches!(type_def.visibility, CoreVisibility::Public))
    {
        exports.type_names.insert(type_def.name.clone());
    }
    for constructor in &metadata.summary.exported_constructors {
        if let Some(parent_name) =
            parent_type_name_for_constructor(&metadata.summary, &constructor.exported_name)
        {
            exports
                .constructor_parent_names
                .insert(constructor.exported_name.clone(), parent_name);
        }
    }

    let module_root = path.parent().unwrap_or_else(|| Path::new("."));
    let crate_root = discover_crate_root(module_root);
    for snippet in extract_pub_use_snippets(source) {
        let trimmed = snippet.trim_start();
        let Ok(import_spec) = parse_ordinary_import(trimmed) else {
            continue;
        };
        let Ok((module_segments, search_roots)) = import_resolution_roots(
            &import_spec.module_segments,
            module_root,
            crate_root.as_deref(),
        ) else {
            continue;
        };
        let Ok(Some(target_path)) = resolve_module_path(&module_segments, &search_roots) else {
            continue;
        };
        let Ok(target_source) = std::fs::read_to_string(&target_path) else {
            continue;
        };
        let Ok(target_metadata) =
            collect_module_type_metadata_from_module_file(&target_path, &target_source)
        else {
            continue;
        };
        let child_exports = collect_public_import_visibility_exports(
            &target_path,
            &target_source,
            &target_metadata,
            visited,
        );
        merge_pub_use_visibility_exports(&mut exports, &child_exports, import_spec.selections);
    }

    exports
}

fn merge_pub_use_visibility_exports(
    exports: &mut PublicImportVisibilityExports,
    child: &PublicImportVisibilityExports,
    selections: Vec<ImportSelection>,
) {
    for selection in selections {
        match selection {
            ImportSelection::Glob => {
                exports.type_names.extend(child.type_names.iter().cloned());
                exports.constructor_parent_names.extend(
                    child
                        .constructor_parent_names
                        .iter()
                        .map(|(name, parent)| (name.clone(), parent.clone())),
                );
            }
            ImportSelection::Named { name, alias } => {
                let visible_name = alias.clone().unwrap_or_else(|| name.clone());
                if child.type_names.contains(&name) {
                    exports.type_names.insert(visible_name);
                    continue;
                }
                if let Some(parent_name) = child.constructor_parent_names.get(&name) {
                    exports
                        .constructor_parent_names
                        .insert(name, parent_name.clone());
                }
            }
        }
    }
}

fn add_resolved_import_selection_visibility(
    info: &mut ImportVisibilityInfo,
    target_source: &str,
    target_metadata: &ash_parser::lower::LoweredTypeMetadata,
    target_exports: &PublicImportVisibilityExports,
    selections: Vec<ImportSelection>,
) {
    let target_callables = public_callable_signatures(target_source);

    for selection in selections {
        match selection {
            ImportSelection::Glob => {
                info.known.extend(target_exports.type_names.iter().cloned());
                for callable in &target_callables {
                    info.known.extend(callable_signature_type_names(callable));
                }
            }
            ImportSelection::Named { name, alias } => {
                let visible_name = alias.unwrap_or_else(|| name.clone());
                if target_exports.type_names.contains(&name) {
                    info.known.insert(visible_name.clone());
                    continue;
                }
                if let Some(type_def) = target_metadata
                    .type_defs
                    .iter()
                    .find(|type_def| type_def.name == name)
                {
                    info.known.insert(visible_name.clone());
                    if !matches!(type_def.visibility, CoreVisibility::Public) && !type_def.builtin {
                        info.private.insert(visible_name);
                    }
                    continue;
                }
                if let Some(parent_name) = target_exports.constructor_parent_names.get(&name) {
                    info.known.insert(parent_name.clone());
                    continue;
                }
                if let Some(parent_name) =
                    parent_type_name_for_constructor(&target_metadata.summary, &name)
                {
                    info.known.insert(parent_name);
                    continue;
                }
                if let Some(callable) = target_callables
                    .iter()
                    .find(|callable| callable.exported_name == name)
                {
                    info.known.extend(callable_signature_type_names(callable));
                    // Importing a callable can make the types in its signature
                    // nameable, but the callable's local alias is still not an
                    // ordinary type name.
                    info.unresolved.insert(visible_name);
                    continue;
                }
                info.unresolved.insert(visible_name);
            }
        }
    }
}

fn parent_type_name_for_constructor(
    summary: &ModuleSemanticSummary,
    constructor_name: &str,
) -> Option<String> {
    let constructor = summary
        .exported_constructors
        .iter()
        .find(|constructor| constructor.exported_name == constructor_name)?;
    summary
        .exported_types
        .iter()
        .find(|ty| ty.id == constructor.parent)
        .map(|ty| ty.exported_name.clone())
}

fn type_defs_from_source_for_visibility(source: &str) -> Vec<CoreTypeDef> {
    ash_parser::parse_surface_file(source)
        .ok()
        .map(|module| {
            ash_parser::lower::lower_module_type_metadata(&module, synthetic_visibility_module())
                .type_defs
        })
        .unwrap_or_default()
}

fn synthetic_visibility_module() -> ash_core::semantic_summary::ModuleIdentity {
    ash_core::semantic_summary::ModuleIdentity::new(
        None,
        ash_core::module_graph::ModuleId(0),
        vec!["visibility-check".to_string()],
        ash_core::semantic_summary::ModuleSourceOrigin::Synthetic {
            reason: "public-imported-type-visibility".to_string(),
        },
    )
}

fn public_representation_import_visibility_errors(
    type_defs: &[CoreTypeDef],
    imported_private_types: &HashSet<String>,
) -> Vec<String> {
    let mut errors = Vec::new();
    for type_def in type_defs
        .iter()
        .filter(|type_def| matches!(type_def.visibility, CoreVisibility::Public))
    {
        let mut leaked = Vec::new();
        collect_core_type_body_names(&type_def.body, &mut leaked);
        leaked.retain(|name| {
            imported_private_types.contains(name)
                && !type_def.params.iter().any(|param| param == name)
        });
        leaked.sort_unstable();
        leaked.dedup();
        for name in leaked {
            errors.push(format!(
                "public type '{}' exposes private ordinary type '{}' in its representation",
                type_def.name, name
            ));
        }
    }
    errors
}

/// Parse a module source as a full `ModuleFile` and lower its ordinary type
/// metadata into core declarations plus core-owned semantic summaries.
pub(crate) fn collect_module_type_metadata_from_module_file(
    path: &Path,
    source: &str,
) -> Result<ash_parser::lower::LoweredTypeMetadata, EngineError> {
    let metadata_source = strip_module_metadata_non_definition_lines(source);
    let module = parse_module_file_for_type_metadata(path, &metadata_source)?;
    collect_module_type_metadata_from_parsed_module_file(path, &module)
}

fn collect_module_type_metadata_from_parsed_module_file(
    path: &Path,
    module: &ash_parser::surface::ModuleFile,
) -> Result<ash_parser::lower::LoweredTypeMetadata, EngineError> {
    reject_inline_module_ordinary_types(path, module)?;
    Ok(ash_parser::lower::lower_module_type_metadata(
        module,
        module_identity_for_path(path),
    ))
}

fn reject_inline_module_ordinary_types(
    path: &Path,
    module: &ash_parser::surface::ModuleFile,
) -> Result<(), EngineError> {
    for module_decl in &module.module_decls {
        let ash_parser::module::ModuleSource::Inline(definitions) = &module_decl.source else {
            continue;
        };
        if definitions
            .iter()
            .any(|definition| matches!(definition, ash_parser::surface::Definition::Type(_)))
        {
            return Err(EngineError::Parse(format!(
                "in '{}': inline module '{}' ordinary type declarations are not yet lowered into semantic summaries; move the type declarations to a file module or defer the inline module type surface",
                path.display(),
                module_decl.name
            )));
        }
        if definitions.iter().any(|definition| {
            matches!(definition, ash_parser::surface::Definition::SealedDomain(_))
        }) {
            return Err(EngineError::Parse(format!(
                "in '{}': inline module '{}' sealed domain declarations are not yet lowered into semantic summaries; move the sealed domain declarations to a file module or defer the inline module domain surface",
                path.display(),
                module_decl.name
            )));
        }
    }
    Ok(())
}

pub(crate) fn collect_runtime_stdlib_type_defs_from_module_file(
    module_path: &str,
    source: &str,
) -> Result<Vec<CoreTypeDef>, EngineError> {
    let virtual_path = PathBuf::from(format!("std://{}.ash", module_path.replace("::", "/")));
    let metadata_source = strip_module_metadata_non_definition_lines(source);
    Ok(collect_module_type_metadata_from_module_file(&virtual_path, &metadata_source)?.type_defs)
}

fn strip_module_metadata_non_definition_lines(source: &str) -> String {
    let mut kept = Vec::new();
    let mut skipping_import = false;
    let mut import_brace_depth = 0usize;

    for line in source.lines() {
        let trimmed = line.trim_start();
        if skipping_import {
            import_brace_depth = import_brace_depth_after_line(import_brace_depth, trimmed);
            if trimmed.ends_with(';') || import_brace_depth == 0 {
                skipping_import = false;
            }
            continue;
        }

        if trimmed.starts_with("use ") || trimmed.starts_with("pub use ") {
            import_brace_depth = import_brace_depth_after_line(0, trimmed);
            if !trimmed.ends_with(';') && import_brace_depth > 0 {
                skipping_import = true;
            }
            continue;
        }

        if trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ") {
            continue;
        }

        kept.push(line);
    }

    kept.join("\n")
}

fn import_brace_depth_after_line(current_depth: usize, line: &str) -> usize {
    line.chars().fold(current_depth, |depth, ch| match ch {
        '{' => depth.saturating_add(1),
        '}' => depth.saturating_sub(1),
        _ => depth,
    })
}

fn parse_module_file_for_type_metadata(
    path: &Path,
    source: &str,
) -> Result<ash_parser::surface::ModuleFile, EngineError> {
    match ash_parser::parse_surface_file_with_path(source, Some(path)) {
        Ok(module) => Ok(module),
        Err(errors) => Err(module_type_metadata_parse_error(path, &errors)),
    }
}

fn module_type_metadata_parse_error(
    path: &Path,
    errors: &[ash_parser::error::ParseError],
) -> EngineError {
    EngineError::Parse(format!(
        "in '{}': failed to parse module file for type metadata: {}",
        path.display(),
        format_parse_errors(errors)
    ))
}

fn format_parse_errors(errors: &[ash_parser::error::ParseError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn module_identity_for_path(path: &Path) -> ModuleIdentity {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let path_text = canonical.to_string_lossy().into_owned();
    ModuleIdentity::new(
        None,
        ModuleId(stable_module_id_from_path(&path_text)),
        module_identity_segments(&canonical),
        ModuleSourceOrigin::File(path_text),
    )
}

fn stable_module_id_from_path(path: &str) -> usize {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in path.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    #[cfg(target_pointer_width = "64")]
    {
        usize::try_from(hash).expect("u64 hash fits in usize on 64-bit targets")
    }
    #[cfg(target_pointer_width = "32")]
    {
        let folded = hash ^ (hash >> 32);
        usize::try_from(folded & u64::from(u32::MAX))
            .expect("folded 32-bit hash fits in usize on 32-bit targets")
    }
}

fn module_identity_segments(path: &Path) -> Vec<String> {
    path.file_stem().and_then(|stem| stem.to_str()).map_or_else(
        || vec![path.to_string_lossy().into_owned()],
        |stem| vec![stem.to_string()],
    )
}

/// Count the number of `pub fn` snippets in source text that parse successfully,
/// returning the count and any diagnostics for snippets that failed to parse.
#[must_use]
pub fn count_pub_fn_snippets(source: &str) -> (usize, Vec<PubFnDiagnostic>) {
    let snippets = extract_braced_snippets(source, |trimmed| trimmed.starts_with("pub fn "));
    let mut count = 0;
    let mut diagnostics = Vec::new();
    for snippet in &snippets {
        match parse_supported_pub_fn_callable(snippet) {
            Ok(Some(_)) => count += 1,
            Ok(None) => {} // parsed but no callable -- shouldn't happen
            Err(diag) => diagnostics.push(diag),
        }
    }
    (count, diagnostics)
}

fn is_skippable_prelude_line(line: &str) -> bool {
    line.is_empty()
        || line.starts_with("--")
        || line.starts_with("//")
        || line.starts_with("/*")
        || line.starts_with('*')
}

fn import_needs_more_lines(snippet: &str) -> bool {
    snippet.contains("::{") && !snippet.contains('}') && !snippet.contains(';')
}

/// Parse all imports from a module source file.
/// Returns a list of import specs for `use` and `pub use` statements.
///
/// # Errors
/// Returns an error if an import statement cannot be parsed.
pub fn parse_module_imports(source: &str) -> Result<Vec<ImportSpec>, EngineError> {
    let mut imports = Vec::new();
    let mut lines = source.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if is_skippable_prelude_line(trimmed) {
            continue;
        }
        if trimmed.starts_with("use ") || trimmed.starts_with("pub use ") {
            let mut snippet = line.to_string();
            while import_needs_more_lines(&snippet) {
                let Some(next_line) = lines.next() else {
                    break;
                };
                snippet.push('\n');
                snippet.push_str(next_line);
            }
            imports.push(parse_ordinary_import(snippet.trim())?);
        } else {
            // Stop at first non-import, non-comment line
            break;
        }
    }
    Ok(imports)
}

fn parse_ordinary_import(line: &str) -> Result<ImportSpec, EngineError> {
    if line.contains('@') {
        return parse_versioned_import(line);
    }

    let normalized = {
        let trimmed_line = line.trim_start();
        let import_line = trimmed_line.strip_prefix("pub use ").map_or_else(
            || line.to_string(),
            |rest| {
                let prefix_len = line.len() - trimmed_line.len();
                let prefix = &line[..prefix_len];
                format!("{prefix}use {rest}")
            },
        );
        if import_line.trim_end().ends_with(';') {
            import_line
        } else {
            format!("{import_line};")
        }
    };
    let mut input = new_input(&normalized);
    let use_stmt = parse_use
        .parse_next(&mut input)
        .map_err(|error| EngineError::Parse(format!("failed to parse import '{line}': {error}")))?;
    Ok(convert_use_statement(use_stmt))
}

fn parse_versioned_import(line: &str) -> Result<ImportSpec, EngineError> {
    let import = line
        .trim()
        .strip_prefix("use ")
        .ok_or_else(|| EngineError::Parse(format!("unsupported import syntax '{line}'")))?
        .trim_end_matches(';')
        .trim();

    let (module_path, selection_text) = if let Some((module_path, items)) = import.split_once("::{")
    {
        let items = items
            .strip_suffix('}')
            .ok_or_else(|| EngineError::Parse(format!("unsupported import syntax '{line}'")))?;
        (module_path.trim(), Some(items))
    } else {
        let last_sep = import
            .rfind("::")
            .ok_or_else(|| EngineError::Parse(format!("unsupported import syntax '{line}'")))?;
        let module_path = &import[..last_sep];
        let item = &import[last_sep + 2..];
        (module_path.trim(), Some(item))
    };

    let module_segments = module_path
        .split("::")
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if module_segments.is_empty() {
        return Err(EngineError::Parse(format!(
            "unsupported import syntax '{line}'"
        )));
    }

    let selections = selection_text.map(parse_selection_list).unwrap_or_default();

    Ok(ImportSpec {
        module_segments,
        selections,
    })
}

fn parse_selection_list(selection_text: &str) -> Vec<ImportSelection> {
    if selection_text.trim() == "*" {
        return vec![ImportSelection::Glob];
    }

    let mut selections = Vec::new();
    for raw_item in selection_text.split(',') {
        let item = raw_item.trim();
        if item.is_empty() {
            continue;
        }

        if item == "*" {
            selections.push(ImportSelection::Glob);
            continue;
        }

        let (name, alias) = if let Some((name, alias)) = item.split_once(" as ") {
            (name.trim(), Some(alias.trim().to_string()))
        } else {
            (item, None)
        };

        selections.push(ImportSelection::Named {
            name: name.to_string(),
            alias,
        });
    }

    selections
}

fn convert_use_statement(use_stmt: ash_parser::use_tree::Use) -> ImportSpec {
    use ash_parser::use_tree::UsePath;

    let selections = match &use_stmt.path {
        UsePath::Simple(path) => {
            if path.segments.len() <= 1 {
                vec![ImportSelection::Named {
                    name: path
                        .segments
                        .first()
                        .map(std::string::ToString::to_string)
                        .unwrap_or_default(),
                    alias: use_stmt.alias.map(|alias| alias.to_string()),
                }]
            } else {
                let name = path
                    .segments
                    .last()
                    .expect("segments checked above")
                    .to_string();
                vec![ImportSelection::Named {
                    name,
                    alias: use_stmt.alias.map(|alias| alias.to_string()),
                }]
            }
        }
        UsePath::Glob(_) => vec![ImportSelection::Glob],
        UsePath::Nested(_, items) => items
            .iter()
            .map(|item| ImportSelection::Named {
                name: item.name.as_ref().to_string(),
                alias: item.alias.as_ref().map(|alias| alias.as_ref().to_string()),
            })
            .collect(),
    };

    let module_segments = match &use_stmt.path {
        UsePath::Simple(path) | UsePath::Glob(path) => {
            if path.segments.len() <= 1 {
                path.segments
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect()
            } else {
                path.segments
                    .iter()
                    .take(path.segments.len() - 1)
                    .map(std::string::ToString::to_string)
                    .collect()
            }
        }
        UsePath::Nested(path, _) => path
            .segments
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
    };

    ImportSpec {
        module_segments,
        selections,
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn collect_module_exports(
    path: &Path,
    cache: &mut HashMap<PathBuf, ModuleExports>,
    visiting: &mut HashSet<PathBuf>,
) -> Result<ModuleExports, EngineError> {
    let path = path.to_path_buf();
    let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
    if visiting.contains(&canonical) {
        return Err(EngineError::Parse(format!(
            "cyclic import detected: '{}'",
            path.display()
        )));
    }
    // Always read the source before reusing a module cache entry.  A semantic
    // summary's V7 provider/binding contract (including its closure digest)
    // must be rebuilt after a public source edit; a path-only cache key is not
    // sufficient for that boundary.
    let source = std::fs::read_to_string(&path)?;
    let source_fingerprint = module_source_cache_fingerprint(&source);
    if let Some(exports) = cache.get(&path).cloned() {
        visiting.insert(canonical.clone());
        let reusable =
            module_exports_cache_is_reusable(&exports, &source_fingerprint, cache, visiting)?;
        visiting.remove(&canonical);
        if reusable {
            return Ok(exports);
        }
    }
    visiting.insert(canonical.clone());

    let metadata_source = strip_module_metadata_non_definition_lines(&source);
    let parsed_module = parse_module_file_for_type_metadata(&path, &metadata_source)?;
    let imported_macros =
        collect_imported_macro_entries_with_state(&path, &source, cache, visiting)?;
    let expanded = ash_parser::surface::expand_surface_module_with_imported_macros(
        parsed_module.clone(),
        imported_macros,
    )
    .map_err(|error| {
        EngineError::Parse(format!(
            "in '{}': expanded-surface validation failed: {error}",
            path.display()
        ))
    })?;
    let mut exports = ModuleExports {
        source_fingerprint,
        ..ModuleExports::default()
    };
    let module_effectful_names =
        ash_parser::effectful_names_from_definitions(&expanded.module.definitions);
    let module_runtime_callables = module_runtime_callables_from_definitions(
        &expanded.module.definitions,
        &module_effectful_names,
    );

    let type_metadata =
        collect_module_type_metadata_from_parsed_module_file(&path, &parsed_module)?;
    let mut public_api_errors =
        public_api_visibility_errors(&path, &source, &type_metadata.type_defs);
    public_api_errors.extend(public_callable_signature_resolution_errors(
        &path,
        &source,
        &type_metadata.type_defs,
    ));
    public_api_errors.extend(public_imported_type_visibility_errors(&path, &source));
    if !public_api_errors.is_empty() {
        return Err(EngineError::Parse(format!(
            "in '{}': public API visibility errors: {}",
            path.display(),
            public_api_errors.join("; ")
        )));
    }
    for type_def in &type_metadata.type_defs {
        insert_type_export(&mut exports, type_def)?;
    }
    let opaque_public_signature_type_names =
        public_callable_opaque_type_names(&source, &type_metadata.type_defs);
    exports.semantic_summary = Some(exportable_module_semantic_summary(
        &type_metadata.summary,
        &exports.type_defs,
        &opaque_public_signature_type_names,
    )?);
    attach_public_type_function_summaries(&mut exports, &type_metadata, &path)?;
    attach_public_associated_family_summaries(&mut exports, &type_metadata, &path, &source)?;
    attach_public_macro_summaries(&mut exports, &parsed_module, &path)?;
    attach_public_interface_identity_summaries(&mut exports, &path, &source)?;
    attach_public_proposition_summaries(&mut exports, &type_metadata, &path, &source)?;
    // Later attachment helpers may set their historical payload version. A
    // provider/binding effect-row payload remains a whole-summary V7
    // contract, so it must never be silently downgraded by those helpers.
    if let Some(summary) = exports.semantic_summary.as_mut()
        && !summary.exported_effect_rows.is_empty()
    {
        summary.version = SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7;
    }
    if let Some(summary) = exports.semantic_summary.as_ref() {
        summary
            .validate_summary_version_contract()
            .map_err(|error| {
                EngineError::Parse(format!(
                    "in '{}': invalid module semantic summary version/content contract: {error:?}",
                    path.display()
                ))
            })?;
    }

    for name in extract_public_capability_names(&source) {
        insert_type_export(&mut exports, &capability_type_identity(&name))?;
    }

    for snippet in
        extract_semicolon_snippets(&source, |trimmed| trimmed.starts_with("pub builtin fn "))
    {
        // `module` is left empty here; load_ordinary_file populates the real
        // value from the import path before inserting into imported_callables.
        if let Some(callable) = parse_builtin_fn_callable(&snippet, String::new())? {
            let mut callable = callable.callable;
            callable.effectful_names.clone_from(&module_effectful_names);
            stamp_callable_export_module(&mut callable, exports.semantic_summary.as_ref());
            let exported_name = callable.exported_name.clone();
            insert_callable_export(&mut exports, &exported_name, callable)?;
        }
    }

    for definition in &expanded.module.definitions {
        let Definition::Function(function) = definition else {
            continue;
        };
        if !matches!(function.visibility, ash_parser::surface::Visibility::Public) {
            continue;
        }
        let mut callable = imported_callable_from_fn_def(function.clone()).callable;
        callable.effectful_names.clone_from(&module_effectful_names);
        callable
            .module_runtime_callables
            .clone_from(&module_runtime_callables);
        stamp_callable_export_module(&mut callable, exports.semantic_summary.as_ref());
        let exported_name = callable.exported_name.clone();
        insert_callable_export(&mut exports, &exported_name, callable)?;
    }

    for snippet in extract_braced_snippets(&source, |trimmed| trimmed.starts_with("pub fn ")) {
        let Ok(Some(callable)) = parse_supported_pub_fn_callable(&snippet) else {
            continue;
        };
        let mut callable = callable.callable;
        callable.effectful_names.clone_from(&module_effectful_names);
        callable
            .module_runtime_callables
            .clone_from(&module_runtime_callables);
        stamp_callable_export_module(&mut callable, exports.semantic_summary.as_ref());
        let exported_name = callable.exported_name.clone();
        insert_callable_export(&mut exports, &exported_name, callable)?;
    }

    // Process pub mod <name>; declarations -- load child module exports
    let module_root = path.parent().ok_or_else(|| {
        EngineError::Configuration(format!("module path '{}' has no parent", path.display()))
    })?;

    // Check regular `use` imports for cycles (e.g. a.ash has `use b::{X}`,
    // b.ash has `use a::{Y}` -- both reference each other).
    let crate_root = discover_crate_root(module_root);
    for snippet in extract_import_snippets(&source) {
        let trimmed = snippet.trim();
        if let Ok(import_spec) = parse_ordinary_import(trimmed) {
            let (module_segments, search_roots) = import_resolution_roots(
                &import_spec.module_segments,
                module_root,
                crate_root.as_deref(),
            )?;
            if let Some(target_path) = resolve_module_path(&module_segments, &search_roots)? {
                let target_canonical = target_path
                    .canonicalize()
                    .unwrap_or_else(|_| target_path.clone());
                if visiting.contains(&target_canonical) {
                    return Err(EngineError::Parse(format!(
                        "cyclic import detected: '{}'",
                        target_path.display()
                    )));
                }
            }
        }
    }

    for name in extract_pub_mod_declarations(&source) {
        let child_path = resolve_child_module(path.as_path(), &name)?;
        visiting.insert(canonical.clone());
        let child_exports =
            collect_module_exports(&child_path, cache, visiting).map_err(|error| {
                EngineError::Parse(format!(
                    "in '{}': failed to load public child module '{}': {error}",
                    path.display(),
                    child_path.display()
                ))
            })?;
        visiting.remove(&canonical);
        // Store child exports under the child module name (for qualified access)
        exports.public_dependency_fingerprints.insert(
            child_path,
            module_exports_cache_validation_fingerprint(&child_exports),
        );
        exports.child_modules.insert(name, child_exports);
    }

    for snippet in extract_semicolon_snippets(&source, |trimmed| trimmed.starts_with("pub use ")) {
        let normalized = snippet.trim();
        let mut input = new_input(normalized);
        let use_stmt = parse_use.parse_next(&mut input).map_err(|error| {
            EngineError::Parse(format!(
                "in '{}': failed to parse pub use: {error}",
                path.display()
            ))
        })?;
        let resolved = resolve_use_target(module_root, &use_stmt)?;
        visiting.insert(canonical.clone());
        let mut target_exports = collect_module_exports(&resolved, cache, visiting)?;
        visiting.remove(&canonical);
        exports.public_dependency_fingerprints.insert(
            resolved.clone(),
            module_exports_cache_validation_fingerprint(&target_exports),
        );
        let target_module = module_path_text(&resolved).to_string();
        stamp_builtin_callable_modules(&mut target_exports, &target_module);
        let target_summary = target_exports.semantic_summary.clone();
        merge_use_exports(&mut exports, target_exports, use_stmt)?;
        rewrite_exported_callable_signature_aliases(&mut exports, target_summary.as_ref());
    }

    visiting.remove(&canonical);
    exports.effect_row_contract_fingerprint =
        effect_row_public_contract_fingerprint(exports.semantic_summary.as_ref());
    cache.insert(path.clone(), exports.clone());
    Ok(exports)
}

/// Return whether an in-memory module-cache value can still be used for the
/// source currently at `path`.
///
/// Summary validation is intentionally repeated on a cache hit: callers may
/// have decoded or otherwise supplied stale cache data with a legacy summary
/// version or unknown sanitizer schema.  Such an entry is never a binding
/// source; rebuilding from public source is the only recovery path. Public
/// dependency state is checked recursively so an unchanged facade cannot
/// serve an old provider closure after its provider changes.
fn module_exports_cache_is_reusable(
    exports: &ModuleExports,
    source_fingerprint: &str,
    cache: &mut HashMap<PathBuf, ModuleExports>,
    visiting: &mut HashSet<PathBuf>,
) -> Result<bool, EngineError> {
    if exports.source_fingerprint != source_fingerprint
        || exports
            .semantic_summary
            .as_ref()
            .is_some_and(|summary| summary.validate_summary_version_contract().is_err())
        || exports.effect_row_contract_fingerprint
            != effect_row_public_contract_fingerprint(exports.semantic_summary.as_ref())
    {
        return Ok(false);
    }

    for (dependency_path, expected_fingerprint) in &exports.public_dependency_fingerprints {
        let current = collect_module_exports(dependency_path, cache, visiting)?;
        if module_exports_cache_validation_fingerprint(&current) != *expected_fingerprint {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Compute the source component of the in-memory module-cache validity key.
fn module_source_cache_fingerprint(source: &str) -> String {
    use sha2::{Digest, Sha256};

    format!("sha256:{:x}", Sha256::digest(source.as_bytes()))
}

/// Fingerprint a module's cache-relevant public dependency state.  This
/// private in-memory value is deliberately distinct from serialized semantic
/// summaries: it carries only hashes, never source text or private facts.
fn module_exports_cache_validation_fingerprint(exports: &ModuleExports) -> String {
    use sha2::{Digest, Sha256};

    let mut dependencies = exports
        .public_dependency_fingerprints
        .iter()
        .map(|(path, fingerprint)| (path.to_string_lossy(), fingerprint.as_str()))
        .collect::<Vec<_>>();
    dependencies.sort_unstable();
    let mut canonical = format!("source:{}\n", exports.source_fingerprint);
    for (path, fingerprint) in dependencies {
        writeln!(&mut canonical, "dependency:{}:{}", path.len(), path)
            .expect("writing to String cannot fail");
        writeln!(
            &mut canonical,
            "fingerprint:{}:{}",
            fingerprint.len(),
            fingerprint
        )
        .expect("writing to String cannot fail");
    }
    format!("sha256:{:x}", Sha256::digest(canonical.as_bytes()))
}

/// Digest public effect-row facts exactly as retained by the V7 summary.
/// This gives the cache a separately recomputed integrity check for sanitizer
/// closure metadata without serializing cache state or consulting private
/// source rows.
fn effect_row_public_contract_fingerprint(
    summary: Option<&ModuleSemanticSummary>,
) -> Option<String> {
    use sha2::{Digest, Sha256};

    let summary = summary?;
    if summary.version != SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7
        || summary.exported_effect_rows.is_empty()
    {
        return None;
    }
    let payload = serde_json::to_vec(&summary.exported_effect_rows).ok()?;
    Some(format!("sha256:{:x}", Sha256::digest(payload)))
}

fn resolve_use_target(
    module_root: &Path,
    use_stmt: &ash_parser::use_tree::Use,
) -> Result<PathBuf, EngineError> {
    let segments = match &use_stmt.path {
        ash_parser::use_tree::UsePath::Simple(path) | ash_parser::use_tree::UsePath::Glob(path) => {
            path.segments.clone()
        }
        ash_parser::use_tree::UsePath::Nested(path, _) => path.segments.clone(),
    };

    let module_segments = match &use_stmt.path {
        ash_parser::use_tree::UsePath::Simple(path) => {
            if path.segments.len() <= 1 {
                path.segments.clone()
            } else {
                path.segments[..path.segments.len() - 1].to_vec()
            }
        }
        ash_parser::use_tree::UsePath::Glob(path)
        | ash_parser::use_tree::UsePath::Nested(path, _) => path.segments.clone(),
    };

    let module_segments = module_segments
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();
    let crate_root = discover_crate_root(module_root);
    let (module_segments, search_roots) =
        import_resolution_roots(&module_segments, module_root, crate_root.as_deref())?;

    resolve_module_path(&module_segments, &search_roots)?.ok_or_else(|| {
        EngineError::Parse(format!(
            "module '{}' not found (searched from '{}')",
            segments
                .iter()
                .map(std::convert::AsRef::as_ref)
                .collect::<Vec<_>>()
                .join("::"),
            module_root.display()
        ))
    })
}

#[allow(clippy::too_many_lines)]
fn merge_use_exports(
    exports: &mut ModuleExports,
    target_exports: ModuleExports,
    use_stmt: ash_parser::use_tree::Use,
) -> Result<(), EngineError> {
    use ash_parser::use_tree::UsePath;

    let target_semantic_summary = target_exports.semantic_summary.clone();

    match use_stmt.path {
        UsePath::Glob(_) => {
            for (name, type_def) in target_exports.type_defs {
                insert_type_export_with_name(exports, &name, type_def)?;
            }
            if let Some(summary) = target_semantic_summary.as_ref() {
                for ty in &summary.exported_types {
                    merge_type_summary_export(
                        exports,
                        summary,
                        &ty.exported_name,
                        &ty.exported_name,
                    )?;
                }
            }
            for (name, type_def) in target_exports.constructor_defs {
                insert_constructor_export_with_name(exports, &name, type_def)?;
            }
            for (name, mut callable) in target_exports.callables {
                stamp_callable_export_module(&mut callable, exports.semantic_summary.as_ref());
                insert_callable_export(exports, &name, callable)?;
            }
            for (name, type_function) in target_exports.type_function_summaries {
                insert_type_function_export(exports, &name, type_function)?;
            }
            for (name, family) in target_exports.associated_family_summaries {
                insert_associated_family_export(exports, &name, family)?;
            }
            if let Some(summary) = target_semantic_summary.as_ref() {
                if let Some(selected_summary) = selected_proposition_semantic_summary(Some(summary))
                {
                    merge_selected_summary_export(exports, summary, selected_summary)?;
                }
                for interface in &summary.interface_identities {
                    if let Some(selected_summary) = selected_interface_semantic_summary(
                        summary,
                        &interface.name,
                        &interface.name,
                    ) {
                        merge_selected_summary_export(exports, summary, selected_summary)?;
                    }
                }
                for type_function in &summary.exported_type_functions {
                    if let Some(selected_summary) = selected_type_function_semantic_summary(
                        summary,
                        &type_function.exported_name,
                        &type_function.exported_name,
                    ) {
                        merge_selected_summary_export(exports, summary, selected_summary)?;
                    }
                }
                for family in &summary.exported_associated_families {
                    if let Some(selected_summary) = selected_associated_family_semantic_summary(
                        summary,
                        &family.visible_name,
                        &family.visible_name,
                    ) {
                        merge_selected_summary_export(exports, summary, selected_summary)?;
                    }
                }
                for row in &summary.exported_effect_rows {
                    if let Some(selected_summary) = sanitized_effect_row_semantic_summary(
                        summary,
                        &row.exported_name,
                        &row.exported_name,
                        ash_core::semantic_summary::EffectRowBindingExposure::PublicReExport,
                    )? {
                        merge_selected_summary_export(exports, summary, selected_summary)?;
                    }
                }
                for value in &summary.exported_values {
                    if let Some(selected_summary) = selected_value_semantic_summary(
                        summary,
                        &value.exported_name,
                        &value.exported_name,
                    ) {
                        merge_selected_summary_export(exports, summary, selected_summary)?;
                    }
                }
            }
        }
        UsePath::Simple(path) => {
            let name = path
                .segments
                .last()
                .map(std::string::ToString::to_string)
                .ok_or_else(|| EngineError::Parse("empty use path".to_string()))?;
            let has_alias = use_stmt.alias.is_some();
            let exported_name = use_stmt
                .alias
                .map_or_else(|| name.clone(), |alias| alias.to_string());
            if let Some(type_def) = target_exports.type_defs.get(&name) {
                insert_type_export_with_name(
                    exports,
                    &exported_name,
                    type_def_with_visible_name(type_def, &exported_name),
                )?;
                if let Some(summary) = target_semantic_summary.as_ref() {
                    merge_type_summary_export(exports, summary, &name, &exported_name)?;
                }
            } else if let Some(type_def) = target_exports.constructor_defs.get(&name) {
                if has_alias {
                    return Err(constructor_alias_error(&name));
                }
                insert_constructor_export_with_name(exports, &exported_name, type_def.clone())?;
                if let Some(summary) = target_semantic_summary.as_ref() {
                    merge_constructor_summary_export(exports, summary, &name)?;
                }
            } else if let Some(callable) = target_exports.callables.get(&name) {
                merge_callable_signature_summaries(
                    exports,
                    target_semantic_summary.as_ref(),
                    callable,
                )?;
                let mut callable = callable.clone();
                rewrite_callable_signature_aliases(
                    &mut callable,
                    target_semantic_summary.as_ref(),
                    exports.semantic_summary.as_ref(),
                );
                callable.exported_name.clone_from(&exported_name);
                stamp_callable_export_module(&mut callable, exports.semantic_summary.as_ref());
                insert_callable_export(exports, &exported_name, callable)?;
            } else if let Some(type_function) = target_exports.type_function_summaries.get(&name) {
                if let Some((summary, selected_summary)) =
                    target_semantic_summary.as_ref().and_then(|summary| {
                        selected_type_function_semantic_summary(summary, &name, &exported_name)
                            .map(|selected| (summary, selected))
                    })
                {
                    merge_selected_summary_export(exports, summary, selected_summary)?;
                }
                let mut type_function = type_function.clone();
                type_function.exported_name.clone_from(&exported_name);
                insert_type_function_export(exports, &exported_name, type_function)?;
            } else if let Some(family) = target_exports.associated_family_summaries.get(&name) {
                if let Some((summary, selected_summary)) =
                    target_semantic_summary.as_ref().and_then(|summary| {
                        selected_associated_family_semantic_summary(summary, &name, &exported_name)
                            .map(|selected| (summary, selected))
                    })
                {
                    merge_selected_summary_export(exports, summary, selected_summary)?;
                }
                let mut family = family.clone();
                family.visible_name.clone_from(&exported_name);
                insert_associated_family_export(exports, &exported_name, family)?;
            } else if let Some((summary, selected_summary)) =
                target_semantic_summary.as_ref().and_then(|summary| {
                    selected_interface_semantic_summary(summary, &name, &exported_name)
                        .map(|selected| (summary, selected))
                })
            {
                merge_selected_summary_export(exports, summary, selected_summary)?;
            } else if let Some(summary) = target_semantic_summary.as_ref() {
                if let Some(selected_summary) = sanitized_effect_row_semantic_summary(
                    summary,
                    &name,
                    &exported_name,
                    ash_core::semantic_summary::EffectRowBindingExposure::PublicReExport,
                )? {
                    merge_selected_summary_export(exports, summary, selected_summary)?;
                } else if let Some(selected_summary) =
                    selected_value_semantic_summary(summary, &name, &exported_name)
                {
                    merge_selected_summary_export(exports, summary, selected_summary)?;
                } else {
                    return Err(missing_pub_use_target_error(&name));
                }
            } else {
                return Err(missing_pub_use_target_error(&name));
            }
        }
        UsePath::Nested(_, items) => {
            // Type aliases in a grouped pub-use are made available before callable
            // signatures are rewritten so `pub use inner::{keep as preserve,
            // Token as PublicToken};` behaves the same as the reverse order.
            let grouped_type_aliases = items
                .iter()
                .filter(|item| target_exports.type_defs.contains_key(item.name.as_ref()))
                .map(|item| {
                    (
                        item.name.as_ref().to_string(),
                        item.alias.as_ref().map_or_else(
                            || item.name.to_string(),
                            std::string::ToString::to_string,
                        ),
                    )
                })
                .collect::<HashMap<_, _>>();
            for item in &items {
                let exported_name = item
                    .alias
                    .as_ref()
                    .map_or_else(|| item.name.to_string(), std::string::ToString::to_string);
                if let Some(type_def) = target_exports.type_defs.get(item.name.as_ref()) {
                    insert_type_export_with_name(
                        exports,
                        &exported_name,
                        type_def_with_visible_name_and_aliases(
                            type_def,
                            &exported_name,
                            &grouped_type_aliases,
                        ),
                    )?;
                    if let Some(summary) = target_semantic_summary.as_ref() {
                        merge_type_summary_export_with_aliases(
                            exports,
                            summary,
                            item.name.as_ref(),
                            &exported_name,
                            &grouped_type_aliases,
                        )?;
                    }
                } else if target_exports
                    .constructor_defs
                    .contains_key(item.name.as_ref())
                {
                    if item.alias.is_some() {
                        return Err(constructor_alias_error(item.name.as_ref()));
                    }
                } else if !target_exports.callables.contains_key(item.name.as_ref())
                    && !target_exports
                        .type_function_summaries
                        .contains_key(item.name.as_ref())
                    && !target_exports
                        .associated_family_summaries
                        .contains_key(item.name.as_ref())
                    && !semantic_summary_has_interface(
                        target_semantic_summary.as_ref(),
                        item.name.as_ref(),
                    )
                    && !target_semantic_summary.as_ref().is_some_and(|summary| {
                        summary
                            .exported_effect_rows
                            .iter()
                            .any(|row| row.exported_name == item.name.as_ref())
                            || selected_value_semantic_summary(
                                summary,
                                item.name.as_ref(),
                                item.name.as_ref(),
                            )
                            .is_some()
                    })
                {
                    return Err(missing_pub_use_target_error(item.name.as_ref()));
                }
            }

            for item in items {
                let exported_name = item
                    .alias
                    .as_ref()
                    .map_or_else(|| item.name.to_string(), std::string::ToString::to_string);
                if target_exports.type_defs.contains_key(item.name.as_ref()) {
                    continue;
                }
                if let Some(type_def) = target_exports.constructor_defs.get(item.name.as_ref()) {
                    insert_constructor_export_with_name(exports, &exported_name, type_def.clone())?;
                    if let Some(summary) = target_semantic_summary.as_ref() {
                        merge_constructor_summary_export(exports, summary, item.name.as_ref())?;
                    }
                } else if let Some(callable) = target_exports.callables.get(item.name.as_ref()) {
                    merge_callable_signature_summaries(
                        exports,
                        target_semantic_summary.as_ref(),
                        callable,
                    )?;
                    let mut callable = callable.clone();
                    rewrite_callable_signature_aliases(
                        &mut callable,
                        target_semantic_summary.as_ref(),
                        exports.semantic_summary.as_ref(),
                    );
                    callable.exported_name.clone_from(&exported_name);
                    stamp_callable_export_module(&mut callable, exports.semantic_summary.as_ref());
                    insert_callable_export(exports, &exported_name, callable)?;
                } else if let Some(type_function) = target_exports
                    .type_function_summaries
                    .get(item.name.as_ref())
                {
                    if let Some((summary, selected_summary)) =
                        target_semantic_summary.as_ref().and_then(|summary| {
                            selected_type_function_semantic_summary(
                                summary,
                                item.name.as_ref(),
                                &exported_name,
                            )
                            .map(|selected| (summary, selected))
                        })
                    {
                        merge_selected_summary_export(exports, summary, selected_summary)?;
                    }
                    let mut type_function = type_function.clone();
                    type_function.exported_name.clone_from(&exported_name);
                    insert_type_function_export(exports, &exported_name, type_function)?;
                } else if let Some(family) = target_exports
                    .associated_family_summaries
                    .get(item.name.as_ref())
                {
                    if let Some((summary, selected_summary)) =
                        target_semantic_summary.as_ref().and_then(|summary| {
                            selected_associated_family_semantic_summary(
                                summary,
                                item.name.as_ref(),
                                &exported_name,
                            )
                            .map(|selected| (summary, selected))
                        })
                    {
                        merge_selected_summary_export(exports, summary, selected_summary)?;
                    }
                    let mut family = family.clone();
                    family.visible_name.clone_from(&exported_name);
                    insert_associated_family_export(exports, &exported_name, family)?;
                } else if let Some((summary, selected_summary)) =
                    target_semantic_summary.as_ref().and_then(|summary| {
                        selected_interface_semantic_summary(
                            summary,
                            item.name.as_ref(),
                            &exported_name,
                        )
                        .map(|selected| (summary, selected))
                    })
                {
                    merge_selected_summary_export(exports, summary, selected_summary)?;
                } else if let Some(summary) = target_semantic_summary.as_ref() {
                    if let Some(selected_summary) = sanitized_effect_row_semantic_summary(
                        summary,
                        item.name.as_ref(),
                        &exported_name,
                        ash_core::semantic_summary::EffectRowBindingExposure::PublicReExport,
                    )? {
                        merge_selected_summary_export(exports, summary, selected_summary)?;
                    } else if let Some(selected_summary) =
                        selected_value_semantic_summary(summary, item.name.as_ref(), &exported_name)
                    {
                        merge_selected_summary_export(exports, summary, selected_summary)?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn insert_type_export(
    exports: &mut ModuleExports,
    type_def: &CoreTypeDef,
) -> Result<(), EngineError> {
    // Public ordinary types export their representation. Non-public explicit
    // builtin substrate types may still export an opaque identity so public
    // callable signatures can name existing runtime-managed handles without
    // exposing constructors or representations. Non-public ordinary types are
    // not exported/importable downstream.
    if matches!(type_def.visibility, CoreVisibility::Public) {
        insert_type_export_with_name(exports, &type_def.name, type_def.clone())?;
    } else if type_def.builtin {
        let mut opaque = type_def.clone();
        opaque.body = CoreTypeBody::Struct(vec![]);
        opaque.builtin = true;
        insert_type_export_with_name(exports, &type_def.name, opaque)?;
    } else {
        return Ok(());
    }

    // Only public type definitions expose constructor names/representation to
    // importing modules.
    if let (CoreVisibility::Public, CoreTypeBody::Enum(variants)) =
        (&type_def.visibility, &type_def.body)
    {
        for variant_name in variants.iter().map(|variant| variant.name.clone()) {
            if variant_name == type_def.name {
                continue;
            }
            insert_constructor_export_with_name(exports, &variant_name, type_def.clone())?;
        }
    }
    Ok(())
}

fn missing_pub_use_target_error(name: &str) -> EngineError {
    EngineError::Parse(format!(
        "pub use target '{name}' not found in re-exported module"
    ))
}

fn attach_public_macro_summaries(
    exports: &mut ModuleExports,
    module: &ash_parser::surface::ModuleFile,
    path: &Path,
) -> Result<(), EngineError> {
    let module_path = module_path_text(path).to_string();
    let summaries = ash_parser::surface::collect_public_macro_summaries(module, module_path)
        .map_err(|error| {
            EngineError::Parse(format!(
                "in '{}': invalid public macro summary: {error}",
                path.display()
            ))
        })?;
    let templates = ash_parser::surface::build_local_macro_table(module).map_err(|error| {
        EngineError::Parse(format!(
            "in '{}': invalid public macro template table: {error}",
            path.display()
        ))
    })?;
    for summary in summaries {
        let name = summary.name.to_string();
        let template = templates.resolve(&name).ok_or_else(|| {
            EngineError::Parse(format!(
                "in '{}': public macro summary '{name}' has no template",
                path.display()
            ))
        })?;
        insert_macro_summary_export(exports, summary, template.clone())?;
    }
    Ok(())
}

fn insert_macro_summary_export(
    exports: &mut ModuleExports,
    summary: MacroSummary,
    template: LocalMacroEntry,
) -> Result<(), EngineError> {
    let name = summary.name.to_string();
    validate_macro_summary_template(&summary, &template)?;
    if exports
        .macro_summaries
        .insert(name.clone(), summary)
        .is_some()
    {
        return Err(EngineError::Parse(format!(
            "duplicate public macro summary '{name}'"
        )));
    }
    if exports
        .macro_templates
        .insert(name.clone(), template)
        .is_some()
    {
        return Err(EngineError::Parse(format!(
            "duplicate public macro template '{name}'"
        )));
    }
    Ok(())
}

fn validate_macro_summary_template(
    summary: &MacroSummary,
    template: &LocalMacroEntry,
) -> Result<(), EngineError> {
    let name = summary.name.as_ref();
    if summary.name != template.name {
        return Err(EngineError::Parse(format!(
            "macro summary '{name}' names template '{}'",
            template.name
        )));
    }
    if summary.identity.local_name != summary.name
        || summary.identity.param_count != summary.params.len()
        || summary.identity.origin_span != summary.origin_span
    {
        return Err(EngineError::Parse(format!(
            "macro summary '{name}' identity does not match exported macro shape"
        )));
    }
    match &summary.identity.origin {
        MacroIdentityOrigin::Imported {
            module_path,
            exported_name,
        } if module_path == &summary.module_path && exported_name == &summary.name => {}
        _ => {
            return Err(EngineError::Parse(format!(
                "macro summary '{name}' identity does not match exported macro origin"
            )));
        }
    }
    if summary.params != template.params {
        return Err(EngineError::Parse(format!(
            "macro summary '{name}' parameter list does not match template"
        )));
    }
    if summary.template_fingerprint.param_count != template.params.len()
        || summary.template_fingerprint.body_span != template.body.span()
    {
        return Err(EngineError::Parse(format!(
            "macro summary '{name}' template fingerprint does not match template"
        )));
    }
    if summary.typed_signature != template.typed_signature {
        return Err(EngineError::Parse(format!(
            "macro summary '{name}' typed signature does not match template"
        )));
    }
    Ok(())
}

fn exportable_module_semantic_summary(
    raw: &ModuleSemanticSummary,
    exportable_types: &HashMap<String, CoreTypeDef>,
    opaque_public_signature_type_names: &HashSet<String>,
) -> Result<ModuleSemanticSummary, EngineError> {
    let mut summary = raw.clone();
    summary.exported_types = raw
        .exported_types
        .iter()
        .filter_map(|ty| {
            exportable_type_summary(ty, exportable_types).or_else(|| {
                opaque_public_signature_type_names
                    .contains(&ty.exported_name)
                    .then(|| opaque_public_signature_type_summary(ty))
            })
        })
        .collect();
    summary.exported_constructors = raw
        .exported_constructors
        .iter()
        .filter(|constructor| exportable_types.contains_key(constructor.parent.name.as_str()))
        .cloned()
        .collect();
    prepare_public_effect_row_exports_v7(raw, &mut summary)?;
    summary.exported_values = raw
        .exported_values
        .iter()
        .filter(|value| matches!(value.visibility, CoreVisibility::Public))
        .cloned()
        .collect();
    // Only export sealed domains with public visibility; private/crate domains
    // must not leak through the module export boundary. Public domain fields
    // also must not reference domains outside that public export set.
    let public_domain_names = raw
        .exported_sealed_domains
        .iter()
        .filter(|domain| matches!(domain.visibility, CoreVisibility::Public))
        .map(|domain| domain.exported_name.clone())
        .collect::<HashSet<_>>();
    summary.exported_sealed_domains = raw
        .exported_sealed_domains
        .iter()
        .filter(|domain| matches!(domain.visibility, CoreVisibility::Public))
        .cloned()
        .collect();
    for domain in &summary.exported_sealed_domains {
        for constructor in &domain.constructors {
            for field in &constructor.fields {
                let Some(constraint) = field.domain_constraint.as_ref() else {
                    continue;
                };
                if !public_domain_names.contains(constraint.name.as_str()) {
                    return Err(EngineError::Parse(format!(
                        "public sealed domain '{}' constructor '{}' field '{}' references non-exportable sealed domain '{}'",
                        domain.exported_name,
                        constructor.exported_name,
                        field.name,
                        constraint.name
                    )));
                }
            }
        }
    }
    Ok(summary)
}

/// Filter raw effect-row declarations to the public summary boundary and add
/// the V7 provider-binding closure evidence required by import sanitization.
///
/// This operates before the summary reaches either an importer or a cache, so
/// a public row with an inaccessible dependency fails without transporting a
/// private name or an opaque substitute.
fn prepare_public_effect_row_exports_v7(
    raw: &ModuleSemanticSummary,
    summary: &mut ModuleSemanticSummary,
) -> Result<(), EngineError> {
    summary.exported_effect_rows = raw
        .exported_effect_rows
        .iter()
        .filter(|row| matches!(row.visibility, CoreVisibility::Public))
        .cloned()
        .collect();
    if summary.exported_effect_rows.is_empty() {
        return Ok(());
    }

    if let Some(row) = summary.exported_effect_rows.iter().find(|row| {
        !transitive_public_effect_row_dependency_closure(summary, row.exported_name.as_str())
            .inaccessible_by_row
            .is_empty()
    }) {
        return Err(EngineError::Parse(format!(
            "private-dependency-export-failure: public effect-row export '{}' has an inaccessible dependency",
            row.exported_name
        )));
    }

    summary.version = SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7;
    let public_closure_digests = summary
        .exported_effect_rows
        .iter()
        .map(|row| {
            (
                row.provider.clone(),
                effect_row_public_closure_digest(
                    summary,
                    &transitive_public_effect_row_dependency_closure(
                        summary,
                        row.exported_name.as_str(),
                    )
                    .public_providers,
                ),
            )
        })
        .collect::<HashMap<_, _>>();
    for row in &mut summary.exported_effect_rows {
        row.closure_metadata = Some(ash_core::semantic_summary::EffectRowClosureMetadata {
            sanitizer_schema_version:
                ash_core::semantic_summary::EFFECT_ROW_SANITIZER_SCHEMA_VERSION,
            public_closure_digest: public_closure_digests
                .get(&row.provider)
                .cloned()
                .unwrap_or_default(),
        });
    }
    Ok(())
}

fn public_callable_opaque_type_names(source: &str, type_defs: &[CoreTypeDef]) -> HashSet<String> {
    let private_ordinary_types = private_ordinary_type_names(type_defs);
    let mut names = HashSet::new();
    for callable in public_callable_signatures(source) {
        names.extend(
            callable_signature_type_names(&callable)
                .into_iter()
                .filter(|name| private_ordinary_types.contains(name)),
        );
    }
    names
}

fn opaque_public_signature_type_summary(summary: &TypeDeclSummary) -> TypeDeclSummary {
    let mut opaque = summary.clone();
    opaque.visibility = CoreVisibility::Public;
    opaque.representation_exposure = RepresentationExposure::Opaque;
    opaque.representation = TypeRepresentationSummary::opaque(false);
    opaque
}

fn attach_public_type_function_summaries(
    exports: &mut ModuleExports,
    type_metadata: &ash_parser::lower::LoweredTypeMetadata,
    path: &Path,
) -> Result<(), EngineError> {
    let has_public_type_function = type_metadata
        .type_function_defs
        .iter()
        .any(|def| matches!(def.visibility, ash_parser::surface::Visibility::Public));
    if !has_public_type_function {
        return Ok(());
    }
    let Some(summary) = exports.semantic_summary.as_mut() else {
        return Ok(());
    };

    let mut type_env = ash_typeck::TypeEnv::with_builtin_types();
    type_env
        .register_module_semantic_summary(summary)
        .map_err(|error| {
            EngineError::Parse(format!(
                "public type-function summary substrate registration failed: {error}"
            ))
        })?;
    for type_def in &type_metadata.type_defs {
        if !matches!(type_def.visibility, CoreVisibility::Public) {
            type_env.register_type(type_def).map_err(|error| {
                EngineError::Parse(format!(
                    "public type-function private-type substrate registration failed: {error}"
                ))
            })?;
        }
    }
    for domain in &type_metadata.summary.exported_sealed_domains {
        if !matches!(domain.visibility, CoreVisibility::Public) {
            type_env
                .register_local_sealed_domain_summary(domain)
                .map_err(|error| {
                    EngineError::Parse(format!(
                        "public type-function private-domain substrate registration failed: {error}"
                    ))
                })?;
        }
    }
    type_env
        .register_local_type_functions(&summary.module, &type_metadata.type_function_defs)
        .map_err(|error| {
            EngineError::Parse(format!(
                "in '{}': public type-function export validation failed before downstream use/reduction: {error}; span {:?}",
                path.display(),
                type_env_error_span(&error)
            ))
        })?;
    let type_function_summaries = type_env
        .export_public_type_function_summaries(&summary.module)
        .map_err(|error| {
            EngineError::Parse(format!(
                "public type-function summary export failed: {error}"
            ))
        })?;

    if type_function_summaries.is_empty() {
        return Ok(());
    }

    summary.version = SummaryVersion::SPEC062_TYPE_COMPUTATION_V3;
    summary
        .exported_type_functions
        .clone_from(&type_function_summaries);
    exports.type_function_summaries = type_function_summaries
        .into_iter()
        .map(|type_function| (type_function.exported_name.clone(), type_function))
        .collect();
    Ok(())
}

fn module_has_public_associated_family(module: &ash_parser::surface::ModuleFile) -> bool {
    module.definitions.iter().any(|definition| {
        let Definition::Interface(interface) = definition else {
            return false;
        };
        matches!(
            interface.visibility,
            ash_parser::surface::Visibility::Public
        ) && interface.associated_types.iter().any(|associated| {
            matches!(
                associated.kind,
                ash_parser::surface::AssociatedTypeKind::SealedFamily { .. }
            )
        })
    })
}

fn attach_public_associated_family_summaries(
    exports: &mut ModuleExports,
    type_metadata: &ash_parser::lower::LoweredTypeMetadata,
    path: &Path,
    source: &str,
) -> Result<(), EngineError> {
    let metadata_source = strip_module_metadata_non_definition_lines(source);
    let module = parse_module_file_for_type_metadata(path, &metadata_source)?;
    if !module_has_public_associated_family(&module) {
        return Ok(());
    }
    let Some(summary) = exports.semantic_summary.as_mut() else {
        return Ok(());
    };

    let local_public_interface_names = local_public_interface_names(&module);
    let directly_visible_imported_interfaces =
        directly_visible_imported_interface_names(path, source)?;
    validate_local_interface_constraint_visibility(
        path,
        &module,
        &local_public_interface_names,
        &directly_visible_imported_interfaces,
    )?;

    let mut type_env = ash_typeck::TypeEnv::with_builtin_types();
    register_imported_interface_definitions_for_constraints(&mut type_env, path, source)?;
    type_env.set_current_module_identity(summary.module.clone());
    for definition in &module.definitions {
        if let Definition::Interface(interface) = definition {
            type_env.register_interface(interface).map_err(|error| {
                EngineError::Parse(format!(
                    "in '{}': public associated-family interface registration failed: {error}; span {:?}",
                    path.display(),
                    type_env_error_span(&error)
                ))
            })?;
        }
    }
    type_env
        .register_module_semantic_summary(summary)
        .map_err(|error| {
            EngineError::Parse(format!(
                "public associated-family summary substrate registration failed: {error}"
            ))
        })?;
    for type_def in &type_metadata.type_defs {
        if !matches!(type_def.visibility, CoreVisibility::Public) {
            type_env.register_type(type_def).map_err(|error| {
                EngineError::Parse(format!(
                    "public associated-family private-type substrate registration failed: {error}"
                ))
            })?;
        }
    }
    for domain in &type_metadata.summary.exported_sealed_domains {
        if !matches!(domain.visibility, CoreVisibility::Public) {
            type_env
                .register_local_sealed_domain_summary(domain)
                .map_err(|error| {
                    EngineError::Parse(format!(
                        "public associated-family private-domain substrate registration failed: {error}"
                    ))
                })?;
        }
    }

    for definition in &module.definitions {
        if let Definition::Impl(impl_def) = definition {
            type_env.register_impl(impl_def).map_err(|error| {
                EngineError::Parse(format!(
                    "in '{}': public associated-family impl export validation failed: {error}; span {:?}",
                    path.display(),
                    type_env_error_span(&error)
                ))
            })?;
        }
    }

    let associated_family_summaries = type_env
        .export_public_associated_family_summaries(&summary.module)
        .map_err(|error| {
            EngineError::Parse(format!(
                "public associated-family summary export failed: {error}"
            ))
        })?;
    if associated_family_summaries.is_empty() {
        return Ok(());
    }

    summary.version = SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4;
    summary
        .exported_associated_families
        .clone_from(&associated_family_summaries);
    exports.associated_family_summaries = associated_family_summaries
        .into_iter()
        .filter(|family| !is_dependency_metadata_name(&family.visible_name))
        .map(|family| (family.visible_name.clone(), family))
        .collect();
    Ok(())
}

fn register_imported_interface_definitions_for_constraints(
    type_env: &mut ash_typeck::TypeEnv,
    path: &Path,
    source: &str,
) -> Result<(), EngineError> {
    let mut visiting = HashSet::new();
    register_imported_interface_definitions_for_constraints_inner(
        type_env,
        path,
        source,
        &mut visiting,
    )
}

#[allow(clippy::too_many_lines)]
fn register_imported_interface_definitions_for_constraints_inner(
    type_env: &mut ash_typeck::TypeEnv,
    path: &Path,
    source: &str,
    visiting: &mut HashSet<PathBuf>,
) -> Result<(), EngineError> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !visiting.insert(canonical.clone()) {
        return Ok(());
    }

    let module_root = path.parent().ok_or_else(|| {
        EngineError::Configuration(format!("module path '{}' has no parent", path.display()))
    })?;
    let crate_root = discover_crate_root(module_root);
    for snippet in extract_import_snippets(source) {
        let import_spec = parse_ordinary_import(snippet.trim())?;
        let (module_segments, search_roots) = import_resolution_roots(
            &import_spec.module_segments,
            module_root,
            crate_root.as_deref(),
        )?;
        let Some(target_path) = resolve_module_path(&module_segments, &search_roots)? else {
            continue;
        };
        let target_canonical = target_path
            .canonicalize()
            .unwrap_or_else(|_| target_path.clone());
        if visiting.contains(&target_canonical) {
            return Err(EngineError::Parse(format!(
                "cyclic import detected while loading '{}'",
                target_path.display()
            )));
        }
        let target_source = std::fs::read_to_string(&target_path)?;
        register_imported_interface_definitions_for_constraints_inner(
            type_env,
            &target_path,
            &target_source,
            visiting,
        )?;
        let target_metadata_source = strip_module_metadata_non_definition_lines(&target_source);
        let target_module =
            parse_module_file_for_type_metadata(&target_path, &target_metadata_source)?;
        let target_type_metadata =
            collect_module_type_metadata_from_module_file(&target_path, &target_source)?;
        for type_def in &target_type_metadata.type_defs {
            if !type_env.has_type(&type_def.name) {
                type_env.declare_type_name(&type_def.name);
            }
        }
        for type_def in target_type_metadata.type_defs {
            if type_env.has_full_type(&type_def.name)
                || type_env.type_identity_for_name(&type_def.name).is_some()
            {
                continue;
            }
            let _ = type_env.register_type_identity(&type_def);
            if matches!(type_def.visibility, ash_core::ast::Visibility::Public) {
                let _ = type_env.expose_type_representation(&type_def.name);
            }
        }
        type_env.set_current_module_identity(module_identity_for_path(&target_path));
        for selection in &import_spec.selections {
            match selection {
                ImportSelection::Glob => {
                    for definition in &target_module.definitions {
                        let Definition::Interface(interface) = definition else {
                            continue;
                        };
                        register_public_imported_interface_for_constraints(
                            type_env,
                            path,
                            &target_path,
                            interface,
                        )?;
                    }
                }
                ImportSelection::Named { name, alias } => {
                    for definition in &target_module.definitions {
                        let Definition::Interface(interface) = definition else {
                            continue;
                        };
                        if interface.name.as_ref() != name {
                            continue;
                        }
                        if let Some(alias) = alias {
                            let mut interface = interface.clone();
                            interface.name = alias.as_str().into();
                            register_public_imported_interface_for_constraints(
                                type_env,
                                path,
                                &target_path,
                                &interface,
                            )?;
                        } else {
                            register_public_imported_interface_for_constraints(
                                type_env,
                                path,
                                &target_path,
                                interface,
                            )?;
                        }
                    }
                }
            }
        }
    }
    visiting.remove(&canonical);
    Ok(())
}

fn register_public_imported_interface_for_constraints(
    type_env: &mut ash_typeck::TypeEnv,
    importing_path: &Path,
    target_path: &Path,
    interface: &InterfaceDef,
) -> Result<(), EngineError> {
    if !matches!(
        interface.visibility,
        ash_parser::surface::Visibility::Public
    ) || type_env.lookup_interface(interface.name.as_ref()).is_some()
    {
        return Ok(());
    }
    type_env.register_interface(interface).map_err(|error| {
        EngineError::Parse(format!(
            "in '{}': public interface imported constraint substrate registration failed for '{}': {error}; span {:?}",
            importing_path.display(),
            target_path.display(),
            type_env_error_span(&error)
        ))
    })
}

fn local_public_interface_names(module: &ash_parser::surface::ModuleFile) -> HashSet<String> {
    module
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            Definition::Interface(interface)
                if matches!(
                    interface.visibility,
                    ash_parser::surface::Visibility::Public
                ) =>
            {
                Some(interface.name.to_string())
            }
            _ => None,
        })
        .collect()
}

fn validate_local_interface_constraint_visibility(
    path: &Path,
    module: &ash_parser::surface::ModuleFile,
    local_public_interface_names: &HashSet<String>,
    directly_visible_imported_interfaces: &HashSet<String>,
) -> Result<(), EngineError> {
    for definition in &module.definitions {
        let Definition::Interface(interface) = definition else {
            continue;
        };
        if !matches!(
            interface.visibility,
            ash_parser::surface::Visibility::Public
        ) {
            continue;
        }
        for constraint in &interface.evidence_constraints {
            let Some(required_interface) =
                interface_constraint_required_name_for_loader(&constraint.interface)
            else {
                continue;
            };
            if local_public_interface_names.contains(required_interface)
                || directly_visible_imported_interfaces.contains(required_interface)
            {
                continue;
            }
            return Err(EngineError::Parse(format!(
                "in '{}': interface '{}' evidence constraint requires interface '{}' that is not locally declared or directly imported",
                path.display(),
                interface.name,
                required_interface
            )));
        }
    }
    Ok(())
}

fn directly_visible_imported_interface_names(
    path: &Path,
    source: &str,
) -> Result<HashSet<String>, EngineError> {
    let module_root = path.parent().ok_or_else(|| {
        EngineError::Configuration(format!("module path '{}' has no parent", path.display()))
    })?;
    let crate_root = discover_crate_root(module_root);
    let mut names = HashSet::new();
    for snippet in extract_import_snippets(source) {
        let import_spec = parse_ordinary_import(snippet.trim())?;
        let (module_segments, search_roots) = import_resolution_roots(
            &import_spec.module_segments,
            module_root,
            crate_root.as_deref(),
        )?;
        let Some(target_path) = resolve_module_path(&module_segments, &search_roots)? else {
            continue;
        };
        let target_source = std::fs::read_to_string(&target_path)?;
        let target_source = strip_module_metadata_non_definition_lines(&target_source);
        let target_module = parse_module_file_for_type_metadata(&target_path, &target_source)?;
        for selection in &import_spec.selections {
            match selection {
                ImportSelection::Glob => {
                    for definition in &target_module.definitions {
                        let Definition::Interface(interface) = definition else {
                            continue;
                        };
                        if matches!(
                            interface.visibility,
                            ash_parser::surface::Visibility::Public
                        ) {
                            names.insert(interface.name.to_string());
                        }
                    }
                }
                ImportSelection::Named { name, alias } => {
                    for definition in &target_module.definitions {
                        let Definition::Interface(interface) = definition else {
                            continue;
                        };
                        if interface.name.as_ref() == name
                            && matches!(
                                interface.visibility,
                                ash_parser::surface::Visibility::Public
                            )
                        {
                            names.insert(alias.clone().unwrap_or_else(|| name.clone()));
                        }
                    }
                }
            }
        }
    }
    Ok(names)
}

fn interface_constraint_required_name_for_loader(ty: &ash_parser::surface::Type) -> Option<&str> {
    match ty {
        ash_parser::surface::Type::Name(name) => Some(name.as_ref()),
        ash_parser::surface::Type::Constructor { name, args } if args.is_empty() => {
            Some(name.as_ref())
        }
        _ => None,
    }
}

fn attach_public_interface_identity_summaries(
    exports: &mut ModuleExports,
    path: &Path,
    source: &str,
) -> Result<(), EngineError> {
    let metadata_source = strip_module_metadata_non_definition_lines(source);
    let module = parse_module_file_for_type_metadata(path, &metadata_source)?;
    let has_public_interface = module.definitions.iter().any(|definition| {
        matches!(
            definition,
            Definition::Interface(interface)
                if matches!(interface.visibility, ash_parser::surface::Visibility::Public)
        )
    });
    if !has_public_interface {
        return Ok(());
    }
    let Some(summary) = exports.semantic_summary.as_mut() else {
        return Ok(());
    };

    let source_origin = ash_core::semantic_summary::SourceOrigin::File(path.display().to_string());
    let local_public_interface_names = local_public_interface_names(&module);
    let directly_visible_imported_interfaces =
        directly_visible_imported_interface_names(path, source)?;
    validate_local_interface_constraint_visibility(
        path,
        &module,
        &local_public_interface_names,
        &directly_visible_imported_interfaces,
    )?;
    let mut type_env = ash_typeck::TypeEnv::with_builtin_types();
    register_imported_interface_definitions_for_constraints(&mut type_env, path, source)?;
    type_env.set_current_module_identity(summary.module.clone());
    for definition in &module.definitions {
        if let Definition::Interface(interface) = definition {
            type_env.register_interface(interface).map_err(|error| {
                EngineError::Parse(format!(
                    "in '{}': public interface identity registration failed: {error}; span {:?}",
                    path.display(),
                    type_env_error_span(&error)
                ))
            })?;
        }
    }

    for interface in collect_public_interface_identity_summaries(&type_env, &module, &source_origin)
    {
        if !summary
            .interface_identities
            .iter()
            .any(|existing| existing.id == interface.id)
        {
            summary.interface_identities.push(interface);
        }
    }
    Ok(())
}

fn attach_public_proposition_summaries(
    exports: &mut ModuleExports,
    type_metadata: &ash_parser::lower::LoweredTypeMetadata,
    path: &Path,
    source: &str,
) -> Result<(), EngineError> {
    let metadata_source = strip_module_metadata_non_definition_lines(source);
    let module = parse_module_file_for_type_metadata(path, &metadata_source)?;
    if !module_has_public_proposition_surface(&module) {
        return Ok(());
    }

    let Some(summary) = exports.semantic_summary.as_mut() else {
        return Ok(());
    };
    let source_origin = ash_core::semantic_summary::SourceOrigin::File(path.display().to_string());
    let mut type_env = ash_typeck::TypeEnv::with_builtin_types();
    seed_public_proposition_type_env(&mut type_env, type_metadata, summary, &module, path)?;

    let public_predicates =
        collect_public_proposition_predicate_summaries(&mut type_env, &module, path)?;
    add_public_proposition_obligations(&mut type_env, &module, &source_origin, path)?;
    type_env
        .discharge_required_proposition_obligations()
        .map_err(|error| {
            EngineError::Parse(format!(
                "in '{}': public proposition checking point failed: {error}",
                path.display()
            ))
        })?;
    let public_interface_identities =
        collect_public_interface_identity_summaries(&type_env, &module, &source_origin);
    let proposition_facts = type_env
        .export_public_proposition_fact_summaries(&summary.module)
        .map_err(|error| {
            EngineError::Parse(format!("public proposition summary export failed: {error}"))
        })?;
    if public_predicates.is_empty() && proposition_facts.is_empty() {
        return Ok(());
    }

    attach_exported_proposition_payload(
        summary,
        public_interface_identities,
        public_predicates,
        proposition_facts,
    );
    Ok(())
}

fn module_has_public_proposition_surface(module: &ash_parser::surface::ModuleFile) -> bool {
    module
        .definitions
        .iter()
        .any(|definition| match definition {
            Definition::PropositionPredicate(predicate) => {
                matches!(
                    predicate.visibility,
                    ash_parser::surface::Visibility::Public
                )
            }
            Definition::Function(function) => {
                matches!(function.visibility, ash_parser::surface::Visibility::Public)
                    && function.proposition_tail.is_some()
            }
            Definition::BuiltinFn(function) => {
                matches!(function.visibility, ash_parser::surface::Visibility::Public)
                    && function.proposition_tail.is_some()
            }
            Definition::TypeFn(type_fn) => {
                matches!(type_fn.visibility, ash_parser::surface::Visibility::Public)
                    && type_fn.proposition_tail.is_some()
            }
            _ => false,
        })
}

fn seed_public_proposition_type_env(
    type_env: &mut ash_typeck::TypeEnv,
    type_metadata: &ash_parser::lower::LoweredTypeMetadata,
    summary: &ModuleSemanticSummary,
    module: &ash_parser::surface::ModuleFile,
    path: &Path,
) -> Result<(), EngineError> {
    type_env.set_current_module_identity(summary.module.clone());
    type_env
        .register_module_semantic_summary(summary)
        .map_err(|error| {
            EngineError::Parse(format!(
                "public proposition summary substrate registration failed: {error}"
            ))
        })?;
    register_private_type_metadata_for_propositions(type_env, type_metadata)?;
    for definition in &module.definitions {
        if let Definition::Interface(interface) = definition {
            type_env.register_interface(interface).map_err(|error| {
                EngineError::Parse(format!(
                    "in '{}': public proposition interface substrate registration failed: {error}; span {:?}",
                    path.display(),
                    type_env_error_span(&error)
                ))
            })?;
        }
    }
    Ok(())
}

fn register_private_type_metadata_for_propositions(
    type_env: &mut ash_typeck::TypeEnv,
    type_metadata: &ash_parser::lower::LoweredTypeMetadata,
) -> Result<(), EngineError> {
    for type_def in &type_metadata.type_defs {
        if !matches!(type_def.visibility, CoreVisibility::Public) {
            type_env.register_type(type_def).map_err(|error| {
                EngineError::Parse(format!(
                    "public proposition private-type substrate registration failed: {error}"
                ))
            })?;
        }
    }
    for domain in &type_metadata.summary.exported_sealed_domains {
        if !matches!(domain.visibility, CoreVisibility::Public) {
            type_env
                .register_local_sealed_domain_summary(domain)
                .map_err(|error| {
                    EngineError::Parse(format!(
                        "public proposition private-domain substrate registration failed: {error}"
                    ))
                })?;
        }
    }
    Ok(())
}

fn collect_public_proposition_predicate_summaries(
    type_env: &mut ash_typeck::TypeEnv,
    module: &ash_parser::surface::ModuleFile,
    path: &Path,
) -> Result<Vec<ash_core::semantic_summary::PropositionPredicateSummary>, EngineError> {
    let mut public_predicates = Vec::new();
    for definition in &module.definitions {
        let Definition::PropositionPredicate(predicate) = definition else {
            continue;
        };
        let id = type_env
            .register_proposition_predicate_decl(predicate)
            .map_err(|error| {
                EngineError::Parse(format!(
                    "in '{}': public proposition predicate registration failed: {error}",
                    path.display()
                ))
            })?;
        if matches!(
            predicate.visibility,
            ash_parser::surface::Visibility::Public
        ) && let Some(info) = type_env.proposition_predicate_by_id(&id)
        {
            public_predicates.push(info.summary.clone());
        }
    }
    Ok(public_predicates)
}

fn add_public_proposition_obligations(
    type_env: &mut ash_typeck::TypeEnv,
    module: &ash_parser::surface::ModuleFile,
    source_origin: &ash_core::semantic_summary::SourceOrigin,
    path: &Path,
) -> Result<(), EngineError> {
    for (index, definition) in module.definitions.iter().enumerate() {
        add_public_proposition_obligation(
            type_env,
            definition,
            source_origin.clone(),
            index,
            path,
        )?;
    }
    Ok(())
}

fn add_public_proposition_obligation(
    type_env: &mut ash_typeck::TypeEnv,
    definition: &Definition,
    source_origin: ash_core::semantic_summary::SourceOrigin,
    index: usize,
    path: &Path,
) -> Result<(), EngineError> {
    let Some((tail, site_tag, label)) = public_definition_proposition_tail(definition) else {
        return Ok(());
    };
    type_env
        .add_proposition_obligations_from_tail(
            tail,
            source_origin,
            ash_typeck::type_env::PropositionCheckingSite::new(
                site_tag + index as u64,
                ash_typeck::type_env::PropositionCheckingSiteKind::ExplicitRequirement,
                Some(label),
            ),
        )
        .map_err(|error| {
            EngineError::Parse(format!(
                "in '{}': public proposition export failed: {error}",
                path.display()
            ))
        })
}

fn public_definition_proposition_tail(
    definition: &Definition,
) -> Option<(&ash_parser::surface::PropositionTail, u64, String)> {
    match definition {
        Definition::Function(function)
            if matches!(function.visibility, ash_parser::surface::Visibility::Public) =>
        {
            function.proposition_tail.as_ref().map(|tail| {
                (
                    tail,
                    0x8791_0000u64,
                    format!("public function {}", function.name),
                )
            })
        }
        Definition::BuiltinFn(function)
            if matches!(function.visibility, ash_parser::surface::Visibility::Public) =>
        {
            function.proposition_tail.as_ref().map(|tail| {
                (
                    tail,
                    0x8792_0000u64,
                    format!("public builtin function {}", function.name),
                )
            })
        }
        Definition::TypeFn(type_fn)
            if matches!(type_fn.visibility, ash_parser::surface::Visibility::Public) =>
        {
            type_fn.proposition_tail.as_ref().map(|tail| {
                (
                    tail,
                    0x8793_0000u64,
                    format!("public type function {}", type_fn.name),
                )
            })
        }
        _ => None,
    }
}

fn collect_public_interface_identity_summaries(
    type_env: &ash_typeck::TypeEnv,
    module: &ash_parser::surface::ModuleFile,
    source_origin: &ash_core::semantic_summary::SourceOrigin,
) -> Vec<InterfaceIdentitySummary> {
    module
        .definitions
        .iter()
        .filter_map(|definition| {
            public_interface_identity_summary(type_env, definition, source_origin.clone())
        })
        .collect()
}

fn public_interface_identity_summary(
    type_env: &ash_typeck::TypeEnv,
    definition: &Definition,
    source_origin: ash_core::semantic_summary::SourceOrigin,
) -> Option<InterfaceIdentitySummary> {
    let Definition::Interface(interface) = definition else {
        return None;
    };
    if !matches!(
        interface.visibility,
        ash_parser::surface::Visibility::Public
    ) {
        return None;
    }
    let id = type_env
        .interface_identity_for_name(interface.name.as_ref())
        .cloned()?;
    let evidence_constraints = interface
        .evidence_constraints
        .iter()
        .map(|constraint| {
            InterfaceEvidenceConstraintSummary::new(
                ash_parser::lower::lower_surface_type(&constraint.subject),
                ash_parser::lower::lower_surface_type(&constraint.interface),
            )
        })
        .collect();
    Some(
        InterfaceIdentitySummary::new(
            id,
            interface.name.to_string(),
            vec![interface.name.to_string()],
            ash_core::semantic_summary::SourceAnchor::new(
                source_origin,
                None,
                format!("interface {}", interface.name),
            ),
        )
        .with_evidence_constraints(evidence_constraints),
    )
}

fn attach_exported_proposition_payload(
    summary: &mut ModuleSemanticSummary,
    public_interface_identities: Vec<InterfaceIdentitySummary>,
    public_predicates: Vec<ash_core::semantic_summary::PropositionPredicateSummary>,
    proposition_facts: Vec<ash_core::semantic_summary::PropositionFactSummary>,
) {
    summary.version = SummaryVersion::SPEC064_PROPOSITIONS_V5;
    for interface in public_interface_identities {
        if !summary
            .interface_identities
            .iter()
            .any(|existing| existing.id == interface.id)
        {
            summary.interface_identities.push(interface);
        }
    }
    summary.exported_proposition_predicates = public_predicates;
    summary.exported_proposition_facts = proposition_facts;
}

fn exportable_type_summary(
    ty: &TypeDeclSummary,
    exportable_types: &HashMap<String, CoreTypeDef>,
) -> Option<TypeDeclSummary> {
    let exported = exportable_types.get(ty.exported_name.as_str())?;
    let mut summary = ty.clone();
    if !matches!(exported.visibility, CoreVisibility::Public) {
        summary.representation_exposure =
            ash_core::semantic_summary::RepresentationExposure::Opaque;
        summary.representation = TypeRepresentationSummary::opaque(exported.builtin);
    }
    Some(summary)
}

fn selected_import_type_semantic_summary(
    summary: &ModuleSemanticSummary,
    source_name: &str,
    imported_name: &str,
) -> Option<ModuleSemanticSummary> {
    let alias_map = HashMap::from([(source_name.to_string(), imported_name.to_string())]);
    selected_type_semantic_summary_with_aliases(
        summary,
        source_name,
        imported_name,
        &alias_map,
        true,
    )
}

/// Select one public provider binding and the complete public closure needed
/// to inspect it.  This is the only effect-row summary selection operation:
/// named imports, globs, and public re-exports differ solely in the visible
/// binding/exposure passed to it.
fn sanitized_effect_row_semantic_summary(
    summary: &ModuleSemanticSummary,
    source_name: &str,
    imported_name: &str,
    exposure: ash_core::semantic_summary::EffectRowBindingExposure,
) -> Result<Option<ModuleSemanticSummary>, EngineError> {
    let selected_provider = summary
        .exported_effect_rows
        .iter()
        .find(|row| row.exported_name == source_name)
        .map(|row| row.provider.clone());
    let Some(selected_provider) = selected_provider else {
        return Ok(None);
    };

    let dependency_closure = transitive_public_effect_row_dependency_closure(summary, source_name);
    if !dependency_closure.inaccessible_by_row.is_empty() {
        // This error deliberately says nothing about the inaccessible row or
        // its provider.  In particular, do not turn it into an opaque summary
        // and then merge/cache it: that would make a non-usable private
        // boundary observable to later consumers.
        return Err(EngineError::Parse(format!(
            "private-dependency-export-failure: effect-row binding '{imported_name}' has an inaccessible dependency"
        )));
    }

    let closure_digest =
        effect_row_public_closure_digest(summary, &dependency_closure.public_providers);
    let mut selected = ModuleSemanticSummary::new(summary.module.clone());
    selected.version = SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7;
    // Select identities first, then retain the provider's public declaration
    // order.  This avoids the former root-first traversal and makes import
    // order irrelevant.
    selected.exported_effect_rows = summary
        .exported_effect_rows
        .iter()
        .filter(|candidate| {
            dependency_closure
                .public_providers
                .contains(&candidate.provider)
        })
        .cloned()
        .map(|mut row| {
            let visible_name = if row.provider == selected_provider {
                imported_name.to_string()
            } else {
                row.exported_name.clone()
            };
            // A facade's public binding is itself part of the exported
            // contract.  A later consumer may select that binding, but cannot
            // erase the fact that it came through a public re-export.
            let binding_exposure = if matches!(
                row.binding.exposure,
                ash_core::semantic_summary::EffectRowBindingExposure::PublicReExport
            ) {
                ash_core::semantic_summary::EffectRowBindingExposure::PublicReExport
            } else {
                exposure
            };
            row.set_visible_binding(visible_name, binding_exposure);
            row.closure_metadata = Some(ash_core::semantic_summary::EffectRowClosureMetadata {
                sanitizer_schema_version:
                    ash_core::semantic_summary::EFFECT_ROW_SANITIZER_SCHEMA_VERSION,
                public_closure_digest: closure_digest.clone(),
            });
            row
        })
        .collect();
    copy_summary_side_metadata(summary, &mut selected);
    Ok(Some(selected))
}

/// Return the public named-row closure needed to validate a selected row.
///
/// Effect-row summary items retain source text, so only the two established
/// named-row spellings (`Name` and `group Name`) participate in this transport
/// closure.  The provider summary has already filtered private rows at its
/// export boundary; consequently this never makes a private dependency
/// importable or source-visible.  Iterating the provider's summary preserves
/// its declaration order in the transported dependency metadata.
struct EffectRowDependencyClosure {
    /// The closure's semantic identity. Visible names are only used to parse
    /// a source row item at this module boundary; aliases/facades cannot
    /// change the provider identities carried from here.
    public_providers: HashSet<ash_core::semantic_summary::EffectRowProviderIdentity>,
    inaccessible_by_row: HashMap<String, Vec<String>>,
}

fn effect_row_public_closure_digest(
    summary: &ModuleSemanticSummary,
    public_providers: &HashSet<ash_core::semantic_summary::EffectRowProviderIdentity>,
) -> String {
    use sha2::{Digest, Sha256};

    // Canonicalize by provider identity, never by source traversal.  Each
    // retained record contains only public summary data: provider identity,
    // source classification, public binding exposure, and ordered row items.
    // Delimiter lengths make the encoding unambiguous without serializing
    // diagnostic anchors or opaque/private dependency details.
    let mut rows = summary
        .exported_effect_rows
        .iter()
        .filter(|row| {
            matches!(
                row.binding.closure_status,
                ash_core::semantic_summary::EffectRowClosureStatus::Complete
            ) && public_providers.contains(&row.provider)
        })
        .collect::<Vec<_>>();
    rows.sort_unstable_by(|left, right| {
        effect_row_provider_sort_key(&left.provider)
            .cmp(&effect_row_provider_sort_key(&right.provider))
    });

    let mut canonical = String::from("ash-effect-row-public-closure/v1\n");
    for row in rows {
        append_canonical_effect_row_field(
            &mut canonical,
            "provider",
            &effect_row_provider_sort_key(&row.provider),
        );
        append_canonical_effect_row_field(
            &mut canonical,
            "classification",
            match row.classification {
                ash_core::semantic_summary::EffectRowExportClassification::TransparentAlias => {
                    "transparent_alias"
                }
                ash_core::semantic_summary::EffectRowExportClassification::DiagnosticGroup => {
                    "diagnostic_group"
                }
            },
        );
        append_canonical_effect_row_field(
            &mut canonical,
            "binding_exposure",
            match row.binding.exposure {
                ash_core::semantic_summary::EffectRowBindingExposure::Declaration => "declaration",
                ash_core::semantic_summary::EffectRowBindingExposure::NamedImport => "named_import",
                ash_core::semantic_summary::EffectRowBindingExposure::GlobImport => "glob_import",
                ash_core::semantic_summary::EffectRowBindingExposure::PublicReExport => {
                    "public_re_export"
                }
            },
        );
        for item in &row.row_items {
            append_canonical_effect_row_field(&mut canonical, "row_item", &item.text);
        }
        canonical.push('\n');
    }

    format!("sha256:{:x}", Sha256::digest(canonical.as_bytes()))
}

fn effect_row_provider_sort_key(
    provider: &ash_core::semantic_summary::EffectRowProviderIdentity,
) -> String {
    let crate_id = provider
        .module
        .crate_id
        .map_or_else(|| "none".to_string(), |id| id.0.to_string());
    format!(
        "crate={crate_id};module={};name={}",
        provider.module.module_id.0, provider.declaration_name
    )
}

fn append_canonical_effect_row_field(output: &mut String, label: &str, value: &str) {
    output.push_str(label);
    output.push(':');
    output.push_str(&value.len().to_string());
    output.push(':');
    output.push_str(value);
    output.push('\n');
}

fn transitive_public_effect_row_dependency_closure(
    summary: &ModuleSemanticSummary,
    source_name: &str,
) -> EffectRowDependencyClosure {
    let Some(source_provider) = summary
        .exported_effect_rows
        .iter()
        .find(|row| row.exported_name == source_name)
        .map(|row| row.provider.clone())
    else {
        return EffectRowDependencyClosure {
            public_providers: HashSet::new(),
            inaccessible_by_row: HashMap::new(),
        };
    };
    let mut providers = HashSet::from([source_provider]);
    let mut inaccessible_by_row = HashMap::new();
    let mut changed = true;

    while changed {
        changed = false;
        for row in &summary.exported_effect_rows {
            if !providers.contains(&row.provider) {
                continue;
            }
            for item in &row.row_items {
                let text = item.text.trim();
                let referenced_name = text.strip_prefix("group ").map(str::trim).or_else(|| {
                    // Qualified symbolic operations (for example
                    // `PosixFs::read`) and raw evidence atoms are row content,
                    // not named row references.  Only the grammar's bare
                    // identifier form participates in the provider closure.
                    is_bare_effect_row_name(text).then_some(text)
                });
                let Some(referenced_name) = referenced_name else {
                    continue;
                };
                // Predicate-like item families are validated by the type
                // checker as row-content errors.  They are not named-row
                // dependencies, even when their compact spelling contains no
                // whitespace (for example `requires_proof`).
                if is_predicate_like_effect_row_item(referenced_name) {
                    continue;
                }
                if let Some(candidate) = summary
                    .exported_effect_rows
                    .iter()
                    .find(|candidate| candidate.exported_name == referenced_name)
                {
                    changed |= providers.insert(candidate.provider.clone());
                } else {
                    let inaccessible = inaccessible_by_row
                        .entry(row.exported_name.clone())
                        .or_insert_with(Vec::new);
                    if !inaccessible.iter().any(|name| name == referenced_name) {
                        inaccessible.push(referenced_name.to_string());
                    }
                }
            }
        }
    }

    EffectRowDependencyClosure {
        public_providers: providers,
        inaccessible_by_row,
    }
}

fn is_bare_effect_row_name(item: &str) -> bool {
    let mut chars = item.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_alphabetic())
        && chars.all(|character| character == '_' || character.is_alphanumeric())
}

fn is_predicate_like_effect_row_item(item: &str) -> bool {
    [
        "requires",
        "ensures",
        "invariant",
        "law",
        "proof",
        "contract",
    ]
    .into_iter()
    .any(|family| {
        item == family
            || item
                .strip_prefix(family)
                .is_some_and(|suffix| suffix.starts_with('_') || suffix.starts_with("::"))
    })
}

fn selected_value_semantic_summary(
    summary: &ModuleSemanticSummary,
    source_name: &str,
    imported_name: &str,
) -> Option<ModuleSemanticSummary> {
    let mut value = summary
        .exported_values
        .iter()
        .find(|value| value.exported_name == source_name)?
        .clone();
    value.exported_name = imported_name.into();

    let mut selected = ModuleSemanticSummary::new(summary.module.clone());
    selected.version = summary.version;
    selected.exported_values.push(value);
    copy_summary_side_metadata(summary, &mut selected);
    Some(selected)
}

fn selected_type_semantic_summary_with_aliases(
    summary: &ModuleSemanticSummary,
    source_name: &str,
    imported_name: &str,
    alias_map: &HashMap<String, String>,
    hide_dependency_metadata: bool,
) -> Option<ModuleSemanticSummary> {
    let selected = summary
        .exported_types
        .iter()
        .find(|ty| ty.exported_name == source_name)?;
    let selected_types = selected_type_and_representation_dependencies(
        summary,
        selected,
        imported_name,
        alias_map,
        hide_dependency_metadata,
    );
    let mut selected_summary = ModuleSemanticSummary::new(summary.module.clone());
    selected_summary.version = summary.version;
    selected_summary.exported_types = selected_types;
    selected_summary.exported_constructors = summary
        .exported_constructors
        .iter()
        .filter(|constructor| constructor.parent == selected.id)
        .map(|constructor| {
            let mut constructor = constructor.clone();
            if let Some(alias) = alias_map.get(&constructor.exported_name) {
                constructor.exported_name = alias.clone();
            }
            constructor
        })
        .collect();
    copy_summary_side_metadata(summary, &mut selected_summary);
    Some(selected_summary)
}

fn selected_constructor_semantic_summary(
    summary: &ModuleSemanticSummary,
    constructor_name: &str,
) -> Option<ModuleSemanticSummary> {
    selected_constructor_semantic_summary_with_dependency_visibility(
        summary,
        constructor_name,
        false,
    )
}

fn selected_import_constructor_semantic_summary(
    summary: &ModuleSemanticSummary,
    constructor_name: &str,
) -> Option<ModuleSemanticSummary> {
    selected_constructor_semantic_summary_with_dependency_visibility(
        summary,
        constructor_name,
        true,
    )
}

fn selected_constructor_semantic_summary_with_dependency_visibility(
    summary: &ModuleSemanticSummary,
    constructor_name: &str,
    hide_dependency_metadata: bool,
) -> Option<ModuleSemanticSummary> {
    let constructor = summary
        .exported_constructors
        .iter()
        .find(|constructor| constructor.exported_name == constructor_name)?;
    let parent = summary
        .exported_types
        .iter()
        .find(|ty| ty.id == constructor.parent)?;
    let alias_map = HashMap::new();
    let selected_types = selected_type_and_representation_dependencies(
        summary,
        parent,
        &parent.exported_name,
        &alias_map,
        hide_dependency_metadata,
    );
    let mut selected_summary = ModuleSemanticSummary::new(summary.module.clone());
    selected_summary.version = summary.version;
    selected_summary.exported_types = selected_types;
    selected_summary.exported_constructors = vec![constructor.clone()];
    copy_summary_side_metadata(summary, &mut selected_summary);
    Some(selected_summary)
}

fn rewrite_type_function_summary_visible_type_names(
    type_function: &mut TypeFunctionSummary,
    alias_map: &HashMap<String, String>,
) {
    rewrite_canonical_type_expr_visible_type_names(&mut type_function.return_type, alias_map);
    for param in &mut type_function.params {
        rewrite_canonical_type_expr_visible_type_names(&mut param.ty, alias_map);
    }
    for equation in &mut type_function.equations {
        rewrite_type_function_result_visible_type_names(&mut equation.result, alias_map);
    }
}

fn rewrite_canonical_type_expr_visible_type_names(
    expr: &mut CanonicalTypeExpr,
    alias_map: &HashMap<String, String>,
) {
    match expr {
        CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => {}
        CanonicalTypeExpr::NominalApp {
            visible_name, args, ..
        } => {
            if let Some(alias) = alias_map.get(visible_name) {
                *visible_name = alias.clone();
            }
            for arg in args {
                rewrite_canonical_type_expr_visible_type_names(arg, alias_map);
            }
        }
        CanonicalTypeExpr::Projection { args, .. }
        | CanonicalTypeExpr::ComputationHeadApp { args, .. } => {
            for arg in args {
                rewrite_canonical_type_expr_visible_type_names(arg, alias_map);
            }
        }
        CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
            for arg in &mut app.args {
                rewrite_canonical_type_expr_visible_type_names(arg, alias_map);
            }
        }
        CanonicalTypeExpr::ConstructorVariableApp(app) => {
            for arg in &mut app.args {
                rewrite_canonical_type_expr_visible_type_names(arg, alias_map);
            }
        }
    }
}

fn rewrite_type_function_result_visible_type_names(
    expr: &mut TypeFunctionResultExpr,
    alias_map: &HashMap<String, String>,
) {
    match expr {
        TypeFunctionResultExpr::Primitive { .. } | TypeFunctionResultExpr::Var { .. } => {}
        TypeFunctionResultExpr::NominalApp {
            visible_name, args, ..
        } => {
            if let Some(alias) = alias_map.get(visible_name) {
                *visible_name = alias.clone();
            }
            for arg in args {
                rewrite_type_function_result_visible_type_names(arg, alias_map);
            }
        }
        TypeFunctionResultExpr::DomainConstructorApp { args, .. }
        | TypeFunctionResultExpr::PromotedDataConstructorApp { args, .. }
        | TypeFunctionResultExpr::Projection { args, .. }
        | TypeFunctionResultExpr::ComputationHeadApp { args, .. } => {
            for arg in args {
                rewrite_type_function_result_visible_type_names(arg, alias_map);
            }
        }
    }
}

fn selected_type_function_semantic_summary(
    summary: &ModuleSemanticSummary,
    type_function_name: &str,
    imported_name: &str,
) -> Option<ModuleSemanticSummary> {
    let selected = summary
        .exported_type_functions
        .iter()
        .find(|type_function| type_function.exported_name == type_function_name)?;
    let closure = transitive_type_function_dependency_summaries(summary, selected);
    let mut dependencies = type_function_summary_dependencies(&closure);
    expand_promoted_data_kind_dependency_closure(summary, &mut dependencies);
    let mut selected_summary = ModuleSemanticSummary::new(summary.module.clone());
    selected_summary.version = summary.version;
    selected_summary.exported_sealed_domains = summary
        .exported_sealed_domains
        .iter()
        .filter(|domain| dependencies.sealed_domains.contains(&domain.id))
        .cloned()
        .collect();
    let selected_exported_types = summary
        .exported_types
        .iter()
        .filter(|ty| dependencies.types.contains(&ty.id))
        .cloned()
        .collect::<Vec<_>>();
    let dependency_type_aliases = selected_exported_types
        .iter()
        .map(|ty| {
            (
                ty.exported_name.clone(),
                dependency_metadata_name(&ty.exported_name),
            )
        })
        .collect::<HashMap<_, _>>();
    selected_summary.exported_types = selected_exported_types
        .into_iter()
        .map(|mut ty| {
            if let Some(metadata_name) = dependency_type_aliases.get(&ty.exported_name) {
                ty.exported_name = metadata_name.clone();
            }
            rewrite_type_representation_aliases(&mut ty.representation, &dependency_type_aliases);
            ty
        })
        .collect();
    selected_summary.exported_constructors = summary
        .exported_constructors
        .iter()
        .filter(|constructor| dependencies.types.contains(&constructor.parent))
        .cloned()
        .collect();
    selected_summary.exported_promoted_data_kinds =
        hidden_promoted_data_kind_dependencies(summary, &dependencies.promoted_data_kinds);
    selected_summary.exported_type_functions = closure
        .into_iter()
        .map(|type_function| {
            let mut type_function = type_function.clone();
            rewrite_type_function_summary_visible_type_names(
                &mut type_function,
                &dependency_type_aliases,
            );
            if type_function.head == selected.head {
                type_function.exported_name = imported_name.to_string();
            } else {
                type_function.exported_name =
                    dependency_metadata_name(&type_function.exported_name);
            }
            type_function
        })
        .collect();
    copy_type_function_summary_side_metadata(summary, &mut selected_summary, &dependencies);
    Some(selected_summary)
}

fn semantic_summary_has_interface(summary: Option<&ModuleSemanticSummary>, name: &str) -> bool {
    summary.is_some_and(|summary| {
        summary
            .interface_identities
            .iter()
            .any(|identity| identity.name == name)
    })
}

fn selected_interface_semantic_summary(
    summary: &ModuleSemanticSummary,
    interface_name: &str,
    imported_name: &str,
) -> Option<ModuleSemanticSummary> {
    let identity = summary
        .interface_identities
        .iter()
        .find(|identity| identity.name == interface_name)?;
    let mut selected = ModuleSemanticSummary::new(summary.module.clone());
    selected.version = summary.version;
    let mut identity = identity.clone();
    identity.name = imported_name.to_string();
    identity.path = vec![imported_name.to_string()];
    selected.interface_identities.push(identity);
    Some(selected)
}

fn selected_associated_family_semantic_summary(
    summary: &ModuleSemanticSummary,
    family_name: &str,
    imported_name: &str,
) -> Option<ModuleSemanticSummary> {
    let selected = summary
        .exported_associated_families
        .iter()
        .find(|family| family.visible_name == family_name)?;
    let associated_family_closure =
        transitive_associated_family_dependency_summaries(summary, selected);
    let mut ordinary_types = HashSet::new();
    let mut sealed_domains = HashSet::new();
    let mut type_functions = HashSet::new();
    let mut associated_family_heads = HashSet::new();
    for family in &associated_family_closure {
        ordinary_types.extend(family.dependency_closure.ordinary_types.iter().cloned());
        sealed_domains.extend(family.dependency_closure.sealed_domains.iter().cloned());
        type_functions.extend(family.dependency_closure.type_functions.iter().cloned());
        associated_family_heads.insert(family.head.clone());
    }

    let mut selected_summary = ModuleSemanticSummary::new(summary.module.clone());
    selected_summary.version = SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4;
    selected_summary.exported_types = summary
        .exported_types
        .iter()
        .filter(|ty| ordinary_types.contains(&ty.id))
        .cloned()
        .collect();
    selected_summary.exported_constructors = summary
        .exported_constructors
        .iter()
        .filter(|constructor| ordinary_types.contains(&constructor.parent))
        .cloned()
        .collect();
    selected_summary.exported_sealed_domains = summary
        .exported_sealed_domains
        .iter()
        .filter(|domain| sealed_domains.contains(&domain.id))
        .cloned()
        .collect();
    selected_summary.exported_type_functions = summary
        .exported_type_functions
        .iter()
        .filter(|type_function| type_functions.contains(&type_function.head))
        .cloned()
        .collect();
    selected_summary.exported_associated_families = associated_family_closure
        .into_iter()
        .cloned()
        .map(|mut family| {
            if family.head == selected.head {
                family.visible_name = imported_name.to_string();
            } else {
                family.visible_name = dependency_metadata_name(&family.visible_name);
            }
            family
        })
        .collect();
    copy_summary_side_metadata(summary, &mut selected_summary);
    selected_summary.interface_identities.retain(|identity| {
        associated_family_heads
            .iter()
            .any(|head| head.interface == identity.id)
    });
    selected_summary
        .associated_member_identities
        .retain(|identity| {
            associated_family_heads
                .iter()
                .any(|head| head.member == identity.id)
        });
    Some(selected_summary)
}

fn transitive_associated_family_dependency_summaries<'a>(
    summary: &'a ModuleSemanticSummary,
    selected: &AssociatedFamilySummary,
) -> Vec<&'a AssociatedFamilySummary> {
    let mut selected_summaries = Vec::new();
    let mut included_heads = HashSet::new();
    let mut pending = vec![selected.head.clone()];

    while let Some(head) = pending.pop() {
        if !included_heads.insert(head.clone()) {
            continue;
        }
        let Some(family) = summary
            .exported_associated_families
            .iter()
            .find(|candidate| candidate.head == head)
        else {
            continue;
        };
        pending.extend(
            family
                .dependency_closure
                .associated_families
                .iter()
                .map(|dependency| dependency.family.clone()),
        );
        selected_summaries.push(family);
    }

    selected_summaries
}

#[derive(Default)]
struct TypeFunctionDependencyIds {
    types: HashSet<TypeDeclId>,
    sealed_domains: HashSet<SealedDomainId>,
    promoted_data_kinds: HashSet<PromotedDataKindId>,
    promoted_constructors: HashSet<PromotedConstructorId>,
    interfaces: HashSet<InterfaceIdentityId>,
    associated_members: HashSet<AssociatedMemberIdentityId>,
}

fn type_function_summary_dependencies(
    summaries: &[&TypeFunctionSummary],
) -> TypeFunctionDependencyIds {
    let mut dependencies = TypeFunctionDependencyIds::default();
    for summary in summaries {
        for param in &summary.params {
            collect_canonical_type_dependencies(&param.ty, &mut dependencies);
            if let Some(domain) = &param.domain_constraint {
                dependencies.sealed_domains.insert(domain.clone());
            }
        }
        collect_canonical_type_dependencies(&summary.return_type, &mut dependencies);
        collect_result_constraint_dependencies(&summary.result_constraint, &mut dependencies);
        for equation in &summary.equations {
            for pattern in &equation.patterns {
                collect_pattern_dependencies(pattern, &mut dependencies);
            }
            collect_result_expr_dependencies(&equation.result, &mut dependencies);
        }
    }
    dependencies
}

fn proposition_summary_dependencies(summary: &ModuleSemanticSummary) -> TypeFunctionDependencyIds {
    let mut dependencies = TypeFunctionDependencyIds::default();
    for predicate in &summary.exported_proposition_predicates {
        for param in &predicate.params {
            collect_canonical_type_dependencies(&param.ty, &mut dependencies);
        }
    }
    for fact in &summary.exported_proposition_facts {
        collect_type_proposition_dependencies(&fact.proposition, &mut dependencies);
    }
    dependencies
}

fn expand_promoted_data_kind_dependency_closure(
    summary: &ModuleSemanticSummary,
    dependencies: &mut TypeFunctionDependencyIds,
) {
    let promoted_summaries = summary
        .exported_promoted_data_kinds
        .iter()
        .map(|data_kind| (data_kind.id.clone(), data_kind.clone()))
        .collect::<HashMap<_, _>>();
    expand_promoted_data_kind_dependencies_from_map(&promoted_summaries, dependencies);
}

fn expand_promoted_data_kind_dependencies_from_map(
    promoted_summaries: &HashMap<
        PromotedDataKindId,
        ash_core::semantic_summary::PromotedDataKindSummary,
    >,
    dependencies: &mut TypeFunctionDependencyIds,
) {
    let mut changed = true;
    while changed {
        changed = false;
        let data_kind_ids = dependencies
            .promoted_data_kinds
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        for data_kind_id in data_kind_ids {
            let Some(data_kind) = promoted_summaries.get(&data_kind_id) else {
                continue;
            };
            dependencies.types.insert(data_kind.source_type.clone());
            for constructor in &data_kind.constructors {
                dependencies
                    .promoted_constructors
                    .insert(constructor.id.clone());
                dependencies
                    .types
                    .insert(constructor.source_constructor.parent.clone());
                for field in &constructor.fields {
                    if let Some(field_data_kind) = &field.data_kind_constraint
                        && dependencies
                            .promoted_data_kinds
                            .insert(field_data_kind.clone())
                    {
                        changed = true;
                    }
                }
            }
        }
    }
}

fn hidden_promoted_data_kind_dependencies(
    summary: &ModuleSemanticSummary,
    promoted_data_kinds: &HashSet<PromotedDataKindId>,
) -> Vec<ash_core::semantic_summary::PromotedDataKindSummary> {
    summary
        .exported_promoted_data_kinds
        .iter()
        .filter(|data_kind| promoted_data_kinds.contains(&data_kind.id))
        .cloned()
        .map(|mut data_kind| {
            data_kind.exported_name = dependency_metadata_name(&data_kind.exported_name);
            data_kind
        })
        .collect()
}

fn collect_type_proposition_dependencies(
    proposition: &TypeProposition,
    dependencies: &mut TypeFunctionDependencyIds,
) {
    match proposition {
        TypeProposition::Equality(proposition) => {
            collect_type_proposition_term_dependencies(&proposition.lhs, dependencies);
            collect_type_proposition_term_dependencies(&proposition.rhs, dependencies);
        }
        TypeProposition::Disequality(proposition) => {
            collect_type_proposition_term_dependencies(&proposition.lhs, dependencies);
            collect_type_proposition_term_dependencies(&proposition.rhs, dependencies);
        }
        TypeProposition::InterfaceBound(proposition) => {
            dependencies
                .interfaces
                .insert(proposition.interface.clone());
            collect_type_proposition_term_dependencies(&proposition.subject, dependencies);
            for arg in &proposition.interface_args {
                collect_type_proposition_term_dependencies(arg, dependencies);
            }
        }
        TypeProposition::NamedPredicate(proposition) => {
            for arg in &proposition.args {
                collect_type_proposition_term_dependencies(arg, dependencies);
            }
        }
    }
}

fn collect_type_proposition_term_dependencies(
    term: &TypePropositionTerm,
    dependencies: &mut TypeFunctionDependencyIds,
) {
    match term {
        TypePropositionTerm::Canonical(expr) => {
            collect_canonical_type_dependencies(expr, dependencies);
        }
        TypePropositionTerm::DomainConstructorApp { domain, args, .. } => {
            dependencies.sealed_domains.insert(domain.clone());
            for arg in args {
                collect_type_proposition_term_dependencies(arg, dependencies);
            }
        }
    }
}

fn collect_canonical_type_dependencies(
    expr: &CanonicalTypeExpr,
    dependencies: &mut TypeFunctionDependencyIds,
) {
    match expr {
        CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => {}
        CanonicalTypeExpr::NominalApp { origin, args, .. } => {
            dependencies.types.insert(origin.clone());
            for arg in args {
                collect_canonical_type_dependencies(arg, dependencies);
            }
        }
        CanonicalTypeExpr::Projection {
            interface,
            member,
            args,
            ..
        } => {
            dependencies.interfaces.insert(interface.clone());
            dependencies.associated_members.insert(member.clone());
            for arg in args {
                collect_canonical_type_dependencies(arg, dependencies);
            }
        }
        CanonicalTypeExpr::ComputationHeadApp { args, .. } => {
            for arg in args {
                collect_canonical_type_dependencies(arg, dependencies);
            }
        }
        CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
            dependencies
                .promoted_data_kinds
                .insert(app.data_kind.clone());
            dependencies
                .promoted_constructors
                .insert(app.constructor.clone());
            dependencies.types.insert(app.data_kind.source_type.clone());
            dependencies
                .types
                .insert(app.constructor.source_constructor.parent.clone());
            for arg in &app.args {
                collect_canonical_type_dependencies(arg, dependencies);
            }
        }
        CanonicalTypeExpr::ConstructorVariableApp(app) => {
            for arg in &app.args {
                collect_canonical_type_dependencies(arg, dependencies);
            }
        }
    }
}

fn collect_pattern_dependencies(
    pattern: &TypeFunctionPattern,
    dependencies: &mut TypeFunctionDependencyIds,
) {
    match pattern {
        TypeFunctionPattern::DomainConstructor {
            constructor: _,
            domain,
            fields,
            constraint,
            ..
        } => {
            dependencies.sealed_domains.insert((**domain).clone());
            collect_pattern_constraint_dependencies(constraint, dependencies);
            for field in fields {
                collect_pattern_dependencies(field, dependencies);
            }
        }
        TypeFunctionPattern::Var { constraint, .. }
        | TypeFunctionPattern::Wildcard { constraint, .. } => {
            collect_pattern_constraint_dependencies(constraint, dependencies);
        }
    }
}

fn collect_pattern_constraint_dependencies(
    constraint: &TypeFunctionPatternConstraint,
    dependencies: &mut TypeFunctionDependencyIds,
) {
    match constraint {
        TypeFunctionPatternConstraint::Kind(_) => {}
        TypeFunctionPatternConstraint::Domain(domain) => {
            dependencies.sealed_domains.insert(domain.clone());
        }
    }
}

fn collect_result_constraint_dependencies(
    constraint: &TypeFunctionResultConstraint,
    dependencies: &mut TypeFunctionDependencyIds,
) {
    match constraint {
        TypeFunctionResultConstraint::Kind(_) => {}
        TypeFunctionResultConstraint::Domain(domain) => {
            dependencies.sealed_domains.insert(domain.clone());
        }
    }
}

fn collect_result_expr_dependencies(
    expr: &TypeFunctionResultExpr,
    dependencies: &mut TypeFunctionDependencyIds,
) {
    match expr {
        TypeFunctionResultExpr::Primitive { constraint, .. }
        | TypeFunctionResultExpr::Var { constraint, .. } => {
            collect_result_constraint_dependencies(constraint, dependencies);
        }
        TypeFunctionResultExpr::NominalApp {
            origin,
            args,
            constraint,
            ..
        } => {
            dependencies.types.insert(origin.clone());
            collect_result_constraint_dependencies(constraint, dependencies);
            for arg in args {
                collect_result_expr_dependencies(arg, dependencies);
            }
        }
        TypeFunctionResultExpr::DomainConstructorApp {
            constructor: _,
            domain,
            args,
            constraint,
            ..
        } => {
            dependencies.sealed_domains.insert(domain.clone());
            collect_result_constraint_dependencies(constraint, dependencies);
            for arg in args {
                collect_result_expr_dependencies(arg, dependencies);
            }
        }
        TypeFunctionResultExpr::PromotedDataConstructorApp {
            constructor,
            data_kind,
            args,
            constraint,
            ..
        } => {
            dependencies
                .promoted_data_kinds
                .insert((**data_kind).clone());
            dependencies
                .promoted_constructors
                .insert((**constructor).clone());
            dependencies.types.insert(data_kind.source_type.clone());
            dependencies
                .types
                .insert(constructor.source_constructor.parent.clone());
            collect_result_constraint_dependencies(constraint, dependencies);
            for arg in args {
                collect_result_expr_dependencies(arg, dependencies);
            }
        }
        TypeFunctionResultExpr::Projection {
            interface,
            member,
            args,
            constraint,
            ..
        } => {
            dependencies.interfaces.insert(interface.clone());
            dependencies.associated_members.insert(member.clone());
            collect_result_constraint_dependencies(constraint, dependencies);
            for arg in args {
                collect_result_expr_dependencies(arg, dependencies);
            }
        }
        TypeFunctionResultExpr::ComputationHeadApp {
            args, constraint, ..
        } => {
            collect_result_constraint_dependencies(constraint, dependencies);
            for arg in args {
                collect_result_expr_dependencies(arg, dependencies);
            }
        }
    }
}

fn copy_type_function_summary_side_metadata(
    source: &ModuleSemanticSummary,
    target: &mut ModuleSemanticSummary,
    dependencies: &TypeFunctionDependencyIds,
) {
    target.re_exports.clone_from(&source.re_exports);
    target
        .imported_summary_refs
        .clone_from(&source.imported_summary_refs);
    target.interface_identities = source
        .interface_identities
        .iter()
        .filter(|identity| dependencies.interfaces.contains(&identity.id))
        .cloned()
        .collect();
    target.associated_member_identities = source
        .associated_member_identities
        .iter()
        .filter(|identity| dependencies.associated_members.contains(&identity.id))
        .cloned()
        .collect();
    target.reserved_identity_slots = source.reserved_identity_slots.clone();
    target
        .diagnostic_anchors
        .clone_from(&source.diagnostic_anchors);
}

fn transitive_type_function_dependency_summaries<'a>(
    summary: &'a ModuleSemanticSummary,
    selected: &TypeFunctionSummary,
) -> Vec<&'a TypeFunctionSummary> {
    let mut selected_summaries = Vec::new();
    let mut included_heads = HashSet::new();
    let mut pending = vec![selected.head.clone()];

    while let Some(head) = pending.pop() {
        if !included_heads.insert(head.clone()) {
            continue;
        }
        let Some(type_function) = summary
            .exported_type_functions
            .iter()
            .find(|candidate| candidate.head == head)
        else {
            continue;
        };
        for param in &type_function.params {
            collect_canonical_type_function_heads(&param.ty, &mut pending);
        }
        collect_canonical_type_function_heads(&type_function.return_type, &mut pending);
        for equation in &type_function.equations {
            collect_result_type_function_heads(&equation.result, &mut pending);
        }
        selected_summaries.push(type_function);
    }

    selected_summaries
}

fn collect_canonical_type_function_heads(
    expr: &CanonicalTypeExpr,
    heads: &mut Vec<TypeComputationHeadId>,
) {
    match expr {
        CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::Var(_) => {}
        CanonicalTypeExpr::NominalApp { args, .. } | CanonicalTypeExpr::Projection { args, .. } => {
            for arg in args {
                collect_canonical_type_function_heads(arg, heads);
            }
        }
        CanonicalTypeExpr::ComputationHeadApp { head, args, .. } => {
            heads.push(head.clone());
            for arg in args {
                collect_canonical_type_function_heads(arg, heads);
            }
        }
        CanonicalTypeExpr::PromotedDataConstructorApp(app) => {
            for arg in &app.args {
                collect_canonical_type_function_heads(arg, heads);
            }
        }
        CanonicalTypeExpr::ConstructorVariableApp(app) => {
            for arg in &app.args {
                collect_canonical_type_function_heads(arg, heads);
            }
        }
    }
}

fn collect_result_type_function_heads(
    expr: &TypeFunctionResultExpr,
    heads: &mut Vec<TypeComputationHeadId>,
) {
    match expr {
        TypeFunctionResultExpr::Primitive { .. } | TypeFunctionResultExpr::Var { .. } => {}
        TypeFunctionResultExpr::NominalApp { args, .. }
        | TypeFunctionResultExpr::DomainConstructorApp { args, .. }
        | TypeFunctionResultExpr::PromotedDataConstructorApp { args, .. }
        | TypeFunctionResultExpr::Projection { args, .. } => {
            for arg in args {
                collect_result_type_function_heads(arg, heads);
            }
        }
        TypeFunctionResultExpr::ComputationHeadApp { head, args, .. } => {
            heads.push(head.clone());
            for arg in args {
                collect_result_type_function_heads(arg, heads);
            }
        }
    }
}

fn copy_summary_side_metadata(source: &ModuleSemanticSummary, target: &mut ModuleSemanticSummary) {
    target.re_exports.clone_from(&source.re_exports);
    target
        .imported_summary_refs
        .clone_from(&source.imported_summary_refs);
    target
        .interface_identities
        .clone_from(&source.interface_identities);
    target
        .associated_member_identities
        .clone_from(&source.associated_member_identities);
    target.reserved_identity_slots = source.reserved_identity_slots.clone();
    target
        .diagnostic_anchors
        .clone_from(&source.diagnostic_anchors);
}

fn merge_callable_signature_summaries(
    exports: &mut ModuleExports,
    target_summary: Option<&ModuleSemanticSummary>,
    callable: &InlineCallable,
) -> Result<(), EngineError> {
    let Some(summary) = target_summary else {
        return Ok(());
    };
    if callable_signature_has_proposition_requirements(callable)
        && let Some(selected_summary) = selected_proposition_semantic_summary(Some(summary))
    {
        merge_selected_summary_export(exports, summary, selected_summary)?;
    }
    let mut names = callable_signature_type_names(callable);
    names.sort_unstable();
    names.dedup();
    for name in names {
        let Some(source_ty) = summary
            .exported_types
            .iter()
            .find(|ty| ty.exported_name == name)
        else {
            continue;
        };
        let exported_name = exports
            .semantic_summary
            .as_ref()
            .and_then(|exported_summary| {
                exported_summary
                    .exported_types
                    .iter()
                    .find(|existing| existing.id == source_ty.id)
                    .map(|existing| existing.exported_name.clone())
            })
            .unwrap_or_else(|| name.clone());
        merge_type_summary_export(exports, summary, &name, &exported_name)?;
    }
    Ok(())
}

fn selected_type_and_representation_dependencies(
    summary: &ModuleSemanticSummary,
    selected: &TypeDeclSummary,
    imported_name: &str,
    alias_map: &HashMap<String, String>,
    hide_dependency_metadata: bool,
) -> Vec<TypeDeclSummary> {
    let mut metadata_alias_map = alias_map.clone();
    if hide_dependency_metadata {
        metadata_alias_map.extend(representation_dependency_metadata_aliases(
            summary,
            &selected.exported_name,
            alias_map,
        ));
    }

    let mut selected_type = selected.clone();
    if let TypeRepresentationSummary::Exposed(body) = &mut selected_type.representation {
        rewrite_core_type_body_aliases(body, &metadata_alias_map);
    }
    selected_type.exported_name = imported_name.into();

    let mut selected_types = vec![selected_type];
    for dependency in transitive_representation_dependency_summaries(summary, selected) {
        let mut dependency = dependency.clone();
        if let TypeRepresentationSummary::Exposed(body) = &mut dependency.representation {
            rewrite_core_type_body_aliases(body, &metadata_alias_map);
        }
        dependency.exported_name = metadata_alias_map
            .get(&dependency.exported_name)
            .cloned()
            .unwrap_or_else(|| dependency.exported_name.clone());
        selected_types.push(dependency);
    }

    selected_types
}

fn representation_dependency_names(ty: &TypeDeclSummary) -> Vec<String> {
    let mut names = Vec::new();
    if let TypeRepresentationSummary::Exposed(body) = &ty.representation {
        collect_type_body_dependency_names(body, &mut names);
    }
    names.retain(|name| !ty.params.iter().any(|param| param == name) && name != &ty.exported_name);
    names.sort_unstable();
    names.dedup();
    names
}

fn collect_type_body_dependency_names(body: &CoreTypeBody, names: &mut Vec<String>) {
    match body {
        CoreTypeBody::Struct(fields) => {
            for (_, field_ty) in fields {
                collect_type_expr_dependency_names(field_ty, names);
            }
        }
        CoreTypeBody::Enum(variants) => {
            for variant in variants {
                for (_, field_ty) in &variant.fields {
                    collect_type_expr_dependency_names(field_ty, names);
                }
                collect_variant_payload_dependency_names(&variant.payload, names);
            }
        }
        CoreTypeBody::Alias(target) => collect_type_expr_dependency_names(target, names),
    }
}

fn collect_variant_payload_dependency_names(payload: &CoreVariantPayload, names: &mut Vec<String>) {
    match payload {
        CoreVariantPayload::Unit => {}
        CoreVariantPayload::Record(fields) => {
            for (_, field_ty) in fields {
                collect_type_expr_dependency_names(field_ty, names);
            }
        }
        CoreVariantPayload::Tuple(items) => {
            for item in items {
                collect_type_expr_dependency_names(item, names);
            }
        }
    }
}

fn collect_type_expr_dependency_names(expr: &CoreTypeExpr, names: &mut Vec<String>) {
    match expr {
        CoreTypeExpr::Named(name) => names.push(name.clone()),
        CoreTypeExpr::Constructor { name, args } => {
            names.push(name.clone());
            for arg in args {
                collect_type_expr_dependency_names(arg, names);
            }
        }
        CoreTypeExpr::Tuple(items) => {
            for item in items {
                collect_type_expr_dependency_names(item, names);
            }
        }
        CoreTypeExpr::Record(fields) => {
            for (_, field_ty) in fields {
                collect_type_expr_dependency_names(field_ty, names);
            }
        }
        CoreTypeExpr::Associated { base, .. } => {
            collect_type_expr_dependency_names(base, names);
        }
    }
}

fn imported_summary_key(summary: &ModuleSemanticSummary) -> ImportedSummaryKey {
    summary.semantic_cache_key()
}

fn merge_type_summary_export(
    exports: &mut ModuleExports,
    target_summary: &ModuleSemanticSummary,
    source_name: &str,
    exported_name: &str,
) -> Result<(), EngineError> {
    let alias_map = HashMap::from([(source_name.to_string(), exported_name.to_string())]);
    merge_type_summary_export_with_aliases(
        exports,
        target_summary,
        source_name,
        exported_name,
        &alias_map,
    )
}

fn merge_constructor_summary_export(
    exports: &mut ModuleExports,
    target_summary: &ModuleSemanticSummary,
    constructor_name: &str,
) -> Result<(), EngineError> {
    let Some(selected_summary) =
        selected_constructor_semantic_summary(target_summary, constructor_name)
    else {
        return Ok(());
    };
    merge_selected_summary_export(exports, target_summary, selected_summary)
}

fn merge_type_summary_export_with_aliases(
    exports: &mut ModuleExports,
    target_summary: &ModuleSemanticSummary,
    source_name: &str,
    exported_name: &str,
    alias_map: &HashMap<String, String>,
) -> Result<(), EngineError> {
    let Some(mut selected_summary) = selected_type_semantic_summary_with_aliases(
        target_summary,
        source_name,
        exported_name,
        alias_map,
        false,
    ) else {
        return Ok(());
    };
    advance_nominal_newtype_public_reexport_hops(&mut selected_summary.exported_types);
    merge_selected_summary_export(exports, target_summary, selected_summary)
}

/// Re-exporting a nominal newtype through this public module advances only the
/// pattern-admission provenance carried with its summary. The canonical type
/// identity and constructor contract remain provider-owned.
fn advance_nominal_newtype_public_reexport_hops(types: &mut [TypeDeclSummary]) {
    for ty in types {
        if ty.declaration_kind == ash_core::semantic_summary::TypeDeclarationKind::NominalNewtype {
            ty.nominal_newtype_public_reexport_hops =
                ty.nominal_newtype_public_reexport_hops.saturating_add(1);
        }
    }
}

fn merge_selected_summary_export(
    exports: &mut ModuleExports,
    target_summary: &ModuleSemanticSummary,
    mut selected_summary: ModuleSemanticSummary,
) -> Result<(), EngineError> {
    let selected_types = std::mem::take(&mut selected_summary.exported_types);
    let selected_constructors = std::mem::take(&mut selected_summary.exported_constructors);
    let summary = exports
        .semantic_summary
        .get_or_insert_with(|| ModuleSemanticSummary::new(target_summary.module.clone()));

    merge_selected_type_exports(summary, selected_types, selected_constructors)?;
    merge_selected_summary_payloads(summary, selected_summary)?;
    update_summary_version_for_selected_payloads(summary);
    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
fn merge_selected_type_exports(
    summary: &mut ModuleSemanticSummary,
    exported_types: Vec<TypeDeclSummary>,
    selected_constructors: Vec<ConstructorSummary>,
) -> Result<(), EngineError> {
    for ty in exported_types {
        if let Some(existing_index) = summary
            .exported_types
            .iter()
            .position(|existing| existing.id == ty.id)
        {
            if summary.exported_types[existing_index].exported_name == ty.exported_name {
                continue;
            }
            summary.exported_types.remove(existing_index);
            summary
                .exported_constructors
                .retain(|constructor| constructor.parent != ty.id);
        }
        if let Some(existing) = summary
            .exported_types
            .iter()
            .find(|existing| existing.exported_name == ty.exported_name)
        {
            if existing.id == ty.id {
                continue;
            }
            // Skip duplicate name with different ID — the existing type is
            // already available for resolution. This handles the case where
            // a dependency type has the same name as a directly-imported type
            // (e.g., GenContext from both context and strategy modules).
            continue;
        }
        summary.exported_types.push(ty);
    }

    for constructor in selected_constructors {
        if !summary
            .exported_constructors
            .iter()
            .any(|existing| existing.id == constructor.id)
        {
            summary.exported_constructors.push(constructor);
        }
    }
    Ok(())
}

fn merge_selected_summary_payloads(
    summary: &mut ModuleSemanticSummary,
    selected_summary: ModuleSemanticSummary,
) -> Result<(), EngineError> {
    // This replaces the former first-wins `id` de-duplication and ensures a
    // failed re-export cannot publish a partial conflicting row surface.
    validate_effect_row_visible_binding_contracts(
        &summary.exported_effect_rows,
        &selected_summary.exported_effect_rows,
    )?;

    for mut row in selected_summary.exported_effect_rows {
        // A public re-export becomes part of this module's public surface.
        // Preserve the source anchor but rehome the summary identity so the
        // receiving TypeEnv can validate it against the enclosing facade.
        row.id.module = summary.module.clone();
        row.id.name.clone_from(&row.exported_name);
        if !summary
            .exported_effect_rows
            .iter()
            .any(|existing| existing.binding.visible_name == row.binding.visible_name)
        {
            summary.exported_effect_rows.push(row);
        }
    }
    for value in selected_summary.exported_values {
        if !summary
            .exported_values
            .iter()
            .any(|existing| existing.exported_name == value.exported_name)
        {
            summary.exported_values.push(value);
        }
    }
    for domain in selected_summary.exported_sealed_domains {
        if !summary
            .exported_sealed_domains
            .iter()
            .any(|existing| existing.id == domain.id)
        {
            summary.exported_sealed_domains.push(domain);
        }
    }
    for data_kind in selected_summary.exported_promoted_data_kinds {
        if !summary
            .exported_promoted_data_kinds
            .iter()
            .any(|existing| existing.id == data_kind.id)
        {
            summary.exported_promoted_data_kinds.push(data_kind);
        }
    }
    for identity in selected_summary.interface_identities {
        if !summary
            .interface_identities
            .iter()
            .any(|existing| existing.id == identity.id)
        {
            summary.interface_identities.push(identity);
        }
    }
    for identity in selected_summary.associated_member_identities {
        if !summary
            .associated_member_identities
            .iter()
            .any(|existing| existing.id == identity.id)
        {
            summary.associated_member_identities.push(identity);
        }
    }
    for type_function in selected_summary.exported_type_functions {
        if !existing_has_type_function_summary(summary, &type_function) {
            summary.exported_type_functions.push(type_function);
        }
    }
    for family in selected_summary.exported_associated_families {
        if !summary
            .exported_associated_families
            .iter()
            .any(|existing| existing.head == family.head)
        {
            summary.exported_associated_families.push(family);
        }
    }
    for predicate in selected_summary.exported_proposition_predicates {
        if !summary
            .exported_proposition_predicates
            .iter()
            .any(|existing| existing.id == predicate.id)
        {
            summary.exported_proposition_predicates.push(predicate);
        }
    }
    for fact in selected_summary.exported_proposition_facts {
        if !summary
            .exported_proposition_facts
            .iter()
            .any(|existing| existing == &fact)
        {
            summary.exported_proposition_facts.push(fact);
        }
    }
    Ok(())
}

const fn update_summary_version_for_selected_payloads(summary: &mut ModuleSemanticSummary) {
    if !summary.exported_effect_rows.is_empty() {
        summary.version = SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7;
    } else if !summary.exported_promoted_data_kinds.is_empty() {
        summary.version = SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6;
    } else if !summary.exported_proposition_predicates.is_empty()
        || !summary.exported_proposition_facts.is_empty()
    {
        summary.version = SummaryVersion::SPEC064_PROPOSITIONS_V5;
    } else if !summary.exported_associated_families.is_empty() {
        summary.version = SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4;
    } else if !summary.exported_type_functions.is_empty() {
        summary.version = SummaryVersion::SPEC062_TYPE_COMPUTATION_V3;
    }
}

fn insert_type_export_with_name(
    exports: &mut ModuleExports,
    name: &str,
    type_def: CoreTypeDef,
) -> Result<(), EngineError> {
    if let Some(existing) = exports.type_defs.get(name) {
        if existing == &type_def {
            return Ok(());
        }
        return Err(EngineError::Configuration(format!(
            "duplicate exported type '{name}'"
        )));
    }
    exports.type_defs.insert(name.to_string(), type_def);
    Ok(())
}

fn insert_constructor_export_with_name(
    exports: &mut ModuleExports,
    name: &str,
    type_def: CoreTypeDef,
) -> Result<(), EngineError> {
    if let Some(existing) = exports.constructor_defs.get(name) {
        if existing == &type_def {
            return Ok(());
        }
        return Err(EngineError::Configuration(format!(
            "duplicate exported constructor '{name}'"
        )));
    }
    exports.constructor_defs.insert(name.to_string(), type_def);
    Ok(())
}

fn insert_type_function_export(
    exports: &mut ModuleExports,
    name: &str,
    type_function: TypeFunctionSummary,
) -> Result<(), EngineError> {
    if let Some(existing) = exports.type_function_summaries.get(name) {
        if existing.head == type_function.head {
            return Ok(());
        }
        return Err(EngineError::Configuration(format!(
            "duplicate exported type function '{name}'"
        )));
    }
    exports
        .type_function_summaries
        .insert(name.to_string(), type_function);
    Ok(())
}

fn insert_associated_family_export(
    exports: &mut ModuleExports,
    name: &str,
    family: AssociatedFamilySummary,
) -> Result<(), EngineError> {
    if let Some(existing) = exports.associated_family_summaries.get(name) {
        if existing.head == family.head {
            return Ok(());
        }
        return Err(EngineError::Configuration(format!(
            "duplicate exported associated family '{name}'"
        )));
    }
    exports
        .associated_family_summaries
        .insert(name.to_string(), family);
    Ok(())
}

fn insert_callable_export(
    exports: &mut ModuleExports,
    name: &str,
    callable: InlineCallable,
) -> Result<(), EngineError> {
    if let Some(existing) = exports.callables.get(name) {
        if existing.exported_name == callable.exported_name {
            return Ok(());
        }
        return Err(EngineError::Configuration(format!(
            "duplicate exported callable '{name}'"
        )));
    }
    exports.callables.insert(name.to_string(), callable);
    Ok(())
}

fn module_runtime_callables_from_definitions(
    definitions: &[Definition],
    module_effectful_names: &HashSet<String>,
) -> HashMap<String, Box<InlineCallable>> {
    definitions
        .iter()
        .filter_map(|definition| {
            let Definition::Function(function) = definition else {
                return None;
            };
            let mut callable = imported_callable_from_fn_def(function.clone()).callable;
            callable.effectful_names.clone_from(module_effectful_names);
            callable.module_runtime_callables.clear();
            Some((function.name.to_string(), Box::new(callable)))
        })
        .collect()
}

mod callable_exports;
use callable_exports::{
    PubFnDiagnostic, capability_type_identity, extract_public_capability_names,
    imported_callable_from_fn_def, module_path_text, parse_builtin_fn_callable,
    parse_supported_pub_fn_callable,
};

mod source_scan;
pub use source_scan::extract_braced_snippets;
use source_scan::{
    extract_import_snippets, extract_pub_mod_declarations, extract_pub_use_snippets,
    extract_semicolon_snippets, resolve_child_module,
};

pub mod import_resolution;
use import_resolution::{discover_crate_root, import_resolution_roots, resolve_module_path};

pub(crate) fn core_type_defs_from_definitions(
    definitions: &[ash_parser::surface::Definition],
) -> Result<Vec<CoreTypeDef>, EngineError> {
    definitions
        .iter()
        .filter_map(|definition| match definition {
            ash_parser::surface::Definition::Type(type_def) => Some(type_def),
            _ => None,
        })
        .map(convert_surface_type_def)
        .collect()
}

fn convert_surface_type_def(
    parsed: &ash_parser::surface::TypeDef,
) -> Result<CoreTypeDef, EngineError> {
    Ok(CoreTypeDef {
        name: parsed.name.to_string(),
        params: parsed.params.iter().map(ToString::to_string).collect(),
        body: match &parsed.body {
            ash_parser::surface::TypeBody::Struct(fields) => {
                CoreTypeBody::Struct(convert_surface_type_fields(fields)?)
            }
            ash_parser::surface::TypeBody::Enum(variants) => CoreTypeBody::Enum(
                variants
                    .iter()
                    .map(|variant| {
                        Ok(CoreVariantDef {
                            name: variant.name.to_string(),
                            fields: convert_surface_type_fields(&variant.fields)?,
                            payload: match &variant.payload {
                                ash_parser::surface::VariantPayload::Unit => {
                                    CoreVariantPayload::Unit
                                }
                                ash_parser::surface::VariantPayload::Record(fields) => {
                                    CoreVariantPayload::Record(convert_surface_type_fields(fields)?)
                                }
                                ash_parser::surface::VariantPayload::Tuple(items) => {
                                    CoreVariantPayload::Tuple(convert_surface_type_items(items)?)
                                }
                            },
                        })
                    })
                    .collect::<Result<Vec<_>, EngineError>>()?,
            ),
            ash_parser::surface::TypeBody::Alias(target) => {
                CoreTypeBody::Alias(convert_surface_type_expr(target)?)
            }
        },
        visibility: match parsed.visibility {
            ash_parser::surface::Visibility::Public => CoreVisibility::Public,
            ash_parser::surface::Visibility::Crate
            | ash_parser::surface::Visibility::Super { .. }
            | ash_parser::surface::Visibility::Restricted { .. } => CoreVisibility::Crate,
            ash_parser::surface::Visibility::Inherited | ash_parser::surface::Visibility::Self_ => {
                CoreVisibility::Private
            }
        },
        builtin: parsed.builtin,
    })
}

fn convert_surface_type_fields(
    fields: &[ash_parser::surface::TypeField],
) -> Result<Vec<(String, CoreTypeExpr)>, EngineError> {
    fields
        .iter()
        .map(|field| {
            Ok((
                field.name.to_string(),
                convert_surface_type_expr(&field.ty)?,
            ))
        })
        .collect()
}

fn convert_surface_type_items(
    items: &[ash_parser::surface::Type],
) -> Result<Vec<CoreTypeExpr>, EngineError> {
    items.iter().map(convert_surface_type_expr).collect()
}

fn convert_surface_type_expr(
    parsed: &ash_parser::surface::Type,
) -> Result<CoreTypeExpr, EngineError> {
    match parsed {
        ash_parser::surface::Type::Name(name) => Ok(CoreTypeExpr::Named(name.to_string())),
        ash_parser::surface::Type::List(item) => Ok(CoreTypeExpr::Constructor {
            name: "List".to_string(),
            args: vec![convert_surface_type_expr(item)?],
        }),
        ash_parser::surface::Type::Tuple(items) => {
            Ok(CoreTypeExpr::Tuple(convert_surface_type_items(items)?))
        }
        ash_parser::surface::Type::Record(fields) => Ok(CoreTypeExpr::Record(
            fields
                .iter()
                .map(|(name, ty)| Ok((name.to_string(), convert_surface_type_expr(ty)?)))
                .collect::<Result<Vec<_>, EngineError>>()?,
        )),
        ash_parser::surface::Type::Constructor { name, args } => Ok(CoreTypeExpr::Constructor {
            name: name.to_string(),
            args: convert_surface_type_items(args)?,
        }),
        ash_parser::surface::Type::Associated { base, name } => Ok(CoreTypeExpr::Associated {
            base: Box::new(convert_surface_type_expr(base)?),
            name: name.to_string(),
        }),
        ash_parser::surface::Type::Hole { span } => Err(EngineError::Parse(format!(
            "type holes are not supported in type definitions at {span:?}"
        ))),
        ash_parser::surface::Type::Fn(_, _, _)
        | ash_parser::surface::Type::Capability(_)
        | ash_parser::surface::Type::AssociatedFamilyProjection { .. } => Err(EngineError::Parse(
            format!("unsupported type definition field type: {parsed:?}"),
        )),
    }
}

#[cfg(test)]
mod tests;
thread_local! {
    static MODULE_ROOT_OVERRIDE: RefCell<Option<ModuleRootOverride>> = const { RefCell::new(None) };
}

#[derive(Debug, Clone)]
struct ModuleRootOverride {
    dependency_roots: Vec<PathBuf>,
    stdlib_root: Option<PathBuf>,
}

/// Run module-loading code with explicit dependency and stdlib roots.
///
/// This is the Phase 127 installed-toolchain seam used by `ashgrove` and tests
/// so callers do not need process-global environment mutation.
///
/// # Errors
///
/// Returns any error produced by `operation`.
pub fn with_module_roots<T>(
    dependency_roots: Vec<PathBuf>,
    stdlib_root: Option<PathBuf>,
    operation: impl FnOnce() -> Result<T, EngineError>,
) -> Result<T, EngineError> {
    MODULE_ROOT_OVERRIDE.with(|slot| {
        let previous = slot.replace(Some(ModuleRootOverride {
            dependency_roots,
            stdlib_root,
        }));
        let result = operation();
        slot.replace(previous);
        result
    })
}
