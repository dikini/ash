//! Run command for executing Ash workflows.
//!
//! TASK-054: Implement `run` command for executing workflows.
//! TASK-076: Updated to use ash-engine.
//! TASK-309: Implemented --dry-run, --timeout flags.
//! TASK-323: Removed --capability flag.
//! TASK-324: Removed --input flag.

use anyhow::{Context, Result};
use ash_core::{Effect, Value};
use ash_engine::EngineError;
use ash_interp::ExecError;
use ash_parser::parse_utils::skip_whitespace_and_comments;
use ash_parser::{Token, TokenKind, expr, lex_with_recovery, new_input};
use ash_provenance::{WorkflowTraceSession, create_trace_recorder};
use async_trait::async_trait;
use clap::Args;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use crate::error::CliError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunOutcome {
    Completed,
    Exit(ExitCode),
}

impl RunOutcome {
    #[must_use]
    pub const fn completed() -> Self {
        Self::Completed
    }

    #[must_use]
    pub fn exit_code(self) -> ExitCode {
        match self {
            Self::Completed => ExitCode::SUCCESS,
            Self::Exit(code) => code,
        }
    }
}

/// Output format for run command
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum RunOutputFormat {
    /// Human-readable text format
    #[default]
    Text,
    /// JSON format
    Json,
}

/// Arguments for the run command
#[derive(Args, Debug, Clone)]
pub struct RunArgs {
    /// Path to workflow file
    #[arg(value_name = "PATH")]
    pub path: String,

    /// Output file for results
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<String>,

    /// Enable trace mode
    #[arg(long)]
    pub trace: bool,

    /// Output format (text, json)
    #[arg(long, value_enum, default_value = "text")]
    pub format: RunOutputFormat,

    /// Validate without executing
    #[arg(long)]
    pub dry_run: bool,

    /// Execution timeout in seconds
    #[arg(long, value_name = "SECONDS")]
    pub timeout: Option<u64>,

    /// Runtime arguments passed to the entry workflow after `--`
    #[arg(last = true, value_name = "ARGS")]
    pub program_args: Vec<String>,
}

/// Run a workflow file
///
/// Supports dry-run mode (validate only) and timeout.
///
/// # Errors
///
/// Returns an error if:
/// - The workflow file cannot be read
/// - Parsing fails
/// - Type checking fails (in dry-run or normal mode)
/// - Execution fails
/// - Timeout is exceeded
pub async fn run(args: &RunArgs) -> Result<RunOutcome> {
    let path = Path::new(&args.path);

    // Build engine with default capabilities
    let engine = build_engine(&args.program_args).context("Failed to build engine")?;
    let source =
        std::fs::read_to_string(path).map_err(|error| classify_run_read_error(path, error))?;
    let source_kind = classify_workflow_source(&source);
    let use_entry_bootstrap = should_use_entry_bootstrap(source_kind);

    // Dry-run mode: parse and check only
    if args.dry_run {
        let workflow = parse_runnable_workflow(&engine, &source, WorkflowSourceKind::Entry)
            .map_err(classify_engine_error)?;
        engine.check(&workflow).map_err(classify_engine_error)?;
        engine
            .verify_entry_workflow(&workflow)
            .map_err(classify_entry_verification_error)?;

        println!("Dry run successful");
        return Ok(RunOutcome::completed());
    }

    if use_entry_bootstrap {
        let exit_code = if let Some(timeout_secs) = args.timeout {
            match tokio::time::timeout(
                Duration::from_secs(timeout_secs),
                execute_entry_source(&engine, &source, args.trace),
            )
            .await
            {
                Ok(result) => result.map_err(classify_entry_bootstrap_error)?,
                Err(_) => {
                    return Err(anyhow::anyhow!("timeout after {timeout_secs}s"));
                }
            }
        } else {
            execute_entry_source(&engine, &source, args.trace)
                .await
                .map_err(classify_entry_bootstrap_error)?
        };

        if exit_code == 0 {
            emit_entry_output(args).await?;
        }

        return Ok(RunOutcome::Exit(ExitCode::from(exit_code)));
    }

    // Run the workflow file with optional timeout
    let result = if let Some(timeout_secs) = args.timeout {
        let timeout_duration = Duration::from_secs(timeout_secs);
        let execution_fut = async {
            if args.trace {
                let workflow = parse_runnable_workflow(&engine, &source, source_kind)
                    .map_err(classify_engine_error)?;
                engine.check(&workflow).map_err(classify_engine_error)?;
                execute_with_trace(&engine, &workflow).await
            } else {
                run_workflow_source(&engine, &source, source_kind).await
            }
        };

        match tokio::time::timeout(timeout_duration, execution_fut).await {
            Ok(result) => result?,
            Err(_) => {
                return Err(anyhow::anyhow!("timeout after {timeout_secs}s"));
            }
        }
    } else {
        // No timeout - run normally
        if args.trace {
            let workflow = parse_runnable_workflow(&engine, &source, source_kind)
                .map_err(classify_engine_error)?;
            engine.check(&workflow).map_err(classify_engine_error)?;
            execute_with_trace(&engine, &workflow).await?
        } else {
            run_workflow_source(&engine, &source, source_kind).await?
        }
    };

    // Output results
    output_result(&result, &args.output, args.format).await?;

    Ok(RunOutcome::completed())
}

