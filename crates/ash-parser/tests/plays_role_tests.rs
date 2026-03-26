//! Tests for parsing `plays role(R)` clause (TASK-259)
//!
//! These tests verify parsing of role inclusion in workflow headers:
//! - Single `plays role(ident)` clause
//! - Multiple `plays role` clauses
//! - Integration with capabilities and other workflow clauses
//! - Error cases

use ash_parser::input::new_input;
use ash_parser::parse_workflow::workflow_def;
use winnow::prelude::*;

// ============================================================================
// Success Cases
// ============================================================================

#[test]
fn test_parse_single_plays_role() {
    let input = r#"workflow ai_agent plays role(ai_agent) { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let workflow = result.unwrap();
    assert_eq!(workflow.name.as_ref(), "ai_agent");
    assert_eq!(workflow.plays_roles.len(), 1);
    assert_eq!(workflow.plays_roles[0].name.as_ref(), "ai_agent");
}

#[test]
fn test_parse_multiple_plays_roles() {
    let input = r#"workflow manager plays role(supervisor) plays role(approver) { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let workflow = result.unwrap();
    assert_eq!(workflow.name.as_ref(), "manager");
    assert_eq!(workflow.plays_roles.len(), 2);
    assert_eq!(workflow.plays_roles[0].name.as_ref(), "supervisor");
    assert_eq!(workflow.plays_roles[1].name.as_ref(), "approver");
}

#[test]
fn test_parse_plays_role_with_params() {
    let input = r#"workflow handler(x: Int) plays role(processor) { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let workflow = result.unwrap();
    assert_eq!(workflow.name.as_ref(), "handler");
    assert_eq!(workflow.params.len(), 1);
    assert_eq!(workflow.plays_roles.len(), 1);
    assert_eq!(workflow.plays_roles[0].name.as_ref(), "processor");
}

#[test]
fn test_parse_plays_role_with_contract() {
    let input =
        r#"workflow secure_handler plays role(guard) requires: true ensures: true { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let workflow = result.unwrap();
    assert_eq!(workflow.name.as_ref(), "secure_handler");
    assert_eq!(workflow.plays_roles.len(), 1);
    assert!(workflow.contract.is_some());
}

#[test]
fn test_parse_plays_role_complex_workflow() {
    let input = r#"workflow process_order(order_id: Int)
        plays role(order_processor)
        plays role(auditor)
        requires: order_id > 0
    {
        done
    }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let workflow = result.unwrap();
    assert_eq!(workflow.name.as_ref(), "process_order");
    assert_eq!(workflow.params.len(), 1);
    assert_eq!(workflow.plays_roles.len(), 2);
    assert_eq!(workflow.plays_roles[0].name.as_ref(), "order_processor");
    assert_eq!(workflow.plays_roles[1].name.as_ref(), "auditor");
}

#[test]
fn test_parse_plays_role_with_snake_case_name() {
    let input = r#"workflow my_handler plays role(my_role) { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let workflow = result.unwrap();
    assert_eq!(workflow.plays_roles[0].name.as_ref(), "my_role");
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_parse_plays_role_missing_role_keyword() {
    let input = r#"workflow handler plays missing_role() { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    // Should fail because "role" keyword is missing
    assert!(
        result.is_err(),
        "Expected parse to fail without 'role' keyword"
    );
}

#[test]
fn test_parse_plays_role_missing_open_paren() {
    let input = r#"workflow handler plays role handler_role) { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_err(),
        "Expected parse to fail without opening paren"
    );
}

#[test]
fn test_parse_plays_role_missing_close_paren() {
    let input = r#"workflow handler plays role(handler_role { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_err(),
        "Expected parse to fail without closing paren"
    );
}

#[test]
fn test_parse_plays_role_empty_name() {
    let input = r#"workflow handler plays role() { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(
        result.is_err(),
        "Expected parse to fail with empty role name"
    );
}

// ============================================================================
// Property Tests
// ============================================================================

#[test]
fn test_parse_plays_role_various_identifiers() {
    let valid_identifiers = [
        "a",
        "A",
        "abc",
        "ABC",
        "a_b_c",
        "A_B_C",
        "abc123",
        "a1b2c3",
        "_private",
        "my_role",
        "ai_agent",
        "order_processor",
    ];

    for ident in &valid_identifiers {
        let input = format!(r#"workflow w plays role({}) {{ done }}"#, ident);
        let mut inp = new_input(&input);
        let result = workflow_def.parse_next(&mut inp);

        assert!(
            result.is_ok(),
            "Expected successful parse for identifier '{}', got: {:?}",
            ident,
            result
        );

        let workflow = result.unwrap();
        assert_eq!(
            workflow.plays_roles[0].name.as_ref(),
            *ident,
            "Role name mismatch for identifier '{}'",
            ident
        );
    }
}

#[test]
fn test_parse_plays_role_preserves_span() {
    let input = r#"workflow handler plays role(my_role) { done }"#;
    let mut input = new_input(input);
    let result = workflow_def.parse_next(&mut input);

    assert!(result.is_ok());
    let workflow = result.unwrap();
    assert_eq!(workflow.plays_roles.len(), 1);

    // Span should be set (not default)
    let span = workflow.plays_roles[0].span;
    assert!(span.start < span.end, "Span should have valid range");
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_parse_workflow_without_plays_roles() {
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
    assert!(workflow.plays_roles.is_empty());
}

#[test]
fn test_parse_plays_role_whitespace_variations() {
    // Test various whitespace patterns
    let inputs = [
        r#"workflow w plays role(r) { done }"#,
        r#"workflow w  plays  role(  r  )  { done }"#,
        r#"workflow w
            plays role(r)
        { done }"#,
        r#"workflow w plays role(r) plays role(s) { done }"#,
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
