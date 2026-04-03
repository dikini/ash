//! Entry workflow signature verification.
//!
//! This module validates the canonical entry contract against parsed surface
//! workflow metadata only. It does not resolve imports, load the standard
//! library, or start bootstrap execution.

use crate::EngineError;
use ash_core::Value;
use ash_parser::surface::{Type, WorkflowDef};
use thiserror::Error;

const EXPECTED_ENTRY_RETURN_TYPE: &str = "Result<(), RuntimeError>";
const RUNTIME_ENTRY_STDLIB_MODULES: [(&str, &str); 4] = [
    ("result", "result.ash"),
    ("runtime", "runtime/mod.ash"),
    ("runtime::error", "runtime/error.ash"),
    ("runtime::args", "runtime/args.ash"),
];

/// Narrow runtime stdlib source bundle used by the entry path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEntryStdlibSource {
    /// Canonical module path exposed to entry sources.
    pub module_path: &'static str,
    /// Raw module source loaded from the workspace stdlib.
    pub source: String,
}

/// Load the minimal stdlib sources needed by the runtime entry path.
///
/// This helper is intentionally narrow: it only loads the `result` and
/// `runtime` modules required by entry verification/bootstrap prerequisites.
/// It does not attempt to resolve imports or register modules in a general
/// module graph.
///
/// # Errors
///
/// Returns [`EngineError::Io`] if any required stdlib file cannot be read.
pub fn load_runtime_entry_stdlib_sources() -> Result<Vec<RuntimeEntryStdlibSource>, EngineError> {
    let stdlib_root = runtime_entry_stdlib_root();

    RUNTIME_ENTRY_STDLIB_MODULES
        .iter()
        .map(|(module_path, relative_path)| {
            let source = std::fs::read_to_string(stdlib_root.join(relative_path))?;
            Ok(RuntimeEntryStdlibSource {
                module_path,
                source,
            })
        })
        .collect()
}

pub(crate) fn validate_runtime_entry_import_prelude(
    source: &str,
    mut has_registered_module: impl FnMut(&str) -> bool,
) -> Result<(), EngineError> {
    let prelude = leading_entry_use_prelude(source);

    for import in prelude.imports {
        let module_path = runtime_entry_registry_module_for_import(import)?;
        if !has_registered_module(module_path) {
            return Err(EngineError::Parse(format!(
                "entry import '{import}' requires registered runtime stdlib module '{module_path}'"
            )));
        }
    }

    Ok(())
}

pub(crate) fn strip_leading_entry_use_lines(source: &str) -> &str {
    leading_entry_use_prelude(source).body
}

struct LeadingEntryUsePrelude<'a> {
    imports: Vec<&'a str>,
    body: &'a str,
}

fn leading_entry_use_prelude(source: &str) -> LeadingEntryUsePrelude<'_> {
    let mut saw_use_line = false;
    let mut offset = 0usize;
    let mut imports = Vec::new();
    let mut remaining = source;

    loop {
        let trimmed = trim_entry_prelude_trivia(remaining);
        offset += remaining.len() - trimmed.len();
        remaining = trimmed;

        if let Some(import) = parse_leading_entry_use_import(remaining) {
            saw_use_line = true;
            let (use_line, rest) = split_entry_use_line(remaining);
            imports.push(import[..import.len() - rest.len()].trim());
            offset += use_line.len();
            remaining = rest;
            continue;
        }

        break;
    }

    LeadingEntryUsePrelude {
        imports,
        body: if saw_use_line {
            &source[offset..]
        } else {
            source
        },
    }
}

fn parse_leading_entry_use_import(input: &str) -> Option<&str> {
    let rest = input.strip_prefix("use")?;
    let whitespace_len = rest
        .char_indices()
        .take_while(|(_, ch)| is_entry_horizontal_whitespace(*ch))
        .last()
        .map_or(0, |(index, ch)| index + ch.len_utf8());

    if whitespace_len == 0 {
        return None;
    }

    Some(&rest[whitespace_len..])
}

const fn is_entry_horizontal_whitespace(ch: char) -> bool {
    ch.is_whitespace() && !matches!(ch, '\n' | '\r')
}

