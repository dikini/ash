//! Ordinary file loader for import-backed execution.
//!
//! This loader supports a constrained executable subset:
//! - contiguous leading `use` imports on ordinary workflow files
//! - module resolution from the workflow tree, `ASH_LIBRARY_PATH`, and the built-in stdlib
//! - imported `pub type` definitions from resolved modules
//! - imported callable bodies from local workflows and stdlib `pub fn` / `pub use`

use crate::EngineError;
use ash_core::ast::{
    TypeBody as CoreTypeBody, TypeDef as CoreTypeDef, TypeExpr as CoreTypeExpr,
    VariantDef as CoreVariantDef, VariantPayload as CoreVariantPayload,
    Visibility as CoreVisibility,
};
use ash_parser::input::new_input;
use ash_parser::parse_module::parse_fn_definition;
use ash_parser::parse_type_def::{
    TypeBody as ParsedTypeBody, TypeDef as ParsedTypeDef, TypeExpr as ParsedTypeExpr,
    VariantPayload as ParsedVariantPayload, Visibility as ParsedVisibility, parse_type_def,
};
use ash_parser::parse_use::parse_use;
use ash_parser::parse_workflow::workflow_def;
use ash_parser::surface::{Definition, Expr, MatchArm as SurfaceMatchArm, Workflow, WorkflowDef};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use winnow::prelude::Parser;

/// Ordinary-file loading output after import stripping and dependency collection.
#[derive(Debug, Clone)]
pub struct LoadedOrdinaryFile {
    /// Workflow source with the leading `use` prelude removed.
    pub workflow_source: String,
    /// Imported public type definitions collected from resolved modules.
    pub imported_type_defs: Vec<CoreTypeDef>,
    /// Imported callable bodies keyed by the imported name.
    pub imported_callables: HashMap<String, InlineCallable>,
}

/// Imported callable body and parameter list.
#[derive(Debug, Clone)]
pub struct InlineCallable {
    /// Name the callable was imported under.
    pub exported_name: String,
    /// Parameter names in call order.
    pub params: Vec<String>,
    /// Callable body expression.
    pub body: Expr,
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
struct ModuleExports {
    type_defs: HashMap<String, CoreTypeDef>,
    callables: HashMap<String, InlineCallable>,
}

/// Load an ordinary workflow file together with its imported metadata.
///
/// # Errors
///
/// Returns [`EngineError`] if the workflow file cannot be read, an import
/// cannot be resolved, or an imported module cannot be parsed into the
/// supported type/callable subset.
pub fn load_ordinary_file(path: &Path) -> Result<LoadedOrdinaryFile, EngineError> {
    let source = std::fs::read_to_string(path)?;
    let entry_root = path.parent().ok_or_else(|| {
        EngineError::Configuration(format!("workflow path '{}' has no parent", path.display()))
    })?;
    let mut module_cache = HashMap::new();

    let mut imports = Vec::new();
    let mut kept_lines = Vec::new();
    let mut seen_non_import = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if !seen_non_import && is_skippable_prelude_line(trimmed) {
            kept_lines.push(line.to_string());
            continue;
        }

        if !seen_non_import && trimmed.starts_with("use ") {
            imports.push(parse_ordinary_import(trimmed)?);
            continue;
        }

        seen_non_import = true;
        kept_lines.push(line.to_string());
    }

    let mut imported_type_defs = Vec::new();
    let mut imported_callables = HashMap::new();

