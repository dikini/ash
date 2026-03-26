//! Run command for executing Ash workflows.
//!
//! TASK-054: Implement `run` command for executing workflows.
//! TASK-076: Updated to use ash-engine.
//! TASK-278: Added input binding support for --input flag.

use anyhow::{Context, Result};
use ash_core::Value;
use ash_engine::EngineError;
use ash_interp::ExecError;
use ash_provenance::{WorkflowTraceSession, create_trace_recorder};
use clap::Args;
use std::collections::HashMap;
use std::path::Path;

use crate::value_convert::json_to_value;

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

    /// Input parameters as JSON object
    #[arg(short, long, value_name = "JSON")]
    pub input: Option<String>,

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

    /// Grant capability (repeatable)
    #[arg(long, value_name = "NAME")]
    pub capability: Vec<String>,
}

/// Run a workflow file
pub async fn run(args: &RunArgs) -> Result<()> {
    let path = Path::new(&args.path);

    // Parse input parameters from JSON
    let input_values = parse_input(&args.input)?;

    // Create engine with capabilities
    let engine = ash_engine::Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build()
        .context("Failed to build engine")?;

    // Run the workflow file with input bindings
    let result = if args.trace {
        let workflow = engine.parse_file(path).map_err(classify_engine_error)?;
        engine.check(&workflow).map_err(classify_engine_error)?;
        execute_with_trace_and_input(&engine, &workflow, input_values).await?
    } else {
        engine
            .run_file_with_input(path, input_values)
            .await
            .map_err(classify_exec_error)?
    };

    // Output results
    output_result(&result, &args.output, args.format).await?;

    Ok(())
}

/// Parse input JSON into a HashMap<String, Value>
///
/// This function parses a JSON string and converts it into a HashMap
/// that can be used as input bindings for workflow execution.
pub fn parse_input(input: &Option<String>) -> Result<HashMap<String, Value>> {
    match input {
        Some(json_str) => {
            let json_value: serde_json::Value =
                serde_json::from_str(json_str).context("Invalid JSON input")?;
            json_to_hashmap(json_value)
        }
        None => Ok(HashMap::new()),
    }
}

/// Convert JSON value to HashMap<String, Value>
///
/// The input must be a JSON object (key-value pairs).
pub fn json_to_hashmap(value: serde_json::Value) -> Result<HashMap<String, Value>> {
    match value {
        serde_json::Value::Object(map) => {
            let mut result = HashMap::new();
            for (k, v) in map {
                result.insert(k, json_to_value(v));
            }
            Ok(result)
        }
        _ => Err(anyhow::anyhow!("Input must be a JSON object")),
    }
}

/// Execute a workflow with tracing enabled and input bindings
async fn execute_with_trace_and_input(
    engine: &ash_engine::Engine,
    workflow: &ash_engine::Workflow,
    input_bindings: HashMap<String, Value>,
) -> Result<Value> {
    use ash_core::WorkflowId;

    let workflow_id = WorkflowId::new();
    let recorder = create_trace_recorder(workflow_id);
    let session = WorkflowTraceSession::start(recorder, "main")?;

    match engine.execute_with_input(workflow, input_bindings).await {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input_empty() {
        let result = parse_input(&None).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_input_valid_json() {
        let json = r#"{"x": 42, "name": "test"}"#.to_string();
        let result = parse_input(&Some(json)).unwrap();
        assert_eq!(result.get("x"), Some(&Value::Int(42)));
        assert_eq!(result.get("name"), Some(&Value::String("test".to_string())));
    }

    #[test]
    fn test_json_to_ash_value_conversions() {
        use crate::value_convert::json_to_value;

        assert_eq!(json_to_value(serde_json::Value::Null), Value::Null);
        assert_eq!(
            json_to_value(serde_json::Value::Bool(true)),
            Value::Bool(true)
        );
        assert_eq!(json_to_value(serde_json::json!(42)), Value::Int(42));
        assert_eq!(
            json_to_value(serde_json::json!("hello")),
            Value::String("hello".to_string())
        );
        assert_eq!(
            json_to_value(serde_json::json!([1, true, "hello"])),
            Value::List(Box::new(vec![
                Value::Int(1),
                Value::Bool(true),
                Value::String("hello".to_string()),
            ]))
        );
        assert_eq!(
            json_to_value(serde_json::json!({"nested": {"value": 42}})),
            Value::Record(Box::new(HashMap::from([(
                "nested".to_string(),
                Value::Record(Box::new(HashMap::from([(
                    "value".to_string(),
                    Value::Int(42),
                )]))),
            )])))
        );
    }

    #[test]
    fn test_run_args_parsing() {
        let args = RunArgs {
            path: "test.ash".to_string(),
            input: Some(r#"{"x": 1}"#.to_string()),
            output: Some("out.json".to_string()),
            trace: true,
            format: RunOutputFormat::Text,
            dry_run: false,
            timeout: Some(30),
            capability: vec!["fs".to_string(), "http".to_string()],
        };

        assert_eq!(args.path, "test.ash");
        assert!(args.trace);
        assert!(args.input.is_some());
        assert!(args.output.is_some());
        assert!(matches!(args.format, RunOutputFormat::Text));
        assert!(!args.dry_run);
        assert_eq!(args.timeout, Some(30));
        assert_eq!(args.capability.len(), 2);
    }

    #[test]
    fn test_run_args_format_json() {
        let args = RunArgs {
            path: "test.ash".to_string(),
            input: None,
            output: None,
            trace: false,
            format: RunOutputFormat::Json,
            dry_run: true,
            timeout: None,
            capability: vec![],
        };

        assert!(matches!(args.format, RunOutputFormat::Json));
        assert!(args.dry_run);
        assert!(args.capability.is_empty());
    }
}