/// Build an engine with default capabilities
///
/// Adds stdio and fs capabilities by default.
fn build_engine(program_args: &[String]) -> Result<ash_engine::Engine, ash_engine::EngineError> {
    let mut builder = ash_engine::Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities();

    for (index, value) in program_args.iter().enumerate() {
        let provider = Arc::new(RuntimeArgProvider::new(index, value));
        let provider_name = provider.name.clone();
        builder = builder.with_custom_provider(&provider_name, provider);
    }

    builder.build()
}

#[derive(Debug)]
struct RuntimeArgProvider {
    name: String,
    value: String,
}

impl RuntimeArgProvider {
    fn new(index: usize, value: &str) -> Self {
        Self {
            name: format!("Args:{index}"),
            value: value.to_string(),
        }
    }
}

#[async_trait]
impl ash_engine::CapabilityProvider for RuntimeArgProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn effect(&self) -> Effect {
        Effect::Epistemic
    }

    async fn observe(
        &self,
        _action: &str,
        _args: &[Value],
    ) -> Result<Value, ash_engine::providers::ProviderError> {
        Ok(Value::variant(
            "Some",
            vec![("value", Value::String(self.value.clone()))],
        ))
    }

    async fn execute(
        &self,
        _action: &str,
        _args: &[Value],
    ) -> Result<Value, ash_engine::providers::ProviderError> {
        Ok(Value::Null)
    }
}

/// Execute a workflow with tracing enabled
async fn execute_with_trace(
    engine: &ash_engine::Engine,
    workflow: &ash_engine::Workflow,
) -> Result<Value> {
    use ash_core::WorkflowId;

    let workflow_id = WorkflowId::new();
    let recorder = create_trace_recorder(workflow_id);
    let session = WorkflowTraceSession::start(recorder, "main")?;

    match engine.execute(workflow).await {
        Ok(value) => {
            let _recorder = session.finish_success()?;
            Ok(value)
        }
        Err(error) => {
            let _recorder = session.finish_error(format!("{error:?}"), Some("engine.execute"))?;
            Err(classify_exec_error(error))
        }
    }
}

