//! Integration Tests for Role Runtime Semantics (TASK-304)
//!
//! Integration tests for role runtime semantics per SPEC-019.

use ash_core::{Capability, Effect, Role, RoleObligationRef, Value};
use ash_engine::{Engine, HttpConfig};
use ash_interp::role_context::DischargeError;
use ash_interp::{
    CapabilityError, CapabilityGrant, RoleContext, RoleError, RoleRegistry, RuntimeCapabilitySet,
};
use ash_parser::surface::{CapabilityDecl, RoleDef, RoleRef, WorkflowDef};
use ash_parser::token::Span;

// ============================================================
// Test Helpers
// ============================================================

fn test_span() -> Span {
    Span::default()
}

fn create_test_role_def(name: &str, capabilities: Vec<&str>) -> RoleDef {
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

fn create_test_workflow_def(name: &str, plays_roles: Vec<&str>) -> WorkflowDef {
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

fn create_test_role(name: &str, authority: Vec<&str>, obligations: Vec<&str>) -> Role {
    Role {
        name: name.to_string(),
        authority: authority
            .into_iter()
            .map(|c| Capability {
                name: c.to_string(),
                effect: Effect::Operational,
                constraints: vec![],
            })
            .collect(),
        obligations: obligations
            .into_iter()
            .map(|o| RoleObligationRef {
                name: o.to_string(),
            })
            .collect(),
    }
}

// ============================================================
// Role Assignment Tests
// ============================================================

#[test]
fn test_assign_role_to_workflow_at_runtime() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def(
        "admin",
        vec!["read", "write", "delete"],
    ));

    let workflow = create_test_workflow_def("secure_workflow", vec!["admin"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("read"));
    assert!(caps.has_capability("write"));
    assert!(caps.has_capability("delete"));
}

#[test]
fn test_assign_single_role() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("user", vec!["read"]));

    let workflow = create_test_workflow_def("reader", vec!["user"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("read"));
    assert!(!caps.has_capability("write"));
}

#[test]
fn test_assign_multiple_roles() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("reader", vec!["read"]));
    registry.register(create_test_role_def("writer", vec!["write"]));

    let workflow = create_test_workflow_def("rw_workflow", vec!["reader", "writer"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("read"));
    assert!(caps.has_capability("write"));
}

#[test]
fn test_verify_role_capability_set() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def(
        "superuser",
        vec!["read", "write", "execute", "admin"],
    ));

    let workflow = create_test_workflow_def("admin_workflow", vec!["superuser"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    let granted = caps.granted_capabilities();
    assert_eq!(granted.len(), 4);
    assert!(granted.contains(&&"read".to_string()));
    assert!(granted.contains(&&"write".to_string()));
    assert!(granted.contains(&&"execute".to_string()));
    assert!(granted.contains(&&"admin".to_string()));
}

#[test]
fn test_compound_roles_capability_union() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def(
        "db_admin",
        vec!["db_read", "db_write"],
    ));
    registry.register(create_test_role_def(
        "file_admin",
        vec!["file_read", "file_write"],
    ));

    let workflow = create_test_workflow_def("compound", vec!["db_admin", "file_admin"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("db_read"));
    assert!(caps.has_capability("db_write"));
    assert!(caps.has_capability("file_read"));
    assert!(caps.has_capability("file_write"));
}

#[test]
fn test_role_inheritance_simulation() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("base", vec!["read"]));
    registry.register(create_test_role_def("extended", vec!["read", "write"]));

    let base_workflow = create_test_workflow_def("base_wf", vec!["base"]);
    let extended_workflow = create_test_workflow_def("extended_wf", vec!["extended"]);

    let base_caps = registry.resolve_workflow_roles(&base_workflow).unwrap();
    let extended_caps = registry.resolve_workflow_roles(&extended_workflow).unwrap();

    assert!(base_caps.has_capability("read"));
    assert!(!base_caps.has_capability("write"));

    assert!(extended_caps.has_capability("read"));
    assert!(extended_caps.has_capability("write"));
}

#[test]
fn test_role_with_no_authority() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("observer", vec![]));

    let workflow = create_test_workflow_def("observer_wf", vec!["observer"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.is_empty());
    assert_eq!(caps.len(), 0);
}

