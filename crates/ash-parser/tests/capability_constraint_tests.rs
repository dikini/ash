//! Tests for parsing `capabilities: [...]` with `@` constraints (TASK-260)
//!
//! These tests verify parsing of capability declarations in workflow headers
//! according to SPEC-024 Section 3:
//!
//! ```text
//! capability-decl ::= "capabilities" ":" "[" capability-ref* "]"
//! capability-ref ::= ident constraint-refinement?
//! constraint-refinement ::= "@" "{" field-value* "}"
//! field-value ::= ident ":" constraint-value
//! constraint-value ::= bool | int | string | array | object
//! ```
//!
//! Example:
//! ```ash
//! capabilities: [
//!     file @ { paths: ["/tmp/*"], read: true },
//!     network @ { hosts: ["*.example.com"], ports: [443] }
//! ]
//! ```

use ash_parser::input::new_input;
use ash_parser::parse_workflow::workflow_def;
use ash_parser::surface::ConstraintValue;
use winnow::prelude::*;

// ============================================================================
// Success Cases - Bare Capabilities
// ============================================================================

#[test]
fn test_parse_bare_capability_without_constraints() {
    let input = r#"workflow test capabilities: [file] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let workflow = result.unwrap();
    assert_eq!(workflow.name.as_ref(), "test");
    assert_eq!(workflow.capabilities.len(), 1);
    assert_eq!(workflow.capabilities[0].capability.as_ref(), "file");
    assert!(workflow.capabilities[0].constraints.is_none());
}

#[test]
fn test_parse_multiple_bare_capabilities() {
    let input = r#"workflow test capabilities: [file, network, database] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let workflow = result.unwrap();
    assert_eq!(workflow.capabilities.len(), 3);
    assert_eq!(workflow.capabilities[0].capability.as_ref(), "file");
    assert_eq!(workflow.capabilities[1].capability.as_ref(), "network");
    assert_eq!(workflow.capabilities[2].capability.as_ref(), "database");
}

#[test]
fn test_parse_empty_capabilities() {
    let input = r#"workflow test capabilities: [] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let workflow = result.unwrap();
    assert!(workflow.capabilities.is_empty());
}

// ============================================================================
// Success Cases - Capabilities with Constraints
// ============================================================================

#[test]
fn test_parse_capability_with_single_constraint() {
    let input = r#"workflow test capabilities: [file @ { paths: ["/tmp/*"] }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let workflow = result.unwrap();
    assert_eq!(workflow.capabilities.len(), 1);
    assert_eq!(workflow.capabilities[0].capability.as_ref(), "file");

    let constraints = workflow.capabilities[0].constraints.as_ref().unwrap();
    assert_eq!(constraints.fields.len(), 1);
    assert_eq!(constraints.fields[0].name.as_ref(), "paths");
}

#[test]
fn test_parse_capability_with_multiple_constraints() {
    let input =
        r#"workflow test capabilities: [file @ { paths: ["/tmp/*"], read: true }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let workflow = result.unwrap();
    let constraints = workflow.capabilities[0].constraints.as_ref().unwrap();
    assert_eq!(constraints.fields.len(), 2);
    assert_eq!(constraints.fields[0].name.as_ref(), "paths");
    assert_eq!(constraints.fields[1].name.as_ref(), "read");
}

#[test]
fn test_parse_multiple_capabilities_with_mixed_constraints() {
    let input =
        r#"workflow test capabilities: [file, network @ { hosts: ["*.example.com"] }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let workflow = result.unwrap();
    assert_eq!(workflow.capabilities.len(), 2);
    assert_eq!(workflow.capabilities[0].capability.as_ref(), "file");
    assert!(workflow.capabilities[0].constraints.is_none());
    assert_eq!(workflow.capabilities[1].capability.as_ref(), "network");
    assert!(workflow.capabilities[1].constraints.is_some());
}

// ============================================================================
// Success Cases - All Constraint Value Types
// ============================================================================

#[test]
fn test_parse_constraint_bool_true() {
    let input = r#"workflow test capabilities: [file @ { read: true }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    let constraints = result.capabilities[0].constraints.as_ref().unwrap();
    assert!(matches!(
        constraints.fields[0].value,
        ConstraintValue::Bool(true)
    ));
}

