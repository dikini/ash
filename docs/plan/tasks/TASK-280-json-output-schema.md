# TASK-280: Fix JSON Output Schema

## Status: 📝 Planned

## Description

Fix the CLI JSON output to match the documented machine-readable schema. Currently emits only `{ file, success, strict, error }` which is too lossy and hides required verification warnings.

## Specification Reference

- SPEC-005: CLI Specification - Section 5 (Output Formats)
- SPEC-021: Observable Behavior Specification

## Dependencies

- ✅ TASK-053: CLI check command
- ✅ TASK-279: CLI spec compliance (output format infrastructure)

## Requirements

### Functional Requirements

1. JSON output must include all fields from SPEC-005 schema
2. Verification warnings must be present in output
3. Error details must include location (file, line, column)
4. Schema version must be included
5. Nested error structures for multiple errors

### SPEC-005 JSON Schema

```json
{
  "schema_version": "1.0",
  "file": "path/to/workflow.ash",
  "success": false,
  "strict": true,
  "exit_code": 3,
  "errors": [
    {
      "severity": "error",
      "code": "E0032",
      "message": "type mismatch",
      "location": {
        "file": "workflow.ash",
        "line": 42,
        "column": 15
      },
      "context": "let x = \"string\" + 42",
      "help": "cannot add String and Int"
    }
  ],
  "warnings": [
    {
      "severity": "warning",
      "code": "W001",
      "message": "unused variable",
      "location": { ... }
    }
  ],
  "verification": {
    "obligations": 5,
    "satisfied": 4,
    "pending": ["audit_trail"]
  },
  "timing": {
    "parse_ms": 12,
    "typecheck_ms": 45,
    "total_ms": 58
  }
}
```

### Current State (Broken)

**File:** `crates/ash-cli/src/commands/check.rs`

```rust
fn output_json(result: &CheckResult) {
    let output = serde_json::json!({
        "file": result.file,
        "success": result.success,
        "strict": result.strict,
        "error": result.error_message(), // Lossy! Loses structure
    });
    println!("{}", output);
}
```

### Target State (Fixed)

**File:** `crates/ash-cli/src/output/json.rs` (new file)

```rust
//! JSON output format implementation

use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
pub struct JsonOutput {
    pub schema_version: String,
    pub file: String,
    pub success: bool,
    pub strict: bool,
    pub exit_code: u8,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<JsonError>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<JsonWarning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<JsonVerification>,
    pub timing: JsonTiming,
}

#[derive(Serialize)]
pub struct JsonError {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub location: JsonLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

#[derive(Serialize)]
pub struct JsonWarning {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub location: JsonLocation,
}

#[derive(Serialize)]
pub struct JsonLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Serialize)]
pub struct JsonVerification {
    pub obligations: usize,
    pub satisfied: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pending: Vec<String>,
}

#[derive(Serialize)]
pub struct JsonTiming {
    pub parse_ms: u64,
    pub typecheck_ms: u64,
    pub total_ms: u64,
}

impl JsonOutput {
    pub fn from_check_result(result: &CheckResult, timing: &Timing) -> Self {
        Self {
            schema_version: "1.0".to_string(),
            file: result.file.display().to_string(),
            success: result.errors.is_empty(),
            strict: result.strict,
            exit_code: result.exit_code(),
            errors: result.errors.iter().map(JsonError::from).collect(),
            warnings: result.warnings.iter().map(JsonWarning::from).collect(),
            verification: result.verification.as_ref().map(JsonVerification::from),
            timing: JsonTiming {
                parse_ms: timing.parse.as_millis() as u64,
                typecheck_ms: timing.typecheck.as_millis() as u64,
                total_ms: timing.total.as_millis() as u64,
            },
        }
    }
}

impl From<&TypeError> for JsonError {
    fn from(err: &TypeError) -> Self {
        JsonError {
            severity: "error".to_string(),
            code: err.code(),
            message: err.to_string(),
            location: JsonLocation {
                file: err.file().display().to_string(),
                line: err.line(),
                column: err.column(),
            },
            context: err.context(),
            help: err.help(),
        }
    }
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-cli/tests/json_output_schema_test.rs`