fn trim_entry_prelude_trivia(mut input: &str) -> &str {
    loop {
        let trimmed = input.trim_start_matches(char::is_whitespace);
        if trimmed.len() != input.len() {
            input = trimmed;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("--") {
            input = rest.find('\n').map_or("", |index| &rest[index + 1..]);
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("/*") {
            input = skip_entry_block_comment(rest);
            continue;
        }

        return trimmed;
    }
}

fn skip_entry_block_comment(input: &str) -> &str {
    let mut depth = 1usize;
    let mut index = 0usize;

    while index < input.len() {
        let remaining = &input[index..];
        if remaining.starts_with("/*") {
            depth += 1;
            index += 2;
            continue;
        }

        if remaining.starts_with("*/") {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return &input[index..];
            }
            continue;
        }

        if let Some(ch) = remaining.chars().next() {
            index += ch.len_utf8();
        } else {
            break;
        }
    }

    ""
}

fn split_entry_use_line(input: &str) -> (&str, &str) {
    let line_end = input.find('\n').map_or(input.len(), |index| index + 1);
    (&input[..line_end], &input[line_end..])
}

fn runtime_entry_registry_module_for_import(import: &str) -> Result<&'static str, EngineError> {
    let import = normalize_runtime_entry_import(import);

    match import.as_str() {
        "result::Result" => Ok("result"),
        "runtime::RuntimeError" => Ok("runtime::error"),
        "runtime::Args" => Ok("runtime::args"),
        _ => Err(EngineError::Parse(format!(
            "unsupported entry runtime import '{import}'; only result::Result, runtime::RuntimeError, and runtime::Args are supported"
        ))),
    }
}

fn normalize_runtime_entry_import(import: &str) -> String {
    let mut normalized = String::with_capacity(import.len());
    let mut remaining = import.trim();

    while !remaining.is_empty() {
        if remaining.starts_with("--") {
            break;
        }

        if let Some(rest) = remaining.strip_prefix("/*") {
            remaining = skip_entry_block_comment(rest);
            continue;
        }

        let Some(ch) = remaining.chars().next() else {
            break;
        };

        remaining = &remaining[ch.len_utf8()..];

        if ch.is_whitespace() || ch == ';' {
            continue;
        }

        normalized.push(ch);
    }

    normalized
}

fn runtime_entry_stdlib_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../std/src")
}

/// Errors produced while validating the canonical entry workflow contract.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EntryVerificationError {
    /// The parsed entry file does not expose a workflow named `main`.
    #[error("entry file has no 'main' workflow")]
    MissingMain,

    /// The workflow metadata is not available in the engine cache.
    #[error("workflow metadata not found in engine cache")]
    MissingWorkflowMetadata,

    /// The entry workflow declared the wrong return type.
    #[error("expected {expected}, found {found}")]
    WrongReturnType {
        /// Canonical required return type.
        expected: String,
        /// Declared return type found in surface metadata.
        found: String,
    },

    /// An entry workflow parameter is not a capability type.
    #[error("parameter '{name}' must be capability type, found {found}")]
    NonCapabilityParameter {
        /// Parameter name.
        name: String,
        /// Surface type found for the parameter.
        found: String,
    },
}