#[test]
fn test_role_with_obligations() {
    let mut registry = RoleRegistry::new();
    let mut role = create_test_role_def("audited_user", vec!["read", "write"]);
    role.obligations = vec!["audit".into(), "log".into()];
    registry.register(role);

    let workflow = create_test_workflow_def("audited", vec!["audited_user"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("read"));
    assert!(caps.has_capability("write"));
}

#[test]
fn test_unknown_role_assignment_error() {
    let registry = RoleRegistry::new();
    let workflow = create_test_workflow_def("unknown_role", vec!["nonexistent"]);
    let result = registry.resolve_workflow_roles(&workflow);

    assert!(result.is_err());
    match result {
        Err(RoleError::UnknownRole { name }) => {
            assert_eq!(name, "nonexistent");
        }
        _ => panic!("Expected UnknownRole error"),
    }
}

// ============================================================
// Role Enforcement Tests
// ============================================================

#[test]
fn test_workflow_without_required_capability_fails() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("limited", vec!["read"]));

    let workflow = create_test_workflow_def("limited_wf", vec!["limited"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    let result = caps.check_use("write", "modify", &Value::Null);
    assert!(result.is_err());
    assert!(matches!(result, Err(CapabilityError::NotGranted)));
}

#[test]
fn test_workflow_with_sufficient_capabilities_succeeds() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def(
        "empowered",
        vec!["read", "write", "execute"],
    ));

    let workflow = create_test_workflow_def("empowered_wf", vec!["empowered"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.check_use("read", "query", &Value::Null).is_ok());
    assert!(caps.check_use("write", "modify", &Value::Null).is_ok());
    assert!(caps.check_use("execute", "run", &Value::Null).is_ok());
}

#[test]
fn test_role_upgrade_scenario() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("user", vec!["read"]));
    registry.register(create_test_role_def(
        "admin",
        vec!["read", "write", "delete"],
    ));

    let user_workflow = create_test_workflow_def("user_wf", vec!["user"]);
    let user_caps = registry.resolve_workflow_roles(&user_workflow).unwrap();
    assert!(!user_caps.has_capability("write"));

    let admin_workflow = create_test_workflow_def("admin_wf", vec!["admin"]);
    let admin_caps = registry.resolve_workflow_roles(&admin_workflow).unwrap();
    assert!(admin_caps.has_capability("write"));
}

#[test]
fn test_role_downgrade_scenario() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def(
        "privileged",
        vec!["read", "write", "admin"],
    ));
    registry.register(create_test_role_def("restricted", vec!["read"]));

    let privileged_workflow = create_test_workflow_def("priv_wf", vec!["privileged"]);
    let privileged_caps = registry
        .resolve_workflow_roles(&privileged_workflow)
        .unwrap();
    assert!(privileged_caps.has_capability("admin"));

    let restricted_workflow = create_test_workflow_def("rest_wf", vec!["restricted"]);
    let restricted_caps = registry
        .resolve_workflow_roles(&restricted_workflow)
        .unwrap();
    assert!(!restricted_caps.has_capability("admin"));
}

#[test]
fn test_capability_use_with_granted_capability() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("operator", vec!["sensor"]));

    let workflow = create_test_workflow_def("operator_wf", vec!["operator"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    let result = caps.check_use("sensor", "read", &Value::Null);
    assert!(result.is_ok());
}

#[test]
fn test_capability_use_without_grant() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("basic", vec!["a"]));

    let workflow = create_test_workflow_def("basic_wf", vec!["basic"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    let result = caps.check_use("b", "operate", &Value::Null);
    assert!(matches!(result, Err(CapabilityError::NotGranted)));
}

#[test]
fn test_multiple_roles_combine_authority() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("role_a", vec!["cap_a"]));
    registry.register(create_test_role_def("role_b", vec!["cap_b"]));
    registry.register(create_test_role_def("role_c", vec!["cap_c"]));

    let workflow = create_test_workflow_def("combined", vec!["role_a", "role_b", "role_c"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.check_use("cap_a", "use", &Value::Null).is_ok());
    assert!(caps.check_use("cap_b", "use", &Value::Null).is_ok());
    assert!(caps.check_use("cap_c", "use", &Value::Null).is_ok());
}

// ============================================================
// Role Context Tests (Obligations and Authority)
// ============================================================

#[test]
fn test_role_context_can_access_with_authority() {
    let role = create_test_role("test_role", vec!["sensor", "actuator"], vec![]);
    let ctx = RoleContext::new(role);

    let sensor_cap = Capability {
        name: "sensor".to_string(),
        effect: Effect::Epistemic,
        constraints: vec![],
    };
    assert!(ctx.can_access(&sensor_cap));

    let unknown_cap = Capability {
        name: "unknown".to_string(),
        effect: Effect::Epistemic,
        constraints: vec![],
    };
    assert!(!ctx.can_access(&unknown_cap));
}