```rust
//! Tests for JSON output schema compliance

use serde_json::Value;
use std::process::Command;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_json_schema_version_present() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() {}").unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ash", "--", "check", "--format", "json"])
        .arg(&workflow)
        .output()
        .unwrap();
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    assert!(json.get("schema_version").is_some());
    assert_eq!(json["schema_version"], "1.0");
}

#[test]
fn test_json_includes_all_required_fields() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() {}").unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ash", "--", "check", "--format", "json"])
        .arg(&workflow)
        .output()
        .unwrap();
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    // Required fields per SPEC-005
    assert!(json.get("file").is_some());
    assert!(json.get("success").is_some());
    assert!(json.get("strict").is_some());
    assert!(json.get("exit_code").is_some());
    assert!(json.get("timing").is_some());
}

#[test]
fn test_json_error_structure() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("bad.ash");
    fs::write(&workflow, r#"
        workflow test {
            let x = "string" + 42;
        }
    "#).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ash", "--", "check", "--format", "json"])
        .arg(&workflow)
        .output()
        .unwrap();
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    assert!(!json["success"].as_bool().unwrap());
    assert!(json["errors"].is_array());
    
    let error = &json["errors"][0];
    assert!(error.get("severity").is_some());
    assert!(error.get("code").is_some());
    assert!(error.get("message").is_some());
    assert!(error.get("location").is_some());
}

#[test]
fn test_json_location_structure() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("bad.ash");
    fs::write(&workflow, r#"
        workflow test {
            let x = "string" + 42;
        }
    "#).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ash", "--", "check", "--format", "json"])
        .arg(&workflow)
        .output()
        .unwrap();
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    let location = &json["errors"][0]["location"];
    assert!(location.get("file").is_some());
    assert!(location.get("line").is_some());
    assert!(location.get("column").is_some());
}

#[test]
fn test_json_timing_structure() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() {}").unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ash", "--", "check", "--format", "json"])
        .arg(&workflow)
        .output()
        .unwrap();
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    let timing = &json["timing"];
    assert!(timing.get("parse_ms").is_some());
    assert!(timing.get("typecheck_ms").is_some());
    assert!(timing.get("total_ms").is_some());
}

#[test]
fn test_json_warnings_present() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("warn.ash");
    // Create workflow that generates warnings
    fs::write(&workflow, r#"
        workflow test {
            let unused = 42;
        }
    "#).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ash", "--", "check", "--format", "json", "--strict"])
        .arg(&workflow)
        .output()
        .unwrap();
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    assert!(json.get("warnings").is_some());
}
```

### Step 2: Implement JSON Output Module

**File:** `crates/ash-cli/src/output/json.rs`

```rust
use serde::Serialize;
use ash_typeck::TypeError;
use std::path::Path;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub struct JsonOutput {
    pub schema_version: &'static str,
    pub file: String,
    pub success: bool,
    pub strict: bool,
    pub exit_code: u8,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<JsonDiagnostic>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<JsonDiagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<JsonVerification>,
    pub timing: JsonTiming,
}

#[derive(Serialize)]
pub struct JsonDiagnostic {
    pub severity: &'static str,
    pub code: String,
    pub message: String,
    pub location: JsonLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

#[derive(Serialize)]
pub struct JsonLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Serialize)]
pub struct JsonVerification {
    pub obligations: usize,
    pub satisfied: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pending: Vec<String>,
}

#[derive(Serialize)]
pub struct JsonTiming {
    pub parse_ms: u64,
    pub typecheck_ms: u64,
    pub total_ms: u64,
}

impl JsonOutput {
    pub fn new(file: &Path) -> Self {
        Self {
            schema_version: "1.0",
            file: file.display().to_string(),
            success: true,
            strict: false,
            exit_code: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
            verification: None,
            timing: JsonTiming {
                parse_ms: 0,
                typecheck_ms: 0,
                total_ms: 0,
            },
        }
    }
    
    pub fn with_error(mut self, error: &dyn Diagnostic) -> Self {
        self.success = false;
        self.errors.push(JsonDiagnostic::from_diagnostic(error, "error"));
        self
    }
    
    pub fn with_warning(mut self, warning: &dyn Diagnostic) -> Self {
        self.warnings.push(JsonDiagnostic::from_diagnostic(warning, "warning"));
        self
    }
}
```

### Step 3: Update Check Command

**File:** `crates/ash-cli/src/commands/check.rs`

```rust
use crate::output::json::JsonOutput;
use std::time::Instant;

pub fn execute(args: CheckArgs) -> Result<ExitCode, CliError> {
    let start = Instant::now();
    
    let parse_start = Instant::now();
    let source = fs::read_to_string(&args.file)?;
    let workflow = parse(&source)?;
    let parse_time = parse_start.elapsed();
    
    let tc_start = Instant::now();
    let mut checker = TypeChecker::new();
    let result = checker.type_check_workflow(&workflow);
    let tc_time = tc_start.elapsed();
    
    let total_time = start.elapsed();
    
    match args.format {
        OutputFormat::Text => output_text(&result, &args),
        OutputFormat::Json => {
            let json = JsonOutput::new(&args.file)
                .with_timing(parse_time, tc_time, total_time);
            
            let json = match &result {
                Ok(tc_result) => {
                    json.with_verification(&tc_result.verification)
                }
                Err(errors) => {
                    errors.iter().fold(json, |j, e| j.with_error(e))
                }
            };
            
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
    }
    
    Ok(ExitCode::from(if result.is_ok() { 0 } else { 3 }))
}
```

## Verification Steps

- [ ] `cargo test -p ash-cli --test json_output_schema_test` passes
- [ ] JSON output validates against SPEC-005 schema
- [ ] `cargo test -p ash-cli` all tests pass
- [ ] Manual verification of JSON structure
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- SPEC-005 compliant JSON output
- Rich error/warning structure
- Timing information

Required by:
- Tooling integration (IDEs, CI/CD)

## Notes

**Tooling Impact**: Current lossy JSON breaks IDE integration and automated tooling that expects structured errors.

**Design Decision**: Use serde for clean, maintainable JSON serialization with conditional field skipping.

**Edge Cases**:
- Large error lists - streaming output if needed
- Missing source locations - use defaults
- Unicode in paths - proper encoding
