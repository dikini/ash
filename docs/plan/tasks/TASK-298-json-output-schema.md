# TASK-298: Fix JSON Output Schema for ash check

## Status: 📝 Planned

## Description

Fix the `ash check --format json` command which does not match the documented tooling schema. The implementation emits only `{ file, success, strict, error }` instead of diagnostics with severity, location, and counts, and it does not expose warnings as required by SPEC-005/SPEC-021.

## Specification Reference

- SPEC-005: CLI Specification
- SPEC-021: Observable Behavior Specification

## Dependencies

- ✅ TASK-053: CLI check command
- ✅ TASK-025: Type errors

## Critical File Locations

- `crates/ash-cli/src/commands/check.rs:132` - JSON output implementation

## Requirements

### Functional Requirements

1. JSON output must match SPEC-005 schema
2. Diagnostics must include severity, location, and message
3. Warnings must be included in output
4. Summary counts must be provided
5. SPEC-021 observable behavior contract must be met

### Current State (Broken)

**File:** `crates/ash-cli/src/commands/check.rs:132`

```rust
fn output_json(result: &CheckResult) -> String {
    // WRONG: Minimal output missing required fields
    serde_json::to_string(&serde_json::json!({
        "file": result.file,
        "success": result.success,
        "strict": result.strict,
        "error": result.error_message(),  // Only one error!
    })).unwrap()
}
```

Problems:
1. No diagnostics array
2. No severity levels
3. No location information
4. Only one error shown
5. Warnings not included
6. SPEC-005 schema not followed

### Target State (Fixed)

```rust
// SPEC-005 compliant JSON schema

#[derive(Serialize)]
struct CheckOutput {
    /// File that was checked
    file: PathBuf,
    
    /// Whether checking succeeded
    success: bool,
    
    /// Whether in strict mode
    strict: bool,
    
    /// Array of all diagnostics
    diagnostics: Vec<Diagnostic>,
    
    /// Summary counts
    summary: Summary,
}

#[derive(Serialize)]
struct Diagnostic {
    /// Severity level
    severity: Severity,
    
    /// Error/warning code
    code: Option<String>,
    
    /// Human-readable message
    message: String,
    
    /// Source location
    location: Location,
    
    /// Suggested fix (if available)
    suggestion: Option<Suggestion>,
}

#[derive(Serialize)]
enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Serialize)]
struct Location {
    /// Path to file
    file: PathBuf,
    
    /// Line number (1-indexed)
    line: usize,
    
    /// Column number (1-indexed)
    column: usize,
    
    /// Byte offset from start of file
    offset: usize,
    
    /// Length of the problematic span
    length: usize,
}

#[derive(Serialize)]
struct Suggestion {
    /// Description of the fix
    message: String,
    
    /// Replacement text
    replacement: String,
}

#[derive(Serialize)]
struct Summary {
    /// Total number of errors
    error_count: usize,
    
    /// Total number of warnings
    warning_count: usize,
    
    /// Total number of info messages
    info_count: usize,
    
    /// Total number of diagnostics
    total_count: usize,
}

fn output_json(result: &CheckResult) -> String {
    let output = CheckOutput {
        file: result.file.clone(),
        success: result.success(),
        strict: result.strict,
        diagnostics: result.diagnostics.iter().map(|d| Diagnostic {
            severity: match d.severity {
                DiagnosticSeverity::Error => Severity::Error,
                DiagnosticSeverity::Warning => Severity::Warning,
                DiagnosticSeverity::Info => Severity::Info,
            },
            code: d.code.clone(),
            message: d.message.clone(),
            location: Location {
                file: d.location.file.clone(),
                line: d.location.line,
                column: d.location.column,
                offset: d.location.offset,
                length: d.location.length,
            },
            suggestion: d.suggestion.as_ref().map(|s| Suggestion {
                message: s.message.clone(),
                replacement: s.replacement.clone(),
            }),
        }).collect(),
        summary: Summary {
            error_count: result.error_count(),
            warning_count: result.warning_count(),
            info_count: result.info_count(),
            total_count: result.diagnostics.len(),
        },
    };
    
    serde_json::to_string_pretty(&output).unwrap()
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-cli/tests/check_json_output_test.rs`