#[test]
fn test_parse_constraint_bool_false() {
    let input = r#"workflow test capabilities: [file @ { write: false }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    let constraints = result.capabilities[0].constraints.as_ref().unwrap();
    assert!(matches!(
        constraints.fields[0].value,
        ConstraintValue::Bool(false)
    ));
}

#[test]
fn test_parse_constraint_int_positive() {
    let input = r#"workflow test capabilities: [network @ { port: 443 }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    let constraints = result.capabilities[0].constraints.as_ref().unwrap();
    match &constraints.fields[0].value {
        ConstraintValue::Int(n) => assert_eq!(*n, 443),
        _ => panic!("Expected Int constraint value"),
    }
}

#[test]
fn test_parse_constraint_int_negative() {
    let input = r#"workflow test capabilities: [sensor @ { offset: -10 }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    let constraints = result.capabilities[0].constraints.as_ref().unwrap();
    match &constraints.fields[0].value {
        ConstraintValue::Int(n) => assert_eq!(*n, -10),
        _ => panic!("Expected Int constraint value"),
    }
}

#[test]
fn test_parse_constraint_string() {
    let input = r#"workflow test capabilities: [file @ { path: "/tmp/data" }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    let constraints = result.capabilities[0].constraints.as_ref().unwrap();
    match &constraints.fields[0].value {
        ConstraintValue::String(s) => assert_eq!(s, "/tmp/data"),
        _ => panic!(
            "Expected String constraint value, got {:?}",
            constraints.fields[0].value
        ),
    }
}

#[test]
fn test_parse_constraint_string_with_special_chars() {
    let input = r#"workflow test capabilities: [file @ { pattern: "*.log" }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    let constraints = result.capabilities[0].constraints.as_ref().unwrap();
    match &constraints.fields[0].value {
        ConstraintValue::String(s) => assert_eq!(s, "*.log"),
        _ => panic!("Expected String constraint value"),
    }
}

#[test]
fn test_parse_constraint_array_empty() {
    let input = r#"workflow test capabilities: [file @ { paths: [] }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    let constraints = result.capabilities[0].constraints.as_ref().unwrap();
    match &constraints.fields[0].value {
        ConstraintValue::Array(arr) => assert!(arr.is_empty()),
        _ => panic!("Expected Array constraint value"),
    }
}

#[test]
fn test_parse_constraint_array_of_strings() {
    let input =
        r#"workflow test capabilities: [file @ { paths: ["/tmp/*", "/var/log/*"] }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    let constraints = result.capabilities[0].constraints.as_ref().unwrap();
    match &constraints.fields[0].value {
        ConstraintValue::Array(arr) => {
            assert_eq!(arr.len(), 2);
            match &arr[0] {
                ConstraintValue::String(s) => assert_eq!(s, "/tmp/*"),
                _ => panic!("Expected String in array"),
            }
        }
        _ => panic!("Expected Array constraint value"),
    }
}

#[test]
fn test_parse_constraint_array_of_ints() {
    let input = r#"workflow test capabilities: [network @ { ports: [80, 443, 8080] }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    let constraints = result.capabilities[0].constraints.as_ref().unwrap();
    match &constraints.fields[0].value {
        ConstraintValue::Array(arr) => {
            assert_eq!(arr.len(), 3);
            match &arr[0] {
                ConstraintValue::Int(n) => assert_eq!(*n, 80),
                _ => panic!("Expected Int in array"),
            }
        }
        _ => panic!("Expected Array constraint value"),
    }
}

#[test]
fn test_parse_constraint_array_of_mixed_types() {
    let input = r#"workflow test capabilities: [db @ { config: [true, 42, "test"] }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    let constraints = result.capabilities[0].constraints.as_ref().unwrap();
    match &constraints.fields[0].value {
        ConstraintValue::Array(arr) => {
            assert_eq!(arr.len(), 3);
            assert!(matches!(arr[0], ConstraintValue::Bool(true)));
            assert!(matches!(arr[1], ConstraintValue::Int(42)));
            assert!(matches!(&arr[2], ConstraintValue::String(s) if s == "test"));
        }
        _ => panic!("Expected Array constraint value"),
    }
}