/// Errors produced while bootstrapping an entry workflow.
#[derive(Debug, Error)]
pub enum EntryBootstrapError {
    /// Failed to load stdlib prerequisites or the entry source itself.
    #[error(transparent)]
    Engine(#[from] EngineError),

    /// The canonical `main` workflow contract was not satisfied.
    #[error(transparent)]
    Verification(#[from] EntryVerificationError),

    /// Workflow execution failed before a terminal entry result was produced.
    #[error("execution failed: {0}")]
    Execution(String),

    /// The runtime error payload contained an exit code outside `0..=255`.
    #[error("invalid runtime exit code {code}; expected 0..=255")]
    InvalidExitCode {
        /// Exit code value returned by the workflow payload.
        code: i64,
    },
}

/// Verify the canonical entry workflow contract using parsed surface metadata.
///
/// # Errors
///
/// Returns [`EntryVerificationError`] if the workflow is not named `main`, if
/// its declared return type is not exactly `Result<(), RuntimeError>`, or if
/// any parameter is not a usage-site capability type.
pub fn verify_entry_workflow_def(def: &WorkflowDef) -> Result<(), EntryVerificationError> {
    if def.name.as_ref() != "main" {
        return Err(EntryVerificationError::MissingMain);
    }

    let declared_return_type = def.declared_return_type.as_ref().ok_or_else(|| {
        EntryVerificationError::WrongReturnType {
            expected: EXPECTED_ENTRY_RETURN_TYPE.to_string(),
            found: "<missing>".to_string(),
        }
    })?;

    if !is_canonical_entry_return_type(declared_return_type) {
        return Err(EntryVerificationError::WrongReturnType {
            expected: EXPECTED_ENTRY_RETURN_TYPE.to_string(),
            found: format_type(declared_return_type),
        });
    }

    for param in &def.params {
        if !matches!(param.ty, Type::Capability(_)) {
            return Err(EntryVerificationError::NonCapabilityParameter {
                name: param.name.to_string(),
                found: format_type(&param.ty),
            });
        }
    }

    Ok(())
}

pub(crate) fn entry_input_bindings(def: &WorkflowDef) -> std::collections::HashMap<String, Value> {
    def.params
        .iter()
        .filter_map(|param| match &param.ty {
            Type::Capability(capability) => {
                Some((param.name.to_string(), Value::Cap(capability.to_string())))
            }
            _ => None,
        })
        .collect()
}

pub(crate) fn derive_entry_exit_code(result: &Value) -> Result<u8, EntryBootstrapError> {
    match result {
        Value::Variant { name, fields } if name == "Err" => {
            let Some((_, Value::Variant { name, fields })) =
                fields.iter().find(|(field_name, _)| field_name == "error")
            else {
                return Ok(0);
            };

            if name != "RuntimeError" {
                return Ok(0);
            }

            let Some((_, Value::Int(code))) = fields
                .iter()
                .find(|(field_name, _)| field_name == "exit_code")
            else {
                return Ok(0);
            };

            u8::try_from(*code).map_err(|_| EntryBootstrapError::InvalidExitCode { code: *code })
        }
        _ => Ok(0),
    }
}

fn is_canonical_entry_return_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Constructor { name, args }
            if name.as_ref() == "Result"
                && args.len() == 2
                && matches!(&args[0], Type::Name(unit) if unit.as_ref() == "()")
                && matches!(&args[1], Type::Name(error) if error.as_ref() == "RuntimeError")
    )
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::Name(name) => name.to_string(),
        Type::List(inner) => format!("[{}]", format_type(inner)),
        Type::Record(fields) => {
            let fields = fields
                .iter()
                .map(|(name, ty)| format!("{name}: {}", format_type(ty)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{fields}}}")
        }
        Type::Capability(name) => format!("cap {name}"),
        Type::Constructor { name, args } => {
            let args = args.iter().map(format_type).collect::<Vec<_>>().join(", ");
            format!("{name}<{args}>")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::{new_input, workflow_def};
    use winnow::Parser;

    fn parse_workflow_def(source: &str) -> WorkflowDef {
        workflow_def
            .parse(new_input(source))
            .expect("workflow should parse")
    }

    #[test]
    fn accepts_canonical_entry_workflow() {
        let def = parse_workflow_def(
            "workflow main(args: cap Args) -> Result<(), RuntimeError> { done; }",
        );

        assert!(verify_entry_workflow_def(&def).is_ok());
    }

    #[test]
    fn rejects_non_canonical_return_type() {
        let def = parse_workflow_def("workflow main() -> Int { done; }");

        let err = verify_entry_workflow_def(&def).expect_err("verification should fail");

        assert!(matches!(
            err,
            EntryVerificationError::WrongReturnType { .. }
        ));
    }

    #[test]
    fn derives_exit_code_from_runtime_error_variant() {
        let value = Value::variant(
            "Err",
            vec![(
                "error",
                Value::variant(
                    "RuntimeError",
                    vec![
                        ("exit_code", Value::Int(42)),
                        ("message", Value::String("boom".to_string())),
                    ],
                ),
            )],
        );

        assert_eq!(derive_entry_exit_code(&value).expect("exit code"), 42);
    }

    #[test]
    fn derives_success_exit_code_for_non_error_result() {
        let value = Value::variant("Ok", vec![("value", Value::Null)]);

        assert_eq!(derive_entry_exit_code(&value).expect("exit code"), 0);
    }

    #[test]
    fn validates_runtime_entry_imports_against_registered_modules() {
        let source = r"
            use result::Result
            use runtime::RuntimeError
            use runtime::Args

            workflow main(args: cap Args) -> Result<(), RuntimeError> { done; }
        ";

        let mut requested_modules = Vec::new();
        validate_runtime_entry_import_prelude(source, |module_path| {
            requested_modules.push(module_path.to_string());
            true
        })
        .expect("runtime imports should validate");

        assert_eq!(
            requested_modules,
            vec![
                "result".to_string(),
                "runtime::error".to_string(),
                "runtime::args".to_string(),
            ]
        );
    }

    #[test]
    fn rejects_unsupported_runtime_entry_imports() {
        let err = validate_runtime_entry_import_prelude(
            "use runtime::system_supervisor\nworkflow main() -> Result<(), RuntimeError> { done; }",
            |_| true,
        )
        .expect_err("unsupported imports should be rejected");

        assert!(matches!(err, EngineError::Parse(_)));
    }
}
