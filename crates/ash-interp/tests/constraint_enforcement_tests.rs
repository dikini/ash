//! Integration tests for constraint enforcement
//!
//! These tests verify that capability constraints are properly enforced at runtime.

use ash_core::Value;
use ash_interp::constraint_enforcement::{
    ConstraintEnforcer, ConstraintViolation, args_from_pairs, constraint_block_from_fields,
};
use ash_parser::surface::ConstraintValue;
use std::collections::HashMap;

// =============================================================================
// Path Constraint Tests
// =============================================================================

#[test]
fn test_path_constraint_allows_matching() {
    let constraints = constraint_block_from_fields(vec![(
        "paths",
        ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
    )]);

    let args = args_from_pairs(vec![("path", "/tmp/data.txt")]);

    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(result.is_ok(), "Expected /tmp/data.txt to match /tmp/*");
}

#[test]
fn test_path_constraint_allows_multiple_patterns() {
    let constraints = constraint_block_from_fields(vec![(
        "paths",
        ConstraintValue::Array(vec![
            ConstraintValue::String("/tmp/*".to_string()),
            ConstraintValue::String("/var/log/*".to_string()),
        ]),
    )]);

    // Should match first pattern
    let args1 = args_from_pairs(vec![("path", "/tmp/data.txt")]);
    assert!(ConstraintEnforcer::check("read", &args1, &constraints).is_ok());

    // Should match second pattern
    let args2 = args_from_pairs(vec![("path", "/var/log/app.log")]);
    assert!(ConstraintEnforcer::check("read", &args2, &constraints).is_ok());
}

#[test]
fn test_path_constraint_denies_non_matching() {
    let constraints = constraint_block_from_fields(vec![(
        "paths",
        ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
    )]);

    let args = args_from_pairs(vec![("path", "/etc/passwd")]);

    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(result.is_err(), "Expected /etc/passwd to not match /tmp/*");

    let err = result.unwrap_err();
    assert!(
        matches!(err, ConstraintViolation::PathNotAllowed { .. }),
        "Expected PathNotAllowed error"
    );
}

#[test]
fn test_path_constraint_denies_subdirectories_with_single_glob() {
    let constraints = constraint_block_from_fields(vec![(
        "paths",
        ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
    )]);

    // /tmp/* should not match /tmp/subdir/file.txt
    let args = args_from_pairs(vec![("path", "/tmp/subdir/file.txt")]);

    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(
        result.is_err(),
        "Expected /tmp/subdir/file.txt to not match /tmp/*"
    );
}

#[test]
fn test_path_constraint_allows_subdirectories_with_double_glob() {
    let constraints = constraint_block_from_fields(vec![(
        "paths",
        ConstraintValue::Array(vec![ConstraintValue::String("/tmp/**".to_string())]),
    )]);

    // /tmp/** should match /tmp/subdir/file.txt
    let args = args_from_pairs(vec![("path", "/tmp/subdir/file.txt")]);

    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(
        result.is_ok(),
        "Expected /tmp/subdir/file.txt to match /tmp/**"
    );
}

#[test]
fn test_path_constraint_allows_exact_match() {
    let constraints = constraint_block_from_fields(vec![(
        "paths",
        ConstraintValue::Array(vec![ConstraintValue::String("/etc/hosts".to_string())]),
    )]);

    let args = args_from_pairs(vec![("path", "/etc/hosts")]);

    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(result.is_ok(), "Expected exact match to succeed");
}

#[test]
fn test_path_constraint_missing_path_argument() {
    let constraints = constraint_block_from_fields(vec![(
        "paths",
        ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
    )]);

    // No path argument provided
    let args = Value::Record(Box::default());

    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(result.is_err(), "Expected error for missing path argument");

    let err = result.unwrap_err();
    assert!(
        matches!(err, ConstraintViolation::MissingArgument("path")),
        "Expected MissingArgument error"
    );
}

#[test]
fn test_path_constraint_not_applied_to_non_file_operations() {
    let constraints = constraint_block_from_fields(vec![(
        "paths",
        ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
    )]);

    // Network operations should not be constrained by paths
    let args = args_from_pairs(vec![("host", "example.com")]);

    let result = ConstraintEnforcer::check("get", &args, &constraints);
    assert!(
        result.is_ok(),
        "Expected path constraints to not apply to network operations"
    );
}

