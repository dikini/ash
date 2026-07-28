//! Integration Tests for Role Runtime Semantics (TASK-304)
//!
//! Integration tests for role runtime semantics per SPEC-019.

use ash_core::{Capability, Effect, Role, RoleObligationRef, Value};
use ash_engine::{Engine, HttpConfig};
use ash_parser::surface::{CapabilityDecl, RoleDef, RoleRef};
use ash_parser::token::Span;
use ash_runtime::role_context::DischargeError;
use ash_runtime::{
    CapabilityError, CapabilityGrant, ExecError, RoleContext, RoleError, RoleRegistry,
    RuntimeCapabilitySet,
};

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

fn role_refs(roles: Vec<&str>) -> Vec<RoleRef> {
    roles
        .into_iter()
        .map(|role| RoleRef {
            name: role.into(),
            span: test_span(),
        })
        .collect()
}

fn resolve_role_refs(registry: &RoleRegistry, roles: Vec<&str>) -> RuntimeCapabilitySet {
    let refs = role_refs(roles);
    registry.resolve_role_bindings(&refs, &[]).unwrap()
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

const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

/// Role/provider construction remains observable at the source boundary, but generic source
/// execution must reject until it carries a validated checked Core/CPS admission.
async fn assert_generic_source_execution_rejects_closed(engine: &Engine) {
    let mut application = engine.parse("fn main() { 42 }").expect("source parses");
    engine.check(&mut application).expect("source typechecks");

    let error = engine
        .execute(&application)
        .await
        .expect_err("generic source execution must reject without checked Core/CPS admission");
    assert!(
        matches!(error, ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR),
        "generic source execution must expose the exact canonical closed-admission error"
    );
}

// ============================================================
// Role Assignment Tests
// ============================================================

#[test]
fn test_assign_role_to_entry_at_runtime() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def(
        "admin",
        vec!["read", "write", "delete"],
    ));

    let caps = resolve_role_refs(&registry, vec!["admin"]);

    assert!(caps.has_capability("read"));
    assert!(caps.has_capability("write"));
    assert!(caps.has_capability("delete"));
}

#[test]
fn test_assign_single_role() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("user", vec!["read"]));

    let caps = resolve_role_refs(&registry, vec!["user"]);

    assert!(caps.has_capability("read"));
    assert!(!caps.has_capability("write"));
}

#[test]
fn test_assign_multiple_roles() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("reader", vec!["read"]));
    registry.register(create_test_role_def("writer", vec!["write"]));

    let caps = resolve_role_refs(&registry, vec!["reader", "writer"]);

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

    let caps = resolve_role_refs(&registry, vec!["superuser"]);

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

    let caps = resolve_role_refs(&registry, vec!["db_admin", "file_admin"]);

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

    let base_caps = resolve_role_refs(&registry, vec!["base"]);
    let extended_caps = resolve_role_refs(&registry, vec!["extended"]);

    assert!(base_caps.has_capability("read"));
    assert!(!base_caps.has_capability("write"));

    assert!(extended_caps.has_capability("read"));
    assert!(extended_caps.has_capability("write"));
}

#[test]
fn test_role_with_no_authority() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("observer", vec![]));

    let caps = resolve_role_refs(&registry, vec!["observer"]);

    assert!(caps.is_empty());
    assert_eq!(caps.len(), 0);
}

#[test]
fn test_role_with_obligations() {
    let mut registry = RoleRegistry::new();
    let mut role = create_test_role_def("audited_user", vec!["read", "write"]);
    role.obligations = vec!["audit".into(), "log".into()];
    registry.register(role);

    let caps = resolve_role_refs(&registry, vec!["audited_user"]);

    assert!(caps.has_capability("read"));
    assert!(caps.has_capability("write"));
}

#[test]
fn test_unknown_role_assignment_error() {
    let registry = RoleRegistry::new();
    let refs = role_refs(vec!["nonexistent"]);
    let result = registry.resolve_role_bindings(&refs, &[]);

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
fn test_entry_without_required_capability_fails() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("limited", vec!["read"]));

    let caps = resolve_role_refs(&registry, vec!["limited"]);

    let result = caps.check_use("write", "modify", &Value::Null);
    assert!(result.is_err());
    assert!(matches!(result, Err(CapabilityError::NotGranted)));
}

#[test]
fn test_entry_with_sufficient_capabilities_succeeds() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def(
        "empowered",
        vec!["read", "write", "execute"],
    ));

    let caps = resolve_role_refs(&registry, vec!["empowered"]);

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

    let user_caps = resolve_role_refs(&registry, vec!["user"]);
    assert!(!user_caps.has_capability("write"));

    let admin_caps = resolve_role_refs(&registry, vec!["admin"]);
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

    let privileged_caps = resolve_role_refs(&registry, vec!["privileged"]);
    assert!(privileged_caps.has_capability("admin"));

    let restricted_caps = resolve_role_refs(&registry, vec!["restricted"]);
    assert!(!restricted_caps.has_capability("admin"));
}

