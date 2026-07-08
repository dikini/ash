//! Tests for CLI value conversion functionality

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

    let items = value
        .list_to_vec()
        .unwrap_or_else(|| panic!("Expected List, got {:?}", value));
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], Value::Int(1));
    assert_eq!(items[1], Value::Int(2));
    assert_eq!(items[2], Value::Int(3));
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

/// Integration test: Verify that the CLI can execute a simple entry source.
#[test]
fn test_run_simple_entry_source() {
    let temp = TempDir::new().unwrap();

    // Create a simple canonical entry source.
    let entry_source = r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
    "#;
    let entry_path = temp.path().join("simple.ash");
    fs::write(&entry_path, entry_source).unwrap();

    // Run the entry source.
    let output = Command::new(env!("CARGO_BIN_EXE_ash"))
        .args(["run"])
        .arg(&entry_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8(output.stdout).unwrap();
    // The entry source should execute successfully without emitting a final value.
    assert!(
        output.status.success() && stdout.is_empty(),
        "Entry source should execute successfully with empty stdout. stdout: {}, stderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}
