//! Tests for JSON output schema compliance (TASK-280)
//!
//! These tests verify that the `ash check --format json` command
//! produces output matching the SPEC-005 JSON schema.

use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the ash binary
fn ash_binary() -> String {
    // Use CARGO_BIN_EXE_ash if available (set by cargo test)
    std::env::var("CARGO_BIN_EXE_ash").unwrap_or_else(|_| "target/debug/ash".to_string())
}

/// Extract JSON from command output, filtering out log lines
fn extract_json(stdout: &str) -> Option<String> {
    // Find the first line that starts with '{' - that's the JSON output
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') {
            // Found the start of JSON, collect all lines until we close all braces
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
fn test_json_schema_version_present() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() {}").unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json"])
        .arg(&workflow)
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON output");

    assert!(
        json.get("schema_version").is_some(),
        "schema_version field is required"
    );
    assert_eq!(
        json["schema_version"], "1.0",
        "schema_version should be 1.0"
    );
}

#[test]
fn test_json_includes_all_required_fields() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() {}").unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json"])
        .arg(&workflow)
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON output");

    // Required fields per SPEC-005
    assert!(json.get("file").is_some(), "file field is required");
    assert!(json.get("success").is_some(), "success field is required");
    assert!(json.get("strict").is_some(), "strict field is required");
    assert!(
        json.get("exit_code").is_some(),
        "exit_code field is required"
    );
    assert!(json.get("timing").is_some(), "timing field is required");
}

#[test]
fn test_json_error_structure() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("bad.ash");
    fs::write(
        &workflow,
        r#"
        workflow test {
            let x = "string" + 42;
        }
    "#,
    )
    .unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json"])
        .arg(&workflow)
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON output");

    // Should have success = false when there are errors
    assert!(
        !json["success"].as_bool().unwrap_or(true),
        "success should be false when there are errors"
    );

    // Should have diagnostics array (SPEC-005 compliant)
    assert!(
        json.get("diagnostics").is_some(),
        "diagnostics field should be present when there are errors"
    );
    assert!(
        json["diagnostics"].is_array(),
        "diagnostics should be an array"
    );

    // Check diagnostic structure if there are errors
    if let Some(diagnostics) = json["diagnostics"].as_array() {
        if !diagnostics.is_empty() {
            let error = &diagnostics[0];
            assert!(
                error.get("severity").is_some(),
                "error should have severity field"
            );
            assert!(error.get("code").is_some(), "error should have code field");
            assert!(
                error.get("message").is_some(),
                "error should have message field"
            );
            assert!(
                error.get("location").is_some(),
                "error should have location field"
            );
        }
    }
}

#[test]
fn test_json_location_structure() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("bad.ash");
    fs::write(
        &workflow,
        r#"
        workflow test {
            let x = "string" + 42;
        }
    "#,
    )
    .unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json"])
        .arg(&workflow)
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON output");

    // Check location structure if there are errors
    if let Some(diagnostics) = json["diagnostics"].as_array() {
        if !diagnostics.is_empty() {
            let location = &diagnostics[0]["location"];
            assert!(
                location.get("file").is_some(),
                "location should have file field"
            );
            assert!(
                location.get("line").is_some(),
                "location should have line field"
            );
            assert!(
                location.get("column").is_some(),
                "location should have column field"
            );
        }
    }
}

#[test]
fn test_json_timing_structure() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() {}").unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json"])
        .arg(&workflow)
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON output");

    assert!(json.get("timing").is_some(), "timing field is required");

    let timing = &json["timing"];
    assert!(
        timing.get("parse_ms").is_some(),
        "timing should have parse_ms field"
    );
    assert!(
        timing.get("typecheck_ms").is_some(),
        "timing should have typecheck_ms field"
    );
    assert!(
        timing.get("total_ms").is_some(),
        "timing should have total_ms field"
    );

    // Verify they are numbers
    assert!(
        timing["parse_ms"].is_number(),
        "parse_ms should be a number"
    );
    assert!(
        timing["typecheck_ms"].is_number(),
        "typecheck_ms should be a number"
    );
    assert!(
        timing["total_ms"].is_number(),
        "total_ms should be a number"
    );
}