#[test]
fn test_role_context_obligation_discharge() {
    let role = create_test_role("obligated", vec!["cap"], vec!["audit", "log"]);
    let ctx = RoleContext::new(role);

    assert!(!ctx.is_discharged("audit"));
    assert!(!ctx.is_discharged("log"));

    assert!(ctx.discharge("audit").is_ok());
    assert!(ctx.is_discharged("audit"));
    assert!(!ctx.is_discharged("log"));

    assert!(ctx.discharge("log").is_ok());
    assert!(ctx.is_discharged("log"));
    assert!(ctx.all_discharged());
}

#[test]
fn test_role_context_discharge_linear_semantics() {
    let role = create_test_role("linear", vec![], vec!["obligation"]);
    let ctx = RoleContext::new(role);

    assert!(ctx.discharge("obligation").is_ok());
    assert_eq!(
        ctx.discharge("obligation"),
        Err(DischargeError::AlreadyDischarged)
    );
}

#[test]
fn test_role_context_undeclared_discharge_fails() {
    let role = create_test_role("simple", vec![], vec![]);
    let ctx = RoleContext::new(role);

    assert_eq!(
        ctx.discharge("unknown"),
        Err(DischargeError::UndeclaredObligation)
    );
}

#[test]
fn test_role_context_pending_obligations() {
    let role = create_test_role("multi_obligated", vec![], vec!["a", "b", "c"]);
    let ctx = RoleContext::new(role);

    let pending = ctx.pending_obligations();
    assert_eq!(pending.len(), 3);

    ctx.discharge("b").unwrap();

    let pending = ctx.pending_obligations();
    assert_eq!(pending.len(), 2);
    assert!(!pending.contains(&"b".to_string()));
}

// ============================================================
// Runtime Role Queries
// ============================================================

#[test]
fn test_query_role_capabilities() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def(
        "queried",
        vec!["cap1", "cap2", "cap3"],
    ));

    let workflow = create_test_workflow_def("queried_wf", vec!["queried"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    let granted = caps.granted_capabilities();
    assert_eq!(granted.len(), 3);

    assert!(caps.get_grant("cap1").is_some());
    assert!(caps.get_grant("cap2").is_some());
    assert!(caps.get_grant("cap3").is_some());
    assert!(caps.get_grant("cap4").is_none());
}

#[test]
fn test_capability_grant_tracking() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("grantor", vec!["shared_cap"]));

    let workflow = create_test_workflow_def("grantee", vec!["grantor"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    let grant = caps.get_grant("shared_cap").unwrap();
    assert_eq!(grant.capability, "shared_cap");
    assert_eq!(grant.granted_by, vec!["grantor"]);
}

#[test]
fn test_multiple_roles_grant_same_capability() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("admin", vec!["delete"]));
    registry.register(create_test_role_def("moderator", vec!["delete"]));

    let workflow = create_test_workflow_def("multi_grant", vec!["admin", "moderator"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    let grant = caps.get_grant("delete").unwrap();
    assert_eq!(grant.granted_by.len(), 2);
    assert!(grant.granted_by.contains(&"admin".to_string()));
    assert!(grant.granted_by.contains(&"moderator".to_string()));
}

#[test]
fn test_role_comparison_equality() {
    let role1 = create_test_role("same", vec!["a", "b"], vec![]);
    let role2 = create_test_role("same", vec!["a", "b"], vec![]);
    let role3 = create_test_role("different", vec!["a"], vec![]);

    assert_eq!(role1.name, role2.name);
    assert_ne!(role1.name, role3.name);
}

#[test]
fn test_role_registry_queries() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("role1", vec!["a"]));
    registry.register(create_test_role_def("role2", vec!["b"]));
    registry.register(create_test_role_def("role3", vec!["c"]));

    assert!(registry.is_registered("role1"));
    assert!(registry.is_registered("role2"));
    assert!(!registry.is_registered("role4"));
    assert_eq!(registry.len(), 3);

    let role = registry.get_role("role2");
    assert!(role.is_some());
    assert_eq!(role.unwrap().name, "role2".into());
}

// ============================================================
// Explicit Workflow Capabilities
// ============================================================

