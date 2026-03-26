//! Tests for implicit default role generation (TASK-261)
//!
//! These tests verify that `capabilities: [...]` desugars to an implicit role
//! per SPEC-024 Section 5.1.
//!
//! ```ash
//! -- Surface:
//! workflow X capabilities: [C1, C2] { ... }
//!
//! -- Lowered:
//! role X_default { capabilities: [C1, C2] }
//! workflow X plays role(X_default) { ... }
//! ```

use ash_core::Effect;
use ash_parser::input::new_input;
use ash_parser::lower::{LoweredWorkflow, lower_workflow_def};
use ash_parser::parse_workflow::workflow_def;
use winnow::prelude::*;

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_and_lower(input: &str) -> LoweredWorkflow {
    let mut input = new_input(input);
    let workflow_def = workflow_def.parse_next(&mut input).expect("parse failed");
    lower_workflow_def(&workflow_def).expect("lower failed")
}

// ============================================================================
// Success Cases - Basic Implicit Role Generation
// ============================================================================

#[test]
fn test_capabilities_creates_implicit_role() {
    let result = parse_and_lower(r#"workflow processor capabilities: [file] { done }"#);

    // Should have generated implicit role
    let implicit_role = result
        .implicit_role
        .expect("expected implicit role to be generated");
    assert_eq!(implicit_role.name, "processor_default");

    // Role should have the capability
    assert_eq!(implicit_role.authority.len(), 1);
    assert_eq!(implicit_role.authority[0].name, "file");

    // Workflow should play the implicit role
    assert!(
        result
            .plays_roles
            .contains(&"processor_default".to_string())
    );
}

#[test]
fn test_capabilities_multiple_creates_implicit_role() {
    let result =
        parse_and_lower(r#"workflow processor capabilities: [file, network, database] { done }"#);

    let implicit_role = result
        .implicit_role
        .expect("expected implicit role to be generated");
    assert_eq!(implicit_role.name, "processor_default");
    assert_eq!(implicit_role.authority.len(), 3);
    assert_eq!(implicit_role.authority[0].name, "file");
    assert_eq!(implicit_role.authority[1].name, "network");
    assert_eq!(implicit_role.authority[2].name, "database");

    assert!(
        result
            .plays_roles
            .contains(&"processor_default".to_string())
    );
}

// ============================================================================
// Success Cases - Constraints Preserved in Generated Role
// ============================================================================

#[test]
fn test_capabilities_with_constraints_in_role() {
    let result = parse_and_lower(
        r#"workflow api capabilities: [network @ { hosts: ["*.example.com"] }] { done }"#,
    );

    let implicit_role = result
        .implicit_role
        .expect("expected implicit role to be generated");
    assert_eq!(implicit_role.name, "api_default");

    // Capability should have constraints
    assert_eq!(implicit_role.authority.len(), 1);
    let capability = &implicit_role.authority[0];
    assert_eq!(capability.name, "network");
    assert!(
        !capability.constraints.is_empty(),
        "expected constraints to be preserved"
    );
}

#[test]
fn test_capabilities_with_multiple_constraints_in_role() {
    let result = parse_and_lower(
        r#"workflow processor capabilities: [file @ { paths: ["/tmp/*"], read: true }] { done }"#,
    );

    let implicit_role = result
        .implicit_role
        .expect("expected implicit role to be generated");
    assert_eq!(implicit_role.name, "processor_default");

    // Should have multiple constraints
    let capability = &implicit_role.authority[0];
    assert_eq!(capability.constraints.len(), 2);
}

// ============================================================================
// Success Cases - Works Alongside Explicit plays role
// ============================================================================

#[test]
fn test_explicit_roles_preserved() {
    let result = parse_and_lower(
        r#"workflow processor plays role(ai_agent) capabilities: [network] { done }"#,
    );

    // Should have both explicit and implicit roles
    assert!(
        result.plays_roles.contains(&"ai_agent".to_string()),
        "explicit role should be preserved"
    );
    assert!(
        result
            .plays_roles
            .contains(&"processor_default".to_string()),
        "implicit role should be added"
    );

    let implicit_role = result
        .implicit_role
        .expect("expected implicit role to be generated");
    assert_eq!(implicit_role.name, "processor_default");
}

#[test]
fn test_multiple_explicit_roles_with_capabilities() {
    let result = parse_and_lower(
        r#"workflow processor plays role(supervisor) plays role(validator) capabilities: [file, network] { done }"#,
    );

    // Should have all roles
    assert!(result.plays_roles.contains(&"supervisor".to_string()));
    assert!(result.plays_roles.contains(&"validator".to_string()));
    assert!(
        result
            .plays_roles
            .contains(&"processor_default".to_string())
    );
    assert_eq!(result.plays_roles.len(), 3);
}

// ============================================================================
// Success Cases - No Capabilities, No Implicit Role
// ============================================================================

#[test]
fn test_no_capabilities_no_implicit_role() {
    let result = parse_and_lower(r#"workflow simple { done }"#);

    // No implicit role should be generated
    assert!(
        result.implicit_role.is_none(),
        "expected no implicit role when no capabilities"
    );
    assert!(
        result.plays_roles.is_empty(),
        "expected no plays_roles when no explicit roles or capabilities"
    );
}

#[test]
fn test_explicit_role_without_capabilities() {
    let result = parse_and_lower(r#"workflow simple plays role(agent) { done }"#);

    // No implicit role should be generated, but explicit role preserved
    assert!(result.implicit_role.is_none());
    assert_eq!(result.plays_roles.len(), 1);
    assert!(result.plays_roles.contains(&"agent".to_string()));
}

// ============================================================================
// Success Cases - Name Collision Handling
// ============================================================================

#[test]
fn test_name_collision_handling() {
    // This test verifies the naming scheme works correctly.
    // The current implementation uses simple "{name}_default" naming.
    // If a role with that name already exists from another source,
    // the collision handling would need to be implemented.

    let result = parse_and_lower(r#"workflow test capabilities: [file] { done }"#);

    let implicit_role = result
        .implicit_role
        .expect("expected implicit role to be generated");
    assert_eq!(implicit_role.name, "test_default");
}

// ============================================================================
// Success Cases - Multiple Workflows Independent
// ============================================================================

#[test]
fn test_multiple_workflows_independent() {
    let result1 = parse_and_lower(r#"workflow workflow_a capabilities: [file] { done }"#);
    let result2 = parse_and_lower(r#"workflow workflow_b capabilities: [network] { done }"#);

    let role1 = result1
        .implicit_role
        .expect("expected implicit role for workflow_a");
    let role2 = result2
        .implicit_role
        .expect("expected implicit role for workflow_b");

    assert_eq!(role1.name, "workflow_a_default");
    assert_eq!(role2.name, "workflow_b_default");

    assert_eq!(role1.authority[0].name, "file");
    assert_eq!(role2.authority[0].name, "network");
}

// ============================================================================
// Success Cases - Complex Workflows
// ============================================================================

#[test]
fn test_complex_workflow_with_all_features() {
    let result = parse_and_lower(
        r#"workflow process_data(input: String)
            plays role(processor)
            capabilities: [
                filesystem @ { paths: ["/data/*"], read: true },
                network @ { hosts: ["api.example.com"] },
                cache
            ]
        {
            done
        }"#,
    );

    // Should have generated role
    let implicit_role = result
        .implicit_role
        .expect("expected implicit role to be generated");
    assert_eq!(implicit_role.name, "process_data_default");

    // Should have all capabilities
    assert_eq!(implicit_role.authority.len(), 3);
    assert_eq!(implicit_role.authority[0].name, "filesystem");
    assert_eq!(implicit_role.authority[1].name, "network");
    assert_eq!(implicit_role.authority[2].name, "cache");

    // Should preserve explicit role
    assert!(result.plays_roles.contains(&"processor".to_string()));
    assert!(
        result
            .plays_roles
            .contains(&"process_data_default".to_string())
    );
}

// ============================================================================
// Property Tests - Generated Role Properties
// ============================================================================

#[test]
fn test_generated_role_has_empty_obligations() {
    let result = parse_and_lower(r#"workflow test capabilities: [file] { done }"#);

    let implicit_role = result
        .implicit_role
        .expect("expected implicit role to be generated");
    assert!(
        implicit_role.obligations.is_empty(),
        "implicit role should have no obligations"
    );
}

#[test]
fn test_generated_role_capabilities_have_correct_effect() {
    let result = parse_and_lower(r#"workflow test capabilities: [file, network] { done }"#);

    let implicit_role = result
        .implicit_role
        .expect("expected implicit role to be generated");

    // All capabilities should have Epistemic effect (default for workflow capabilities)
    for capability in &implicit_role.authority {
        assert_eq!(
            capability.effect,
            Effect::Epistemic,
            "capability {} should have Epistemic effect",
            capability.name
        );
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_capabilities_no_implicit_role() {
    let result = parse_and_lower(r#"workflow test capabilities: [] { done }"#);

    // Empty capabilities list should not generate a role
    assert!(
        result.implicit_role.is_none(),
        "expected no implicit role for empty capabilities"
    );
}

#[test]
fn test_workflow_body_is_lowered() {
    let result = parse_and_lower(r#"workflow test capabilities: [file] { ret 42 }"#);

    // The workflow body should still be lowered correctly
    use ash_core::Workflow as CoreWorkflow;
    assert!(
        matches!(result.workflow, CoreWorkflow::Ret { .. }),
        "workflow body should be lowered to Ret"
    );
}

// ============================================================================
// Integration Tests - Constraint Value Types
// ============================================================================

#[test]
fn test_constraints_bool_values() {
    let result = parse_and_lower(
        r#"workflow test capabilities: [file @ { read: true, write: false }] { done }"#,
    );

    let implicit_role = result
        .implicit_role
        .expect("expected implicit role to be generated");
    assert_eq!(implicit_role.authority[0].constraints.len(), 2);
}

#[test]
fn test_constraints_int_values() {
    let result =
        parse_and_lower(r#"workflow test capabilities: [network @ { port: 443 }] { done }"#);

    let implicit_role = result
        .implicit_role
        .expect("expected implicit role to be generated");
    assert_eq!(implicit_role.authority[0].constraints.len(), 1);
}

#[test]
fn test_constraints_string_values() {
    let result =
        parse_and_lower(r#"workflow test capabilities: [file @ { path: "/tmp/data" }] { done }"#);

    let implicit_role = result
        .implicit_role
        .expect("expected implicit role to be generated");
    assert_eq!(implicit_role.authority[0].constraints.len(), 1);
}

#[test]
fn test_constraints_array_values() {
    let result = parse_and_lower(
        r#"workflow test capabilities: [file @ { paths: ["/tmp/*", "/var/log/*"] }] { done }"#,
    );

    let implicit_role = result
        .implicit_role
        .expect("expected implicit role to be generated");
    assert_eq!(implicit_role.authority[0].constraints.len(), 1);
}
