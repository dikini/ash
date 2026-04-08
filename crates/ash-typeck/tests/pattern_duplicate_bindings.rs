//! Test for duplicate pattern binding rejection (TASK-005)
//!
//! These tests verify that duplicate bindings within a single pattern are rejected,
//! while shadowing across statements is still allowed.

use ash_parser::{input::new_input, workflow_def};
use ash_typeck::type_check_workflow;

fn parse_and_check(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = new_input(source);
    let def = workflow_def(&mut input).map_err(|e| format!("Parse error: {:?}", e))?;
    let _ =
        type_check_workflow(&def.body, None).map_err(|e| format!("Type check error: {:?}", e))?;
    Ok(())
}

#[test]
fn test_duplicate_in_tuple_pattern_rejected() {
    // Duplicate bindings in a tuple pattern should be rejected
    let source = r#"
workflow main {
  let [x, x] = [1, 2]
  ret x
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_err(),
        "Duplicate binding in pattern [x, x] should be rejected: {:?}",
        result.err()
    );

    // Verify the error mentions duplicate binding
    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(
        err_msg.contains("duplicate") || err_msg.contains("Duplicate"),
        "Error should mention duplicate: {}",
        err_msg
    );
}

#[test]
fn test_duplicate_in_record_pattern_rejected() {
    // Duplicate bindings in a record pattern should be rejected
    let source = r#"
workflow main {
  let {a: x, b: x} = {a: 1, b: 2}
  ret x
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_err(),
        "Duplicate binding in pattern {{a: x, b: x}} should be rejected: {:?}",
        result.err()
    );
}

#[test]
fn test_duplicate_in_list_pattern_rejected() {
    // Duplicate bindings in a list pattern should be rejected
    let source = r#"
workflow main {
  let [x, y, x] = [1, 2, 3]
  ret x
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_err(),
        "Duplicate binding in pattern [x, y, x] should be rejected: {:?}",
        result.err()
    );
}

#[test]
fn test_shadowing_across_statements_allowed() {
    // Shadowing across different statements should be allowed
    let source = r#"
workflow main {
  let x = 1
  let x = 2
  ret x
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_ok(),
        "Shadowing across statements should be allowed: {:?}",
        result.err()
    );
}

#[test]
fn test_unique_pattern_bindings_allowed() {
    // Unique bindings in a pattern should be allowed
    let source = r#"
workflow main {
  let [x, y] = [1, 2]
  ret x + y
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_ok(),
        "Unique bindings in pattern should be allowed: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_pattern_duplicate_rejected() {
    // Duplicate bindings in nested patterns should be rejected
    let source = r#"
workflow main {
  let [[x, y], [x, z]] = [[1, 2], [3, 4]]
  ret x
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_err(),
        "Duplicate binding in nested pattern should be rejected: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_pattern_unique_allowed() {
    // Unique bindings in nested patterns should be allowed
    let source = r#"
workflow main {
  let [[x, y], [z, w]] = [[1, 2], [3, 4]]
  ret x + y + z + w
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_ok(),
        "Unique bindings in nested pattern should be allowed: {:?}",
        result.err()
    );
}
