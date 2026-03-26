//! Type tests for capability constraint validation (TASK-263)
//!
//! Tests for constraint checking per SPEC-017 and SPEC-024:
//! - Verify capability constraints match capability schema
//! - Validate field names are valid for capability type
//! - Check value types match expected
//! - Error messages for unknown caps, invalid fields, type mismatches

use ash_parser::surface::{
    CapabilityDecl, CapabilityDef, ConstraintBlock, ConstraintField, ConstraintValue, EffectType,
    Visibility,
};
use ash_parser::token::Span;
use ash_typeck::constraint_checking::{
    ConstraintCheckError, ConstraintChecker, ExpectedConstraintType,
};
use std::collections::HashMap;

fn test_span() -> Span {
    Span::new(0, 0, 1, 1)
}

fn create_capability_def(name: &str) -> CapabilityDef {
    CapabilityDef {
        visibility: Visibility::Public,
        name: name.into(),
        effect: EffectType::Operational,
        params: vec![],
        return_type: None,
        constraints: vec![],
        span: test_span(),
    }
}

fn create_capability_decl(name: &str, constraints: Option<ConstraintBlock>) -> CapabilityDecl {
    CapabilityDecl {
        capability: name.into(),
        constraints,
        span: test_span(),
    }
}

fn create_constraint_block(fields: Vec<(&str, ConstraintValue)>) -> ConstraintBlock {
    let fields = fields
        .into_iter()
        .map(|(name, value)| ConstraintField {
            name: name.into(),
            value,
            span: test_span(),
        })
        .collect();

    ConstraintBlock {
        fields,
        span: test_span(),
    }
}

#[test]
fn test_valid_file_constraints() {
    let mut cap_defs = HashMap::new();
    cap_defs.insert("file".to_string(), create_capability_def("file"));

    let checker = ConstraintChecker::new(&cap_defs);

    // Valid file constraints with all valid fields
    let decl = create_capability_decl(
        "file",
        Some(create_constraint_block(vec![
            (
                "paths",
                ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
            ),
            ("read", ConstraintValue::Bool(true)),
            ("write", ConstraintValue::Bool(false)),
        ])),
    );

    let result = checker.check_capability_decl(&decl);
    assert!(
        result.is_ok(),
        "Expected valid file constraints to be accepted"
    );
}

#[test]
fn test_valid_file_constraints_paths_only() {
    let mut cap_defs = HashMap::new();
    cap_defs.insert("file".to_string(), create_capability_def("file"));

    let checker = ConstraintChecker::new(&cap_defs);

    // Valid file constraints with only paths
    let decl = create_capability_decl(
        "file",
        Some(create_constraint_block(vec![(
            "paths",
            ConstraintValue::Array(vec![
                ConstraintValue::String("/tmp/*".to_string()),
                ConstraintValue::String("/var/log/*.log".to_string()),
            ]),
        )])),
    );

    let result = checker.check_capability_decl(&decl);
    assert!(result.is_ok());
}

#[test]
fn test_valid_file_constraints_read_only() {
    let mut cap_defs = HashMap::new();
    cap_defs.insert("file".to_string(), create_capability_def("file"));

    let checker = ConstraintChecker::new(&cap_defs);

    // Valid read-only file constraints
    let decl = create_capability_decl(
        "file",
        Some(create_constraint_block(vec![
            (
                "paths",
                ConstraintValue::Array(vec![ConstraintValue::String("/data/*".to_string())]),
            ),
            ("read", ConstraintValue::Bool(true)),
        ])),
    );

    let result = checker.check_capability_decl(&decl);
    assert!(result.is_ok());
}

#[test]
fn test_invalid_constraint_field() {
    let mut cap_defs = HashMap::new();
    cap_defs.insert("file".to_string(), create_capability_def("file"));

    let checker = ConstraintChecker::new(&cap_defs);

    // Invalid field for file capability
    let decl = create_capability_decl(
        "file",
        Some(create_constraint_block(vec![(
            "invalid_field",
            ConstraintValue::Bool(true),
        )])),
    );

    let result = checker.check_capability_decl(&decl);
    assert!(result.is_err(), "Expected invalid field to be rejected");

    match result.unwrap_err() {
        ConstraintCheckError::InvalidConstraintField {
            capability,
            field,
            span,
        } => {
            assert_eq!(capability, "file");
            assert_eq!(field, "invalid_field");
            assert_eq!(span, test_span());
        }
        _ => panic!("Expected InvalidConstraintField error"),
    }
}

