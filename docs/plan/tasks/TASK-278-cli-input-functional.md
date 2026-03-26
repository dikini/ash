# TASK-278: Make CLI --input Functional

## Status: 📝 Planned

## Description

Fix the CLI issue where `ash run --input file.json` parses the JSON but then discards it before execution. The `_input_values` are parsed and stored but never bound to the workflow's input parameter.

## Specification Reference

- SPEC-005: CLI Specification - Section 2.2 (Input Handling)

## Dependencies

- ✅ TASK-054: CLI run command
- ✅ TASK-274: Engine provider wiring (input binding pattern)

## Requirements

### Functional Requirements

1. CLI --input JSON must be bound to workflow input parameter
2. JSON types must map correctly to Ash Value types
3. Input binding must occur before workflow execution
4. Missing input for parameterized workflow must be an error
5. Excess input fields should be handled gracefully (warn or ignore)

### Current State (Broken)

**File:** `crates/ash-cli/src/commands/run.rs`

```rust
pub fn execute(args: RunArgs) -> Result<(), CliError> {
    let source = fs::read_to_string(&args.file)?;
    let workflow = parse(&source)?;
    
    // Parse input but discard it!
    let _input_values = if let Some(input_path) = &args.input {
        let input_json = fs::read_to_string(input_path)?;
        serde_json::from_str(&input_json)? // Parsed but not used!
    } else {
        serde_json::json!(null)
    };
    
    // Execute without input binding
    let engine = Engine::new();
    let result = engine.execute(&workflow, Value::Null)?; // Always Null!
    
    println!("{}", result.value);
    Ok(())
}
```

### Target State (Fixed)

```rust
pub fn execute(args: RunArgs) -> Result<(), CliError> {
    let source = fs::read_to_string(&args.file)?;
    let workflow = parse(&source)?;
    
    // Parse and convert input
    let input_value = if let Some(input_path) = &args.input {
        let input_json = fs::read_to_string(input_path)?;
        json_to_value(serde_json::from_str(&input_json)?)
    } else {
        Value::Null
    };
    
    // Validate input against workflow signature
    if let Some(ref params) = workflow.params {
        validate_input(&input_value, params)?;
    }
    
    // Execute WITH input
    let engine = Engine::new();
    let result = engine.execute(&workflow, input_value)?;
    
    println!("{}", result.value);
    Ok(())
}

fn json_to_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Float(n.as_f64().unwrap())
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            Value::List(arr.into_iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(obj) => {
            Value::Record(obj.into_iter()
                .map(|(k, v)| (k, json_to_value(v)))
                .collect())
        }
    }
}

fn validate_input(input: &Value, params: &Params) -> Result<(), CliError> {
    // Check required parameters are present
    for param in &params.required {
        if !input.has_field(&param.name) {
            return Err(CliError::MissingInput {
                param: param.name.clone(),
            });
        }
    }
    Ok(())
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-cli/tests/input_functional_test.rs`

```rust
//! Tests for CLI --input functionality

use std::process::Command;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_input_bound_to_workflow() {
    let temp = TempDir::new().unwrap();
    
    // Create workflow file
    let workflow = r#"
        workflow greet(name: String) {
            act print("Hello, " + name);
        }
    "#;
    let workflow_path = temp.path().join("greet.ash");
    fs::write(&workflow_path, workflow).unwrap();
    
    // Create input file
    let input = r#"{"name": "World"}"#;
    let input_path = temp.path().join("input.json");
    fs::write(&input_path, input).unwrap();
    
    // Run with input
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ash", "--", "run"])
        .arg(&workflow_path)
        .arg("--input")
        .arg(&input_path)
        .output()
        .expect("Failed to execute");
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Hello, World"));
}

#[test]
fn test_missing_input_error() {
    let temp = TempDir::new().unwrap();
    
    // Workflow requires input
    let workflow = r#"
        workflow test(required_param: Int) {
            decide required_param
        }
    "#;
    let workflow_path = temp.path().join("test.ash");
    fs::write(&workflow_path, workflow).unwrap();
    
    // Run without input
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ash", "--", "run"])
        .arg(&workflow_path)
        .output()
        .expect("Failed to execute");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MissingInput") || stderr.contains("required"));
}

#[test]
fn test_json_types_mapped_correctly() {
    let test_cases = vec![
        (r#"null"#, Value::Null),
        (r#"true"#, Value::Bool(true)),
        (r#"42"#, Value::Int(42)),
        (r#"3.14"#, Value::Float(3.14)),
        (r#""hello""#, Value::String("hello".to_string())),
        (r#"[1, 2, 3]"#, Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])),
    ];
    
    for (json, expected) in test_cases {
        let parsed = serde_json::from_str(json).unwrap();
        let value = json_to_value(parsed);
        assert_eq!(value, expected);
    }
}

#[test]
fn test_nested_json_object() {
    let json = r#"{"user": {"name": "Alice", "age": 30}}"#;
    let parsed = serde_json::from_str(json).unwrap();
    let value = json_to_value(parsed);
    
    if let Value::Record(fields) = value {
        assert!(fields.contains_key("user"));
    } else {
        panic!("Expected Record");
    }
}
```