    for import in imports {
        let module_path = resolve_module_path(&import.module_segments, &search_roots(entry_root))
            .ok_or_else(|| {
            EngineError::Parse(format!(
                "module '{}' not found",
                import.module_segments.join("::")
            ))
        })?;
        let exports = collect_module_exports(&module_path, &mut module_cache)?;

        for selection in import.selections {
            match selection {
                ImportSelection::Glob => {
                    imported_type_defs.extend(exports.type_defs.values().cloned());
                    imported_callables.extend(exports.callables.clone());
                }
                ImportSelection::Named { name, alias } => {
                    let exported_name = alias.unwrap_or_else(|| name.clone());
                    if let Some(type_def) = exports.type_defs.get(&name) {
                        imported_type_defs.push(type_def.clone());
                    } else if let Some(callable) = exports.callables.get(&name) {
                        let mut callable = callable.clone();
                        callable.exported_name.clone_from(&exported_name);
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
        imported_callables,
    })
}

pub(crate) fn collect_public_type_defs_from_source(
    source: &str,
) -> Result<Vec<CoreTypeDef>, EngineError> {
    let mut type_defs = Vec::new();
    for snippet in extract_semicolon_snippets(source, |trimmed| trimmed.starts_with("pub type ")) {
        type_defs.extend(parse_public_type_defs(&snippet)?);
    }
    Ok(type_defs)
}

fn is_skippable_prelude_line(line: &str) -> bool {
    line.is_empty() || line.starts_with("--") || line.starts_with("/*") || line.starts_with('*')
}

fn parse_ordinary_import(line: &str) -> Result<ImportSpec, EngineError> {
    if line.contains('@') {
        return parse_versioned_import(line);
    }

    let normalized = if line.trim_end().ends_with(';') {
        line.to_string()
    } else {
        format!("{line};")
    };
    let mut input = new_input(&normalized);
    let use_stmt = parse_use
        .parse_next(&mut input)
        .map_err(|error| EngineError::Parse(format!("{error}")))?;
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

fn collect_module_exports(
    path: &Path,
    cache: &mut HashMap<PathBuf, ModuleExports>,
) -> Result<ModuleExports, EngineError> {
    let path = path.to_path_buf();
    if let Some(exports) = cache.get(&path) {
        return Ok(exports.clone());
    }

    let source = std::fs::read_to_string(&path)?;
    let mut exports = ModuleExports::default();

    for snippet in extract_semicolon_snippets(&source, |trimmed| trimmed.starts_with("pub type ")) {
        for type_def in parse_public_type_defs(&snippet)? {
            insert_type_export(&mut exports, &type_def)?;
        }
    }

    for snippet in extract_braced_snippets(&source, |trimmed| trimmed.starts_with("workflow ")) {
        if let Some(callable) = parse_workflow_callable(&snippet)? {
            insert_callable_export(&mut exports, &callable.name, callable.callable)?;
        }
    }

    for snippet in extract_braced_snippets(&source, |trimmed| trimmed.starts_with("pub fn ")) {
        if let Some(callable) = parse_supported_pub_fn_callable(&snippet) {
            insert_callable_export(&mut exports, &callable.name, callable.callable)?;
        }
    }

    let module_root = path.parent().ok_or_else(|| {
        EngineError::Configuration(format!("module path '{}' has no parent", path.display()))
    })?;
    for snippet in extract_semicolon_snippets(&source, |trimmed| trimmed.starts_with("pub use ")) {
        let normalized = snippet.trim();
        let mut input = new_input(normalized);
        let use_stmt = parse_use
            .parse_next(&mut input)
            .map_err(|error| EngineError::Parse(format!("{error}")))?;
        let resolved = resolve_use_target(module_root, &use_stmt)?;
        let target_exports = collect_module_exports(&resolved, cache)?;
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

    resolve_module_path(
        &module_segments
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>(),
        &search_roots(module_root),
    )
    .ok_or_else(|| {
        EngineError::Parse(format!(
            "module '{}' not found",
            segments
                .iter()
                .map(std::convert::AsRef::as_ref)
                .collect::<Vec<_>>()
                .join("::")
        ))
    })
}

fn merge_use_exports(
    exports: &mut ModuleExports,
    target_exports: ModuleExports,
    use_stmt: ash_parser::use_tree::Use,
) -> Result<(), EngineError> {
    use ash_parser::use_tree::UsePath;

    match use_stmt.path {
        UsePath::Glob(_) => {
            for (name, type_def) in target_exports.type_defs {
                insert_type_export_with_name(exports, &name, type_def)?;
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
            } else if let Some(callable) = target_exports.callables.get(&name) {
                let mut callable = callable.clone();
                callable.exported_name.clone_from(&exported_name);
                insert_callable_export(exports, &exported_name, callable)?;
            } else {
                return Err(EngineError::Parse(format!("item '{name}' not found")));
            }
        }
        UsePath::Nested(_, items) => {
            for item in items {
                let exported_name = item
                    .alias
                    .map_or_else(|| item.name.to_string(), |alias| alias.to_string());
                if let Some(type_def) = target_exports.type_defs.get(item.name.as_ref()) {
                    insert_type_export_with_name(exports, &exported_name, type_def.clone())?;
                } else if let Some(callable) = target_exports.callables.get(item.name.as_ref()) {
                    let mut callable = callable.clone();
                    callable.exported_name.clone_from(&exported_name);
                    insert_callable_export(exports, &exported_name, callable)?;
                } else {
                    return Err(EngineError::Parse(format!(
                        "item '{}' not found in re-export target",
                        item.name
                    )));
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
    let variant_names = match &type_def.body {
        CoreTypeBody::Enum(variants) => variants
            .iter()
            .map(|variant| variant.name.clone())
            .collect(),
        _ => Vec::new(),
    };

    insert_type_export_with_name(exports, &type_def.name, type_def.clone())?;
    for variant_name in variant_names {
        if variant_name == type_def.name {
            continue;
        }
        insert_type_export_with_name(exports, &variant_name, type_def.clone())?;
    }
    Ok(())
}

fn insert_type_export_with_name(
    exports: &mut ModuleExports,
    name: &str,
    type_def: CoreTypeDef,
) -> Result<(), EngineError> {
    if exports
        .type_defs
        .insert(name.to_string(), type_def)
        .is_some()
    {
        return Err(EngineError::Configuration(format!(
            "duplicate exported type '{name}'"
        )));
    }
    Ok(())
}

fn insert_callable_export(
    exports: &mut ModuleExports,
    name: &str,
    callable: InlineCallable,
) -> Result<(), EngineError> {
    if exports
        .callables
        .insert(name.to_string(), callable)
        .is_some()
    {
        return Err(EngineError::Configuration(format!(
            "duplicate exported callable '{name}'"
        )));
    }
    Ok(())
}

fn parse_public_type_defs(snippet: &str) -> Result<Vec<CoreTypeDef>, EngineError> {
    let mut input = new_input(snippet.trim());
    let parsed = parse_type_def
        .parse_next(&mut input)
        .map_err(|error| EngineError::Parse(format!("{error}")))?;
    Ok(vec![convert_type_def(&parsed)])
}

fn parse_workflow_callable(snippet: &str) -> Result<Option<ImportedCallableExport>, EngineError> {
    let mut input = new_input(snippet.trim());
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

    Ok(Some(ImportedCallableExport {
        name: name.clone(),
        callable: InlineCallable {
            exported_name: name,
            params,
            body: normalize_imported_callable_expr(&function.body),
        },
    }))
}

fn normalize_imported_callable_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Block {
            statements,
            tail_expr,
            span,
        } => {
            let mut normalized = tail_expr.as_deref().map_or_else(
                || Expr::Literal(ash_parser::surface::Literal::Null),
                normalize_imported_callable_expr,
            );

            for statement in statements.iter().rev() {
                let ash_parser::surface::BlockStmt::Let {
                    pattern,
                    expr,
                    span: stmt_span,
                } = statement;
                normalized = Expr::Match {
                    scrutinee: Box::new(normalize_imported_callable_expr(expr)),
                    arms: vec![SurfaceMatchArm {
                        pattern: pattern.clone(),
                        body: Box::new(normalized),
                        span: *stmt_span,
                    }],
                    span: *span,
                };
            }

            normalized
        }
        _ => expr.clone(),
    }
}

fn parse_supported_pub_fn_callable(snippet: &str) -> Option<ImportedCallableExport> {
    parse_pub_fn_callable(snippet).ok().flatten()
}

#[derive(Debug, Clone)]
struct ImportedCallableExport {
    name: String,
    callable: InlineCallable,
}

fn extract_callable_from_workflow(
    workflow: WorkflowDef,
) -> Result<Option<ImportedCallableExport>, EngineError> {
    let WorkflowDef {
        name, params, body, ..
    } = workflow;

    let Workflow::Ret { expr, .. } = body else {
        return Err(EngineError::Parse(format!(
            "imported callable '{name}' must use a single ret expression"
        )));
    };

    Ok(Some(ImportedCallableExport {
        name: name.to_string(),
        callable: InlineCallable {
            exported_name: name.to_string(),
            params: params
                .into_iter()
                .map(|param| param.name.to_string())
                .collect(),
            body: expr,
        },
    }))
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

fn extract_braced_snippets<F>(source: &str, predicate: F) -> Vec<String>
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
