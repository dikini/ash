//! Ordinary file loader for import-backed execution.
//!
//! This loader supports a constrained executable subset:
//! - contiguous leading `use` imports on ordinary workflow files
//! - module resolution from the workflow tree, `ASH_LIBRARY_PATH`, and the built-in stdlib
//! - imported `pub type` definitions from resolved modules
//! - imported callable bodies from local workflows and stdlib `pub fn` / `pub use`

use crate::EngineError;
use crate::legacy_workflow_adapter::legacy_workflow_def_to_workflow_form;
use ash_core::ast::{
    TypeBody as CoreTypeBody, TypeDef as CoreTypeDef, TypeExpr as CoreTypeExpr,
    VariantDef as CoreVariantDef, VariantPayload as CoreVariantPayload,
    Visibility as CoreVisibility,
};
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin, TypeDeclSummary,
    TypeRepresentationSummary,
};
use ash_core::workflow_carrier::{
    CoverageEvidence, OpenPostcondition, ProcContractSummary, ProcFailureSummary, ProcLowerSummary,
    ProcProvenanceSummary, ProcResourceAuthoritySummary, ProjectionEvent, ProjectionEventKind,
    ProjectionKind, PublicWorkflowSummary, SourceOrigin, WorkflowBinder, WorkflowForm,
    WorkflowNodeId, WorkflowScope, lower_workflow_form,
};
use ash_parser::input::new_input;
use ash_parser::parse_module::{parse_builtin_fn_definition, parse_fn_definition};
use ash_parser::parse_type_def::{
    TypeBody as ParsedTypeBody, TypeDef as ParsedTypeDef, TypeExpr as ParsedTypeExpr,
    VariantPayload as ParsedVariantPayload, Visibility as ParsedVisibility, parse_type_def,
};
use ash_parser::parse_use::parse_use;
use ash_parser::parse_workflow::workflow_def;
use ash_parser::surface::{Definition, Expr, Type, Workflow, WorkflowDef};
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use winnow::prelude::Parser;

thread_local! {
    static LEGACY_TYPE_SNIPPET_COMPAT_SCOPE: Cell<usize> = const { Cell::new(0) };
}

/// Executes `f` with the legacy ordinary-type snippet compatibility APIs enabled.
///
/// This is an explicit TASK-789 quarantine fence. Normal module checking,
/// import/export collection, and stdlib discovery must not enter this scope;
/// they use parsed `ModuleFile` metadata and semantic summaries instead.
pub fn with_legacy_type_snippet_compat<T>(f: impl FnOnce() -> T) -> T {
    struct ScopeGuard;

    impl Drop for ScopeGuard {
        fn drop(&mut self) {
            LEGACY_TYPE_SNIPPET_COMPAT_SCOPE.with(|scope| {
                let depth = scope.get();
                scope.set(depth.saturating_sub(1));
            });
        }
    }

    LEGACY_TYPE_SNIPPET_COMPAT_SCOPE.with(|scope| scope.set(scope.get() + 1));
    let _guard = ScopeGuard;
    f()
}

/// Ordinary-file loading output after import stripping and dependency collection.
#[derive(Debug, Clone)]
pub struct LoadedOrdinaryFile {
    /// Workflow source with the leading `use` prelude removed.
    pub workflow_source: String,
    /// Imported public type definitions collected from resolved modules.
    pub imported_type_defs: Vec<CoreTypeDef>,
    /// Imported semantic summaries collected from resolved modules.
    pub imported_semantic_summaries: Vec<ModuleSemanticSummary>,
    /// Imported callable bodies keyed by the imported name.
    pub imported_callables: HashMap<String, InlineCallable>,
}

/// Check whether a source file is a valid importable module surface.
///
/// # Errors
///
/// Returns [`EngineError`] if module exports cannot be collected.
pub fn check_importable_module_file(path: &Path) -> Result<(), EngineError> {
    let source = std::fs::read_to_string(path)?;
    let contains_workflow = source_contains_workflow_keyword(&source);
    let mut cache = HashMap::new();
    let mut visiting = HashSet::new();
    let exports = collect_module_exports(path, &mut cache, &mut visiting).map_err(|error| {
        EngineError::Parse(format!(
            "in '{}': failed to collect module exports: {error}",
            path.display()
        ))
    })?;
    if contains_workflow
        && exports.type_defs.is_empty()
        && exports.constructor_defs.is_empty()
        && exports.callables.is_empty()
        && exports.child_modules.is_empty()
    {
        return Err(EngineError::Parse(format!(
            "in '{}': workflow module contains no importable exports",
            path.display()
        )));
    }
    Ok(())
}

fn source_contains_workflow_keyword(source: &str) -> bool {
    source.lines().any(|line| {
        let code = strip_line_comment(line);
        code.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
            .any(|token| token == "workflow")
    })
}

