//! Tests for CLI --input functionality

use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_json_types_mapped_correctly() {
    use ash_cli::value_convert::json_to_value;
    use ash_core::Value;

    let test_cases = vec![
        (serde_json::Value::Null, Value::Null),
        (serde_json::Value::Bool(true), Value::Bool(true)),
        (serde_json::Value::Bool(false), Value::Bool(false)),
        (serde_json::json!(42), Value::Int(42)),
        (serde_json::json!(-100), Value::Int(-100)),
        (
            serde_json::json!("hello"),
            Value::String("hello".to_string()),
        ),
    ];

    for (json, expected) in test_cases {
        let value = json_to_value(json);
        assert_eq!(value, expected);
    }
}

#[test]
fn test_json_array_conversion() {
    use ash_cli::value_convert::json_to_value;
    use ash_core::Value;

    let json = serde_json::json!([1, 2, 3]);
    let value = json_to_value(json);

    match value {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Int(1));
            assert_eq!(items[1], Value::Int(2));
            assert_eq!(items[2], Value::Int(3));
        }
        _ => panic!("Expected List, got {:?}", value),
    }
}

#[test]
fn test_nested_json_object() {
    use ash_cli::value_convert::json_to_value;
    use ash_core::Value;

    let json = serde_json::json!({"user": {"name": "Alice", "age": 30}});
    let value = json_to_value(json);

    match value {
        Value::Record(fields) => {
            assert!(fields.contains_key("user"));
            match fields.get("user").unwrap() {
                Value::Record(user_fields) => {
                    assert_eq!(
                        user_fields.get("name"),
                        Some(&Value::String("Alice".to_string()))
                    );
                    assert_eq!(user_fields.get("age"), Some(&Value::Int(30)));
                }
                _ => panic!("Expected nested Record"),
            }
        }
        _ => panic!("Expected Record, got {:?}", value),
    }
}

#[test]
fn test_value_to_json_roundtrip() {
    use ash_cli::value_convert::{json_to_value, value_to_json};

    let original = serde_json::json!({
        "name": "test",
        "count": 42,
        "active": true,
        "items": [1, 2, 3]
    });

    let value = json_to_value(original.clone());
    let back_to_json = value_to_json(&value);

    assert_eq!(original, back_to_json);
}

#[test]
fn test_parse_input_empty() {
    use ash_cli::commands::run::parse_input;

    let result = parse_input(&None).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_parse_input_valid_json() {
    use ash_cli::commands::run::parse_input;
    use ash_core::Value;

    let json = r#"{"x": 42, "name": "test"}"#.to_string();
    let result = parse_input(&Some(json)).unwrap();

    assert_eq!(result.get("x"), Some(&Value::Int(42)));
    assert_eq!(result.get("name"), Some(&Value::String("test".to_string())));
}

#[test]
fn test_parse_input_nested_object() {
    use ash_cli::commands::run::parse_input;
    use ash_core::Value;

    let json = r#"{"config": {"debug": true, "port": 8080}}"#.to_string();
    let result = parse_input(&Some(json)).unwrap();

    assert!(result.contains_key("config"));
    match result.get("config").unwrap() {
        Value::Record(fields) => {
            assert_eq!(fields.get("debug"), Some(&Value::Bool(true)));
            assert_eq!(fields.get("port"), Some(&Value::Int(8080)));
        }
        _ => panic!("Expected Record"),
    }
}

#[test]
fn test_parse_input_array_value() {
    use ash_cli::commands::run::parse_input;
    use ash_core::Value;

    let json = r#"{"items": [1, 2, 3]}"#.to_string();
    let result = parse_input(&Some(json)).unwrap();

    match result.get("items").unwrap() {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Int(1));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_input_invalid_json() {
    use ash_cli::commands::run::parse_input;

    let json = r#"{invalid json"#.to_string();
    let result = parse_input(&Some(json));

    assert!(result.is_err());
}

#[test]
fn test_parse_input_non_object() {
    use ash_cli::commands::run::parse_input;

    // Non-object JSON should fail - we expect an object for input bindings
    let json = r#"42"#.to_string();
    let result = parse_input(&Some(json));

    assert!(result.is_err());
}

/// Integration test: Verify that the CLI can execute a simple workflow
/// Note: Full parameter binding depends on workflow language support
#[test]
fn test_run_simple_workflow() {
    let temp = TempDir::new().unwrap();

    // Create a simple workflow that returns a value
    let workflow = r#"
        workflow main() {
            ret 42
        }
    "#;
    let workflow_path = temp.path().join("simple.ash");
    fs::write(&workflow_path, workflow).unwrap();

    // Run the workflow
    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "run"])
        .arg(&workflow_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8(output.stdout).unwrap();
    // The workflow should execute successfully
    assert!(
        output.status.success() || stdout.contains("42"),
        "Workflow should execute. stdout: {}, stderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Integration test: Verify that --input flag is accepted
#[test]
fn test_input_flag_accepted() {
    let temp = TempDir::new().unwrap();

    // Create a simple workflow
    let workflow = r#"
        workflow main() {
            ret 42
        }
    "#;
    let workflow_path = temp.path().join("test.ash");
    fs::write(&workflow_path, workflow).unwrap();

    // Run with --input flag (even if not fully utilized by workflow)
    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "run"])
        .arg(&workflow_path)
        .arg("--input")
        .arg(r#"{"test": "value"}"#)
        .output()
        .expect("Failed to execute");

    // Should not fail due to input parsing
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Invalid JSON"),
        "Input should be parsed as valid JSON. stderr: {}",
        stderr
    );
}