### Step 2: Implement JSON to Value Conversion

**File:** `crates/ash-cli/src/value_convert.rs` (new file)

```rust
//! JSON to Ash Value conversion

use ash_core::Value;
use serde_json;

pub fn json_to_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(u) = n.as_u64() {
                // Handle large unsigned values
                Value::Int(i64::try_from(u).unwrap_or(i64::MAX))
            } else {
                Value::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            Value::List(arr.into_iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(obj) => {
            let fields: Vec<(String, Value)> = obj
                .into_iter()
                .map(|(k, v)| (k, json_to_value(v)))
                .collect();
            Value::Record(fields.into_iter().collect())
        }
    }
}

pub fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(i) => serde_json::Value::Number((*i).into()),
        Value::Float(f) => serde_json::Value::Number(
            serde_json::Number::from_f64(*f).unwrap_or(0.into())
        ),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::List(items) => {
            serde_json::Value::Array(items.iter().map(value_to_json).collect())
        }
        Value::Record(fields) => {
            let map: serde_json::Map<String, serde_json::Value> = fields
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        _ => serde_json::Value::Null, // Other variants as needed
    }
}
```

### Step 3: Update Run Command

**File:** `crates/ash-cli/src/commands/run.rs`

```rust
use crate::value_convert::json_to_value;

pub fn execute(args: RunArgs) -> Result<ExitCode, CliError> {
    let source = fs::read_to_string(&args.file)?;
    let workflow = parse(&source)?;
    
    // Parse and convert input
    let input_value = if let Some(input_path) = &args.input {
        let input_json = fs::read_to_string(input_path)?;
        let json: serde_json::Value = serde_json::from_str(&input_json)
            .map_err(|e| CliError::InvalidInput {
                path: input_path.clone(),
                message: e.to_string(),
            })?;
        json_to_value(json)
    } else {
        Value::Null
    };
    
    // Validate input matches workflow signature
    validate_workflow_input(&workflow, &input_value)?;
    
    // Execute with input
    let engine = Engine::new();
    let result = engine.execute(&workflow, input_value)?;
    
    // Output result
    match args.format {
        OutputFormat::Text => println!("{}", result.value),
        OutputFormat::Json => {
            let json = value_to_json(&result.value);
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
    }
    
    Ok(ExitCode::SUCCESS)
}

fn validate_workflow_input(workflow: &Workflow, input: &Value) -> Result<(), CliError> {
    // Check if workflow has parameters
    let param_count = workflow.params.as_ref().map(|p| p.len()).unwrap_or(0);
    
    if param_count == 0 {
        return Ok(()); // No params needed
    }
    
    // For single param workflows, input can be direct value
    if param_count == 1 && !matches!(input, Value::Record(_)) {
        return Ok(());
    }
    
    // For multi-param workflows, input should be a record
    if let Value::Record(fields) = input {
        // Could validate field names match param names here
        Ok(())
    } else {
        Err(CliError::InputTypeMismatch {
            expected: "object with named parameters".to_string(),
            got: input.type_name(),
        })
    }
}
```

### Step 4: Add Error Types

**File:** `crates/ash-cli/src/error.rs`

```rust
#[derive(Debug, Error)]
pub enum CliError {
    // ... existing errors ...
    
    #[error("invalid input file '{path}': {message}")]
    InvalidInput {
        path: PathBuf,
        message: String,
    },
    
    #[error("missing required input parameter: {param}")]
    MissingInput {
        param: String,
    },
    
    #[error("input type mismatch: expected {expected}, got {got}")]
    InputTypeMismatch {
        expected: String,
        got: String,
    },
}
```

## Verification Steps

- [ ] `cargo test -p ash-cli --test input_functional_test` passes
- [ ] `cargo test -p ash-cli --test cli` passes
- [ ] Manual test: `ash run workflow.ash --input input.json`
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Functional CLI --input parameter
- JSON to Value conversion utilities
- Input validation

Required by:
- TASK-279: CLI spec compliance

## Notes

**Critical Issue**: SPEC-005 promises input functionality that doesn't work. This breaks automation and scripting use cases.

**Design Decision**: Support both direct values (for single param) and record objects (for multi-param) for flexibility.

**Edge Cases**:
- Large JSON files - streaming parse if needed
- Type mismatches - clear error messages
- Missing fields - validate against workflow signature
- Extra fields - ignore or warn