// =============================================================================
// Host Constraint Tests
// =============================================================================

#[test]
fn test_host_constraint_allows_matching() {
    let constraints = constraint_block_from_fields(vec![(
        "hosts",
        ConstraintValue::Array(vec![ConstraintValue::String("*.example.com".to_string())]),
    )]);

    let args = args_from_pairs(vec![("host", "api.example.com")]);

    let result = ConstraintEnforcer::check("get", &args, &constraints);
    assert!(
        result.is_ok(),
        "Expected api.example.com to match *.example.com"
    );
}

#[test]
fn test_host_constraint_allows_exact_match() {
    let constraints = constraint_block_from_fields(vec![(
        "hosts",
        ConstraintValue::Array(vec![ConstraintValue::String("example.com".to_string())]),
    )]);

    let args = args_from_pairs(vec![("host", "example.com")]);

    let result = ConstraintEnforcer::check("get", &args, &constraints);
    assert!(result.is_ok(), "Expected exact host match to succeed");
}

#[test]
fn test_host_constraint_denies_non_matching() {
    let constraints = constraint_block_from_fields(vec![(
        "hosts",
        ConstraintValue::Array(vec![ConstraintValue::String("*.example.com".to_string())]),
    )]);

    let args = args_from_pairs(vec![("host", "evil.com")]);

    let result = ConstraintEnforcer::check("get", &args, &constraints);
    assert!(
        result.is_err(),
        "Expected evil.com to not match *.example.com"
    );

    let err = result.unwrap_err();
    assert!(
        matches!(err, ConstraintViolation::HostNotAllowed { .. }),
        "Expected HostNotAllowed error"
    );
}

#[test]
fn test_host_constraint_wildcard_does_not_match_parent() {
    let constraints = constraint_block_from_fields(vec![(
        "hosts",
        ConstraintValue::Array(vec![ConstraintValue::String("*.example.com".to_string())]),
    )]);

    // *.example.com should NOT match example.com itself
    let args = args_from_pairs(vec![("host", "example.com")]);

    let result = ConstraintEnforcer::check("get", &args, &constraints);
    assert!(
        result.is_err(),
        "Expected example.com to not match *.example.com"
    );
}

#[test]
fn test_host_constraint_missing_host_argument() {
    let constraints = constraint_block_from_fields(vec![(
        "hosts",
        ConstraintValue::Array(vec![ConstraintValue::String("*.example.com".to_string())]),
    )]);

    // No host argument provided
    let args = Value::Record(Box::default());

    let result = ConstraintEnforcer::check("get", &args, &constraints);
    assert!(result.is_err(), "Expected error for missing host argument");

    let err = result.unwrap_err();
    assert!(
        matches!(err, ConstraintViolation::MissingArgument("host")),
        "Expected MissingArgument error"
    );
}

#[test]
fn test_host_constraint_not_applied_to_file_operations() {
    let constraints = constraint_block_from_fields(vec![(
        "hosts",
        ConstraintValue::Array(vec![ConstraintValue::String("*.example.com".to_string())]),
    )]);

    // File operations should not be constrained by hosts
    let args = args_from_pairs(vec![("path", "/tmp/data.txt")]);

    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(
        result.is_ok(),
        "Expected host constraints to not apply to file operations"
    );
}

// =============================================================================
// Permission Constraint Tests
// =============================================================================

#[test]
fn test_permission_read_allowed() {
    let constraints = constraint_block_from_fields(vec![("read", ConstraintValue::Bool(true))]);

    let args = args_from_pairs(vec![("path", "/tmp/data.txt")]);

    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(
        result.is_ok(),
        "Expected read to be allowed when read: true"
    );
}

#[test]
fn test_permission_read_denied() {
    let constraints = constraint_block_from_fields(vec![("read", ConstraintValue::Bool(false))]);

    let args = args_from_pairs(vec![("path", "/tmp/data.txt")]);

    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(
        result.is_err(),
        "Expected read to be denied when read: false"
    );

    let err = result.unwrap_err();
    assert!(
        matches!(err, ConstraintViolation::PermissionDenied { .. }),
        "Expected PermissionDenied error"
    );
}