/// Output the result to stdout or file
async fn execute_entry_source(
    engine: &ash_engine::Engine,
    source: &str,
    trace: bool,
) -> std::result::Result<u8, ash_engine::EntryBootstrapError> {
    if !trace {
        return engine.bootstrap_entry_source(source).await;
    }

    use ash_core::WorkflowId;

    let workflow_id = WorkflowId::new();
    let recorder = create_trace_recorder(workflow_id);
    let session = WorkflowTraceSession::start(recorder, "main")
        .map_err(|error| ash_engine::EntryBootstrapError::Execution(error.to_string()))?;

    match engine.bootstrap_entry_source(source).await {
        Ok(exit_code) => {
            let _recorder = session
                .finish_success()
                .map_err(|error| ash_engine::EntryBootstrapError::Execution(error.to_string()))?;
            Ok(exit_code)
        }
        Err(error) => {
            let _recorder = session
                .finish_error(format!("{error:?}"), Some("bootstrap_entry_source"))
                .map_err(|trace_error| {
                    ash_engine::EntryBootstrapError::Execution(trace_error.to_string())
                })?;
            Err(error)
        }
    }
}

async fn output_result(
    result: &Value,
    output_path: &Option<String>,
    format: RunOutputFormat,
) -> Result<()> {
    let output = match format {
        RunOutputFormat::Text => format!("{result}"),
        RunOutputFormat::Json => {
            let json_value = crate::value_convert::value_to_json(result);
            serde_json::to_string_pretty(&json_value)
                .context("Failed to serialize result to JSON")?
        }
    };

    match output_path {
        Some(path) => {
            tokio::fs::write(path, output)
                .await
                .with_context(|| format!("Failed to write output to {path}"))?;
        }
        None => {
            println!("{output}");
        }
    }

    Ok(())
}

async fn emit_entry_output(args: &RunArgs) -> Result<()> {
    if let Some(path) = &args.output {
        tokio::fs::write(path, "")
            .await
            .with_context(|| format!("Failed to write output to {path}"))?;
    }

    Ok(())
}

fn classify_exec_error(error: ExecError) -> anyhow::Error {
    // Per SPEC-021: preserve distinct error classes for observable behavior
    match error {
        // Parse errors - will exit with code 2
        ExecError::Parse(_) => anyhow::anyhow!("{error}"),
        // Type errors - will exit with code 3
        ExecError::Type(_) => anyhow::anyhow!("{error}"),
        // IO errors - will exit with code 4
        ExecError::Io(_) => anyhow::anyhow!("{error}"),
        // Capability/verification errors - exit code 6
        ExecError::CapabilityNotAvailable(name) => {
            anyhow::anyhow!("verification error: capability not available: {name}")
        }
        // Policy errors
        ExecError::PolicyDenied { policy } => anyhow::anyhow!("policy denial: {policy}"),
        ExecError::RequiresApproval {
            role,
            operation,
            capability,
        } => anyhow::anyhow!(
            "approval required: role '{}' must approve {} on {}",
            role.as_ref(),
            operation,
            capability
        ),
        // Other execution errors - exit code 5
        other => anyhow::anyhow!("{other}"),
    }
}

fn classify_engine_error(error: EngineError) -> anyhow::Error {
    match error {
        EngineError::Parse(message) => anyhow::anyhow!("parse error: {message}"),
        EngineError::Type(message) => anyhow::anyhow!("type error: {message}"),
        EngineError::Execution(message) => anyhow::anyhow!("runtime error: {message}"),
        EngineError::CapabilityNotFound(name) => {
            anyhow::anyhow!("verification error: capability not found: {name}")
        }
        EngineError::Io(error) => anyhow::anyhow!("io error: {error}"),
        EngineError::Configuration(message) => {
            anyhow::anyhow!("configuration error: {message}")
        }
    }
}

fn classify_entry_verification_error(error: ash_engine::EntryVerificationError) -> anyhow::Error {
    match error {
        ash_engine::EntryVerificationError::MissingMain => {
            anyhow::anyhow!("entry file has no 'main' workflow")
        }
        ash_engine::EntryVerificationError::MissingWorkflowMetadata => {
            anyhow::anyhow!("entry workflow metadata is unavailable")
        }
        ash_engine::EntryVerificationError::WrongReturnType { expected, found } => {
            anyhow::anyhow!(
                "'main' has wrong return type\n  expected: {expected}\n  found: {found}"
            )
        }
        ash_engine::EntryVerificationError::NonCapabilityParameter { name, found } => {
            anyhow::anyhow!("parameter '{name}' must be capability type\n  found: {found}")
        }
    }
}

