//! Type tests for role inclusion (TASK-262)
//!
//! Tests for role checking per SPEC-019 and SPEC-024:
//! - Verify each role exists
//! - Compose capabilities from all included roles
//! - Check for capability conflicts

use ash_parser::surface::{CapabilityDecl, RoleDef, RoleRef, WorkflowDef};
use ash_parser::token::Span;
use ash_typeck::role_checking::{EffectiveCapabilities, RoleCheckError, RoleChecker};
use std::collections::HashMap;

fn test_span() -> Span {
    Span::new(0, 0, 1, 1)
}

fn create_role_def(name: &str, capabilities: Vec<&str>) -> RoleDef {
    RoleDef {
        name: name.into(),
        capabilities: capabilities
            .into_iter()
            .map(|cap| CapabilityDecl {
                capability: cap.into(),
                constraints: None,
                span: test_span(),
            })
            .collect(),
        obligations: vec![],
        span: test_span(),
    }
}

fn create_workflow_def_with_roles(role_names: Vec<&str>) -> WorkflowDef {
    let plays_roles: Vec<RoleRef> = role_names
        .into_iter()
        .map(|name| RoleRef {
            name: name.into(),
            span: test_span(),
        })
        .collect();

    WorkflowDef {
        name: "test_workflow".into(),
        params: vec![],
        plays_roles,
        capabilities: vec![],
        body: ash_parser::surface::Workflow::Done { span: test_span() },
        contract: None,
        span: test_span(),
    }
}

#[test]
fn test_valid_role_inclusion() {
    let mut role_defs = HashMap::new();
    role_defs.insert(
        "ai_agent".to_string(),
        create_role_def("ai_agent", vec!["network", "file"]),
    );

    let checker = RoleChecker::new(&role_defs);
    let workflow = create_workflow_def_with_roles(vec!["ai_agent"]);

    let result = checker.check_workflow_roles(&workflow);
    assert!(result.is_ok());

    let effective = result.unwrap();
    assert!(effective.has("network"));
    assert!(effective.has("file"));
    assert_eq!(effective.len(), 2);
}

#[test]
fn test_unknown_role_error() {
    let role_defs = HashMap::new();
    let checker = RoleChecker::new(&role_defs);
    let workflow = create_workflow_def_with_roles(vec!["unknown_role"]);

    let result = checker.check_workflow_roles(&workflow);
    assert!(result.is_err());

    match result.unwrap_err() {
        RoleCheckError::UnknownRole { name, .. } => {
            assert_eq!(name, "unknown_role");
        }
        _ => panic!("Expected UnknownRole error"),
    }
}

#[test]
fn test_multiple_role_capabilities_composed() {
    let mut role_defs = HashMap::new();
    role_defs.insert(
        "ai_agent".to_string(),
        create_role_def("ai_agent", vec!["network", "file"]),
    );
    role_defs.insert(
        "network_client".to_string(),
        create_role_def("network_client", vec!["http", "websocket"]),
    );

    let checker = RoleChecker::new(&role_defs);
    let workflow = create_workflow_def_with_roles(vec!["ai_agent", "network_client"]);

    let result = checker.check_workflow_roles(&workflow);
    assert!(result.is_ok());

    let effective = result.unwrap();
    assert!(effective.has("network"));
    assert!(effective.has("file"));
    assert!(effective.has("http"));
    assert!(effective.has("websocket"));
    assert_eq!(effective.len(), 4);
}