#[test]
fn test_permission_write_allowed() {
    let constraints = constraint_block_from_fields(vec![("write", ConstraintValue::Bool(true))]);

    let args = args_from_pairs(vec![("path", "/tmp/data.txt")]);

    let result = ConstraintEnforcer::check("write", &args, &constraints);
    assert!(
        result.is_ok(),
        "Expected write to be allowed when write: true"
    );
}

#[test]
fn test_permission_write_denied() {
    let constraints = constraint_block_from_fields(vec![("write", ConstraintValue::Bool(false))]);

    let args = args_from_pairs(vec![("path", "/tmp/data.txt")]);

    let result = ConstraintEnforcer::check("write", &args, &constraints);
    assert!(
        result.is_err(),
        "Expected write to be denied when write: false"
    );
}

#[test]
fn test_permission_spawn_allowed() {
    let constraints = constraint_block_from_fields(vec![("spawn", ConstraintValue::Bool(true))]);

    let args = Value::Null;

    let result = ConstraintEnforcer::check("spawn", &args, &constraints);
    assert!(
        result.is_ok(),
        "Expected spawn to be allowed when spawn: true"
    );
}

#[test]
fn test_permission_spawn_denied() {
    let constraints = constraint_block_from_fields(vec![("spawn", ConstraintValue::Bool(false))]);

    let args = Value::Null;

    let result = ConstraintEnforcer::check("spawn", &args, &constraints);
    assert!(
        result.is_err(),
        "Expected spawn to be denied when spawn: false"
    );
}

#[test]
fn test_permission_kill_allowed() {
    let constraints = constraint_block_from_fields(vec![("kill", ConstraintValue::Bool(true))]);

    let args = Value::Null;

    let result = ConstraintEnforcer::check("kill", &args, &constraints);
    assert!(
        result.is_ok(),
        "Expected kill to be allowed when kill: true"
    );
}

#[test]
fn test_permission_kill_denied() {
    let constraints = constraint_block_from_fields(vec![("kill", ConstraintValue::Bool(false))]);

    let args = Value::Null;

    let result = ConstraintEnforcer::check("kill", &args, &constraints);
    assert!(
        result.is_err(),
        "Expected kill to be denied when kill: false"
    );
}

#[test]
fn test_permission_combined() {
    // Test combined read: true, write: false
    let constraints = constraint_block_from_fields(vec![
        ("read", ConstraintValue::Bool(true)),
        ("write", ConstraintValue::Bool(false)),
    ]);

    let args = args_from_pairs(vec![("path", "/tmp/test.txt")]);

    // Read should succeed
    assert!(
        ConstraintEnforcer::check("read", &args, &constraints).is_ok(),
        "Expected read to succeed when read: true"
    );

    // Write should fail
    let write_result = ConstraintEnforcer::check("write", &args, &constraints);
    assert!(
        write_result.is_err(),
        "Expected write to fail when write: false"
    );
}

#[test]
fn test_permission_operation_mapping_get() {
    // get operation maps to read permission
    let constraints = constraint_block_from_fields(vec![("read", ConstraintValue::Bool(false))]);

    let args = args_from_pairs(vec![("host", "example.com")]);

    let result = ConstraintEnforcer::check("get", &args, &constraints);
    assert!(
        result.is_err(),
        "Expected get to be denied when read: false"
    );
}

#[test]
fn test_permission_operation_mapping_post() {
    // post operation maps to write permission
    let constraints = constraint_block_from_fields(vec![("write", ConstraintValue::Bool(false))]);

    let args = args_from_pairs(vec![("host", "example.com")]);

    let result = ConstraintEnforcer::check("post", &args, &constraints);
    assert!(
        result.is_err(),
        "Expected post to be denied when write: false"
    );
}

// =============================================================================
// Error Message Tests
// =============================================================================