#[test]
fn test_parse_constraint_object_empty() {
    let input = r#"workflow test capabilities: [api @ { options: {} }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    let constraints = result.capabilities[0].constraints.as_ref().unwrap();
    match &constraints.fields[0].value {
        ConstraintValue::Object(obj) => assert!(obj.is_empty()),
        _ => panic!("Expected Object constraint value"),
    }
}

#[test]
fn test_parse_constraint_object_with_values() {
    let input =
        r#"workflow test capabilities: [api @ { limits: { requests: 100, period: 60 } }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    let constraints = result.capabilities[0].constraints.as_ref().unwrap();
    match &constraints.fields[0].value {
        ConstraintValue::Object(obj) => {
            assert_eq!(obj.len(), 2);
            assert_eq!(obj[0].0, "requests");
            match &obj[0].1 {
                ConstraintValue::Int(n) => assert_eq!(*n, 100),
                _ => panic!("Expected Int in object"),
            }
        }
        _ => panic!("Expected Object constraint value"),
    }
}

#[test]
fn test_parse_constraint_nested_array() {
    let input = r#"workflow test capabilities: [matrix @ { rows: [[1, 2], [3, 4]] }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    let constraints = result.capabilities[0].constraints.as_ref().unwrap();
    match &constraints.fields[0].value {
        ConstraintValue::Array(outer) => {
            assert_eq!(outer.len(), 2);
            match &outer[0] {
                ConstraintValue::Array(inner) => {
                    assert_eq!(inner.len(), 2);
                    assert!(matches!(inner[0], ConstraintValue::Int(1)));
                }
                _ => panic!("Expected nested array"),
            }
        }
        _ => panic!("Expected Array constraint value"),
    }
}

// ============================================================================
// Integration Tests - Combined with Other Workflow Features
// ============================================================================

#[test]
fn test_capabilities_with_plays_role() {
    let input = r#"workflow test plays role(agent) capabilities: [file, network] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    assert_eq!(result.plays_roles.len(), 1);
    assert_eq!(result.capabilities.len(), 2);
}

#[test]
fn test_capabilities_with_params() {
    let input = r#"workflow test(x: Int) capabilities: [file] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    assert_eq!(result.params.len(), 1);
    assert_eq!(result.capabilities.len(), 1);
}

#[test]
fn test_capabilities_with_contract() {
    let input = r#"workflow test capabilities: [file] requires: true { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    assert_eq!(result.capabilities.len(), 1);
    assert!(result.contract.is_some());
}

#[test]
fn test_capabilities_complex_workflow() {
    let input = r#"workflow process_data(input: String)
        plays role(processor)
        plays role(validator)
        capabilities: [
            filesystem @ { paths: ["/data/*"], read: true, write: false },
            network @ { hosts: ["api.example.com"], ports: [443] },
            cache
        ]
        requires: input != ""
        ensures: true
    {
        done
    }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    assert_eq!(result.name.as_ref(), "process_data");
    assert_eq!(result.params.len(), 1);
    assert_eq!(result.plays_roles.len(), 2);
    assert_eq!(result.capabilities.len(), 3);
    assert!(result.contract.is_some());

    // Check filesystem capability
    let fs = &result.capabilities[0];
    assert_eq!(fs.capability.as_ref(), "filesystem");
    let fs_constraints = fs.constraints.as_ref().unwrap();
    assert_eq!(fs_constraints.fields.len(), 3);

    // Check cache capability (no constraints)
    let cache = &result.capabilities[2];
    assert_eq!(cache.capability.as_ref(), "cache");
    assert!(cache.constraints.is_none());
}

// ============================================================================
// Property Tests - Path Patterns and Identifiers
// ============================================================================