#[test]
fn test_explicit_capability_declaration() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("base", vec!["read"]));

    let mut workflow = create_test_workflow_def("with_explicit", vec!["base"]);
    workflow.capabilities.push(CapabilityDecl {
        capability: "custom".into(),
        constraints: None,
        span: test_span(),
    });

    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("read"));
    assert!(caps.has_capability("custom"));
}

// ============================================================
// Capability Grant Operations
// ============================================================

#[test]
fn test_capability_grant_merge() {
    let mut grant = CapabilityGrant::new("test".to_string());
    grant.add_granting_role("role1".to_string());
    grant.add_granting_role("role2".to_string());

    assert_eq!(grant.granted_by.len(), 2);
    assert!(grant.granted_by.contains(&"role1".to_string()));
    assert!(grant.granted_by.contains(&"role2".to_string()));
}

#[test]
fn test_runtime_capability_set_grant_by_name() {
    let mut caps = RuntimeCapabilitySet::new();

    caps.grant_by_name("cap1", "role1");
    caps.grant_by_name("cap1", "role2");
    caps.grant_by_name("cap2", "role1");

    assert!(caps.has_capability("cap1"));
    assert!(caps.has_capability("cap2"));
    assert!(!caps.has_capability("cap3"));

    let grant = caps.get_grant("cap1").unwrap();
    assert_eq!(grant.granted_by.len(), 2);
}

#[test]
fn test_runtime_capability_set_operations() {
    let mut caps = RuntimeCapabilitySet::new();
    assert!(caps.is_empty());

    let decl = CapabilityDecl {
        capability: "test".into(),
        constraints: None,
        span: test_span(),
    };
    caps.grant(&decl).unwrap();

    assert!(!caps.is_empty());
    assert_eq!(caps.len(), 1);
    assert!(caps.has_capability("test"));
}

// ============================================================
// Error Handling
// ============================================================

#[test]
fn test_role_error_display() {
    let err = RoleError::UnknownRole {
        name: "missing".to_string(),
    };
    assert_eq!(err.to_string(), "unknown role: missing");

    let err = RoleError::IncompatibleGrants {
        capability: "conflict".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "incompatible grants for capability: conflict"
    );
}

#[test]
fn test_capability_error_display() {
    let err = CapabilityError::NotGranted;
    assert_eq!(err.to_string(), "capability not granted");

    let err = CapabilityError::ConstraintViolation {
        reason: "access denied".to_string(),
    };
    assert_eq!(err.to_string(), "constraint violation: access denied");
}

// ============================================================
// Complex Role Scenarios
// ============================================================

#[test]
fn test_role_hierarchy_simulation() {
    let mut registry = RoleRegistry::new();

    registry.register(create_test_role_def("guest", vec!["read_public"]));
    registry.register(create_test_role_def(
        "user",
        vec!["read_public", "read_private", "comment"],
    ));
    registry.register(create_test_role_def(
        "moderator",
        vec!["read_public", "read_private", "comment", "moderate"],
    ));
    registry.register(create_test_role_def(
        "admin",
        vec![
            "read_public",
            "read_private",
            "comment",
            "moderate",
            "administrate",
            "delete",
        ],
    ));

    let guest_wf = create_test_workflow_def("guest_wf", vec!["guest"]);
    let user_wf = create_test_workflow_def("user_wf", vec!["user"]);
    let mod_wf = create_test_workflow_def("mod_wf", vec!["moderator"]);
    let admin_wf = create_test_workflow_def("admin_wf", vec!["admin"]);

    let guest_caps = registry.resolve_workflow_roles(&guest_wf).unwrap();
    let user_caps = registry.resolve_workflow_roles(&user_wf).unwrap();
    let mod_caps = registry.resolve_workflow_roles(&mod_wf).unwrap();
    let admin_caps = registry.resolve_workflow_roles(&admin_wf).unwrap();

    assert!(guest_caps.has_capability("read_public"));
    assert!(!guest_caps.has_capability("comment"));

    assert!(user_caps.has_capability("comment"));
    assert!(!user_caps.has_capability("moderate"));

    assert!(mod_caps.has_capability("moderate"));
    assert!(!mod_caps.has_capability("administrate"));

    assert!(admin_caps.has_capability("administrate"));
    assert!(admin_caps.has_capability("delete"));
}