#[test]
fn test_path_error_message_contains_path() {
    let constraints = constraint_block_from_fields(vec![(
        "paths",
        ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
    )]);

    let args = args_from_pairs(vec![("path", "/etc/passwd")]);

    let err = ConstraintEnforcer::check("read", &args, &constraints).unwrap_err();
    let msg = err.to_string();

    assert!(
        msg.contains("/etc/passwd"),
        "Error message should contain the denied path"
    );
    assert!(
        msg.contains("/tmp/*"),
        "Error message should contain allowed patterns"
    );
}

#[test]
fn test_host_error_message_contains_host() {
    let constraints = constraint_block_from_fields(vec![(
        "hosts",
        ConstraintValue::Array(vec![ConstraintValue::String("*.example.com".to_string())]),
    )]);

    let args = args_from_pairs(vec![("host", "evil.com")]);

    let err = ConstraintEnforcer::check("get", &args, &constraints).unwrap_err();
    let msg = err.to_string();

    assert!(
        msg.contains("evil.com"),
        "Error message should contain the denied host"
    );
    assert!(
        msg.contains("*.example.com"),
        "Error message should contain allowed patterns"
    );
}

#[test]
fn test_permission_error_message_contains_operation() {
    let constraints = constraint_block_from_fields(vec![("write", ConstraintValue::Bool(false))]);

    let args = args_from_pairs(vec![("path", "/tmp/test.txt")]);

    let err = ConstraintEnforcer::check("write", &args, &constraints).unwrap_err();
    let msg = err.to_string();

    assert!(
        msg.contains("write"),
        "Error message should contain the operation"
    );
    assert!(
        msg.contains("permission"),
        "Error message should mention permission"
    );
}

// =============================================================================
// Complex Constraint Tests
// =============================================================================

#[test]
fn test_combined_path_and_permission_constraints() {
    // Both path and permission constraints must be satisfied
    let constraints = constraint_block_from_fields(vec![
        (
            "paths",
            ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
        ),
        ("read", ConstraintValue::Bool(true)),
    ]);

    let args = args_from_pairs(vec![("path", "/tmp/data.txt")]);

    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(result.is_ok(), "Expected both constraints to pass");
}

#[test]
fn test_combined_path_and_permission_denied() {
    let constraints = constraint_block_from_fields(vec![
        (
            "paths",
            ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
        ),
        ("write", ConstraintValue::Bool(false)),
    ]);

    let args = args_from_pairs(vec![("path", "/tmp/data.txt")]);

    // Path matches but write permission denied
    let result = ConstraintEnforcer::check("write", &args, &constraints);
    assert!(result.is_err(), "Expected write permission denial");
}