#[test]
fn test_parse_capability_various_identifiers() {
    let valid_identifiers = [
        "a",
        "abc",
        "abc123",
        "a1b2c3",
        "my_capability",
        "file_system",
        "http_client",
        "aws_s3",
    ];

    for ident in &valid_identifiers {
        let input = format!(r#"workflow test capabilities: [{}] {{ done }}"#, ident);
        let mut inp = new_input(&input);
        let result = workflow_def.parse_next(&mut inp);

        assert!(
            result.is_ok(),
            "Expected successful parse for identifier '{}', got: {:?}",
            ident,
            result
        );

        let workflow = result.unwrap();
        assert_eq!(workflow.capabilities[0].capability.as_ref(), *ident);
    }
}

#[test]
fn test_parse_path_patterns() {
    let patterns = [
        r#"/tmp/*"#,
        r#"/var/log/**/*.log"#,
        r#"/home/user/data"#,
        r#"C:\\Windows\\Temp"#,
        r#"s3://bucket/prefix/*"#,
        r#"gs://bucket/path"#,
    ];

    for pattern in &patterns {
        let input = format!(
            r#"workflow test capabilities: [storage @ {{ path: "{}" }}] {{ done }}"#,
            pattern
        );
        let mut inp = new_input(&input);
        let result = workflow_def.parse_next(&mut inp);

        assert!(
            result.is_ok(),
            "Expected successful parse for pattern '{}', got: {:?}",
            pattern,
            result
        );
    }
}

#[test]
fn test_parse_host_patterns() {
    let hosts = [
        r#"*.example.com"#,
        r#"api.example.com"#,
        r#"192.168.1.1"#,
        r#"*.internal.service"#,
    ];

    for host in &hosts {
        let input = format!(
            r#"workflow test capabilities: [network @ {{ hosts: ["{}"] }}] {{ done }}"#,
            host
        );
        let mut inp = new_input(&input);
        let result = workflow_def.parse_next(&mut inp);

        assert!(
            result.is_ok(),
            "Expected successful parse for host '{}', got: {:?}",
            host,
            result
        );
    }
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_parse_capabilities_missing_colon() {
    let input = r#"workflow test capabilities [file] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(result.is_err(), "Expected parse to fail without colon");
}

#[test]
fn test_parse_capabilities_missing_brackets() {
    let input = r#"workflow test capabilities: file { done }"#;
    let mut input = new_input(input);
    let _result = workflow_def.parse_next(&mut input);

    // This might succeed or fail depending on parsing strategy
    // but it shouldn't panic
}

#[test]
fn test_parse_constraint_missing_brace() {
    let input = r#"workflow test capabilities: [file @ { read: true ] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_err(),
        "Expected parse to fail with missing closing brace"
    );
}

#[test]
fn test_parse_constraint_missing_colon() {
    let input = r#"workflow test capabilities: [file @ { read true }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_err(),
        "Expected parse to fail without colon in field"
    );
}

// ============================================================================
// Whitespace and Formatting Tests
// ============================================================================

#[test]
fn test_parse_capabilities_whitespace_variations() {
    let inputs = [
        r#"workflow test capabilities:[file]{done}"#,
        r#"workflow test capabilities: [ file ] { done }"#,
        r#"workflow test capabilities: [
            file,
            network
        ] { done }"#,
        r#"workflow test capabilities: [
            file @ {
                read: true
            }
        ] { done }"#,
    ];

    for input in &inputs {
        let mut inp = new_input(input);
        let result = workflow_def.parse_next(&mut inp);
        assert!(
            result.is_ok(),
            "Expected successful parse for input '{}', got: {:?}",
            input,
            result
        );
    }
}

// ============================================================================
// Span Tests
// ============================================================================

#[test]
fn test_parse_capability_preserves_span() {
    let input = r#"workflow test capabilities: [file @ { read: true }] { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input).unwrap();

    assert_eq!(result.capabilities.len(), 1);

    // Span should be set (not default)
    let span = result.capabilities[0].span;
    assert!(span.start < span.end, "Span should have valid range");

    // Constraint block should also have span
    let constraint_span = result.capabilities[0].constraints.as_ref().unwrap().span;
    assert!(
        constraint_span.start < constraint_span.end,
        "Constraint span should have valid range"
    );
}

// ============================================================================
// Backward Compatibility Tests
// ============================================================================

#[test]
fn test_workflow_without_capabilities() {
    // Ensure existing workflows still work
    let input = r#"workflow simple { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let workflow = result.unwrap();
    assert!(workflow.capabilities.is_empty());
}

#[test]
fn test_workflow_with_only_plays_role() {
    let input = r#"workflow test plays role(agent) { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(result.is_ok());
    let workflow = result.unwrap();
    assert_eq!(workflow.plays_roles.len(), 1);
    assert!(workflow.capabilities.is_empty());
}