fn strip_line_comment(line: &str) -> &str {
    let dash = line.find("--");
    let slash = line.find("//");
    match (dash, slash) {
        (Some(a), Some(b)) => &line[..a.min(b)],
        (Some(i), None) | (None, Some(i)) => &line[..i],
        (None, None) => line,
    }
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
    /// treated as effectful when lowering nested `act { ... }` expressions.
    pub effectful_names: HashSet<String>,
    /// Whether this callable has an Ash body or is a bodyless builtin.
    pub kind: CallableKind,
    /// Full declared type signature for imported callables.
    pub signature: Option<CallableSignature>,
    /// Public first-class workflow summary for imported `Workflow<A>` exports.
    pub workflow_summary: Option<PublicWorkflowSummary>,
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
struct ImportSpec {
    module_segments: Vec<String>,
    selections: Vec<ImportSelection>,
}

#[derive(Debug, Clone)]
enum ImportSelection {
    Named { name: String, alias: Option<String> },
    Glob,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ModuleExports {
    pub(crate) type_defs: HashMap<String, CoreTypeDef>,
    pub(crate) constructor_defs: HashMap<String, CoreTypeDef>,
    pub(crate) callables: HashMap<String, InlineCallable>,
    /// Core-owned ordinary type semantic summary lowered from the parsed `ModuleFile`.
    pub(crate) semantic_summary: Option<ModuleSemanticSummary>,
    /// Child module exports loaded via `pub mod <name>;` declarations.
    ///
    /// Populated by TASK-540 but not yet consumed by `merge_use_exports` --
    /// `pub use` resolution currently goes through filesystem path resolution
    /// via `resolve_use_target`. This field is infrastructure for future
    /// qualified module path access (`llm::types::Role`).
    pub(crate) child_modules: HashMap<String, Self>,
}

/// Parse source containing zero or more function definitions followed by a
/// workflow definition.
///
/// This handles the extended syntax where `fn` definitions precede the
/// `workflow` block.
///
/// # Errors
///
/// Returns a string describing the parse error if the source is invalid.
///
/// # Panics
///
/// Panics if no workflow definition is found after the check above passes
/// (should be unreachable).
pub fn parse_program_with_functions(source: &str) -> Result<ash_parser::surface::Program, String> {
    use ash_parser::input::new_input;
    use ash_parser::parse_module::parse_fn_definition;
    use ash_parser::parse_utils::skip_whitespace_and_comments;
    use ash_parser::parse_workflow::workflow_def;
    use winnow::Parser;

    let mut input = new_input(source);
    skip_whitespace_and_comments(&mut input);

    // Parse leading fn definitions
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

    // Parse zero or more named workflow definitions (helper workflows).
    // Each must have a name (i.e., not be an anonymous `workflow { ... }`).
    let mut all_workflows = Vec::new();
    loop {
        let snapshot = input.clone();
        if let Ok(wf) = workflow_def.parse_next(&mut input) {
            skip_whitespace_and_comments(&mut input);
            // If there is more input, this is a helper workflow; if EOF, it's
            // the entry workflow. We collect all and split at the end.
            all_workflows.push(wf);
        } else {
            input = snapshot;
            break;
        }
    }

    if all_workflows.is_empty() {
        return Err("expected at least one workflow definition".to_string());
    }

    // The last workflow is the entry point; preceding ones are helpers.
    let workflow = all_workflows.pop().expect("at least one workflow");
    let helper_workflows = all_workflows;

    if !input.input.is_empty() {
        return Err("unexpected trailing input after workflow definition".to_string());
    }

    Ok(ash_parser::surface::Program {
        definitions,
        helper_workflows,
        workflow,
    })
}

/// Load an ordinary workflow file together with its imported metadata.
///
/// # Errors
///
/// Returns [`EngineError`] if the workflow file cannot be read, an import
/// cannot be resolved, or an imported module cannot be parsed into the
/// supported type/callable subset.
#[allow(clippy::too_many_lines)]
pub fn load_ordinary_file(path: &Path) -> Result<LoadedOrdinaryFile, EngineError> {
    let source = std::fs::read_to_string(path)?;
    let canonical_entry = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let entry_root = path.parent().ok_or_else(|| {
        EngineError::Configuration(format!("workflow path '{}' has no parent", path.display()))
    })?;
    let mut module_cache = HashMap::new();
    let mut visiting = HashSet::new();
    visiting.insert(canonical_entry);

    let mut imports = Vec::new();
    let mut kept_lines = Vec::new();
    let mut seen_non_import = false;

    let mut lines = source.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if !seen_non_import && is_skippable_prelude_line(trimmed) {
            kept_lines.push(line.to_string());
            continue;
        }

        if !seen_non_import && (trimmed.starts_with("use ") || trimmed.starts_with("pub use ")) {
            let mut snippet = line.to_string();
            while import_needs_more_lines(&snippet) {
                let Some(next_line) = lines.next() else {
                    break;
                };
                snippet.push('\n');
                snippet.push_str(next_line);
            }
            imports.push(parse_ordinary_import(snippet.trim())?);
            continue;
        }

        seen_non_import = true;
        if let Some(rest) = line.trim_start().strip_prefix("pub workflow ") {
            let indent_len = line.len() - line.trim_start().len();
            kept_lines.push(format!("{}workflow {rest}", &line[..indent_len]));
        } else {
            kept_lines.push(line.to_string());
        }
    }

    let mut imported_type_defs = Vec::new();
    let mut imported_type_names = HashSet::new();
    let mut imported_semantic_summaries = Vec::new();
    let mut imported_summary_keys = HashSet::new();
    let mut imported_callables = HashMap::new();

    let crate_root = discover_crate_root(entry_root);
    for import in imports {
        let absolute_roots = search_roots(entry_root);
        let (module_segments, search_roots) = normalize_import_resolution(
            &import.module_segments,
            entry_root,
            crate_root.as_deref(),
            &absolute_roots,
        );
        let module_path =
            resolve_module_path(&module_segments, &search_roots).ok_or_else(|| {
                EngineError::Parse(format!(
                    "module '{}' not found",
                    import.module_segments.join("::")
                ))
            })?;
        let exports = collect_module_exports(&module_path, &mut module_cache, &mut visiting)?;

        for selection in import.selections {
            match selection {
                ImportSelection::Glob => {
                    for type_def in exports.type_defs.values() {
                        if imported_type_names.insert(type_def.name.clone()) {
                            imported_type_defs.push(type_def.clone());
                        }
                    }
                    if let Some(summary) = exports.semantic_summary.clone() {
                        let key = imported_summary_key(&summary);
                        if imported_summary_keys.insert(key) {
                            imported_semantic_summaries.push(summary);
                        }
                    }
                    for (k, mut v) in exports.callables.clone() {
                        if let CallableKind::Builtin { ref mut module } = v.kind {
                            *module = module_segments.join("::");
                        }
                        imported_callables.insert(k, v);
                    }
                }
                ImportSelection::Named { name, alias } => {
                    let exported_name = alias.unwrap_or_else(|| name.clone());
                    if let Some(type_def) = exports.type_defs.get(&name) {
                        let mut imported_type = type_def.clone();
                        imported_type.name.clone_from(&exported_name);
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
                        if imported_type_names.insert(type_def.name.clone()) {
                            imported_type_defs.push(type_def.clone());
                        }
                        push_selected_semantic_summary(
                            &mut imported_semantic_summaries,
                            &mut imported_summary_keys,
                            exports.semantic_summary.as_ref(),
                            &type_def.name,
                            &type_def.name,
                        );
                    } else if let Some(callable) = exports.callables.get(&name) {
                        push_signature_semantic_summaries(
                            &mut imported_semantic_summaries,
                            &mut imported_summary_keys,
                            exports.semantic_summary.as_ref(),
                            callable,
                        );
                        // Imported callable signatures may mention private/opaque
                        // module-local type identities (for example std::act::Act
                        // and Policy). Bring legacy identities into the type
                        // environment without exposing constructors while semantic
                        // summaries carry selected ordinary identities.
                        for type_def in exports.type_defs.values() {
                            if imported_type_names.insert(type_def.name.clone()) {
                                imported_type_defs.push(type_def.clone());
                            }
                        }
                        let mut callable = callable.clone();
                        callable.exported_name.clone_from(&exported_name);
                        if let CallableKind::Builtin { ref mut module } = callable.kind {
                            *module = module_segments.join("::");
                        }
                        imported_callables.insert(exported_name, callable);
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
        workflow_source: kept_lines.join("\n"),
        imported_type_defs,
        imported_semantic_summaries,
        imported_callables,
    })
}

type ImportedSummaryKey = Vec<(String, String, Option<usize>, usize, String)>;

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
    let Some(selected) = selected_type_semantic_summary(summary, source_name, imported_name) else {
        return;
    };
    let key = imported_summary_key(&selected);
    if imported_summary_keys.insert(key) {
        imported_semantic_summaries.push(selected);
    }
}

fn push_signature_semantic_summaries(
    imported_semantic_summaries: &mut Vec<ModuleSemanticSummary>,
    imported_summary_keys: &mut HashSet<ImportedSummaryKey>,
    summary: Option<&ModuleSemanticSummary>,
    callable: &InlineCallable,
) {
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
        Type::Name(name) | Type::Capability(name) => names.push(name.to_string()),
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
        Type::Fn(params, ret) => {
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
            exported_summary
                .exported_types
                .iter()
                .find(|exported_ty| exported_ty.id == source_ty.id)
                .map(|exported_ty| {
                    (
                        source_ty.exported_name.clone(),
                        exported_ty.exported_name.clone(),
                    )
                })
        })
        .filter(|(source, exported)| source != exported)
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

fn rewrite_surface_type_aliases(ty: &mut Type, aliases: &HashMap<String, String>) {
    match ty {
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
        Type::Fn(params, ret) => {
            for param in params {
                rewrite_surface_type_aliases(param, aliases);
            }
            rewrite_surface_type_aliases(ret, aliases);
        }
    }
}

pub(crate) fn public_callable_signature_visibility_errors(
    source: &str,
    type_defs: &[CoreTypeDef],
) -> Vec<String> {
    let private_ordinary_types = type_defs
        .iter()
        .filter(|type_def| {
            !matches!(type_def.visibility, CoreVisibility::Public)
                && !type_def.builtin
                && !is_existing_opaque_compatibility_exception(type_def)
        })
        .map(|type_def| type_def.name.clone())
        .collect::<HashSet<_>>();
    if private_ordinary_types.is_empty() {
        return Vec::new();
    }

    let mut errors = Vec::new();
    for snippet in extract_braced_snippets(source, is_workflow_export_start) {
        let callable = parse_workflow_callable(&snippet)
            .ok()
            .flatten()
            .or_else(|| parse_workflow_signature_callable(&snippet));
        if let Some(callable) = callable {
            append_callable_signature_visibility_errors(
                &callable.callable,
                &private_ordinary_types,
                &mut errors,
            );
        }
    }
    for snippet in extract_braced_snippets(source, |trimmed| trimmed.starts_with("pub fn ")) {
        if let Ok(Some(callable)) = parse_supported_pub_fn_callable(&snippet) {
            append_callable_signature_visibility_errors(
                &callable.callable,
                &private_ordinary_types,
                &mut errors,
            );
        }
    }
    for snippet in
        extract_semicolon_snippets(source, |trimmed| trimmed.starts_with("pub builtin fn "))
    {
        if let Ok(Some(callable)) = parse_builtin_fn_callable(&snippet, String::new()) {
            append_callable_signature_visibility_errors(
                &callable.callable,
                &private_ordinary_types,
                &mut errors,
            );
        }
    }
    errors
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

/// Compatibility-only extractor for legacy tests that intentionally exercise
/// pre-ModuleFile `pub type` snippet parsing.
///
/// Normal module checking, export collection, and runtime stdlib discovery must
/// use [`collect_module_type_metadata_from_module_file`] (or its runtime wrapper)
/// so `ModuleFile` parsing and semantic summaries are authoritative.
///
/// # Errors
///
/// Returns [`EngineError::Parse`] if any extracted type snippet contains invalid syntax.
pub fn collect_public_type_defs_from_source_compat(
    source: &str,
) -> Result<Vec<CoreTypeDef>, EngineError> {
    ensure_legacy_type_snippet_compat_scope()?;
    let mut type_defs = Vec::new();
    for snippet in extract_semicolon_snippets(source, is_public_type_definition_start) {
        type_defs.push(parse_type_def_snippet(&snippet)?);
    }
    Ok(type_defs)
}

/// Compatibility-only extractor for legacy tests that intentionally exercise
/// pre-ModuleFile ordinary type identity snippet parsing.
///
/// This includes both `type` and `pub type` declarations. It is not a normal
/// semantic path for module checking, imports/exports, or stdlib discovery.
///
/// # Errors
///
/// Returns [`EngineError::Parse`] if any extracted type snippet contains invalid syntax.
pub fn collect_type_identity_defs_from_source_compat(
    source: &str,
) -> Result<Vec<CoreTypeDef>, EngineError> {
    ensure_legacy_type_snippet_compat_scope()?;
    let mut type_defs = Vec::new();
    for snippet in extract_semicolon_snippets(source, is_type_definition_start) {
        type_defs.push(parse_type_def_snippet(&snippet)?);
    }
    Ok(type_defs)
}

fn ensure_legacy_type_snippet_compat_scope() -> Result<(), EngineError> {
    let enabled = LEGACY_TYPE_SNIPPET_COMPAT_SCOPE.with(|scope| scope.get() > 0);
    if enabled {
        Ok(())
    } else {
        Err(EngineError::Parse(
            "legacy ordinary type snippet compatibility path requires explicit with_legacy_type_snippet_compat scope; normal type metadata must use ModuleFile semantic summaries".into(),
        ))
    }
}

/// Parse a module source as a full `ModuleFile` and lower its ordinary type
/// metadata into core declarations plus core-owned semantic summaries.
pub(crate) fn collect_module_type_metadata_from_module_file(
    path: &Path,
    source: &str,
) -> Result<ash_parser::lower::LoweredTypeMetadata, EngineError> {
    let module = parse_module_file_for_type_metadata(path, source)?;
    Ok(ash_parser::lower::lower_module_type_metadata(
        &module,
        module_identity_for_path(path),
    ))
}

pub(crate) fn collect_runtime_stdlib_type_defs_from_module_file(
    module_path: &str,
    source: &str,
) -> Result<Vec<CoreTypeDef>, EngineError> {
    let virtual_path = PathBuf::from(format!("std://{}.ash", module_path.replace("::", "/")));
    Ok(collect_module_type_metadata_from_module_file(&virtual_path, source)?.type_defs)
}

fn parse_module_file_for_type_metadata(
    path: &Path,
    source: &str,
) -> Result<ash_parser::surface::ModuleFile, EngineError> {
    match ash_parser::parse_surface_file_with_path(source, Some(path)) {
        Ok(module) => Ok(module),
        Err(first_errors) if source_contains_workflow_keyword(source) => {
            let projected = module_source_without_legacy_workflow_exports(source);
            if projected == source {
                return Err(module_type_metadata_parse_error(path, &first_errors));
            }
            ash_parser::parse_surface_file_with_path(&projected, Some(path))
                .map_err(|_| module_type_metadata_parse_error(path, &first_errors))
        }
        Err(errors) => Err(module_type_metadata_parse_error(path, &errors)),
    }
}

/// Compatibility fence for Phase 108 legacy `pub workflow` and unsupported
/// `pub fn` export snippets.
///
/// TASK-785 routes ordinary type metadata through parsed `ModuleFile` lowering.
/// Some legacy workflow-export and broken/unsupported public function snippets
/// are still collected by existing snippet paths and are not accepted by full
/// `ModuleFile` parsing. This projection removes only braced non-type exports,
/// then retries `ModuleFile` parsing for ordinary type metadata. It is
/// intentionally narrow and must not be generalized into ordinary type snippet
/// scanning.
fn module_source_without_legacy_workflow_exports(source: &str) -> String {
    let mut projected = source.to_string();
    for snippet in extract_braced_snippets(source, is_workflow_export_start) {
        projected = projected.replace(&snippet, "");
    }
    for snippet in extract_braced_snippets(source, |trimmed| trimmed.starts_with("pub fn ")) {
        projected = projected.replace(&snippet, "");
    }
    projected
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

fn module_identity_for_path(path: &Path) -> ModuleIdentity {
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

fn is_public_type_definition_start(trimmed: &str) -> bool {
    trimmed.starts_with("pub type ") || trimmed.starts_with("pub builtin type ")
}

fn is_type_definition_start(trimmed: &str) -> bool {
    trimmed.starts_with("type ")
        || trimmed.starts_with("pub type ")
        || trimmed.starts_with("builtin type ")
        || trimmed.starts_with("pub builtin type ")
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
    if let Some(exports) = cache.get(&path) {
        return Ok(exports.clone());
    }

    let source = std::fs::read_to_string(&path)?;
    let mut exports = ModuleExports::default();
    let module_effectful_names = ash_parser::parse_surface_file(&source)
        .ok()
        .map(|module| ash_parser::effectful_names_from_definitions(&module.definitions))
        .unwrap_or_default();

    let type_metadata = collect_module_type_metadata_from_module_file(&path, &source)?;
    for type_def in &type_metadata.type_defs {
        insert_type_export(&mut exports, type_def)?;
    }
    exports.semantic_summary = Some(exportable_module_semantic_summary(
        &type_metadata.summary,
        &exports.type_defs,
    ));

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
            let exported_name = callable.exported_name.clone();
            insert_callable_export(&mut exports, &exported_name, callable)?;
        }
    }

    for snippet in extract_braced_snippets(&source, is_workflow_export_start) {
        if let Ok(Some(callable)) = parse_workflow_callable(&snippet) {
            let mut callable = callable.callable;
            callable.effectful_names.clone_from(&module_effectful_names);
            let exported_name = callable.exported_name.clone();
            if let Some(summary) = callable.workflow_summary.as_mut() {
                stamp_workflow_summary_import_origin(
                    summary,
                    module_path_text(path.as_path()),
                    &exported_name,
                );
            }
            insert_callable_export(&mut exports, &exported_name, callable)?;
        } else if let Some(callable) = parse_workflow_signature_callable(&snippet) {
            let mut callable = callable.callable;
            callable.effectful_names.clone_from(&module_effectful_names);
            let exported_name = callable.exported_name.clone();
            if let Some(summary) = callable.workflow_summary.as_mut() {
                stamp_workflow_summary_import_origin(
                    summary,
                    module_path_text(path.as_path()),
                    &exported_name,
                );
            }
            insert_callable_export(&mut exports, &exported_name, callable)?;
        }
        // Silently skip workflows that fail to parse during module export collection.
        // This mirrors the graceful handling of pub fn parse failures above.
    }

    for snippet in extract_braced_snippets(&source, |trimmed| trimmed.starts_with("pub fn ")) {
        match parse_supported_pub_fn_callable(&snippet) {
            Ok(Some(callable)) => {
                let mut callable = callable.callable;
                callable.effectful_names.clone_from(&module_effectful_names);
                let exported_name = callable.exported_name.clone();
                if let Some(summary) = callable.workflow_summary.as_mut() {
                    stamp_workflow_summary_import_origin(
                        summary,
                        module_path_text(path.as_path()),
                        &exported_name,
                    );
                }
                insert_callable_export(&mut exports, &exported_name, callable)?;
            }
            Ok(None) => {}
            Err(_diag) => {
                // Silently skip unsupported pub fn during module loading.
                // Diagnostics are surfaced via check_module_file.
            }
        }
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
            let absolute_roots = search_roots(module_root);
            let (module_segments, search_roots) = normalize_import_resolution(
                &import_spec.module_segments,
                module_root,
                crate_root.as_deref(),
                &absolute_roots,
            );
            if let Some(target_path) = resolve_module_path(&module_segments, &search_roots) {
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
        let child_path = resolve_child_module(module_root, &name)?;
        visiting.insert(canonical.clone());
        let child_exports = collect_module_exports(&child_path, cache, visiting)?;
        visiting.remove(&canonical);
        // Store child exports under the child module name (for qualified access)
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
        let target_exports = collect_module_exports(&resolved, cache, visiting)?;
        visiting.remove(&canonical);
        merge_use_exports(&mut exports, target_exports, use_stmt)?;
    }

    cache.insert(path.clone(), exports.clone());
    Ok(exports)
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
    let absolute_roots = search_roots(module_root);
    let crate_root = discover_crate_root(module_root);
    let (module_segments, search_roots) = normalize_import_resolution(
        &module_segments,
        module_root,
        crate_root.as_deref(),
        &absolute_roots,
    );

    resolve_module_path(&module_segments, &search_roots).ok_or_else(|| {
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
            for (name, callable) in target_exports.callables {
                insert_callable_export(exports, &name, callable)?;
            }
        }
        UsePath::Simple(path) => {
            let name = path
                .segments
                .last()
                .map(std::string::ToString::to_string)
                .ok_or_else(|| EngineError::Parse("empty use path".to_string()))?;
            let exported_name = use_stmt
                .alias
                .map_or_else(|| name.clone(), |alias| alias.to_string());
            if let Some(type_def) = target_exports.type_defs.get(&name) {
                insert_type_export_with_name(exports, &exported_name, type_def.clone())?;
                if let Some(summary) = target_semantic_summary.as_ref() {
                    merge_type_summary_export(exports, summary, &name, &exported_name)?;
                }
            } else if let Some(type_def) = target_exports.constructor_defs.get(&name) {
                insert_constructor_export_with_name(exports, &exported_name, type_def.clone())?;
            } else if let Some(callable) = target_exports.callables.get(&name) {
                let mut callable = callable.clone();
                rewrite_callable_signature_aliases(
                    &mut callable,
                    target_semantic_summary.as_ref(),
                    exports.semantic_summary.as_ref(),
                );
                callable.exported_name.clone_from(&exported_name);
                insert_callable_export(exports, &exported_name, callable)?;
            } else {
                return Err(missing_pub_use_target_error(&name));
            }
        }
        UsePath::Nested(_, items) => {
            for item in items {
                let exported_name = item
                    .alias
                    .map_or_else(|| item.name.to_string(), |alias| alias.to_string());
                if let Some(type_def) = target_exports.type_defs.get(item.name.as_ref()) {
                    insert_type_export_with_name(exports, &exported_name, type_def.clone())?;
                    if let Some(summary) = target_semantic_summary.as_ref() {
                        merge_type_summary_export(
                            exports,
                            summary,
                            item.name.as_ref(),
                            &exported_name,
                        )?;
                    }
                } else if let Some(type_def) =
                    target_exports.constructor_defs.get(item.name.as_ref())
                {
                    insert_constructor_export_with_name(exports, &exported_name, type_def.clone())?;
                } else if let Some(callable) = target_exports.callables.get(item.name.as_ref()) {
                    let mut callable = callable.clone();
                    rewrite_callable_signature_aliases(
                        &mut callable,
                        target_semantic_summary.as_ref(),
                        exports.semantic_summary.as_ref(),
                    );
                    callable.exported_name.clone_from(&exported_name);
                    insert_callable_export(exports, &exported_name, callable)?;
                } else {
                    return Err(missing_pub_use_target_error(item.name.as_ref()));
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
    } else if type_def.builtin || is_existing_opaque_compatibility_exception(type_def) {
        let mut opaque = type_def.clone();
        opaque.body = CoreTypeBody::Struct(vec![]);
        opaque.builtin = true;
        insert_type_export_with_name(exports, &type_def.name, opaque)?;
    } else {
        return Ok(());
    }

    // Only public type definitions expose constructor names/representation to
    // importing modules.
    if matches!(type_def.visibility, CoreVisibility::Public)
        && let CoreTypeBody::Enum(variants) = &type_def.body
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

fn exportable_module_semantic_summary(
    raw: &ModuleSemanticSummary,
    exportable_types: &HashMap<String, CoreTypeDef>,
) -> ModuleSemanticSummary {
    let mut summary = raw.clone();
    summary.exported_types = raw
        .exported_types
        .iter()
        .filter_map(|ty| exportable_type_summary(ty, exportable_types))
        .collect();
    summary.exported_constructors = raw
        .exported_constructors
        .iter()
        .filter(|constructor| exportable_types.contains_key(constructor.parent.name.as_str()))
        .cloned()
        .collect();
    summary
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

fn selected_type_semantic_summary(
    summary: &ModuleSemanticSummary,
    source_name: &str,
    imported_name: &str,
) -> Option<ModuleSemanticSummary> {
    let selected = summary
        .exported_types
        .iter()
        .find(|ty| ty.exported_name == source_name)?;
    let mut selected_type = selected.clone();
    selected_type.exported_name = imported_name.into();
    let mut selected_summary = ModuleSemanticSummary::new(summary.module.clone());
    selected_summary.version = summary.version;
    selected_summary.exported_types.push(selected_type);
    selected_summary.exported_constructors = summary
        .exported_constructors
        .iter()
        .filter(|constructor| constructor.parent == selected.id)
        .cloned()
        .collect();
    selected_summary.re_exports.clone_from(&summary.re_exports);
    selected_summary
        .imported_summary_refs
        .clone_from(&summary.imported_summary_refs);
    selected_summary.reserved_identity_slots = summary.reserved_identity_slots.clone();
    selected_summary
        .diagnostic_anchors
        .clone_from(&summary.diagnostic_anchors);
    Some(selected_summary)
}

fn imported_summary_key(summary: &ModuleSemanticSummary) -> ImportedSummaryKey {
    let mut key = summary
        .exported_types
        .iter()
        .map(|ty| {
            (
                ty.exported_name.clone(),
                ty.id.name.clone(),
                ty.id.module.crate_id.map(|crate_id| crate_id.0),
                ty.id.module.module_id.0,
                format!("{:?}", ty.id.item_kind),
            )
        })
        .collect::<Vec<_>>();
    key.sort_unstable();
    key
}

fn merge_type_summary_export(
    exports: &mut ModuleExports,
    target_summary: &ModuleSemanticSummary,
    source_name: &str,
    exported_name: &str,
) -> Result<(), EngineError> {
    let Some(mut ty) = target_summary
        .exported_types
        .iter()
        .find(|ty| ty.exported_name == source_name)
        .cloned()
    else {
        return Ok(());
    };
    ty.exported_name = exported_name.into();
    let summary = exports
        .semantic_summary
        .get_or_insert_with(|| ModuleSemanticSummary::new(target_summary.module.clone()));
    if let Some(existing) = summary
        .exported_types
        .iter()
        .find(|existing| existing.exported_name == ty.exported_name)
    {
        if existing.id == ty.id {
            return Ok(());
        }
        return Err(EngineError::Configuration(format!(
            "duplicate exported type semantic summary '{}'",
            ty.exported_name
        )));
    }
    summary.exported_types.push(ty.clone());
    for constructor in target_summary
        .exported_constructors
        .iter()
        .filter(|constructor| constructor.parent == ty.id)
    {
        if !summary
            .exported_constructors
            .iter()
            .any(|existing| existing.id == constructor.id)
        {
            summary.exported_constructors.push(constructor.clone());
        }
    }
    Ok(())
}

fn is_existing_opaque_compatibility_exception(type_def: &CoreTypeDef) -> bool {
    // Phase 97 std::act established an opaque public boundary using a private
    // ordinary `Act<A>` alias over the explicit builtin `ActEnv` substrate. Keep
    // that named identity importable until TASK-787+TASK-790 move the exception
    // into a core-owned summary/diagnostic rule; do not generalize this to other
    // private ordinary types.
    type_def.name == "Act" && !matches!(type_def.visibility, CoreVisibility::Public)
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

fn extract_public_capability_names(source: &str) -> Vec<String> {
    extract_semicolon_snippets(source, |trimmed| trimmed.starts_with("pub capability "))
        .into_iter()
        .filter_map(|snippet| {
            snippet
                .trim()
                .strip_prefix("pub capability ")
                .and_then(|rest| rest.split(':').next())
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(ToOwned::to_owned)
        })
        .collect()
}

fn capability_type_identity(name: &str) -> CoreTypeDef {
    CoreTypeDef {
        name: name.to_string(),
        params: Vec::new(),
        body: CoreTypeBody::Struct(vec![]),
        visibility: CoreVisibility::Public,
        builtin: true,
    }
}

fn parse_type_def_snippet(snippet: &str) -> Result<CoreTypeDef, EngineError> {
    if let Some(type_def) = parse_simple_type_alias_snippet(snippet) {
        return Ok(type_def);
    }

    let mut input = new_input(snippet.trim());
    let parsed = parse_type_def
        .parse_next(&mut input)
        .map_err(|error| EngineError::Parse(format!("{error}")))?;
    Ok(convert_type_def(&parsed))
}

fn parse_simple_type_alias_snippet(snippet: &str) -> Option<CoreTypeDef> {
    let trimmed = snippet.trim().strip_suffix(';')?.trim();
    let rest = trimmed.strip_prefix("pub type ")?.trim();
    let (name, target) = rest.split_once('=')?;
    let name = name.trim();
    let target = target.trim();

    let (name, params) = if let Some((base, params_text)) = name.split_once('<') {
        let params_text = params_text.strip_suffix('>')?;
        (
            base.trim(),
            params_text
                .split(',')
                .map(str::trim)
                .filter(|param| !param.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
        )
    } else {
        (name, Vec::new())
    };

    if name.is_empty() || target.is_empty() || target.contains('{') || target.contains('|') {
        return None;
    }

    Some(CoreTypeDef {
        name: name.to_string(),
        params,
        body: CoreTypeBody::Alias(convert_simple_type_expr(target)?),
        visibility: CoreVisibility::Public,
        builtin: false,
    })
}

fn convert_simple_type_expr(text: &str) -> Option<CoreTypeExpr> {
    let text = text.trim();
    if let Some((name, args_text)) = text.split_once('<') {
        let args_text = args_text.strip_suffix('>')?;
        let args = args_text
            .split(',')
            .map(convert_simple_type_expr)
            .collect::<Option<Vec<_>>>()?;
        return Some(CoreTypeExpr::Constructor {
            name: name.trim().to_string(),
            args,
        });
    }

    if text
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == ':')
    {
        Some(CoreTypeExpr::Named(text.to_string()))
    } else {
        None
    }
}

fn is_workflow_export_start(trimmed: &str) -> bool {
    starts_with_keyword(trimmed, "workflow") || starts_with_keyword(trimmed, "pub workflow")
}

fn starts_with_keyword(text: &str, keyword: &str) -> bool {
    text.strip_prefix(keyword).is_some_and(|rest| {
        rest.chars()
            .next()
            .is_some_and(|ch| ch.is_whitespace() || ch == '(')
    })
}

fn parse_workflow_signature_callable(snippet: &str) -> Option<ImportedCallableExport> {
    let trimmed = snippet.trim();
    let rest = trimmed
        .strip_prefix("pub workflow ")
        .or_else(|| trimmed.strip_prefix("workflow "))?;
    let (name_text, after_name) = rest.split_once('(')?;
    let name = name_text.trim();
    if name.is_empty() {
        return None;
    }
    let (params_text, after_params) = split_balanced_prefix(after_name, '(', ')')?;
    let params = parse_workflow_signature_params(params_text)?;
    let return_type = parse_workflow_signature_return_type(after_params)?;
    let body = workflow_signature_expr_for_body_text(snippet);
    let fn_def = ash_parser::surface::FnDef {
        visibility: ash_parser::surface::Visibility::Public,
        name: name.into(),
        type_params: Vec::new(),
        params,
        return_type: Some(return_type),
        contract: None,
        body: body.clone(),
        span: ash_parser::token::Span::default(),
    };

    Some(ImportedCallableExport {
        callable: InlineCallable {
            exported_name: name.to_string(),
            params: fn_def
                .params
                .iter()
                .map(|param| param.name.to_string())
                .collect(),
            effectful_names: HashSet::new(),
            kind: CallableKind::User { body },
            signature: Some(CallableSignature::Function(fn_def)),
            workflow_summary: None,
        },
    })
}

fn split_balanced_prefix(text: &str, open: char, close: char) -> Option<(&str, &str)> {
    let mut depth = 1usize;
    for (index, ch) in text.char_indices() {
        match ch {
            ch if ch == open => depth += 1,
            ch if ch == close => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some((&text[..index], &text[index + ch.len_utf8()..]));
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_workflow_signature_params(text: &str) -> Option<Vec<ash_parser::surface::Param>> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Some(Vec::new());
    }

    trimmed
        .split(',')
        .map(|param| {
            let (name, ty) = param.split_once(':')?;
            Some(ash_parser::surface::Param {
                name: name.trim().into(),
                ty: parse_workflow_signature_type(ty.trim())?,
            })
        })
        .collect()
}

fn parse_workflow_signature_return_type(text: &str) -> Option<ash_parser::surface::Type> {
    let after_arrow = text.trim_start().strip_prefix("->")?;
    let ty_text = after_arrow.split_once('{')?.0.trim();
    parse_workflow_signature_type(ty_text)
}

fn parse_workflow_signature_type(text: &str) -> Option<ash_parser::surface::Type> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Some(inner) = text.strip_prefix("cap ") {
        return Some(ash_parser::surface::Type::Capability(inner.trim().into()));
    }
    if let Some((name, args_text)) = text.split_once('<') {
        let args_text = args_text.strip_suffix('>')?;
        let args = split_top_level_commas(args_text)
            .into_iter()
            .map(parse_workflow_signature_type)
            .collect::<Option<Vec<_>>>()?;
        return Some(ash_parser::surface::Type::Constructor {
            name: name.trim().into(),
            args,
        });
    }
    Some(ash_parser::surface::Type::Name(text.into()))
}

fn split_top_level_commas(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, ch) in text.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(text[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(text[start..].trim());
    parts
}

const fn workflow_signature_expr_for_body_text(_snippet: &str) -> Expr {
    Expr::Literal(ash_parser::surface::Literal::Null)
}

fn parse_workflow_callable(snippet: &str) -> Result<Option<ImportedCallableExport>, EngineError> {
    let trimmed = snippet.trim();
    let mut normalized = String::new();
    let workflow_source = trimmed
        .strip_prefix("pub workflow ")
        .map_or(trimmed, |rest| {
            normalized = format!("workflow {rest}");
            normalized.as_str()
        });
    let mut input = new_input(workflow_source);
    let parsed = workflow_def
        .parse_next(&mut input)
        .map_err(|error| EngineError::Parse(format!("{error}")))?;
    extract_callable_from_workflow(parsed)
}

fn parse_pub_fn_callable(snippet: &str) -> Result<Option<ImportedCallableExport>, EngineError> {
    let mut input = new_input(snippet.trim());
    let parsed = parse_fn_definition
        .parse_next(&mut input)
        .map_err(|error| EngineError::Parse(format!("{error}")))?;

    let Definition::Function(function) = parsed else {
        return Err(EngineError::Parse(
            "expected pub fn to parse as a function definition".to_string(),
        ));
    };

    let name = function.name.to_string();
    let params = function
        .params
        .iter()
        .map(|param| param.name.to_string())
        .collect::<Vec<_>>();

    let workflow_summary = workflow_returning_pub_fn_summary(&function);

    Ok(Some(ImportedCallableExport {
        callable: InlineCallable {
            exported_name: name,
            params,
            effectful_names: HashSet::new(),
            kind: CallableKind::User {
                body: function.body.clone(),
            },
            signature: Some(CallableSignature::Function(function)),
            workflow_summary,
        },
    }))
}

/// Conservative public-summary adapter for parser-only module export collection.
///
/// This intentionally recognizes only first-class `do:Workflow` expressions
/// whose public contract statements can be classified without typed lowering.
/// Unsupported shapes return `None` rather than inventing public workflow
/// metadata; full typed workflow expression lowering remains owned by typeck.
fn workflow_returning_pub_fn_summary(
    function: &ash_parser::surface::FnDef,
) -> Option<PublicWorkflowSummary> {
    let return_type = function.return_type.as_ref()?;
    if !is_workflow_return_type(return_type) {
        return None;
    }

    let workflow_expr = workflow_summary_source_expr(&function.body)?;
    let form = workflow_expr_summary_form(workflow_expr, function.name.as_ref())?;
    let origin = SourceOrigin::ImportedSummary {
        module: String::new(),
        public_anchor: function.name.to_string(),
    };
    let mut lowered = lower_workflow_form(&form, origin.clone());
    for event in &mut lowered.projection_events {
        event.origin = origin.clone();
    }
    Some(PublicWorkflowSummary {
        node_count: lowered.projection_events.len(),
        projection_events: lowered.projection_events,
        coverage: lowered.coverage,
    })
}

fn workflow_summary_source_expr(expr: &Expr) -> Option<&Expr> {
    match expr {
        Expr::DoBlock { .. } => Some(expr),
        Expr::Block {
            statements,
            tail_expr: Some(tail_expr),
            ..
        } if statements.is_empty() => workflow_summary_source_expr(tail_expr),
        _ => None,
    }
}

fn workflow_expr_summary_form(expr: &Expr, anchor: &str) -> Option<WorkflowForm<()>> {
    match expr {
        Expr::DoBlock { target, stmts, .. } if target.name.as_ref() == "Workflow" => {
            let mut next_node = 1;
            let form = workflow_do_stmts_summary_form(stmts, &mut next_node, anchor)?;
            Some(WorkflowForm::Scope {
                node: WorkflowNodeId(next_node),
                scope: WorkflowScope {
                    name: Some(anchor.to_string()),
                    origin: first_class_workflow_source_origin(),
                },
                body: Box::new(form),
            })
        }
        _ => None,
    }
}

fn workflow_do_stmts_summary_form(
    stmts: &[ash_parser::surface::DoStmt],
    next_node: &mut u64,
    anchor: &str,
) -> Option<WorkflowForm<()>> {
    match stmts {
        [ash_parser::surface::DoStmt::Return { .. }] => Some(WorkflowForm::FromProc {
            node: workflow_summary_node(next_node),
            summary: first_class_body_proc_summary(anchor),
        }),
        [
            ash_parser::surface::DoStmt::WorkflowRequires { expr, .. },
            rest @ ..,
        ] => {
            let node = workflow_summary_node(next_node);
            let requirement =
                ash_parser::workflow_contract_classifier::classify_requirement(expr).ok()?;
            let source = WorkflowForm::Requires { node, requirement };
            let next = workflow_do_stmts_summary_form(rest, next_node, anchor)?;
            Some(WorkflowForm::Bind {
                node: workflow_summary_node(next_node),
                source: Box::new(source),
                binder: WorkflowBinder::Ignored,
                next: Box::new(next),
            })
        }
        [
            ash_parser::surface::DoStmt::WorkflowEnsures { expr, .. },
            rest @ ..,
        ] => {
            let node = workflow_summary_node(next_node);
            let postcondition =
                ash_parser::workflow_contract_classifier::classify_postcondition(expr).ok()?;
            let source = WorkflowForm::Ensures {
                node,
                postcondition: OpenPostcondition {
                    predicate: postcondition,
                },
            };
            let next = workflow_do_stmts_summary_form(rest, next_node, anchor)?;
            Some(WorkflowForm::Bind {
                node: workflow_summary_node(next_node),
                source: Box::new(source),
                binder: WorkflowBinder::Ignored,
                next: Box::new(next),
            })
        }
        _ => None,
    }
}

fn first_class_body_proc_summary(anchor: &str) -> ProcLowerSummary {
    ProcLowerSummary {
        coverage_obligation_nodes: Vec::new(),
        contract_summary: Some(ProcContractSummary {
            obligations: Vec::new(),
            public_anchor: Some(format!("first_class_body_as_proc_summary:{anchor}")),
        }),
        failure_summary: Some(ProcFailureSummary {
            routes: Vec::new(),
            conservative: false,
        }),
        resource_authority_summary: Some(ProcResourceAuthoritySummary {
            resources: Vec::new(),
            conservative: false,
        }),
        provenance_summary: Some(ProcProvenanceSummary {
            event_kinds: Vec::new(),
            conservative: false,
        }),
        source_origin: Some(first_class_workflow_source_origin()),
    }
}

fn first_class_workflow_source_origin() -> SourceOrigin {
    SourceOrigin::Synthetic {
        parent_span: None,
        reason: "first-class do:Workflow public summary adapter".to_string(),
    }
}

const fn workflow_summary_node(next_node: &mut u64) -> WorkflowNodeId {
    let node = WorkflowNodeId(*next_node);
    *next_node += 1;
    node
}

fn is_workflow_return_type(return_type: &Type) -> bool {
    match return_type {
        Type::Constructor { name, args } => name.as_ref() == "Workflow" && args.len() == 1,
        Type::Name(name) => name.as_ref() == "Workflow",
        _ => false,
    }
}

/// Diagnostic produced when a `pub fn` snippet fails to parse.
#[derive(Debug, Clone)]
pub struct PubFnDiagnostic {
    /// Function name extracted from the snippet, if possible.
    pub name: Option<String>,
    /// Human-readable reason for the failure.
    pub reason: String,
}

/// Attempt to extract the function name from a `pub fn` snippet.
fn extract_fn_name_from_snippet(snippet: &str) -> Option<String> {
    let trimmed = snippet.trim();
    trimmed
        .strip_prefix("pub fn ")
        .and_then(|rest| rest.split(|c: char| c.is_whitespace() || c == '(').next())
        .map(std::string::ToString::to_string)
}

fn parse_supported_pub_fn_callable(
    snippet: &str,
) -> Result<Option<ImportedCallableExport>, PubFnDiagnostic> {
    parse_pub_fn_callable(snippet).map_err(|e| PubFnDiagnostic {
        name: extract_fn_name_from_snippet(snippet),
        reason: format!("{e}"),
    })
}

/// Parse a `builtin fn` snippet into an [`ImportedCallableExport`].
///
/// Builtin functions have no Ash-level body, so a null placeholder expression
/// is used.  The important data is the function name and parameter list.
fn parse_builtin_fn_callable(
    snippet: &str,
    module: String,
) -> Result<Option<ImportedCallableExport>, EngineError> {
    let mut input = new_input(snippet.trim());
    let parsed = parse_builtin_fn_definition
        .parse_next(&mut input)
        .map_err(|error| EngineError::Parse(format!("{error}")))?;

    let Definition::BuiltinFn(builtin) = parsed else {
        return Err(EngineError::Parse(
            "expected builtin fn to parse as a BuiltinFn definition".to_string(),
        ));
    };

    let name = builtin.name.to_string();
    let params = builtin
        .params
        .iter()
        .map(|param| param.name.to_string())
        .collect::<Vec<_>>();

    Ok(Some(ImportedCallableExport {
        callable: InlineCallable {
            exported_name: name,
            params,
            effectful_names: HashSet::new(),
            kind: CallableKind::Builtin { module },
            signature: Some(CallableSignature::Builtin(builtin)),
            workflow_summary: None,
        },
    }))
}

#[derive(Debug, Clone)]
struct ImportedCallableExport {
    callable: InlineCallable,
}

#[allow(clippy::unnecessary_wraps)]
fn extract_callable_from_workflow(
    workflow: WorkflowDef,
) -> Result<Option<ImportedCallableExport>, EngineError> {
    let workflow_summary = public_workflow_summary_from_workflow(&workflow);
    let WorkflowDef {
        name,
        params,
        declared_return_type,
        body,
        span,
        ..
    } = workflow;

    let fn_def = workflow_signature_from_parts(
        name.clone(),
        params.clone(),
        declared_return_type,
        &body,
        span,
    );
    let signature = Some(CallableSignature::Function(fn_def.clone()));
    let expr = match body {
        Workflow::Ret { expr, .. } => expr,
        _ => workflow_signature_expr(&fn_def),
    };

    Ok(Some(ImportedCallableExport {
        callable: InlineCallable {
            exported_name: name.to_string(),
            params: params
                .into_iter()
                .map(|param| param.name.to_string())
                .collect(),
            effectful_names: HashSet::new(),
            kind: CallableKind::User { body: expr },
            signature,
            workflow_summary: Some(workflow_summary),
        },
    }))
}

fn public_workflow_summary_from_workflow(workflow: &WorkflowDef) -> PublicWorkflowSummary {
    let origin = SourceOrigin::ImportedSummary {
        module: String::new(),
        public_anchor: workflow.name.to_string(),
    };
    let Ok(form) = legacy_workflow_def_to_workflow_form(workflow) else {
        return public_workflow_summary(workflow.name.as_ref());
    };
    let lowered = lower_workflow_form(&form, origin);
    PublicWorkflowSummary {
        node_count: lowered.projection_events.len(),
        projection_events: lowered.projection_events,
        coverage: lowered.coverage,
    }
}

fn public_workflow_summary(anchor: &str) -> PublicWorkflowSummary {
    PublicWorkflowSummary {
        node_count: 1,
        projection_events: vec![ProjectionEvent {
            node: WorkflowNodeId(0),
            projection: ProjectionKind::Contract,
            origin: SourceOrigin::ImportedSummary {
                module: String::new(),
                public_anchor: anchor.to_string(),
            },
            kind: ProjectionEventKind::Neutral,
        }],
        coverage: CoverageEvidence::default(),
    }
}

fn module_path_text(path: &Path) -> &str {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
}

fn stamp_workflow_summary_import_origin(
    summary: &mut PublicWorkflowSummary,
    module: &str,
    public_anchor: &str,
) {
    let origin = SourceOrigin::ImportedSummary {
        module: module.to_string(),
        public_anchor: public_anchor.to_string(),
    };
    for event in &mut summary.projection_events {
        event.origin = origin.clone();
    }
}

fn workflow_signature_from_parts(
    name: ash_parser::surface::Name,
    params: Vec<ash_parser::surface::Parameter>,
    return_type: Option<ash_parser::surface::Type>,
    body: &Workflow,
    span: ash_parser::token::Span,
) -> ash_parser::surface::FnDef {
    ash_parser::surface::FnDef {
        visibility: ash_parser::surface::Visibility::Public,
        name,
        type_params: Vec::new(),
        params: params
            .into_iter()
            .map(|param| ash_parser::surface::Param {
                name: param.name,
                ty: param.ty,
            })
            .collect(),
        return_type,
        contract: None,
        body: workflow_signature_expr_for_body(body),
        span,
    }
}

fn workflow_signature_expr(fn_def: &ash_parser::surface::FnDef) -> Expr {
    fn_def.body.clone()
}

fn workflow_signature_expr_for_body(body: &Workflow) -> Expr {
    let span = workflow_span(body);
    Expr::Variable {
        name: "__imported_workflow_body_not_inlined".into(),
        span,
    }
}

const fn workflow_span(workflow: &Workflow) -> ash_parser::token::Span {
    match workflow {
        Workflow::Observe { span, .. }
        | Workflow::Orient { span, .. }
        | Workflow::Propose { span, .. }
        | Workflow::Decide { span, .. }
        | Workflow::Check { span, .. }
        | Workflow::Oblige { span, .. }
        | Workflow::Act { span, .. }
        | Workflow::Let { span, .. }
        | Workflow::If { span, .. }
        | Workflow::For { span, .. }
        | Workflow::With { span, .. }
        | Workflow::Maybe { span, .. }
        | Workflow::Must { span, .. }
        | Workflow::Seq { span, .. }
        | Workflow::Done { span, .. }
        | Workflow::Ret { span, .. }
        | Workflow::Set { span, .. }
        | Workflow::Send { span, .. }
        | Workflow::Receive { span, .. }
        | Workflow::Yield { span, .. }
        | Workflow::Resume { span, .. } => *span,
    }
}
fn extract_pub_mod_declarations(source: &str) -> Vec<String> {
    extract_semicolon_snippets(source, |trimmed| trimmed.starts_with("pub mod "))
        .iter()
        .filter_map(|snippet| {
            let trimmed = snippet.trim();
            trimmed
                .strip_prefix("pub mod ")
                .map(|rest| rest.trim().trim_end_matches(';').trim().to_string())
        })
        .filter(|name| !name.is_empty())
        .collect()
}

fn resolve_child_module(module_root: &Path, name: &str) -> Result<PathBuf, EngineError> {
    // Try name.ash first, then name/mod.ash
    let file_candidate = module_root.join(format!("{name}.ash"));
    if file_candidate.is_file() {
        return Ok(file_candidate);
    }
    let mod_candidate = module_root.join(name).join("mod.ash");
    if mod_candidate.is_file() {
        return Ok(mod_candidate);
    }
    Err(EngineError::Parse(format!(
        "pub mod '{name}': module not found (searched {} and {})",
        file_candidate.display(),
        mod_candidate.display()
    )))
}

fn extract_import_snippets(source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut snippets = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        if trimmed.starts_with("--") || trimmed.starts_with("pub use ") {
            index += 1;
            continue;
        }

        if trimmed.starts_with("use ") {
            let mut snippet = lines[index].to_string();
            while import_needs_more_lines(&snippet) {
                index += 1;
                if index >= lines.len() {
                    break;
                }
                snippet.push('\n');
                snippet.push_str(lines[index]);
            }
            snippets.push(snippet);
        }

        index += 1;
    }

    snippets
}

fn extract_semicolon_snippets<F>(source: &str, predicate: F) -> Vec<String>
where
    F: Fn(&str) -> bool,
{
    let lines: Vec<&str> = source.lines().collect();
    let mut snippets = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        if trimmed.starts_with("--") {
            index += 1;
            continue;
        }

        if predicate(trimmed) {
            let mut snippet = String::new();
            while index < lines.len() {
                if !snippet.is_empty() {
                    snippet.push('\n');
                }
                snippet.push_str(lines[index]);
                if lines[index].contains(';') {
                    snippets.push(snippet);
                    break;
                }
                index += 1;
            }
        }

        index += 1;
    }

    snippets
}

/// Extract braced code snippets from a source string whose first line matches
/// the given predicate.
///
/// Walks the source line-by-line looking for lines that satisfy `predicate`,
/// then consumes the matching brace-delimited block and returns it as a
/// standalone snippet string.
pub fn extract_braced_snippets<F>(source: &str, predicate: F) -> Vec<String>
where
    F: Fn(&str) -> bool,
{
    let lines: Vec<&str> = source.lines().collect();
    let mut snippets = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        if trimmed.starts_with("--") {
            index += 1;
            continue;
        }

        if predicate(trimmed) {
            let mut snippet = String::new();
            let mut brace_depth = 0usize;
            let mut seen_open = false;

            while index < lines.len() {
                if !snippet.is_empty() {
                    snippet.push('\n');
                }
                snippet.push_str(lines[index]);

                for ch in lines[index].chars() {
                    match ch {
                        '{' => {
                            brace_depth += 1;
                            seen_open = true;
                        }
                        '}' if brace_depth > 0 => {
                            brace_depth -= 1;
                        }
                        _ => {}
                    }
                }

                if seen_open && brace_depth == 0 {
                    snippets.push(snippet);
                    break;
                }

                index += 1;
            }
        }

        index += 1;
    }

    snippets
}

fn resolve_module_path(module_segments: &[String], search_roots: &[PathBuf]) -> Option<PathBuf> {
    for root in search_roots {
        if let Some(path) = resolve_in_root(root.as_path(), module_segments) {
            return Some(path);
        }
    }
    None
}

fn normalize_import_resolution(
    module_segments: &[String],
    importing_dir: &Path,
    crate_root: Option<&Path>,
    absolute_roots: &[PathBuf],
) -> (Vec<String>, Vec<PathBuf>) {
    let Some(first) = module_segments.first().map(String::as_str) else {
        return (Vec::new(), absolute_roots.to_vec());
    };

    match first {
        "self" => (
            module_segments[1..].to_vec(),
            vec![importing_dir.to_path_buf()],
        ),
        "super" => {
            let mut root = importing_dir.to_path_buf();
            let mut roots = vec![root.clone()];
            let mut index = 0usize;
            while module_segments
                .get(index)
                .is_some_and(|segment| segment == "super")
            {
                root.pop();
                roots.push(root.clone());
                index += 1;
            }
            (module_segments[index..].to_vec(), roots)
        }
        "crate" => (
            module_segments[1..].to_vec(),
            crate_import_roots(importing_dir, crate_root),
        ),
        _ => (module_segments.to_vec(), absolute_roots.to_vec()),
    }
}

fn crate_import_roots(importing_dir: &Path, crate_root: Option<&Path>) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let mut current = Some(importing_dir);
    while let Some(path) = current {
        roots.push(path.to_path_buf());
        current = path.parent();
    }

    if let Some(root) = crate_root
        && !roots.iter().any(|candidate| candidate == root)
    {
        roots.push(root.to_path_buf());
    }

    roots
}

fn discover_crate_root(importing_dir: &Path) -> Option<PathBuf> {
    let mut current = importing_dir;
    let mut best = None;

    loop {
        if is_ash_module_root(current, importing_dir) {
            best = Some(current.to_path_buf());
        }

        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }

    best.or_else(|| fallback_std_module_root(importing_dir))
}

fn fallback_std_module_root(importing_dir: &Path) -> Option<PathBuf> {
    let std_root = builtin_stdlib_root().canonicalize().ok()?;
    let importing_dir = importing_dir.canonicalize().ok()?;
    if importing_dir.starts_with(&std_root) {
        Some(std_root)
    } else {
        None
    }
}

fn is_ash_module_root(path: &Path, importing_dir: &Path) -> bool {
    if path.join("mod.ash").is_file() {
        return true;
    }

    path != importing_dir && contains_ash_files(path)
}

fn contains_ash_files(path: &Path) -> bool {
    std::fs::read_dir(path).is_ok_and(|entries| {
        entries.flatten().any(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "ash")
        })
    })
}

fn search_roots(root: &Path) -> Vec<PathBuf> {
    let mut roots = vec![root.to_path_buf()];
    if let Some(value) = std::env::var_os("ASH_LIBRARY_PATH") {
        roots.extend(std::env::split_paths(&value));
    }
    roots.push(builtin_stdlib_root());
    roots
}

fn resolve_in_root(root: &Path, module_segments: &[String]) -> Option<PathBuf> {
    let joined = module_segments
        .iter()
        .fold(root.to_path_buf(), |mut path, segment| {
            path.push(segment);
            path
        });

    let file_candidate = joined.with_extension("ash");
    if file_candidate.is_file() {
        return Some(file_candidate);
    }

    let mod_candidate = joined.join("mod.ash");
    if mod_candidate.is_file() {
        return Some(mod_candidate);
    }

    None
}

fn builtin_stdlib_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../std/src")
}

fn convert_type_def(parsed: &ParsedTypeDef) -> CoreTypeDef {
    CoreTypeDef {
        name: parsed.name.clone(),
        params: parsed.params.clone(),
        body: match &parsed.body {
            ParsedTypeBody::Struct(fields) => CoreTypeBody::Struct(
                fields
                    .iter()
                    .map(|(name, ty)| (name.clone(), convert_type_expr(ty)))
                    .collect(),
            ),
            ParsedTypeBody::Enum(variants) => CoreTypeBody::Enum(
                variants
                    .iter()
                    .map(|variant| CoreVariantDef {
                        name: variant.name.clone(),
                        fields: variant
                            .fields
                            .iter()
                            .map(|(name, ty)| (name.clone(), convert_type_expr(ty)))
                            .collect(),
                        payload: match &variant.payload {
                            ParsedVariantPayload::Unit => CoreVariantPayload::Unit,
                            ParsedVariantPayload::Record(fields) => CoreVariantPayload::Record(
                                fields
                                    .iter()
                                    .map(|(name, ty)| (name.clone(), convert_type_expr(ty)))
                                    .collect(),
                            ),
                            ParsedVariantPayload::Tuple(items) => CoreVariantPayload::Tuple(
                                items.iter().map(convert_type_expr).collect(),
                            ),
                        },
                    })
                    .collect(),
            ),
            ParsedTypeBody::Alias(target) => CoreTypeBody::Alias(convert_type_expr(target)),
        },
        visibility: match parsed.visibility {
            ParsedVisibility::Public => CoreVisibility::Public,
            ParsedVisibility::Crate => CoreVisibility::Crate,
            ParsedVisibility::Private => CoreVisibility::Private,
        },
        builtin: parsed.builtin,
    }
}

fn convert_type_expr(parsed: &ParsedTypeExpr) -> CoreTypeExpr {
    match parsed {
        ParsedTypeExpr::Named(name) => CoreTypeExpr::Named(name.clone()),
        ParsedTypeExpr::Constructor { name, args } => CoreTypeExpr::Constructor {
            name: name.clone(),
            args: args.iter().map(convert_type_expr).collect(),
        },
        ParsedTypeExpr::Tuple(items) => {
            CoreTypeExpr::Tuple(items.iter().map(convert_type_expr).collect())
        }
        ParsedTypeExpr::Record(fields) => CoreTypeExpr::Record(
            fields
                .iter()
                .map(|(name, ty)| (name.clone(), convert_type_expr(ty)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: `pub mod child;` loads the child module's exports and stores
    /// them in `child_modules` under the child name.
    #[test]
    fn test_pub_mod_types_loads_child_exports() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path();

        // child.ash: defines a public type
        std::fs::write(
            dir.join("child.ash"),
            "pub type Role = System | User | Assistant;",
        )
        .expect("write child");

        // parent.ash: declares pub mod child;
        std::fs::write(dir.join("parent.ash"), "pub mod child;").expect("write parent");

        let mut cache = HashMap::new();
        let exports =
            collect_module_exports(&dir.join("parent.ash"), &mut cache, &mut HashSet::new())
                .expect("collecting parent exports should succeed");

        let child = exports
            .child_modules
            .get("child")
            .expect("child_modules should contain 'child'");
        assert!(
            child.type_defs.contains_key("Role"),
            "child module should export Role"
        );
    }

    /// Test 2: `pub use child::{Role}` re-exports still work alongside
    /// `pub mod child;` -- the parent's `type_defs` contains Role.
    #[test]
    fn test_pub_use_resolves_via_child_module() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path();

        std::fs::write(dir.join("child.ash"), "pub type Role = System | User;")
            .expect("write child");

        // parent.ash: both pub mod child; and pub use child::{Role};
        std::fs::write(
            dir.join("parent.ash"),
            "pub mod child;\npub use child::{Role};",
        )
        .expect("write parent");

        let mut cache = HashMap::new();
        let exports =
            collect_module_exports(&dir.join("parent.ash"), &mut cache, &mut HashSet::new())
                .expect("collecting parent exports should succeed");

        // Role is re-exported via pub use
        assert!(
            exports.type_defs.contains_key("Role"),
            "parent should re-export Role via pub use"
        );
        // Also present in child_modules
        assert!(
            exports.child_modules.contains_key("child"),
            "child_modules should contain 'child'"
        );
    }

    /// Test 3: Child exports are NOT flattened into the parent -- only
    /// explicitly `pub use`d items appear at the parent's top level.
    #[test]
    fn test_child_exports_not_flattened() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path();

        // child.ash: defines two public types
        std::fs::write(
            dir.join("child.ash"),
            "pub type Alpha = A | B;\npub type Beta = C | D;",
        )
        .expect("write child");

        // parent.ash: declares pub mod child; but only re-exports Alpha
        std::fs::write(
            dir.join("parent.ash"),
            "pub mod child;\npub use child::{Alpha};",
        )
        .expect("write parent");

        let mut cache = HashMap::new();
        let exports =
            collect_module_exports(&dir.join("parent.ash"), &mut cache, &mut HashSet::new())
                .expect("collecting parent exports should succeed");

        // Alpha should be re-exported
        assert!(
            exports.type_defs.contains_key("Alpha"),
            "parent should re-export Alpha"
        );
        // Beta should NOT appear in parent's type_defs (not re-exported)
        assert!(
            !exports.type_defs.contains_key("Beta"),
            "Beta should not be flattened into parent -- only pub use items appear"
        );
        // Both Alpha and Beta should exist in the child module
        let child = exports
            .child_modules
            .get("child")
            .expect("child_modules should contain 'child'");
        assert!(
            child.type_defs.contains_key("Alpha"),
            "child should have Alpha"
        );
        assert!(
            child.type_defs.contains_key("Beta"),
            "child should have Beta"
        );
    }

    /// Test 4: A file with `pub mod nonexistent;` should produce an error
    /// because the child module file does not exist.
    #[test]
    fn test_nonexistent_pub_mod_errors() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path();
        std::fs::write(dir.join("parent.ash"), "pub mod nonexistent;").expect("write parent");

        let mut cache = HashMap::new();
        let result =
            collect_module_exports(&dir.join("parent.ash"), &mut cache, &mut HashSet::new());

        let err =
            result.expect_err("collecting exports from file with nonexistent pub mod should fail");
        let msg = format!("{err}");
        assert!(
            msg.contains("pub mod 'nonexistent'") || msg.contains("module not found"),
            "error message should reference the missing module: {msg}",
        );
    }

    /// Test 5: `builtin fn` declarations are extracted as callables.
    #[test]
    fn test_builtin_fn_extraction() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path();

        std::fs::write(
            dir.join("module.ash"),
            "\
pub builtin fn add(x: Int, y: Int) -> Int;
builtin fn private_helper(a: String) -> String;
pub type Role = System | User;",
        )
        .expect("write module");

        let mut cache = HashMap::new();
        let exports =
            collect_module_exports(&dir.join("module.ash"), &mut cache, &mut HashSet::new())
                .expect("collecting exports should succeed");

        // Only pub builtin fn is exported; module-private builtin fn is not.
        assert!(
            exports.callables.contains_key("add"),
            "module should export callable 'add'"
        );
        assert!(
            !exports.callables.contains_key("private_helper"),
            "module-private builtin fn should NOT be exported"
        );

        // Verify parameter names
        let add = exports.callables.get("add").expect("add callable");
        assert_eq!(add.params, vec!["x", "y"]);
        assert_eq!(add.exported_name, "add");

        // Verify type def is also collected (not disrupted by builtin fn extraction)
        assert!(
            exports.type_defs.contains_key("Role"),
            "module should still export type Role"
        );
    }

    /// Test 6: Mixed `pub fn` and `builtin fn` declarations coexist.
    #[test]
    fn test_mixed_fn_and_builtin_fn() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path();

        std::fs::write(
            dir.join("module.ash"),
            "\
pub fn double(x: Int) -> Int { x * 2 }
pub builtin fn triple(x: Int) -> Int;
pub type Flag = On | Off;",
        )
        .expect("write module");

        let mut cache = HashMap::new();
        let exports =
            collect_module_exports(&dir.join("module.ash"), &mut cache, &mut HashSet::new())
                .expect("collecting exports should succeed");

        assert!(
            exports.callables.contains_key("double"),
            "module should export callable 'double' (pub fn)"
        );
        assert!(
            exports.callables.contains_key("triple"),
            "module should export callable 'triple' (pub builtin fn)"
        );
        assert!(
            exports.type_defs.contains_key("Flag"),
            "module should export type Flag"
        );

        // Verify builtin fn has Builtin kind
        let triple = exports.callables.get("triple").expect("triple callable");
        assert!(
            matches!(triple.kind, CallableKind::Builtin { .. }),
            "builtin fn kind should be CallableKind::Builtin"
        );
    }

    /// Test 7: `builtin fn` with type parameters is extracted correctly.
    #[test]
    fn test_builtin_fn_with_type_params() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path();

        std::fs::write(
            dir.join("module.ash"),
            "pub builtin fn identity<T>(value: T) -> T;",
        )
        .expect("write module");

        let mut cache = HashMap::new();
        let exports =
            collect_module_exports(&dir.join("module.ash"), &mut cache, &mut HashSet::new())
                .expect("collecting exports should succeed");

        assert!(
            exports.callables.contains_key("identity"),
            "module should export callable 'identity'"
        );
        let identity = exports
            .callables
            .get("identity")
            .expect("identity callable");
        assert_eq!(identity.params, vec!["value"]);
    }

    #[test]
    fn builtin_fn_callable_kind_carries_module_name() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path();

        std::fs::write(
            dir.join("string.ash"),
            "pub builtin fn concat(a: String, b: String) -> String;\n",
        )
        .expect("write");
        std::fs::write(
            dir.join("caller.ash"),
            "use string::{concat}\nworkflow main { ret 0 }\n",
        )
        .expect("write");

        let result = super::load_ordinary_file(&dir.join("caller.ash")).expect("load");
        let callable = result
            .imported_callables
            .get("concat")
            .expect("concat callable");
        match &callable.kind {
            super::CallableKind::Builtin { module } => {
                assert_eq!(
                    module.as_str(),
                    "string",
                    "module name must be populated from the import path by load_ordinary_file"
                );
            }
            other @ super::CallableKind::User { .. } => {
                panic!("expected Builtin {{ module }}, got: {other:?}")
            }
        }
    }

    #[test]
    fn builtin_fn_glob_import_carries_module_name() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path();

        std::fs::write(
            dir.join("math.ash"),
            "pub builtin fn add(x: Int, y: Int) -> Int;\npub builtin fn sub(x: Int, y: Int) -> Int;\n",
        )
        .expect("write");
        std::fs::write(
            dir.join("caller.ash"),
            "use math::*\nworkflow main { ret 0 }\n",
        )
        .expect("write");

        let result = super::load_ordinary_file(&dir.join("caller.ash")).expect("load");

        for name in &["add", "sub"] {
            let callable = result
                .imported_callables
                .get(*name)
                .unwrap_or_else(|| panic!("'{name}' should be in imported_callables"));
            match &callable.kind {
                super::CallableKind::Builtin { module } => {
                    assert_eq!(
                        module.as_str(),
                        "math",
                        "glob import must stamp module name on Builtin callable '{name}'"
                    );
                }
                other @ super::CallableKind::User { .. } => {
                    panic!("expected Builtin {{ module }} for '{name}', got: {other:?}")
                }
            }
        }
    }

    #[test]
    fn builtin_fn_higher_order_signature_imports_cleanly() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path();

        std::fs::write(
            dir.join("act.ash"),
            "pub builtin fn bind<A, B>(ma: Act<A>, f: Fn(A) -> Act<B>) -> Act<B>;\n",
        )
        .expect("write module");

        let mut cache = HashMap::new();
        let exports = collect_module_exports(&dir.join("act.ash"), &mut cache, &mut HashSet::new())
            .expect(
                "higher-order builtin fn signatures should parse for current std::act placeholders",
            );

        assert!(
            exports.callables.contains_key("bind"),
            "expected higher-order builtin fn to be exported"
        );
    }

    #[test]
    fn type_identity_collector_includes_builtin_type_forms() {
        let defs = with_legacy_type_snippet_compat(|| {
            collect_type_identity_defs_from_source_compat(
                "builtin type ActEnv;\npub builtin type PublicOpaque;\ntype Local = Int;\npub type Exported = String;",
            )
        })
        .expect("collect type identities");

        let names = defs.iter().map(|def| def.name.as_str()).collect::<Vec<_>>();
        assert_eq!(names, vec!["ActEnv", "PublicOpaque", "Local", "Exported"]);
        assert!(
            defs.iter()
                .find(|def| def.name == "ActEnv")
                .unwrap()
                .builtin
        );
        assert!(
            defs.iter()
                .find(|def| def.name == "PublicOpaque")
                .unwrap()
                .builtin
        );
    }

    #[test]
    fn module_exports_include_opaque_private_type_identities_without_representation() {
        let temp = tempfile::tempdir().expect("tempdir");
        let module = temp.path().join("types.ash");
        std::fs::write(
            &module,
            "builtin type PrivateOpaque;\ntype PrivateAlias = Int;\npub builtin type PublicOpaque;\npub type PublicAlias = String;",
        )
        .expect("write module");

        let exports = collect_module_exports(&module, &mut HashMap::new(), &mut HashSet::new())
            .expect("collect exports");

        assert!(exports.type_defs.contains_key("PublicOpaque"));
        assert!(exports.type_defs.contains_key("PublicAlias"));
        let private_opaque = exports
            .type_defs
            .get("PrivateOpaque")
            .expect("private builtin identity should export opaquely");
        assert!(private_opaque.builtin);
        assert!(
            matches!(private_opaque.body, CoreTypeBody::Struct(ref fields) if fields.is_empty())
        );
        assert!(
            !exports.type_defs.contains_key("PrivateAlias"),
            "private ordinary aliases must not be exported/importable downstream"
        );
        assert!(!exports.constructor_defs.contains_key("PrivateOpaque"));
        assert!(!exports.constructor_defs.contains_key("PrivateAlias"));
    }

    #[test]
    fn private_type_identity_can_import_without_representation_or_constructor() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path();
        std::fs::write(
            dir.join("inner.ash"),
            "type Secret = Int;\npub type Public = Int;",
        )
        .expect("write inner");
        std::fs::write(dir.join("outer.ash"), "pub use inner::{Public};").expect("write outer");
        std::fs::write(
            dir.join("caller.ash"),
            "use outer::{Public}\nworkflow main { ret 0 }\n",
        )
        .expect("write caller");

        let loaded = load_ordinary_file(&dir.join("caller.ash"))
            .expect("public type remains importable through public re-export");
        assert!(
            loaded
                .imported_type_defs
                .iter()
                .any(|def| def.name == "Public")
        );
        assert!(
            !loaded
                .imported_type_defs
                .iter()
                .any(|def| def.name == "Secret")
        );

        let reexport_secret_module = dir.join("reexport_secret.ash");
        std::fs::write(&reexport_secret_module, "pub use inner::{Secret};")
            .expect("write re-export secret module");
        let err = collect_module_exports(
            &reexport_secret_module,
            &mut HashMap::new(),
            &mut HashSet::new(),
        )
        .expect_err("private ordinary re-export must fail");
        let msg = err.to_string();
        assert!(
            msg.contains("Secret") && msg.contains("pub use"),
            "private ordinary re-export diagnostic should mention Secret and pub use: {msg}"
        );

        let secret_caller = dir.join("secret_caller.ash");
        std::fs::write(
            &secret_caller,
            "use inner::{Secret}\nworkflow main { ret 0 }\n",
        )
        .expect("write secret caller");
        let err = load_ordinary_file(&secret_caller)
            .expect_err("private ordinary Secret identity should not import");
        let msg = err.to_string();
        assert!(
            msg.contains("Secret") && msg.contains("not found"),
            "private ordinary import diagnostic should mention Secret not found: {msg}"
        );
    }
}