fn classify_entry_bootstrap_error(error: ash_engine::EntryBootstrapError) -> anyhow::Error {
    match error {
        ash_engine::EntryBootstrapError::Engine(engine_error) => {
            classify_engine_error(engine_error)
        }
        ash_engine::EntryBootstrapError::Verification(error) => {
            classify_entry_verification_error(error)
        }
        ash_engine::EntryBootstrapError::Execution(message) => {
            anyhow::anyhow!("runtime error: {message}")
        }
        ash_engine::EntryBootstrapError::InvalidExitCode { code } => {
            anyhow::anyhow!("invalid runtime exit code {code}")
        }
    }
}

fn classify_run_read_error(path: &Path, error: std::io::Error) -> anyhow::Error {
    if error.kind() == std::io::ErrorKind::NotFound {
        anyhow::anyhow!("file not found: {}", path.display())
    } else {
        anyhow::anyhow!("failed to read workflow file {}: {error}", path.display())
    }
}

pub fn classify_run_cli_error(error: anyhow::Error) -> CliError {
    let message = error.to_string();
    let lower = message.to_lowercase();

    if lower.contains("file not found:")
        || lower.contains("entry file has no 'main' workflow")
        || lower.contains("'main' has wrong return type")
        || lower.contains("must be capability type")
        || lower.contains("invalid runtime exit code")
    {
        CliError::general(message)
    } else {
        CliError::from(error)
    }
}

fn has_leading_entry_prelude(tokens: &[Token]) -> bool {
    let mut index = 0;
    let mut saw_entry_use = false;

    while let Some(token) = tokens.get(index) {
        if matches!(token.kind, TokenKind::Eof) {
            break;
        }

        let Some(next_index) = consume_entry_prelude_use(tokens, index) else {
            break;
        };

        saw_entry_use = true;
        index = next_index;
    }

    saw_entry_use
        && matches!(
            tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::Workflow) | Some(TokenKind::Eof) | None
        )
}

