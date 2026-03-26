//! Integration tests for runtime role resolution
//!
//! Tests role resolution, capability grants, and use checking.

use ash_core::Value;
use ash_interp::role_runtime::{
    CapabilityError, CapabilityGrant, RoleError, RoleRegistry, RuntimeCapabilitySet,
};
use ash_parser::surface::{
    CapabilityDecl, ConstraintBlock, ConstraintField, ConstraintValue, RoleDef, RoleRef,
    WorkflowDef,
};
use ash_parser::token::Span;

fn test_span() -> Span {
    Span::default()
}

fn create_test_role(name: &str, capabilities: Vec<&str>) -> RoleDef {
    RoleDef {
        name: name.into(),
        authority: capabilities.into_iter().map(|c| c.into()).collect(),
        obligations: vec![],
        span: test_span(),
    }
}

fn create_test_workflow(name: &str, plays_roles: Vec<&str>) -> WorkflowDef {
    WorkflowDef {
        name: name.into(),
        params: vec![],
        plays_roles: plays_roles
            .into_iter()
            .map(|r| RoleRef {
                name: r.into(),
                span: test_span(),
            })
            .collect(),
        capabilities: vec![],
        body: ash_parser::surface::Workflow::Done { span: test_span() },
        contract: None,
        span: test_span(),
    }
}

/// Test that a role can be resolved to its capabilities
#[test]
fn test_role_resolution() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role("ai_agent", vec!["file", "process"]));

    let workflow = create_test_workflow("test", vec!["ai_agent"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("file"));
    assert!(caps.has_capability("process"));
    assert!(!caps.has_capability("network"));
}

/// Test that an unknown role produces an error
#[test]
fn test_unknown_role_error() {
    let registry = RoleRegistry::new();

    let workflow = create_test_workflow("test", vec!["nonexistent"]);

    let result = registry.resolve_workflow_roles(&workflow);
    assert!(result.is_err());

    match result {
        Err(RoleError::UnknownRole { name }) => {
            assert_eq!(name, "nonexistent");
        }
        _ => panic!("Expected UnknownRole error, got {:?}", result),
    }
}

/// Test that multiple roles combine their capabilities
#[test]
fn test_multiple_roles_combined() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role("file_user", vec!["file"]));
    registry.register(create_test_role("net_user", vec!["network"]));

    let workflow = create_test_workflow("test", vec!["file_user", "net_user"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("file"));
    assert!(caps.has_capability("network"));
    assert_eq!(caps.len(), 2);
}

/// Test capability use checking
#[test]
fn test_capability_use_check() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role("file_user", vec!["file"]));

    let workflow = create_test_workflow("test", vec!["file_user"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    // Should succeed for granted capability
    assert!(caps.check_use("file", "read", &Value::Null).is_ok());

    // Should fail for ungranted capability
    let result = caps.check_use("network", "get", &Value::Null);
    assert!(result.is_err());

    match result {
        Err(CapabilityError::NotGranted) => (), // Expected
        _ => panic!("Expected NotGranted error, got {:?}", result),
    }
}

/// Test that capability grants track which roles granted them
#[test]
fn test_capability_grant_tracking() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role("admin", vec!["file"]));
    registry.register(create_test_role("user", vec!["file"]));

    let workflow = create_test_workflow("test", vec!["admin", "user"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    let grant = caps.get_grant("file").unwrap();
    assert_eq!(grant.granted_by.len(), 2);
    assert!(grant.granted_by.contains(&"admin".to_string()));
    assert!(grant.granted_by.contains(&"user".to_string()));
}

/// Test that explicit workflow capabilities are included
#[test]
fn test_explicit_workflow_capabilities() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role("base", vec!["file"]));

    let mut workflow = create_test_workflow("test", vec!["base"]);
    workflow.capabilities.push(CapabilityDecl {
        capability: "network".into(),
        constraints: None,
        span: test_span(),
    });

    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("file")); // From role
    assert!(caps.has_capability("network")); // From explicit declaration
}