#[test]
fn test_constraint_type_mismatch() {
    let mut cap_defs = HashMap::new();
    cap_defs.insert("file".to_string(), create_capability_def("file"));

    let checker = ConstraintChecker::new(&cap_defs);

    // Type mismatch: read should be bool, not string
    let decl = create_capability_decl(
        "file",
        Some(create_constraint_block(vec![(
            "read",
            ConstraintValue::String("true".to_string()),
        )])),
    );

    let result = checker.check_capability_decl(&decl);
    assert!(result.is_err(), "Expected type mismatch to be detected");

    match result.unwrap_err() {
        ConstraintCheckError::ConstraintTypeMismatch {
            field,
            expected,
            found,
        } => {
            assert_eq!(field, "read");
            assert_eq!(expected, "bool");
            assert_eq!(found, "string");
        }
        _ => panic!("Expected ConstraintTypeMismatch error"),
    }
}

#[test]
fn test_constraint_type_mismatch_paths() {
    let mut cap_defs = HashMap::new();
    cap_defs.insert("file".to_string(), create_capability_def("file"));

    let checker = ConstraintChecker::new(&cap_defs);

    // Type mismatch: paths should be array, not string
    let decl = create_capability_decl(
        "file",
        Some(create_constraint_block(vec![(
            "paths",
            ConstraintValue::String("/tmp/*".to_string()),
        )])),
    );

    let result = checker.check_capability_decl(&decl);
    assert!(result.is_err());

    match result.unwrap_err() {
        ConstraintCheckError::ConstraintTypeMismatch {
            field,
            expected,
            found,
        } => {
            assert_eq!(field, "paths");
            assert_eq!(expected, "array");
            assert_eq!(found, "string");
        }
        _ => panic!("Expected ConstraintTypeMismatch error"),
    }
}

#[test]
fn test_unknown_capability() {
    let cap_defs: HashMap<String, CapabilityDef> = HashMap::new();
    let checker = ConstraintChecker::new(&cap_defs);

    let decl = create_capability_decl("unknown_cap", None);

    let result = checker.check_capability_decl(&decl);
    assert!(
        result.is_err(),
        "Expected unknown capability to be rejected"
    );

    match result.unwrap_err() {
        ConstraintCheckError::UnknownCapability { name, span } => {
            assert_eq!(name, "unknown_cap");
            assert_eq!(span, test_span());
        }
        _ => panic!("Expected UnknownCapability error"),
    }
}

#[test]
fn test_valid_network_constraints() {
    let mut cap_defs = HashMap::new();
    cap_defs.insert("network".to_string(), create_capability_def("network"));

    let checker = ConstraintChecker::new(&cap_defs);

    let decl = create_capability_decl(
        "network",
        Some(create_constraint_block(vec![
            (
                "hosts",
                ConstraintValue::Array(vec![ConstraintValue::String("*.example.com".to_string())]),
            ),
            (
                "ports",
                ConstraintValue::Array(vec![ConstraintValue::Int(443)]),
            ),
            (
                "protocols",
                ConstraintValue::Array(vec![ConstraintValue::String("https".to_string())]),
            ),
        ])),
    );

    let result = checker.check_capability_decl(&decl);
    assert!(
        result.is_ok(),
        "Expected valid network constraints to be accepted"
    );
}

#[test]
fn test_valid_network_constraints_multiple_values() {
    let mut cap_defs = HashMap::new();
    cap_defs.insert("network".to_string(), create_capability_def("network"));

    let checker = ConstraintChecker::new(&cap_defs);

    let decl = create_capability_decl(
        "network",
        Some(create_constraint_block(vec![
            (
                "hosts",
                ConstraintValue::Array(vec![
                    ConstraintValue::String("api.example.com".to_string()),
                    ConstraintValue::String("*.internal.com".to_string()),
                ]),
            ),
            (
                "ports",
                ConstraintValue::Array(vec![
                    ConstraintValue::Int(80),
                    ConstraintValue::Int(443),
                    ConstraintValue::Int(8080),
                ]),
            ),
        ])),
    );

    let result = checker.check_capability_decl(&decl);
    assert!(result.is_ok());
}

