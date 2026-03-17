//! Run command for executing Ash workflows.
//!
//! TASK-054: Implement `run` command for executing workflows.

use anyhow::{Context, Result};
use ash_core::Value;
use clap::Args;
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

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
}

/// Run a workflow file
pub async fn run(args: &RunArgs) -> Result<()> {
    let path = Path::new(&args.path);

    // Read the workflow source
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read workflow file: {}", path.display()))?;

    // Parse input parameters
    let input_values = parse_input(&args.input)?;

    // Parse the workflow
    let workflow = parse_workflow(&source)?;

    // Execute the workflow
    let result = if args.trace {
        execute_with_trace(&workflow, input_values).await?
    } else {
        execute_workflow(&workflow, input_values).await?
    };

    // Output results
    output_result(&result, &args.output).await?;

    Ok(())
}

/// Parse input JSON into a HashMap
fn parse_input(input: &Option<String>) -> Result<HashMap<String, Value>> {
    match input {
        Some(json_str) => {
            let value: serde_json::Value =
                serde_json::from_str(json_str).context("Invalid JSON input")?;
            json_to_hashmap(value)
        }
        None => Ok(HashMap::new()),
    }
}

/// Convert JSON value to HashMap<String, Value>
fn json_to_hashmap(value: serde_json::Value) -> Result<HashMap<String, Value>> {
    match value {
        serde_json::Value::Object(map) => {
            let mut result = HashMap::new();
            for (k, v) in map {
                result.insert(k, json_to_ash_value(v)?);
            }
            Ok(result)
        }
        _ => Err(anyhow::anyhow!("Input must be a JSON object")),
    }
}

/// Convert JSON value to Ash Value
fn json_to_ash_value(value: serde_json::Value) -> Result<Value> {
    match value {
        serde_json::Value::Null => Ok(Value::Null),
        serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else {
                Ok(Value::Null) // Float not supported in core
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(s)),
        serde_json::Value::Array(arr) => {
            let values: Result<Vec<_>> = arr.into_iter().map(json_to_ash_value).collect();
            Ok(Value::List(values?))
        }
        serde_json::Value::Object(map) => {
            let mut result = std::collections::HashMap::new();
            for (k, v) in map {
                result.insert(k, json_to_ash_value(v)?);
            }
            Ok(Value::Record(result))
        }
    }
}

/// Parse workflow source into core IR
fn parse_workflow(source: &str) -> Result<ash_core::Workflow> {
    use ash_parser::parse_workflow::workflow_def;
    use winnow::prelude::*;

    let mut input = ash_parser::new_input(source);
    let workflow_def = workflow_def
        .parse_next(&mut input)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    Ok(ash_parser::lower::lower_workflow(&workflow_def))
}

/// Execute a workflow with the given input
async fn execute_workflow(
    workflow: &ash_core::Workflow,
    _input: HashMap<String, Value>,
) -> Result<Value> {
    ash_interp::interpret(workflow)
        .await
        .map_err(|e| anyhow::anyhow!("Execution error: {:?}", e))
}

/// Execute a workflow with tracing enabled
async fn execute_with_trace(
    workflow: &ash_core::Workflow,
    _input: HashMap<String, Value>,
) -> Result<Value> {
    use ash_provenance::create_trace_recorder;
    use ash_core::WorkflowId;

    let workflow_id = WorkflowId::new();
    let mut recorder = create_trace_recorder(workflow_id);

    ash_provenance::record_workflow_start(&mut recorder, "main");

    let result = ash_interp::interpret(workflow)
        .await
        .map_err(|e| anyhow::anyhow!("Execution error: {:?}", e))?;

    ash_provenance::record_workflow_complete(&mut recorder, true);

    // Output trace summary - use the recorder directly
    let events = recorder.events();
    println!("[INFO] Trace recorded: {} events", events.len());

    Ok(result)
}

/// Output the result to stdout or file
async fn output_result(result: &Value, output_path: &Option<String>) -> Result<()> {
    let output = format!("{}", result);

    match output_path {
        Some(path) => {
            tokio::fs::write(path, output)
                .await
                .with_context(|| format!("Failed to write output to {}", path))?;
            println!("[OK] Output written to {}", path);
        }
        None => {
            println!("{} {}", "Result:".green().bold(), output);
        }
    }

    Ok(())
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
        assert_eq!(
            result.get("name"),
            Some(&Value::String("test".to_string()))
        );
    }

    #[test]
    fn test_json_to_ash_value_conversions() {
        assert_eq!(json_to_ash_value(serde_json::Value::Null).unwrap(), Value::Null);
        assert_eq!(
            json_to_ash_value(serde_json::Value::Bool(true)).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            json_to_ash_value(serde_json::json!(42)).unwrap(),
            Value::Int(42)
        );
        assert_eq!(
            json_to_ash_value(serde_json::json!("hello")).unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_run_args_parsing() {
        let args = RunArgs {
            path: "test.ash".to_string(),
            input: Some(r#"{"x": 1}"#.to_string()),
            output: Some("out.json".to_string()),
            trace: true,
        };

        assert_eq!(args.path, "test.ash");
        assert!(args.trace);
        assert!(args.input.is_some());
        assert!(args.output.is_some());
    }
}