/// Test capability grants with constraints
#[test]
fn test_capability_with_constraints() {
    let mut caps = RuntimeCapabilitySet::new();

    let constraint_block = ConstraintBlock {
        fields: vec![ConstraintField {
            name: "paths".into(),
            value: ConstraintValue::Array(vec![ConstraintValue::String("/tmp/*".to_string())]),
            span: test_span(),
        }],
        span: test_span(),
    };

    let decl = CapabilityDecl {
        capability: "file".into(),
        constraints: Some(constraint_block.clone()),
        span: test_span(),
    };

    caps.grant(&decl).unwrap();

    let grant = caps.get_grant("file").unwrap();
    assert_eq!(grant.capability, "file");
    assert!(grant.constraints.is_some());
}

/// Test error messages are descriptive
#[test]
fn test_error_messages() {
    let role_err = RoleError::UnknownRole {
        name: "missing_role".to_string(),
    };
    assert!(role_err.to_string().contains("missing_role"));

    let cap_err = CapabilityError::NotGranted;
    assert!(cap_err.to_string().contains("not granted"));

    let constraint_err = CapabilityError::ConstraintViolation {
        reason: "path denied".to_string(),
    };
    assert!(constraint_err.to_string().contains("path denied"));
}

/// Test empty role resolution
#[test]
fn test_empty_role_resolution() {
    let registry = RoleRegistry::new();
    let workflow = create_test_workflow("test", vec![]);

    let caps = registry.resolve_workflow_roles(&workflow).unwrap();
    assert!(caps.is_empty());
}

/// Test role with no authority grants no capabilities
#[test]
fn test_role_with_no_authority() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role("powerless", vec![]));

    let workflow = create_test_workflow("test", vec!["powerless"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(!caps.has_capability("anything"));
    assert!(caps.is_empty());
}

/// Test role registry operations
#[test]
fn test_role_registry_operations() {
    let mut registry = RoleRegistry::new();

    // Register roles
    registry.register(create_test_role("role1", vec!["cap1"]));
    registry.register(create_test_role("role2", vec!["cap2"]));

    assert_eq!(registry.len(), 2);
    assert!(registry.is_registered("role1"));
    assert!(registry.is_registered("role2"));

    // Unregister a role
    let removed = registry.unregister("role1");
    assert!(removed.is_some());
    assert_eq!(registry.len(), 1);
    assert!(!registry.is_registered("role1"));

    // Get role
    let role = registry.get_role("role2");
    assert!(role.is_some());
    assert_eq!(role.unwrap().name, "role2".into());

    // Get nonexistent role
    assert!(registry.get_role("nonexistent").is_none());
}

/// Test merging capability grants
#[test]
fn test_capability_grant_merge() {
    let mut grant = CapabilityGrant::new("file".to_string());

    // First merge with constraints
    let decl1 = CapabilityDecl {
        capability: "file".into(),
        constraints: Some(ConstraintBlock {
            fields: vec![ConstraintField {
                name: "read".into(),
                value: ConstraintValue::Bool(true),
                span: test_span(),
            }],
            span: test_span(),
        }),
        span: test_span(),
    };

    grant.merge(&decl1).unwrap();
    assert!(grant.constraints.is_some());

    // Second merge should also succeed
    let decl2 = CapabilityDecl {
        capability: "file".into(),
        constraints: None,
        span: test_span(),
    };

    grant.merge(&decl2).unwrap();
}

/// Test RuntimeCapabilitySet operations
#[test]
fn test_capability_set_operations() {
    let mut caps = RuntimeCapabilitySet::new();
    assert!(caps.is_empty());
    assert_eq!(caps.len(), 0);

    // Grant by name
    caps.grant_by_name("file", "admin");
    assert!(!caps.is_empty());
    assert_eq!(caps.len(), 1);
    assert!(caps.has_capability("file"));

    // Get granted capabilities list
    let granted = caps.granted_capabilities();
    assert_eq!(granted.len(), 1);
    assert!(granted.contains(&&"file".to_string()));

    // Get nonexistent grant
    assert!(caps.get_grant("network").is_none());
}