#[test]
fn test_json_warnings_field_exists() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() {}").unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json"])
        .arg(&workflow)
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON output");

    // Warnings field may be skipped when empty due to skip_serializing_if
    // But if present, it should be an array
    if let Some(warnings) = json.get("warnings") {
        assert!(warnings.is_array(), "warnings should be an array");
    }
}

#[test]
fn test_json_strict_mode_reflected() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() {}").unwrap();

    // Test with --strict flag
    let output = Command::new(ash_binary())
        .args(["check", "--format", "json", "--strict"])
        .arg(&workflow)
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON output");

    assert!(
        json["strict"].as_bool().unwrap_or(false),
        "strict should be true when --strict flag is passed"
    );
}

#[test]
fn test_json_success_true_for_valid_workflow() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("valid.ash");
    fs::write(&workflow, "workflow test() {}").unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json"])
        .arg(&workflow)
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON output");

    assert!(
        json["success"].as_bool().unwrap_or(false),
        "success should be true for valid workflow"
    );
    assert_eq!(
        json["exit_code"], 0,
        "exit_code should be 0 for valid workflow"
    );
}

#[test]
fn test_json_file_path_present() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("my_workflow.ash");
    fs::write(&workflow, "workflow test() {}").unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json"])
        .arg(&workflow)
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON output");

    let file_path = json["file"].as_str().expect("file should be a string");
    assert!(
        file_path.contains("my_workflow.ash"),
        "file path should contain the workflow filename"
    );
}

#[test]
fn test_json_verification_field_structure() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() {}").unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json"])
        .arg(&workflow)
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON output");

    // Verification is optional, but if present should have correct structure
    if let Some(verification) = json.get("verification") {
        assert!(
            verification.get("obligations").is_some(),
            "verification should have obligations field"
        );
        assert!(
            verification.get("satisfied").is_some(),
            "verification should have satisfied field"
        );
        assert!(
            verification.get("pending").is_some(),
            "verification should have pending field"
        );
    }
}

#[test]
fn test_json_errors_array_when_parse_error() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("parse_error.ash");
    // Invalid syntax - missing closing brace
    fs::write(&workflow, "workflow test { { let x = 1;").unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json"])
        .arg(&workflow)
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let json_str = extract_json(&stdout).expect("No JSON found in output");
    let json: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON output");

    assert!(
        !json["success"].as_bool().unwrap_or(true),
        "success should be false for parse error"
    );

    // Should have diagnostics array with at least one error (SPEC-005 compliant)
    if let Some(diagnostics) = json["diagnostics"].as_array() {
        assert!(
            !diagnostics.is_empty(),
            "diagnostics array should not be empty for parse error"
        );
    } else {
        panic!("diagnostics field should be present for parse error");
    }
}

/// Property-based test: JSON should always be valid
#[test]
fn test_json_always_valid_for_any_input() {
    let temp = TempDir::new().unwrap();

    // Test various inputs
    let inputs = [
        "workflow test() {}",
        "workflow test { let x = 1; }",
        "invalid syntax here {{",
        "",
        "workflow",
    ];

    for (i, input) in inputs.iter().enumerate() {
        let workflow = temp.path().join(format!("test_{}.ash", i));
        fs::write(&workflow, input).unwrap();

        let output = Command::new(ash_binary())
            .args(["check", "--format", "json"])
            .arg(&workflow)
            .output()
            .expect("Failed to execute ash check command");

        let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");

        // Extract and validate JSON
        let json_str = extract_json(&stdout).expect("No JSON found in output");
        let result: Result<Value, _> = serde_json::from_str(&json_str);
        assert!(
            result.is_ok(),
            "Output should always be valid JSON, got: {}",
            json_str
        );
    }
}

/// Test that snake_case is used for all field names
#[test]
fn test_json_uses_snake_case() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() {}").unwrap();

    let output = Command::new(ash_binary())
        .args(["check", "--format", "json"])
        .arg(&workflow)
        .output()
        .expect("Failed to execute ash check command");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let json_str = extract_json(&stdout).expect("No JSON found in output");

    // Should not contain camelCase field names
    assert!(
        !json_str.contains("schemaVersion"),
        "Should use snake_case, not camelCase"
    );
    assert!(
        !json_str.contains("exitCode"),
        "Should use snake_case, not camelCase"
    );
    assert!(
        !json_str.contains("parseMs"),
        "Should use snake_case, not camelCase"
    );
}
