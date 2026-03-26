# TASK-292: Make CLI --input Functional

## Status: 📝 Planned

## Description

Fix the critical issue where `ash run --input` is a no-op after parsing. The JSON is parsed into `_input_values` and then discarded. This is a concrete bug and diverges from the CLI spec.

## Specification Reference

- SPEC-005: CLI Specification
- SPEC-010: Engine API Specification

## Dependencies

- ✅ TASK-054: CLI run command
- ✅ TASK-076: CLI engine integration

## Critical File Locations

- `crates/ash-cli/src/commands/run.rs:35` - input parsed but not used
- `crates/ash-cli/src/commands/run.rs:39` - _input_values discarded
- `crates/ash-cli/src/commands/run.rs:50` - execution without input

## Requirements

### Functional Requirements

1. `--input` JSON must be parsed into workflow parameters
2. Parsed input must be passed to workflow execution
3. Type mismatches must produce clear errors
4. Missing required parameters must be reported
5. Extra parameters should be handled gracefully

### Current State (Broken)

**File:** `crates/ash-cli/src/commands/run.rs:35-50`

```rust
pub fn execute(args: RunArgs) -> Result<(), CliError> {
    let workflow_source = fs::read_to_string(&args.file)?;
    
    // Parse input JSON but DON'T use it
    let _input_values = if let Some(input_path) = args.input {
        let input_json = fs::read_to_string(input_path)?;
        parse_input_json(&input_json)?  // Line 39: Parsed into _input_values
    } else {
        vec![]
    };
    
    let engine = Engine::new();
    let workflow = engine.parse(&workflow_source)?;
    
    // MISSING: Pass _input_values to execution
    let result = engine.execute(&workflow)?;  // Line 50: No input passed!
    
    println!("{:?}", result);
    Ok(())
}
```

Problems:
1. Input JSON is parsed but discarded
2. Workflow parameters cannot be provided
3. The --input flag is effectively useless
4. SPEC-005 compliance broken

### Target State (Fixed)

```rust
pub fn execute(args: RunArgs) -> Result<(), CliError> {
    let workflow_source = fs::read_to_string(&args.file)?;
    
    // FIX: Parse and keep input values
    let input_values: HashMap<String, Value> = if let Some(input_path) = args.input {
        let input_json = fs::read_to_string(input_path)?;
        parse_input_json(&input_json)?
    } else {
        HashMap::new()
    };
    
    let engine = Engine::new();
    let workflow = engine.parse(&workflow_source)?;
    
    // FIX: Validate inputs against workflow parameters
    validate_inputs(&workflow, &input_values)?;
    
    // FIX: Pass input values to execution
    let result = engine.execute_with_input(&workflow, input_values)?;
    
    // Format output according to --format flag
    format_output(&result, args.format)?;
    
    Ok(())
}

fn validate_inputs(
    workflow: &Workflow,
    inputs: &HashMap<String, Value>,
) -> Result<(), CliError> {
    // Check all required parameters are provided
    for param in &workflow.params {
        if param.required && !inputs.contains_key(&param.name) {
            return Err(CliError::MissingRequiredParameter {
                name: param.name.clone(),
                workflow: workflow.name.clone(),
            });
        }
    }
    
    // Check provided parameters match expected types
    for (name, value) in inputs {
        let param = workflow.params.iter()
            .find(|p| p.name == *name)
            .ok_or_else(|| CliError::UnknownParameter {
                name: name.clone(),
                workflow: workflow.name.clone(),
            })?;
        
        if !value.matches_type(&param.ty) {
            return Err(CliError::TypeMismatch {
                parameter: name.clone(),
                expected: param.ty.clone(),
                actual: value.type_name(),
            });
        }
    }
    
    Ok(())
}

fn parse_input_json(json: &str) -> Result<HashMap<String, Value>, CliError> {
    let parsed: serde_json::Value = serde_json::from_str(json)?;
    
    let obj = parsed.as_object()
        .ok_or_else(|| CliError::InvalidInputFormat {
            message: "Input must be a JSON object".to_string(),
        })?;
    
    let mut values = HashMap::new();
    for (key, val) in obj {
        values.insert(key.clone(), json_to_value(val)?);
    }
    
    Ok(values)
}

fn json_to_value(json: &serde_json::Value) -> Result<Value, CliError> {
    match json {
        serde_json::Value::Null => Ok(Value::Unit),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(CliError::InvalidInputFormat {
                    message: format!("Unsupported number: {}", n),
                })
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let values: Result<Vec<_>, _> = arr.iter().map(json_to_value).collect();
            Ok(Value::List(values?))
        }
        serde_json::Value::Object(obj) => {
            let mut fields = HashMap::new();
            for (k, v) in obj {
                fields.insert(k.clone(), json_to_value(v)?);
            }
            Ok(Value::Record(fields))
        }
    }
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-cli/tests/cli_input_test.rs`