```rust
//! Tests for ash check --format json output

use std::process::Command;
use std::fs;
use tempfile::TempDir;
use serde_json::Value;

#[test]
fn test_json_output_has_diagnostics_array() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow test {
            let x: Int = "string";
        }
    "#;
    
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "--format", "json", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    // Must have diagnostics array
    assert!(json.get("diagnostics").is_some());
    assert!(json["diagnostics"].is_array());
}

#[test]
fn test_json_output_includes_severity() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow test {
            let x: Int = "string";
        }
    "#;
    
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "--format", "json", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    // Each diagnostic must have severity
    for diagnostic in json["diagnostics"].as_array().unwrap() {
        assert!(diagnostic.get("severity").is_some());
        let severity = diagnostic["severity"].as_str().unwrap();
        assert!(matches!(severity, "Error" | "Warning" | "Info"));
    }
}

#[test]
fn test_json_output_includes_location() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow test {
            let x: Int = "string";
        }
    "#;
    
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "--format", "json", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    for diagnostic in json["diagnostics"].as_array().unwrap() {
        assert!(diagnostic.get("location").is_some());
        let location = &diagnostic["location"];
        assert!(location.get("file").is_some());
        assert!(location.get("line").is_some());
        assert!(location.get("column").is_some());
    }
}

#[test]
fn test_json_output_includes_summary() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow test {
            let x: Int = "string";
        }
    "#;
    
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "--format", "json", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    assert!(json.get("summary").is_some());
    let summary = &json["summary"];
    assert!(summary.get("error_count").is_some());
    assert!(summary.get("warning_count").is_some());
    assert!(summary.get("total_count").is_some());
}

#[test]
fn test_json_output_includes_warnings() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow test {
            let unused = 42;
            act print("hello");
        }
    "#;
    
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "--format", "json", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    // Should have at least one warning
    let warnings: Vec<_> = json["diagnostics"].as_array().unwrap()
        .iter()
        .filter(|d| d["severity"] == "Warning")
        .collect();
    
    assert!(!warnings.is_empty(), "Expected warnings for unused variable");
}

#[test]
fn test_json_output_multiple_errors() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow test {
            let x: Int = "string";
            let y: Bool = 42;
        }
    "#;
    
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "--format", "json", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    let errors = json["diagnostics"].as_array().unwrap();
    assert!(errors.len() >= 2, "Expected multiple errors");
}

#[test]
fn test_json_output_success_true_when_no_errors() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow test {
            act print("hello");
        }
    "#;
    
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "--format", "json", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    assert_eq!(json["success"], true);
}

#[test]
fn test_json_output_success_false_when_errors() {
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"
        workflow test {
            let x: Int = "error";
        }
    "#;
    
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "--format", "json", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    assert_eq!(json["success"], false);
}

#[test]
fn test_json_schema_matches_spec() {
    // Verify output matches SPEC-005 schema
    let temp = TempDir::new().unwrap();
    
    let workflow = r#"workflow test { act print("hello"); }"#;
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "check", "--format", "json", path.to_str().unwrap()])
        .current_dir("/home/dikini/Projects/ash")
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();
    
    // Required fields per SPEC-005
    assert!(json.get("file").is_some());
    assert!(json.get("success").is_some());
    assert!(json.get("strict").is_some());
    assert!(json.get("diagnostics").is_some());
    assert!(json.get("summary").is_some());
}
```

### Step 2: Define JSON Schema Types

**File:** `crates/ash-cli/src/output.rs`

```rust
//! JSON output schema for CLI commands

use serde::Serialize;
use std::path::PathBuf;

/// SPEC-005 compliant check output
#[derive(Serialize)]
pub struct CheckOutput {
    pub file: PathBuf,
    pub success: bool,
    pub strict: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub summary: Summary,
}

#[derive(Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<String>,
    pub message: String,
    pub location: Location,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<Suggestion>,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Serialize)]
pub struct Location {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub length: usize,
}

#[derive(Serialize)]
pub struct Suggestion {
    pub message: String,
    pub replacement: String,
}

#[derive(Serialize)]
pub struct Summary {
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub total_count: usize,
}
```

### Step 3: Update Check Command

**File:** `crates/ash-cli/src/commands/check.rs`

```rust
use crate::output::{CheckOutput, Diagnostic, Severity, Location, Summary};
use ash_engine::{Diagnostic as EngineDiagnostic, DiagnosticSeverity};

fn format_json_output(result: &CheckResult) -> String {
    let output = CheckOutput {
        file: result.file.clone(),
        success: result.success(),
        strict: result.strict,
        diagnostics: result.diagnostics.iter().map(convert_diagnostic).collect(),
        summary: Summary {
            error_count: result.diagnostics.iter()
                .filter(|d| matches!(d.severity, DiagnosticSeverity::Error))
                .count(),
            warning_count: result.diagnostics.iter()
                .filter(|d| matches!(d.severity, DiagnosticSeverity::Warning))
                .count(),
            info_count: result.diagnostics.iter()
                .filter(|d| matches!(d.severity, DiagnosticSeverity::Info))
                .count(),
            total_count: result.diagnostics.len(),
        },
    };
    
    serde_json::to_string_pretty(&output).unwrap()
}

fn convert_diagnostic(d: &EngineDiagnostic) -> Diagnostic {
    Diagnostic {
        severity: match d.severity {
            DiagnosticSeverity::Error => Severity::Error,
            DiagnosticSeverity::Warning => Severity::Warning,
            DiagnosticSeverity::Info => Severity::Info,
        },
        code: d.code.clone(),
        message: d.message.clone(),
        location: Location {
            file: d.location.file.clone(),
            line: d.location.line,
            column: d.location.column,
            offset: d.location.offset,
            length: d.location.length,
        },
        suggestion: d.suggestion.as_ref().map(|s| crate::output::Suggestion {
            message: s.message.clone(),
            replacement: s.replacement.clone(),
        }),
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-cli --test check_json_output_test` passes
- [ ] JSON output has all required fields
- [ ] Diagnostics include severity and location
- [ ] Warnings appear in output
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- SPEC-005 compliant JSON output
- Full diagnostic information

Required by:
- IDE integration
- CI/CD tooling
- Editor plugins

## Notes

**Critical Issue**: JSON output doesn't match specification.

**Risk Assessment**: Medium - affects tooling integration.

**Implementation Strategy**:
1. First: Define schema types
2. Second: Convert engine diagnostics
3. Third: Update output formatting
4. Fourth: Verify against spec