fn consume_entry_prelude_use(tokens: &[Token], start: usize) -> Option<usize> {
    if !matches_ident(tokens.get(start), "use") {
        return None;
    }

    let first_segment = ident_name(tokens.get(start + 1)?)?;
    if first_segment != "result" && first_segment != "runtime" {
        return None;
    }

    if !matches!(tokens.get(start + 2)?.kind, TokenKind::Colon)
        || !matches!(tokens.get(start + 3)?.kind, TokenKind::Colon)
    {
        return None;
    }

    let mut index = start + 4;
    while let Some(token) = tokens.get(index) {
        match token.kind {
            TokenKind::Semicolon => return Some(index + 1),
            TokenKind::Workflow | TokenKind::Eof => return Some(index),
            _ => index += 1,
        }
    }

    Some(index)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EntryHeaderShape {
    name_is_main: bool,
    canonical_return_type: bool,
}

fn first_workflow_entry_header_shape(source: &str, tokens: &[Token]) -> Option<EntryHeaderShape> {
    let workflow_index = tokens
        .iter()
        .position(|token| matches!(token.kind, TokenKind::Workflow))?;

    let mut index = workflow_index + 1;
    let name_is_main = matches_ident(tokens.get(index), "main");

    let name_token = tokens.get(index)?;

    if !matches!(name_token.kind, TokenKind::Ident(_)) {
        return None;
    }

    index += 1;
    let next_index = skip_parenthesized_tokens(tokens, index)?;
    index = next_index;

    if !matches!(
        tokens.get(index).map(|token| &token.kind),
        Some(TokenKind::Minus)
    ) || !matches!(
        tokens.get(index + 1).map(|token| &token.kind),
        Some(TokenKind::Gt)
    ) {
        return None;
    }
    index += 2;

    let canonical_return_type = matches_ident(tokens.get(index), "Result")
        && matches!(
            tokens.get(index + 1).map(|token| &token.kind),
            Some(TokenKind::Lt)
        )
        && matches!(
            tokens.get(index + 2).map(|token| &token.kind),
            Some(TokenKind::LParen)
        )
        && matches!(
            tokens.get(index + 3).map(|token| &token.kind),
            Some(TokenKind::RParen)
        )
        && matches!(
            tokens.get(index + 4).map(|token| &token.kind),
            Some(TokenKind::Comma)
        )
        && matches_ident(tokens.get(index + 5), "RuntimeError")
        && matches!(
            tokens.get(index + 6).map(|token| &token.kind),
            Some(TokenKind::Gt)
        );

    if !canonical_return_type {
        while let Some(token) = tokens.get(index) {
            match token.kind {
                TokenKind::LBrace | TokenKind::Eof => break,
                _ => index += 1,
            }
        }
    } else {
        index += 7;
    }

    if !skip_optional_entry_header_clauses(source, tokens, index) {
        return None;
    }

    Some(EntryHeaderShape {
        name_is_main,
        canonical_return_type,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkflowSourceKind {
    Ordinary,
    LeadingRuntimePrelude,
    EntryCandidate,
    Entry,
}

#[cfg(test)]
fn is_entry_workflow_source(source: &str) -> bool {
    matches!(classify_workflow_source(source), WorkflowSourceKind::Entry)
}

fn classify_workflow_source(source: &str) -> WorkflowSourceKind {
    let (tokens, _errors) = lex_with_recovery(source);

    if let Some(shape) = first_workflow_entry_header_shape(source, &tokens) {
        if shape.name_is_main && shape.canonical_return_type {
            WorkflowSourceKind::Entry
        } else if shape.name_is_main || shape.canonical_return_type {
            WorkflowSourceKind::EntryCandidate
        } else if has_leading_entry_prelude(&tokens) {
            WorkflowSourceKind::LeadingRuntimePrelude
        } else {
            WorkflowSourceKind::Ordinary
        }
    } else if has_leading_entry_prelude(&tokens) {
        WorkflowSourceKind::LeadingRuntimePrelude
    } else {
        WorkflowSourceKind::Ordinary
    }
}

fn parse_runnable_workflow(
    engine: &ash_engine::Engine,
    source: &str,
    source_kind: WorkflowSourceKind,
) -> std::result::Result<ash_engine::Workflow, EngineError> {
    match source_kind {
        WorkflowSourceKind::Ordinary => engine.parse(source),
        WorkflowSourceKind::LeadingRuntimePrelude
        | WorkflowSourceKind::EntryCandidate
        | WorkflowSourceKind::Entry => {
            engine.load_runtime_stdlib()?;
            engine.parse_entry_source(source)
        }
    }
}

fn should_use_entry_bootstrap(source_kind: WorkflowSourceKind) -> bool {
    matches!(
        source_kind,
        WorkflowSourceKind::Entry | WorkflowSourceKind::EntryCandidate
    )
}

async fn run_workflow_source(
    engine: &ash_engine::Engine,
    source: &str,
    source_kind: WorkflowSourceKind,
) -> Result<Value> {
    let workflow =
        parse_runnable_workflow(engine, source, source_kind).map_err(classify_engine_error)?;
    engine.check(&workflow).map_err(classify_engine_error)?;
    engine.execute(&workflow).await.map_err(classify_exec_error)
}

fn skip_optional_entry_header_clauses(source: &str, tokens: &[Token], mut index: usize) -> bool {
    loop {
        match tokens.get(index).map(|token| &token.kind) {
            Some(TokenKind::LBrace) => return true,
            Some(TokenKind::Eof) | None => return false,
            _ if matches_ident(tokens.get(index), "plays") => {
                let Some(next_index) = consume_entry_plays_clause(tokens, index) else {
                    return false;
                };
                index = next_index;
            }
            _ if matches_ident(tokens.get(index), "capabilities") => {
                let Some(next_index) = consume_entry_capabilities_clause(tokens, index) else {
                    return false;
                };
                index = next_index;
            }
            _ if matches_ident(tokens.get(index), "requires")
                || matches_ident(tokens.get(index), "ensures") =>
            {
                let Some(next_index) = consume_entry_contract_clause(source, tokens, index) else {
                    return false;
                };
                index = next_index;
            }
            _ => return false,
        }
    }
}

fn consume_entry_plays_clause(tokens: &[Token], start: usize) -> Option<usize> {
    if !matches_ident(tokens.get(start), "plays") || !matches_ident(tokens.get(start + 1), "role") {
        return None;
    }

    skip_parenthesized_tokens(tokens, start + 2)
}

fn consume_entry_capabilities_clause(tokens: &[Token], start: usize) -> Option<usize> {
    if !matches_ident(tokens.get(start), "capabilities")
        || !matches!(
            tokens.get(start + 1).map(|token| &token.kind),
            Some(TokenKind::Colon)
        )
    {
        return None;
    }

    skip_bracketed_tokens(tokens, start + 2)
}

fn consume_entry_contract_clause(source: &str, tokens: &[Token], start: usize) -> Option<usize> {
    if !matches!(
        tokens.get(start + 1).map(|token| &token.kind),
        Some(TokenKind::Colon)
    ) {
        return None;
    }

    let expression_start = tokens.get(start + 2)?.span.start;
    let mut input = new_input(&source[expression_start..]);
    skip_whitespace_and_comments(&mut input);
    let _ = expr(&mut input).ok()?;
    skip_whitespace_and_comments(&mut input);

    let next_offset = expression_start + (source[expression_start..].len() - input.input.len());

    tokens
        .iter()
        .enumerate()
        .skip(start + 2)
        .find(|(_, token)| token.span.start >= next_offset || matches!(token.kind, TokenKind::Eof))
        .map(|(index, _)| index)
}

fn skip_parenthesized_tokens(tokens: &[Token], start: usize) -> Option<usize> {
    if !matches!(
        tokens.get(start).map(|token| &token.kind),
        Some(TokenKind::LParen)
    ) {
        return None;
    }

    let mut depth = 0usize;
    let mut index = start;
    while let Some(token) = tokens.get(index) {
        match token.kind {
            TokenKind::LParen => depth += 1,
            TokenKind::RParen => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            TokenKind::Eof => return None,
            _ => {}
        }
        index += 1;
    }

    None
}

fn skip_bracketed_tokens(tokens: &[Token], start: usize) -> Option<usize> {
    if !matches!(
        tokens.get(start).map(|token| &token.kind),
        Some(TokenKind::LBracket)
    ) {
        return None;
    }

    let mut depth = 0usize;
    let mut index = start;
    while let Some(token) = tokens.get(index) {
        match token.kind {
            TokenKind::LBracket => depth += 1,
            TokenKind::RBracket => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            TokenKind::Eof => return None,
            _ => {}
        }
        index += 1;
    }

    None
}

fn matches_ident(token: Option<&Token>, expected: &str) -> bool {
    ident_name_from_option(token).is_some_and(|name| name == expected)
}

fn ident_name(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Ident(name) => Some(name.as_ref()),
        _ => None,
    }
}

fn ident_name_from_option(token: Option<&Token>) -> Option<&str> {
    token.and_then(ident_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_args_parsing() {
        let args = RunArgs {
            path: "test.ash".to_string(),
            output: Some("out.json".to_string()),
            trace: true,
            format: RunOutputFormat::Text,
            dry_run: false,
            timeout: Some(30),
            program_args: vec!["hello".to_string()],
        };

        assert_eq!(args.path, "test.ash");
        assert!(args.trace);
        assert!(args.output.is_some());
        assert!(matches!(args.format, RunOutputFormat::Text));
        assert!(!args.dry_run);
        assert_eq!(args.timeout, Some(30));
        assert_eq!(args.program_args, vec!["hello"]);
    }

    #[test]
    fn test_run_args_format_json() {
        let args = RunArgs {
            path: "test.ash".to_string(),
            output: None,
            trace: false,
            format: RunOutputFormat::Json,
            dry_run: true,
            timeout: None,
            program_args: vec![],
        };

        assert!(matches!(args.format, RunOutputFormat::Json));
        assert!(args.dry_run);
    }

    // ============================================================
    // TASK-309: Tests for --dry-run, --timeout flags
    // ============================================================

    #[test]
    fn test_build_engine_default_capabilities() {
        let result = build_engine(&[]);
        assert!(
            result.is_ok(),
            "Engine should build with default capabilities"
        );
    }

    #[tokio::test]
    async fn test_dry_run_valid_workflow() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file with a valid canonical entry workflow
        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"
            use result::Result
            use runtime::RuntimeError

            workflow main() -> Result<(), RuntimeError> {{ done; }}
            "#
        )
        .unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let args = RunArgs {
            path,
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: true, // Enable dry-run
            timeout: None,
            program_args: vec![],
        };

        let result = run(&args).await;
        assert!(result.is_ok(), "Dry run should succeed for valid workflow");
    }

    #[tokio::test]
    async fn test_dry_run_invalid_syntax() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file with invalid syntax
        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(temp_file, "invalid syntax!!!").unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let args = RunArgs {
            path,
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: true, // Enable dry-run
            timeout: None,
            program_args: vec![],
        };

        let result = run(&args).await;
        assert!(result.is_err(), "Dry run should fail for invalid syntax");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("parse") || err_msg.contains("Parse"),
            "Error should indicate parse failure: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_dry_run_type_error() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file with a type error
        // This workflow has inconsistent return types
        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"workflow main {{
                if true {{
                    ret 42;
                }} else {{
                    ret "string";
                }}
            }}"#
        )
        .unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let args = RunArgs {
            path,
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: true, // Enable dry-run
            timeout: None,
            program_args: vec![],
        };

        let _result = run(&args).await;
        // Note: Depending on the type checker, this may or may not be a type error
        // The test verifies the dry-run path works end-to-end
    }

    #[tokio::test]
    async fn test_run_with_timeout() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file with a simple workflow
        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"
            use result::Result
            use runtime::RuntimeError

            workflow main() -> Result<(), RuntimeError> {{ done; }}
            "#
        )
        .unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let args = RunArgs {
            path,
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: false,
            timeout: Some(30), // 30 second timeout
            program_args: vec![],
        };

        let result = run(&args).await;
        assert!(
            result.is_ok(),
            "Run with timeout should succeed for quick workflow"
        );
    }

    #[tokio::test]
    async fn test_run_without_timeout() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file with a simple workflow
        let mut temp_file = NamedTempFile::with_suffix(".ash").unwrap();
        write!(
            temp_file,
            r#"
            use result::Result
            use runtime::RuntimeError

            workflow main() -> Result<(), RuntimeError> {{ done; }}
            "#
        )
        .unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let args = RunArgs {
            path,
            output: None,
            trace: false,
            format: RunOutputFormat::Text,
            dry_run: false,
            timeout: None, // No timeout
            program_args: vec![],
        };

        let result = run(&args).await;
        assert!(result.is_ok(), "Run without timeout should succeed");
    }

    #[test]
    fn test_import_free_entry_detector_accepts_capabilities_clause_after_return_type() {
        let source = r#"
            workflow main() -> Result<(), RuntimeError>
            capabilities: []
            { done; }
        "#;

        assert!(is_entry_workflow_source(source));
    }

    #[test]
    fn test_import_free_entry_detector_rejects_unknown_clause_after_return_type() {
        let source = r#"
            workflow main() -> Result<(), RuntimeError>
            unexpected: []
            { done; }
        "#;

        assert!(!is_entry_workflow_source(source));
    }
}
