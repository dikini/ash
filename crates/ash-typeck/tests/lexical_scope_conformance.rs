//! Lexical scope conformance tests for TASK-445
//!
//! These tests verify that the type checker correctly enforces lexical block scope
//! for bindings, consuming the canonical lowered form consistently.

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
fn test_earlier_let_binding_visible_in_later_statement() {
    // Earlier let bindings should be visible in later statements of the same block
    let source = r#"
workflow test {
  let items = [1, 2, 3]
  let first = items[0]
  done
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_ok(),
        "Earlier let binding should be visible in later statements: {:?}",
        result.err()
    );
}

#[test]
fn test_unbound_name_rejected_after_normalization() {
    // Unbound names should be rejected after normalization
    let source = r#"
workflow test {
  let first = items[0]
  done
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_err(),
        "Unbound name 'items' should be rejected: {:?}",
        result
    );

    // Verify the error mentions the unbound variable
    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(
        err_msg.contains("items") || err_msg.contains("unbound") || err_msg.contains("Unbound"),
        "Error should mention the unbound variable: {}",
        err_msg
    );
}

#[test]
fn test_shadowing_by_later_lexical_binding() {
    // Shadowing should work only by later lexical binding in the same block
    let source = r#"
workflow test {
  let x = 1
  let x = x + 1
  done
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_ok(),
        "Later binding should shadow earlier binding in same block: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_let_bindings_lexical_scope() {
    // Verify that nested let bindings maintain proper lexical scoping
    let source = r#"
workflow test {
  let outer = [1, 2, 3]
  let middle = outer[0]
  let inner = middle + 1
  done
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_ok(),
        "Nested let bindings should maintain proper lexical scope: {:?}",
        result.err()
    );
}

#[test]
fn test_let_with_non_binding_statement_continuation() {
    // Non-binding statements should still work in continuation
    let source = r#"
workflow test {
  let items = [1, 2, 3]
  act print(items[0])
  done
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_ok(),
        "Let followed by non-binding statement should work: {:?}",
        result.err()
    );
}

#[test]
fn test_let_in_if_then_branch() {
    // Let bindings in conditional branches should be scoped to that branch
    let source = r#"
workflow test {
  if true then {
    let x = 1
    done
  }
  done
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_ok(),
        "Let binding in if-then branch should work: {:?}",
        result.err()
    );
}

#[test]
fn test_let_not_visible_outside_block() {
    // Let binding inside a block should not be visible outside
    let source = r#"
workflow test {
  if true then {
    let x = 1
    done
  }
  let y = x + 1
  done
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_err(),
        "Let binding inside block should not be visible outside: {:?}",
        result
    );

    // Verify the error mentions the unbound variable
    if let Err(err) = result {
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("x") || err_msg.contains("unbound") || err_msg.contains("Unbound"),
            "Error should mention the unbound variable 'x': {}",
            err_msg
        );
    }
}

#[test]
fn test_multiple_let_bindings_same_level() {
    // Multiple let bindings at same level should all be visible to later bindings
    let source = r#"
workflow test {
  let a = 1
  let b = 2
  let c = a + b
  done
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_ok(),
        "Multiple let bindings should all be visible to later bindings: {:?}",
        result.err()
    );
}

#[test]
fn test_let_with_observe_binding() {
    // Observe with binding should also establish lexical scope
    let source = r#"
workflow test {
  observe capability_name as result
  act print(result)
  done
}
"#;

    let result = parse_and_check(source);
    // This might fail due to unknown capability, but shouldn't fail due to scope
    // We're mainly testing that the binding is visible in the continuation
    if let Err(err) = result {
        let err_msg = format!("{}", err);
        // Should not be a scope error
        assert!(
            !err_msg.contains("unbound") && !err_msg.contains("Unbound"),
            "Observe binding should be visible in continuation, got: {}",
            err_msg
        );
    }
}

#[test]
fn test_let_binding_in_expression() {
    // Let binding should be usable in subsequent expressions
    let source = r#"
workflow test {
  let x = 5
  let y = x * 2
  let z = y + 1
  done
}
"#;

    let result = parse_and_check(source);
    assert!(
        result.is_ok(),
        "Let bindings should chain correctly: {:?}",
        result.err()
    );
}