```rust
//! Tests for CLI --input functionality

use std::process::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_input_provides_workflow_parameters() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow greet(name: String) {
            act print("Hello, " + name);
        }
    "#;
    
    let workflow_path = temp.path().join("greet.ash");
    fs::write(&workflow_path, workflow).unwrap();
    
    let input = r#"{"name": "World"}"#;
    let input_path = temp.path().join("input.json");
    fs::write(&input_path, input).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "run", "--input", input_path.to_str().unwrap(), workflow_path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello, World"), "Expected 'Hello, World', got: {}", stdout);
}

#[test]
fn test_input_missing_required_parameter() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow greet(name: String) {
            act print("Hello, " + name);
        }
    "#;
    
    let workflow_path = temp.path().join("greet.ash");
    fs::write(&workflow_path, workflow).unwrap();
    
    // No --input provided for required parameter
    let output = Command::new("cargo")
        .args(&["run", "--", "run", workflow_path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing required parameter"));
    assert!(stderr.contains("name"));
}

#[test]
fn test_input_type_mismatch() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow calculate(x: Int) {
            act print(x + 1);
        }
    "#;
    
    let workflow_path = temp.path().join("calc.ash");
    fs::write(&workflow_path, workflow).unwrap();
    
    let input = r#"{"x": "not a number"}"#;  // String instead of Int
    let input_path = temp.path().join("input.json");
    fs.write(&input_path, input).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "run", "--input", input_path.to_str().unwrap(), workflow_path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("type mismatch"));
}

#[test]
fn test_input_unknown_parameter() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow simple {
            act print("hello");
        }
    "#;
    
    let workflow_path = temp.path().join("simple.ash");
    fs::write(&workflow_path, workflow).unwrap();
    
    let input = r#"{"unknown": "value"}"#;
    let input_path = temp.path().join("input.json");
    fs::write(&input_path, input).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "run", "--input", input_path.to_str().unwrap(), workflow_path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown parameter"));
}

#[test]
fn test_input_multiple_parameters() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow configure(host: String, port: Int, debug: Bool) {
            act print("Connecting to " + host + ":" + port);
            if debug {
                act print("Debug mode enabled");
            }
        }
    "#;
    
    let workflow_path = temp.path().join("config.ash");
    fs::write(&workflow_path, workflow).unwrap();
    
    let input = r#"{"host": "localhost", "port": 8080, "debug": true}"#;
    let input_path = temp.path().join("input.json");
    fs::write(&input_path, input).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "run", "--input", input_path.to_str().unwrap(), workflow_path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("localhost:8080"));
    assert!(stdout.contains("Debug mode enabled"));
}

#[test]
fn test_input_complex_types() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow process(items: List<String>, config: {timeout: Int}) {
            act print("Processing " + items.length + " items");
            act print("Timeout: " + config.timeout);
        }
    "#;
    
    let workflow_path = temp.path().join("process.ash");
    fs::write(&workflow_path, workflow).unwrap();
    
    let input = r#"{"items": ["a", "b", "c"], "config": {"timeout": 30}}"#;
    let input_path = temp.path().join("input.json");
    fs::write(&input_path, input).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "run", "--input", input_path.to_str().unwrap(), workflow_path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3 items"));
    assert!(stdout.contains("30"));
}

#[test]
fn test_input_json_array_fails() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow simple {
            act print("hello");
        }
    "#;
    
    let workflow_path = temp.path().join("simple.ash");
    fs::write(&workflow_path, workflow).unwrap();
    
    let input = r#"["not", "an", "object"]"#;  // Array instead of object
    let input_path = temp.path().join("input.json");
    fs::write(&input_path, input).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "run", "--input", input_path.to_str().unwrap(), workflow_path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("must be a JSON object"));
}
```

### Step 2: Update Engine API for Input Support

**File:** `crates/ash-engine/src/lib.rs`

```rust
impl Engine {
    /// Execute workflow with input parameters
    pub fn execute_with_input(
        &self,
        workflow: &Workflow,
        inputs: HashMap<String, Value>,
    ) -> Result<ExecutionResult, EngineError> {
        // Bind inputs to workflow parameters
        let mut ctx = Context::new();
        for (name, value) in inputs {
            ctx.bind(name, value);
        }
        
        let mut state = RuntimeState::with_providers(self.provider_registry.clone());
        
        self.interpreter.execute(
            workflow,
            &mut ctx,
            &mut state,
        )
    }
    
    /// Execute workflow without inputs (for parameterless workflows)
    pub fn execute(&self, workflow: &Workflow) -> Result<ExecutionResult, EngineError> {
        self.execute_with_input(workflow, HashMap::new())
    }
}
```

### Step 3: Update CLI Run Command

**File:** `crates/ash-cli/src/commands/run.rs`

```rust
use std::collections::HashMap;
use ash_engine::Value;

pub fn execute(args: RunArgs) -> Result<ExitCode, CliError> {
    let workflow_source = fs::read_to_string(&args.file)?;
    
    // FIX: Parse and use input values
    let input_values: HashMap<String, Value> = if let Some(input_path) = &args.input {
        let input_json = fs::read_to_string(input_path)?;
        parse_input_json(&input_json)?
    } else {
        HashMap::new()
    };
    
    let engine = Engine::new();
    let workflow = engine.parse(&workflow_source)?;
    
    // FIX: Validate and bind inputs
    validate_inputs(&workflow, &input_values)?;
    
    // FIX: Execute with inputs
    let result = engine.execute_with_input(&workflow, input_values)?;
    
    // Output formatting
    match args.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Pretty => {
            println!("{:#?}", result);
        }
    }
    
    Ok(ExitCode::SUCCESS)
}
```

## Verification Steps

- [ ] `cargo test -p ash-cli --test cli_input_test` passes
- [ ] `--input` JSON is passed to workflow execution
- [ ] Missing parameters produce clear errors
- [ ] Type mismatches are reported
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Functional `--input` flag
- SPEC-005 compliance for CLI run command

Required by:
- Parameterized workflow execution
- CI/CD integration

## Notes

**Critical Issue**: This is a CLI functionality bug. The flag exists but does nothing.

**Risk Assessment**: High - user-facing feature is broken.

**Implementation Strategy**:
1. First: Add input validation
2. Second: Wire inputs through Engine API
3. Third: Update CLI to pass inputs
4. Fourth: Add integration tests