#[test]
fn test_valid_process_constraints() {
    let mut cap_defs = HashMap::new();
    cap_defs.insert("process".to_string(), create_capability_def("process"));

    let checker = ConstraintChecker::new(&cap_defs);

    let decl = create_capability_decl(
        "process",
        Some(create_constraint_block(vec![
            ("spawn", ConstraintValue::Bool(true)),
            ("kill", ConstraintValue::Bool(false)),
            (
                "signal",
                ConstraintValue::Array(vec![ConstraintValue::Int(9), ConstraintValue::Int(15)]),
            ),
        ])),
    );

    let result = checker.check_capability_decl(&decl);
    assert!(
        result.is_ok(),
        "Expected valid process constraints to be accepted"
    );
}

#[test]
fn test_no_constraints_is_valid() {
    let mut cap_defs = HashMap::new();
    cap_defs.insert("file".to_string(), create_capability_def("file"));

    let checker = ConstraintChecker::new(&cap_defs);

    // Capability with no constraints should be valid
    let decl = create_capability_decl("file", None);

    let result = checker.check_capability_decl(&decl);
    assert!(
        result.is_ok(),
        "Expected capability without constraints to be valid"
    );
}

#[test]
fn test_empty_constraints_is_valid() {
    let mut cap_defs = HashMap::new();
    cap_defs.insert("file".to_string(), create_capability_def("file"));

    let checker = ConstraintChecker::new(&cap_defs);

    // Capability with empty constraint block should be valid
    let decl = create_capability_decl("file", Some(create_constraint_block(vec![])));

    let result = checker.check_capability_decl(&decl);
    assert!(
        result.is_ok(),
        "Expected empty constraint block to be valid"
    );
}

#[test]
fn test_network_invalid_field() {
    let mut cap_defs = HashMap::new();
    cap_defs.insert("network".to_string(), create_capability_def("network"));

    let checker = ConstraintChecker::new(&cap_defs);

    // Using file field for network capability
    let decl = create_capability_decl(
        "network",
        Some(create_constraint_block(vec![(
            "paths",
            ConstraintValue::Array(vec![]),
        )])),
    );

    let result = checker.check_capability_decl(&decl);
    assert!(result.is_err());

    match result.unwrap_err() {
        ConstraintCheckError::InvalidConstraintField {
            capability, field, ..
        } => {
            assert_eq!(capability, "network");
            assert_eq!(field, "paths");
        }
        _ => panic!("Expected InvalidConstraintField error"),
    }
}

#[test]
fn test_process_type_mismatch_spawn() {
    let mut cap_defs = HashMap::new();
    cap_defs.insert("process".to_string(), create_capability_def("process"));

    let checker = ConstraintChecker::new(&cap_defs);

    // spawn should be bool, not int
    let decl = create_capability_decl(
        "process",
        Some(create_constraint_block(vec![(
            "spawn",
            ConstraintValue::Int(1),
        )])),
    );

    let result = checker.check_capability_decl(&decl);
    assert!(result.is_err());

    match result.unwrap_err() {
        ConstraintCheckError::ConstraintTypeMismatch {
            field,
            expected,
            found,
        } => {
            assert_eq!(field, "spawn");
            assert_eq!(expected, "bool");
            assert_eq!(found, "int");
        }
        _ => panic!("Expected ConstraintTypeMismatch error"),
    }
}

#[test]
fn test_multiple_capabilities_in_registry() {
    let mut cap_defs = HashMap::new();
    cap_defs.insert("file".to_string(), create_capability_def("file"));
    cap_defs.insert("network".to_string(), create_capability_def("network"));
    cap_defs.insert("process".to_string(), create_capability_def("process"));

    let checker = ConstraintChecker::new(&cap_defs);

    // All should be available
    assert!(checker.has_capability("file"));
    assert!(checker.has_capability("network"));
    assert!(checker.has_capability("process"));
    assert!(!checker.has_capability("unknown"));

    let caps: Vec<_> = checker.available_capabilities().collect();
    assert_eq!(caps.len(), 3);
}

#[test]
fn test_expected_constraint_type_matches_bool() {
    assert!(ExpectedConstraintType::Bool.matches(&ConstraintValue::Bool(true)));
    assert!(ExpectedConstraintType::Bool.matches(&ConstraintValue::Bool(false)));
    assert!(!ExpectedConstraintType::Bool.matches(&ConstraintValue::Int(1)));
    assert!(!ExpectedConstraintType::Bool.matches(&ConstraintValue::String("true".to_string())));
}