#[test]
fn test_role_inclusion_commutativity() {
    let mut role_defs = HashMap::new();
    role_defs.insert(
        "role_a".to_string(),
        create_role_def("role_a", vec!["cap_a"]),
    );
    role_defs.insert(
        "role_b".to_string(),
        create_role_def("role_b", vec!["cap_b"]),
    );

    let checker = RoleChecker::new(&role_defs);

    // Order 1: role_a, role_b
    let workflow1 = create_workflow_def_with_roles(vec!["role_a", "role_b"]);
    let result1 = checker.check_workflow_roles(&workflow1).unwrap();

    // Order 2: role_b, role_a
    let workflow2 = create_workflow_def_with_roles(vec!["role_b", "role_a"]);
    let result2 = checker.check_workflow_roles(&workflow2).unwrap();

    // Should have same capabilities regardless of order
    assert_eq!(result1.len(), result2.len());
    assert!(result1.has("cap_a"));
    assert!(result1.has("cap_b"));
    assert!(result2.has("cap_a"));
    assert!(result2.has("cap_b"));
}

#[test]
fn test_empty_role_inclusion() {
    let role_defs = HashMap::new();
    let checker = RoleChecker::new(&role_defs);
    let workflow = create_workflow_def_with_roles(vec![]);

    let result = checker.check_workflow_roles(&workflow);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_duplicate_role_reference() {
    let mut role_defs = HashMap::new();
    role_defs.insert(
        "ai_agent".to_string(),
        create_role_def("ai_agent", vec!["network"]),
    );

    let checker = RoleChecker::new(&role_defs);
    // Duplicate role reference should be handled gracefully
    let workflow = create_workflow_def_with_roles(vec!["ai_agent", "ai_agent"]);

    let result = checker.check_workflow_roles(&workflow);
    assert!(result.is_ok());

    // Should only count capabilities once
    let effective = result.unwrap();
    assert_eq!(effective.len(), 1);
}

#[test]
fn test_effective_capabilities_get() {
    let effective = EffectiveCapabilities::new();

    // Should return None for non-existent capability
    assert!(effective.get("nonexistent").is_none());

    // Add a capability and verify we can retrieve it
    let _cap = CapabilityDecl {
        capability: "test_cap".into(),
        constraints: None,
        span: test_span(),
    };
    // We can only add via RoleChecker, so let's test via that path
    let mut role_defs = HashMap::new();
    role_defs.insert(
        "test_role".to_string(),
        create_role_def("test_role", vec!["test_cap"]),
    );

    let checker = RoleChecker::new(&role_defs);
    let workflow = create_workflow_def_with_roles(vec!["test_role"]);
    let effective = checker.check_workflow_roles(&workflow).unwrap();

    assert!(effective.get("test_cap").is_some());
    assert_eq!(
        effective.get("test_cap").unwrap().capability,
        "test_cap".into()
    );
}

#[test]
fn test_effective_capabilities_is_empty() {
    let role_defs = HashMap::new();
    let checker = RoleChecker::new(&role_defs);

    // Empty workflow should have empty capabilities
    let workflow = create_workflow_def_with_roles(vec![]);
    let effective = checker.check_workflow_roles(&workflow).unwrap();
    assert!(effective.is_empty());

    // Workflow with role should have non-empty capabilities
    let mut role_defs = HashMap::new();
    role_defs.insert(
        "test_role".to_string(),
        create_role_def("test_role", vec!["cap1"]),
    );
    let checker = RoleChecker::new(&role_defs);
    let workflow = create_workflow_def_with_roles(vec!["test_role"]);
    let effective = checker.check_workflow_roles(&workflow).unwrap();
    assert!(!effective.is_empty());
}

#[test]
fn test_effective_capabilities_capability_names() {
    let mut role_defs = HashMap::new();
    role_defs.insert(
        "test_role".to_string(),
        create_role_def("test_role", vec!["cap1", "cap2", "cap3"]),
    );

    let checker = RoleChecker::new(&role_defs);
    let workflow = create_workflow_def_with_roles(vec!["test_role"]);
    let effective = checker.check_workflow_roles(&workflow).unwrap();

    let names: Vec<_> = effective.capability_names().collect();
    assert_eq!(names.len(), 3);
    assert!(names.contains(&&"cap1".to_string()));
    assert!(names.contains(&&"cap2".to_string()));
    assert!(names.contains(&&"cap3".to_string()));
}

#[test]
fn test_role_checker_has_role() {
    let mut role_defs = HashMap::new();
    role_defs.insert(
        "existing_role".to_string(),
        create_role_def("existing_role", vec![]),
    );

    let checker = RoleChecker::new(&role_defs);

    assert!(checker.has_role("existing_role"));
    assert!(!checker.has_role("nonexistent_role"));
}

#[test]
fn test_role_checker_available_roles() {
    let mut role_defs = HashMap::new();
    role_defs.insert("role1".to_string(), create_role_def("role1", vec![]));
    role_defs.insert("role2".to_string(), create_role_def("role2", vec![]));
    role_defs.insert("role3".to_string(), create_role_def("role3", vec![]));

    let checker = RoleChecker::new(&role_defs);
    let roles: Vec<_> = checker.available_roles().collect();

    assert_eq!(roles.len(), 3);
}

#[test]
fn test_complex_role_composition() {
    // Test a realistic scenario with multiple roles
    let mut role_defs = HashMap::new();

    // AI agent role with AI-related capabilities
    role_defs.insert(
        "ai_agent".to_string(),
        RoleDef {
            name: "ai_agent".into(),
            capabilities: vec![
                CapabilityDecl { capability: "llm".into(), constraints: None, span: test_span() },
                CapabilityDecl { capability: "embedding".into(), constraints: None, span: test_span() },
                CapabilityDecl { capability: "vector_store".into(), constraints: None, span: test_span() },
            ],
            obligations: vec!["response_safety".into()],
            span: test_span(),
        },
    );

    // Network client role with network capabilities
    role_defs.insert(
        "network_client".to_string(),
        RoleDef {
            name: "network_client".into(),
            capabilities: vec![
                CapabilityDecl { capability: "http".into(), constraints: None, span: test_span() },
                CapabilityDecl { capability: "websocket".into(), constraints: None, span: test_span() },
                CapabilityDecl { capability: "tls".into(), constraints: None, span: test_span() },
            ],
            obligations: vec![],
            span: test_span(),
        },
    );

    // File processor role with file capabilities
    role_defs.insert(
        "file_processor".to_string(),
        RoleDef {
            name: "file_processor".into(),
            capabilities: vec![
                CapabilityDecl { capability: "file_read".into(), constraints: None, span: test_span() },
                CapabilityDecl { capability: "file_write".into(), constraints: None, span: test_span() },
            ],
            obligations: vec!["audit_log".into()],
            span: test_span(),
        },
    );

    let checker = RoleChecker::new(&role_defs);
    let workflow =
        create_workflow_def_with_roles(vec!["ai_agent", "network_client", "file_processor"]);

    let result = checker.check_workflow_roles(&workflow);
    assert!(result.is_ok());

    let effective = result.unwrap();
    assert_eq!(effective.len(), 8); // 3 + 3 + 2 capabilities

    // Verify all expected capabilities are present
    assert!(effective.has("llm"));
    assert!(effective.has("embedding"));
    assert!(effective.has("vector_store"));
    assert!(effective.has("http"));
    assert!(effective.has("websocket"));
    assert!(effective.has("tls"));
    assert!(effective.has("file_read"));
    assert!(effective.has("file_write"));
}

#[test]
fn test_partial_unknown_roles() {
    // Test when some roles exist and some don't
    let mut role_defs = HashMap::new();
    role_defs.insert(
        "known_role".to_string(),
        create_role_def("known_role", vec!["cap1"]),
    );

    let checker = RoleChecker::new(&role_defs);
    let workflow = create_workflow_def_with_roles(vec!["known_role", "unknown_role"]);

    let result = checker.check_workflow_roles(&workflow);
    assert!(result.is_err());

    match result.unwrap_err() {
        RoleCheckError::UnknownRole { name, .. } => {
            assert_eq!(name, "unknown_role");
        }
        _ => panic!("Expected UnknownRole error"),
    }
}
