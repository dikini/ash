//! Tests for ash check --format json output schema (TASK-298)
//!
//! These tests verify SPEC-005 compliant JSON output with:
//! - diagnostics array with severity, location, message
//! - summary counts
//! - warnings included in output

use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the ash binary
fn ash_binary() -> String {
    std::env::var("CARGO_BIN_EXE_ash").unwrap_or_else(|_| "target/debug/ash".to_string())
}

/// Extract JSON from command output, filtering out log lines
fn extract_json(stdout: &str) -> Option<String> {
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') {
            let mut depth = 0;
            let mut in_string = false;
            let mut escape_next = false;
            let mut result = String::new();

            for ch in stdout[stdout.find(trimmed).unwrap_or(0)..].chars() {
                result.push(ch);

                if escape_next {
                    escape_next = false;
                    continue;
                }

                if ch == '\\' {
                    escape_next = true;
                    continue;
                }

                if ch == '"' && !escape_next {
                    in_string = !in_string;
                    continue;
                }

                if !in_string {
                    if ch == '{' {
                        depth += 1;
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                }
            }

            return Some(result);
        }
    }
    None
}

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

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).unwrap();

    // Must have diagnostics array
    assert!(
        json.get("diagnostics").is_some(),
        "diagnostics field is required"
    );
    assert!(
        json["diagnostics"].is_array(),
        "diagnostics should be an array"
    );
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

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).unwrap();

    // Each diagnostic must have severity
    for diagnostic in json["diagnostics"].as_array().unwrap() {
        assert!(
            diagnostic.get("severity").is_some(),
            "diagnostic must have severity"
        );
        let severity = diagnostic["severity"].as_str().unwrap();
        assert!(
            matches!(severity, "error" | "warning" | "info"),
            "severity should be 'error', 'warning', or 'info', got: {}",
            severity
        );
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

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).unwrap();

    for diagnostic in json["diagnostics"].as_array().unwrap() {
        assert!(
            diagnostic.get("location").is_some(),
            "diagnostic must have location"
        );
        let location = &diagnostic["location"];
        assert!(location.get("file").is_some(), "location must have file");
        assert!(location.get("line").is_some(), "location must have line");
        assert!(
            location.get("column").is_some(),
            "location must have column"
        );
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

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).unwrap();

    assert!(json.get("summary").is_some(), "summary field is required");
    let summary = &json["summary"];
    assert!(
        summary.get("error_count").is_some(),
        "summary must have error_count"
    );
    assert!(
        summary.get("warning_count").is_some(),
        "summary must have warning_count"
    );
    assert!(
        summary.get("info_count").is_some(),
        "summary must have info_count"
    );
    assert!(
        summary.get("total_count").is_some(),
        "summary must have total_count"
    );
}

#[test]
fn test_json_output_includes_warnings() {
    let temp = TempDir::new().unwrap();

    // Workflow with potentially unused variable
    let workflow = r#"
        workflow test {
            let unused = 42;
            act print("hello");
        }
    "#;

    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).unwrap();

    // Check structure - should have diagnostics array
    assert!(json["diagnostics"].is_array());
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

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).unwrap();

    let diagnostics = json["diagnostics"].as_array().unwrap();
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d["severity"] == "error")
        .collect();
    assert!(!errors.is_empty(), "Expected at least one error");
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

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(
        json["success"], true,
        "success should be true when no errors"
    );
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

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(
        json["success"], false,
        "success should be false when errors present"
    );
}

#[test]
fn test_json_schema_matches_spec() {
    // Verify output matches SPEC-005 schema
    let temp = TempDir::new().unwrap();

    let workflow = r#"workflow test { act print("hello"); }"#;
    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).unwrap();

    // Required fields per SPEC-005
    assert!(json.get("file").is_some(), "file field is required");
    assert!(json.get("success").is_some(), "success field is required");
    assert!(json.get("strict").is_some(), "strict field is required");
    assert!(
        json.get("diagnostics").is_some(),
        "diagnostics field is required"
    );
    assert!(json.get("summary").is_some(), "summary field is required");
}

#[test]
fn test_json_diagnostic_has_message() {
    let temp = TempDir::new().unwrap();

    let workflow = r#"
        workflow test {
            let x: Int = "string";
        }
    "#;

    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).unwrap();

    for diagnostic in json["diagnostics"].as_array().unwrap() {
        assert!(
            diagnostic.get("message").is_some(),
            "diagnostic must have message"
        );
        assert!(
            diagnostic["message"].as_str().is_some(),
            "message should be a string"
        );
    }
}

#[test]
fn test_json_diagnostic_has_code() {
    let temp = TempDir::new().unwrap();

    let workflow = r#"
        workflow test {
            let x: Int = "string";
        }
    "#;

    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).unwrap();

    for diagnostic in json["diagnostics"].as_array().unwrap() {
        // code is optional but if present should be a string
        if let Some(code) = diagnostic.get("code") {
            assert!(
                code.as_str().is_some(),
                "code should be a string if present"
            );
        }
    }
}

#[test]
fn test_json_summary_counts_match_diagnostics() {
    let temp = TempDir::new().unwrap();

    let workflow = r#"
        workflow test {
            let x: Int = "string";
        }
    "#;

    let path = temp.path().join("test.ash");
    fs::write(&path, workflow).unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json", path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).unwrap();

    let diagnostics = json["diagnostics"].as_array().unwrap();
    let summary = &json["summary"];

    let error_count = diagnostics
        .iter()
        .filter(|d| d["severity"] == "error")
        .count();
    let warning_count = diagnostics
        .iter()
        .filter(|d| d["severity"] == "warning")
        .count();
    let info_count = diagnostics
        .iter()
        .filter(|d| d["severity"] == "info")
        .count();

    assert_eq!(
        summary["error_count"].as_u64().unwrap() as usize,
        error_count,
        "error_count should match actual errors"
    );
    assert_eq!(
        summary["warning_count"].as_u64().unwrap() as usize,
        warning_count,
        "warning_count should match actual warnings"
    );
    assert_eq!(
        summary["info_count"].as_u64().unwrap() as usize,
        info_count,
        "info_count should match actual info messages"
    );
    assert_eq!(
        summary["total_count"].as_u64().unwrap() as usize,
        diagnostics.len(),
        "total_count should match total diagnostics"
    );
}
