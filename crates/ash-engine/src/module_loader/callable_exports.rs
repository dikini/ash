//! Public callable, workflow, and capability export helpers.

use super::{
    CallableKind, CallableSignature, CoreTypeBody, CoreTypeDef, CoreVisibility, Definition,
    EngineError, HashMap, HashSet, InlineCallable, Parser, Path,
    callable_row_requirement_from_builtin, callable_row_requirement_from_fn_def,
    extract_semicolon_snippets, new_input, parse_builtin_fn_definition, parse_fn_definition,
};

pub(super) fn extract_public_capability_names(source: &str) -> Vec<String> {
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

pub(super) fn capability_type_identity(name: &str) -> CoreTypeDef {
    CoreTypeDef {
        name: name.to_string(),
        params: Vec::new(),
        body: CoreTypeBody::Struct(vec![]),
        visibility: CoreVisibility::Public,
        builtin: true,
    }
}

pub(super) fn parse_pub_fn_callable(
    snippet: &str,
) -> Result<Option<ImportedCallableExport>, EngineError> {
    let mut input = new_input(snippet.trim());
    let parsed = parse_fn_definition
        .parse_next(&mut input)
        .map_err(|error| EngineError::Parse(format!("{error}")))?;

    let Definition::Function(function) = parsed else {
        return Err(EngineError::Parse(
            "expected pub fn to parse as a function definition".to_string(),
        ));
    };

    Ok(Some(imported_callable_from_fn_def(function)))
}

pub(super) fn imported_callable_from_fn_def(
    function: ash_parser::surface::FnDef,
) -> ImportedCallableExport {
    let name = function.name.to_string();
    let params = function
        .params
        .iter()
        .map(|param| param.name.to_string())
        .collect::<Vec<_>>();
    let body = function.body.clone();
    let row_requirement = callable_row_requirement_from_fn_def(&function);

    ImportedCallableExport {
        callable: InlineCallable {
            exported_name: name,
            params,
            effectful_names: HashSet::new(),
            kind: CallableKind::User { body },
            row_requirement,
            signature: Some(CallableSignature::Function(function)),
            exporting_modules: HashSet::new(),
            module_runtime_callables: HashMap::new(),
        },
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
pub(super) fn extract_fn_name_from_snippet(snippet: &str) -> Option<String> {
    let trimmed = snippet.trim();
    trimmed
        .strip_prefix("pub fn ")
        .and_then(|rest| rest.split(|c: char| c.is_whitespace() || c == '(').next())
        .map(std::string::ToString::to_string)
}

pub(super) fn parse_supported_pub_fn_callable(
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
pub(super) fn parse_builtin_fn_callable(
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

    Ok(Some(imported_callable_from_builtin_fn_def(builtin, module)))
}

pub(super) fn imported_callable_from_builtin_fn_def(
    builtin: ash_parser::surface::BuiltinFnDef,
    module: String,
) -> ImportedCallableExport {
    let name = builtin.name.to_string();
    let params = builtin
        .params
        .iter()
        .map(|param| param.name.to_string())
        .collect::<Vec<_>>();

    ImportedCallableExport {
        callable: InlineCallable {
            exported_name: name,
            params,
            effectful_names: HashSet::new(),
            kind: CallableKind::Builtin { module },
            row_requirement: callable_row_requirement_from_builtin(&builtin),
            signature: Some(CallableSignature::Builtin(builtin)),
            exporting_modules: HashSet::new(),
            module_runtime_callables: HashMap::new(),
        },
    }
}

#[derive(Debug, Clone)]
pub(super) struct ImportedCallableExport {
    pub(super) callable: InlineCallable,
}

pub(super) fn module_path_text(path: &Path) -> &str {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
}