#[test]
fn test_capability_use_with_granted_capability() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("operator", vec!["sensor"]));

    let caps = resolve_role_refs(&registry, vec!["operator"]);

    let result = caps.check_use("sensor", "read", &Value::Null);
    assert!(result.is_ok());
}

#[test]
fn test_capability_use_without_grant() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("basic", vec!["a"]));

    let caps = resolve_role_refs(&registry, vec!["basic"]);

    let result = caps.check_use("b", "operate", &Value::Null);
    assert!(matches!(result, Err(CapabilityError::NotGranted)));
}

#[test]
fn test_multiple_roles_combine_authority() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("role_a", vec!["cap_a"]));
    registry.register(create_test_role_def("role_b", vec!["cap_b"]));
    registry.register(create_test_role_def("role_c", vec!["cap_c"]));

    let caps = resolve_role_refs(&registry, vec!["role_a", "role_b", "role_c"]);

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

    let caps = resolve_role_refs(&registry, vec!["queried"]);

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

    let caps = resolve_role_refs(&registry, vec!["grantor"]);

    let grant = caps.get_grant("shared_cap").unwrap();
    assert_eq!(grant.capability, "shared_cap");
    assert_eq!(grant.granted_by, vec!["grantor"]);
}

#[test]
fn test_multiple_roles_grant_same_capability() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("admin", vec!["delete"]));
    registry.register(create_test_role_def("moderator", vec!["delete"]));

    let caps = resolve_role_refs(&registry, vec!["admin", "moderator"]);

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
// Explicit Entry Capabilities
// ============================================================

#[test]
fn test_explicit_capability_declaration() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("base", vec!["read"]));

    let refs = role_refs(vec!["base"]);
    let explicit_capabilities = vec![CapabilityDecl {
        capability: "custom".into(),
        constraints: None,
        span: test_span(),
    }];

    let caps = registry
        .resolve_role_bindings(&refs, &explicit_capabilities)
        .unwrap();

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

    let guest_caps = resolve_role_refs(&registry, vec!["guest"]);
    let user_caps = resolve_role_refs(&registry, vec!["user"]);
    let mod_caps = resolve_role_refs(&registry, vec!["moderator"]);
    let admin_caps = resolve_role_refs(&registry, vec!["admin"]);

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

    let caps = resolve_role_refs(&registry, vec!["reader", "writer"]);

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

    let caps = resolve_role_refs(&registry, vec!["db_user", "api_user"]);

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

    let refs = role_refs(vec!["to_remove"]);
    let result = registry.resolve_role_bindings(&refs, &[]);
    assert!(result.is_err());
}

#[test]
fn test_role_registry_update_role() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("updatable", vec!["old_cap"]));
    registry.register(create_test_role_def("updatable", vec!["new_cap"]));

    let caps = resolve_role_refs(&registry, vec!["updatable"]);

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
    assert_generic_source_execution_rejects_closed(&engine).await;
}

#[tokio::test]
async fn test_engine_with_stdio_role_simulation() {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .build()
        .expect("engine builds");

    assert_generic_source_execution_rejects_closed(&engine).await;
}

#[tokio::test]
async fn test_engine_with_fs_role_simulation() {
    let engine = Engine::new()
        .with_fs_capabilities()
        .build()
        .expect("engine builds");

    assert_generic_source_execution_rejects_closed(&engine).await;
}

#[tokio::test]
async fn test_engine_with_stdio_fs_capabilities_role_simulation() {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build()
        .expect("engine builds");

    assert_generic_source_execution_rejects_closed(&engine).await;
}

#[test]
fn test_engine_with_http_capabilities_registers_provider() {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .with_http_capabilities(HttpConfig::new())
        .build()
        .expect("engine builds with HTTP capabilities");

    assert!(
        engine.has_provider("http"),
        "Engine should register HTTP provider when HTTP capabilities requested"
    );
}

// ============================================================
// Edge Cases
// ============================================================

#[test]
fn test_role_with_special_characters() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("my-role_v1.0", vec!["cap"]));

    let caps = resolve_role_refs(&registry, vec!["my-role_v1.0"]);

    assert!(caps.has_capability("cap"));
}

#[test]
fn test_capability_with_special_characters() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def(
        "test",
        vec!["db:read", "fs/write", "http.get"],
    ));

    let caps = resolve_role_refs(&registry, vec!["test"]);

    assert!(caps.has_capability("db:read"));
    assert!(caps.has_capability("fs/write"));
    assert!(caps.has_capability("http.get"));
}

#[test]
fn test_empty_role_name() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role_def("", vec!["cap"]));

    let caps = resolve_role_refs(&registry, vec![""]);

    assert!(caps.has_capability("cap"));
}

#[test]
fn test_role_context_clone() {
    let role = create_test_role("cloneable", vec!["a", "b"], vec!["obl"]);
    let ctx = RoleContext::new(role);

    ctx.discharge("obl").unwrap();

    let cloned = ctx.clone();
    // Both original and clone should see the discharged obligation
    assert!(ctx.is_discharged("obl"));
    assert!(cloned.is_discharged("obl"));
    assert!(cloned.can_access(&Capability {
        name: "a".to_string(),
        effect: Effect::Operational,
        constraints: vec![],
    }));
}