#[test]
fn test_expected_constraint_type_matches_int() {
    assert!(ExpectedConstraintType::Int.matches(&ConstraintValue::Int(42)));
    assert!(ExpectedConstraintType::Int.matches(&ConstraintValue::Int(-10)));
    assert!(!ExpectedConstraintType::Int.matches(&ConstraintValue::Bool(true)));
    assert!(!ExpectedConstraintType::Int.matches(&ConstraintValue::String("42".to_string())));
}

#[test]
fn test_expected_constraint_type_matches_string() {
    assert!(ExpectedConstraintType::String.matches(&ConstraintValue::String("test".to_string())));
    assert!(ExpectedConstraintType::String.matches(&ConstraintValue::String("".to_string())));
    assert!(!ExpectedConstraintType::String.matches(&ConstraintValue::Int(42)));
    assert!(!ExpectedConstraintType::String.matches(&ConstraintValue::Bool(false)));
}

#[test]
fn test_expected_constraint_type_matches_array() {
    assert!(ExpectedConstraintType::Array.matches(&ConstraintValue::Array(vec![])));
    assert!(
        ExpectedConstraintType::Array
            .matches(&ConstraintValue::Array(vec![ConstraintValue::Int(1)]))
    );
    assert!(!ExpectedConstraintType::Array.matches(&ConstraintValue::String("[]".to_string())));
}

#[test]
fn test_expected_constraint_type_matches_object() {
    assert!(ExpectedConstraintType::Object.matches(&ConstraintValue::Object(vec![])));
    assert!(
        ExpectedConstraintType::Object.matches(&ConstraintValue::Object(vec![(
            "key".to_string(),
            ConstraintValue::Bool(true)
        )]))
    );
    assert!(!ExpectedConstraintType::Object.matches(&ConstraintValue::String("{}".to_string())));
}

#[test]
fn test_expected_constraint_type_matches_any_of() {
    let any_type = ExpectedConstraintType::AnyOf(vec![
        ExpectedConstraintType::Bool,
        ExpectedConstraintType::Int,
    ]);

    assert!(any_type.matches(&ConstraintValue::Bool(true)));
    assert!(any_type.matches(&ConstraintValue::Int(42)));
    assert!(!any_type.matches(&ConstraintValue::String("test".to_string())));
    assert!(!any_type.matches(&ConstraintValue::Array(vec![])));
}

#[test]
fn test_type_descriptions() {
    assert_eq!(ExpectedConstraintType::Bool.description(), "bool");
    assert_eq!(ExpectedConstraintType::Int.description(), "int");
    assert_eq!(ExpectedConstraintType::String.description(), "string");
    assert_eq!(ExpectedConstraintType::Array.description(), "array");
    assert_eq!(ExpectedConstraintType::Object.description(), "object");

    let any_type = ExpectedConstraintType::AnyOf(vec![
        ExpectedConstraintType::Bool,
        ExpectedConstraintType::Int,
    ]);
    assert_eq!(any_type.description(), "one of: bool, int");
}

#[test]
fn test_error_display_messages() {
    let unknown_cap_err = ConstraintCheckError::UnknownCapability {
        name: "foo".to_string(),
        span: test_span(),
    };
    let msg = format!("{}", unknown_cap_err);
    assert!(msg.contains("Unknown capability"));
    assert!(msg.contains("foo"));

    let invalid_field_err = ConstraintCheckError::InvalidConstraintField {
        capability: "file".to_string(),
        field: "bad_field".to_string(),
        span: test_span(),
    };
    let msg = format!("{}", invalid_field_err);
    assert!(msg.contains("Invalid constraint field"));
    assert!(msg.contains("bad_field"));
    assert!(msg.contains("file"));

    let type_mismatch_err = ConstraintCheckError::ConstraintTypeMismatch {
        field: "read".to_string(),
        expected: "bool".to_string(),
        found: "string".to_string(),
    };
    let msg = format!("{}", type_mismatch_err);
    assert!(msg.contains("Constraint type mismatch"));
    assert!(msg.contains("read"));
    assert!(msg.contains("bool"));
    assert!(msg.contains("string"));
}