#[test]
fn test_role_compound_permissions() {
    let mut registry = RoleRegistry::new();

    registry.register(create_test_role_def("reader", vec!["read", "search"]));
    registry.register(create_test_role_def(
        "writer",
        vec!["write", "create", "delete"],
    ));

    let workflow = create_test_workflow_def("full_access", vec!["reader", "writer"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("read"));
    assert!(caps.has_capability("search"));
    assert!(caps.has_capability("write"));
    assert!(caps.has_capability("create"));
    assert!(caps.has_capability("delete"));
}

#[test]
fn test_role_overlap_capabilities() {
    let mut registry = RoleRegistry::new();

    registry.register(create_test_role_def("db_user", vec!["db_read", "db_write"]));
    registry.register(create_test_role_def(
        "api_user",
        vec!["api_read", "api_write", "db_read"],
    ));

    let workflow = create_test_workflow_def("overlap", vec!["db_user", "api_user"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("db_read"));
    assert!(caps.has_capability("db_write"));
    assert!(caps.has_capability("api_read"));
    assert!(caps.has_capability("api_write"));

    let grant = caps.get_grant("db_read").unwrap();
    assert_eq!(grant.granted_by.len(), 2);
}

// ============================================================
// Role Registry Operations
// ============================================================

#[test]
fn test_role_registry_unregister() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("to_remove", vec!["temp"]));

    assert!(registry.is_registered("to_remove"));

    let removed = registry.unregister("to_remove");
    assert!(removed.is_some());
    assert!(!registry.is_registered("to_remove"));

    let workflow = create_test_workflow_def("after_removal", vec!["to_remove"]);
    let result = registry.resolve_workflow_roles(&workflow);
    assert!(result.is_err());
}

#[test]
fn test_role_registry_update_role() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("updatable", vec!["old_cap"]));
    registry.register(create_test_role_def("updatable", vec!["new_cap"]));

    let workflow = create_test_workflow_def("test", vec!["updatable"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("new_cap"));
    assert!(!caps.has_capability("old_cap"));
}

#[test]
fn test_role_registry_empty() {
    let registry = RoleRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

// ============================================================
// Integration with Engine
// ============================================================

#[tokio::test]
async fn test_engine_default_builds_without_roles() {
    let engine = Engine::default();
    let result = engine.run("workflow main { ret 42; }").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(42));
}

#[tokio::test]
async fn test_engine_with_stdio_role_simulation() {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .build()
        .expect("engine builds");

    let result = engine.run("workflow main { ret 42; }").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_engine_with_fs_role_simulation() {
    let engine = Engine::new()
        .with_fs_capabilities()
        .build()
        .expect("engine builds");

    let result = engine.run("workflow main { ret 42; }").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_engine_with_stdio_fs_capabilities_role_simulation() {
    // HTTP provider not yet implemented - test without it
    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build()
        .expect("engine builds");

    let result = engine.run("workflow main { ret 42; }").await;
    assert!(result.is_ok());
}

#[test]
fn test_engine_with_http_capabilities_returns_error() {
    // HTTP provider not yet implemented - should return error
    let result = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .with_http_capabilities(HttpConfig::new())
        .build();

    assert!(
        result.is_err(),
        "Engine should return error when HTTP capabilities requested"
    );
}

// ============================================================
// Edge Cases
// ============================================================

#[test]
fn test_role_with_special_characters() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("my-role_v1.0", vec!["cap"]));

    let workflow = create_test_workflow_def("special", vec!["my-role_v1.0"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("cap"));
}

#[test]
fn test_capability_with_special_characters() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def(
        "test",
        vec!["db:read", "fs/write", "http.get"],
    ));

    let workflow = create_test_workflow_def("special_caps", vec!["test"]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("db:read"));
    assert!(caps.has_capability("fs/write"));
    assert!(caps.has_capability("http.get"));
}

#[test]
fn test_empty_role_name() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("", vec!["cap"]));

    let workflow = create_test_workflow_def("empty_role", vec![""]);
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();

    assert!(caps.has_capability("cap"));
}

#[test]
fn test_role_context_clone() {
    let role = create_test_role("cloneable", vec!["a", "b"], vec!["obl"]);
    let ctx = RoleContext::new(role);

    ctx.discharge("obl").unwrap();

    let cloned = ctx.clone();
    assert!(cloned.is_discharged("obl"));
    assert!(cloned.can_access(&Capability {
        name: "a".to_string(),
        effect: Effect::Operational,
        constraints: vec![],
    }));
}