#[test]
fn test_combined_host_and_permission_constraints() {
    let constraints = constraint_block_from_fields(vec![
        (
            "hosts",
            ConstraintValue::Array(vec![ConstraintValue::String("*.example.com".to_string())]),
        ),
        ("read", ConstraintValue::Bool(true)),
    ]);

    let args = args_from_pairs(vec![("host", "api.example.com")]);

    let result = ConstraintEnforcer::check("get", &args, &constraints);
    assert!(result.is_ok(), "Expected both constraints to pass");
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_empty_constraints_allow_all() {
    // Empty constraint block should allow everything
    let constraints = constraint_block_from_fields(vec![]);

    let args = args_from_pairs(vec![("path", "/any/path")]);

    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(
        result.is_ok(),
        "Expected empty constraints to allow all operations"
    );
}

#[test]
fn test_unknown_constraint_ignored() {
    // Unknown constraint fields should be ignored
    let constraints = constraint_block_from_fields(vec![(
        "unknown_field",
        ConstraintValue::String("some_value".to_string()),
    )]);

    let args = args_from_pairs(vec![("path", "/tmp/data.txt")]);

    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(result.is_ok(), "Expected unknown constraints to be ignored");
}

#[test]
fn test_non_string_path_value() {
    // Path value that's not a string should cause error
    let constraints = constraint_block_from_fields(vec![(
        "paths",
        ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
    )]);

    let mut map = HashMap::new();
    map.insert("path".to_string(), Value::Int(42));
    let args = Value::Record(Box::new(map));

    // The check_paths won't find a string path, so it returns MissingArgument
    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(result.is_err());
}

#[test]
fn test_paths_constraint_requires_array() {
    // paths constraint with non-array value should error
    let constraints = constraint_block_from_fields(vec![(
        "paths",
        ConstraintValue::String("/tmp/*".to_string()), // Should be array
    )]);

    let args = args_from_pairs(vec![("path", "/tmp/data.txt")]);

    let result = ConstraintEnforcer::check("read", &args, &constraints);
    assert!(
        result.is_err(),
        "Expected error for non-array paths constraint"
    );

    let err = result.unwrap_err();
    assert!(
        matches!(err, ConstraintViolation::InvalidConstraint(_)),
        "Expected InvalidConstraint error"
    );
}

#[test]
fn test_hosts_constraint_requires_array() {
    // hosts constraint with non-array value should error
    let constraints = constraint_block_from_fields(vec![(
        "hosts",
        ConstraintValue::String("*.example.com".to_string()), // Should be array
    )]);

    let args = args_from_pairs(vec![("host", "api.example.com")]);

    let result = ConstraintEnforcer::check("get", &args, &constraints);
    assert!(
        result.is_err(),
        "Expected error for non-array hosts constraint"
    );
}

// =============================================================================
// Glob Pattern Tests
// =============================================================================

#[test]
fn test_glob_pattern_variations() {
    // Test various glob patterns
    assert!(ConstraintEnforcer::path_matches("/tmp/a.txt", "/tmp/*.txt"));
    assert!(ConstraintEnforcer::path_matches(
        "/tmp/data.json",
        "/tmp/*.json"
    ));
    assert!(!ConstraintEnforcer::path_matches(
        "/tmp/data.xml",
        "/tmp/*.json"
    ));

    // Test with more complex patterns
    assert!(ConstraintEnforcer::path_matches(
        "/var/log/app.log",
        "/var/log/*"
    ));
    assert!(!ConstraintEnforcer::path_matches(
        "/var/www/index.html",
        "/var/log/*"
    ));
}

#[test]
fn test_deep_subdirectories_with_double_glob() {
    // /tmp/** should match files at any depth under /tmp/
    assert!(ConstraintEnforcer::path_matches("/tmp/file.txt", "/tmp/**"));
    assert!(ConstraintEnforcer::path_matches(
        "/tmp/a/file.txt",
        "/tmp/**"
    ));
    assert!(ConstraintEnforcer::path_matches(
        "/tmp/a/b/c/d/e/file.txt",
        "/tmp/**"
    ));

    // Should not match files outside /tmp/
    assert!(!ConstraintEnforcer::path_matches(
        "/var/tmp/file.txt",
        "/tmp/**"
    ));
}

// =============================================================================
// Host Wildcard Tests
// =============================================================================

#[test]
fn test_host_wildcard_edge_cases() {
    // Multiple subdomain levels
    assert!(ConstraintEnforcer::host_matches(
        "a.b.c.example.com",
        "*.example.com"
    ));
    assert!(ConstraintEnforcer::host_matches(
        "deep.nested.sub.example.com",
        "*.example.com"
    ));

    // Exact match should also work
    assert!(!ConstraintEnforcer::host_matches(
        "example.com",
        "*.example.com"
    ));

    // Different TLD should not match
    assert!(!ConstraintEnforcer::host_matches(
        "api.example.org",
        "*.example.com"
    ));

    // Prefix matching, not substring
    assert!(!ConstraintEnforcer::host_matches(
        "notexample.com",
        "*.example.com"
    ));
}

// =============================================================================
// ConstraintViolation Clone Tests
// =============================================================================

#[test]
fn test_constraint_violation_clone() {
    let err1 = ConstraintViolation::MissingArgument("path");
    let err1_clone = err1.clone();
    assert_eq!(err1, err1_clone);

    let err2 = ConstraintViolation::PathNotAllowed {
        path: "/etc/passwd".to_string(),
        allowed: vec!["/tmp/*".to_string()],
    };
    let err2_clone = err2.clone();
    assert_eq!(err2, err2_clone);
}

#[test]
fn test_constraint_violation_equality() {
    let err1 = ConstraintViolation::MissingArgument("path");
    let err2 = ConstraintViolation::MissingArgument("path");
    let err3 = ConstraintViolation::MissingArgument("host");

    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}
