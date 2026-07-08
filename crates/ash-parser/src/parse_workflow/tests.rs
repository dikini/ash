//! Tests for `parse_workflow`.

use super::*;
use crate::surface::Pattern;

fn test_input(s: &str) -> ParseInput<'_> {
    crate::input::new_input(s)
}

#[test]
fn test_observe_stmt() {
    let mut input = test_input("observe read_db");
    let result = parse_stmt(&mut input).unwrap();
    assert!(matches!(result, Workflow::Observe { .. }));
}

#[test]
fn test_observe_stmt_with_args_index() {
    let mut input = test_input("observe Args 0");
    let result = parse_stmt(&mut input).unwrap();

    match result {
        Workflow::Observe {
            capability,
            binding,
            ..
        } => {
            assert_eq!(capability.as_ref(), "Args:0");
            assert!(binding.is_none());
        }
        _ => panic!("Expected Observe"),
    }
}

#[test]
fn test_workflow_def_rejects_non_args_observe_index_surface() {
    let mut input = test_input(
        r"
        fn main() {
            observe sensor 0;
            {};
        }
    ",
    );

    let result = workflow_def(&mut input);
    assert!(
        result.is_err(),
        "non-Args indexed observe syntax should remain invalid"
    );
}

#[test]
fn test_observe_stmt_with_args_index_and_binding() {
    let mut input = test_input("observe Args 0 as arg");
    let result = parse_stmt(&mut input).unwrap();

    match result {
        Workflow::Observe {
            capability,
            binding,
            ..
        } => {
            assert_eq!(capability.as_ref(), "Args:0");
            assert!(
                matches!(binding, Some(Pattern::Variable { name, .. }) if name.as_ref() == "arg")
            );
        }
        _ => panic!("Expected Observe"),
    }
}

#[test]
fn test_observe_with_binding() {
    let mut input = test_input("observe read_db as data");
    let result = parse_stmt(&mut input).unwrap();
    match result {
        Workflow::Observe { binding, .. } => {
            assert!(binding.is_some());
        }
        _ => panic!("Expected Observe"),
    }
}

#[test]
fn test_let_stmt() {
    let mut input = test_input("let x = 42");
    let result = parse_stmt(&mut input).unwrap();
    assert!(matches!(result, Workflow::Let { .. }));
}

#[test]
fn test_if_stmt() {
    let mut input = test_input("if true then {}");
    let result = parse_stmt(&mut input).unwrap();
    assert!(matches!(result, Workflow::If { .. }));
}

#[test]
fn test_if_else_stmt() {
    let mut input = test_input("if x > 0 then {} else {}");
    let result = parse_stmt(&mut input).unwrap();
    match result {
        Workflow::If { else_branch, .. } => {
            assert!(else_branch.is_some());
        }
        _ => panic!("Expected If"),
    }
}

#[test]
fn test_act_stmt() {
    let mut input = test_input("act log_event(\"test\")");
    let result = parse_stmt(&mut input).unwrap();
    assert!(matches!(result, Workflow::Act { .. }));
}

#[test]
fn test_for_stmt() {
    let mut input = test_input("for item in items do {}");
    let result = parse_stmt(&mut input).unwrap();
    assert!(matches!(result, Workflow::For { .. }));
}

#[test]
fn test_with_stmt() {
    let mut input = test_input("with db do {}");
    let result = parse_stmt(&mut input).unwrap();
    assert!(matches!(result, Workflow::With { .. }));
}

#[test]
fn test_maybe_stmt() {
    let mut input = test_input("maybe {} else {}");
    let result = parse_stmt(&mut input).unwrap();
    assert!(matches!(result, Workflow::Maybe { .. }));
}

#[test]
fn test_must_stmt() {
    let mut input = test_input("must {}");
    let result = parse_stmt(&mut input).unwrap();
    assert!(matches!(result, Workflow::Must { .. }));
}

#[test]
fn test_pattern_variable() {
    let mut input = test_input("my_var");
    let result = pattern(&mut input).unwrap();
    assert!(matches!(result, Pattern::Variable { name, .. } if name.as_ref() == "my_var"));
}

#[test]
fn test_pattern_variable_named_supervises() {
    let mut input = test_input("supervises");
    let result = pattern(&mut input).unwrap();
    assert!(matches!(result, Pattern::Variable { name, .. } if name.as_ref() == "supervises"));
}

#[test]
fn test_pattern_wildcard() {
    let mut input = test_input("_");
    let result = pattern(&mut input).unwrap();
    assert!(matches!(result, Pattern::Wildcard));
}

#[test]
fn test_pattern_tuple() {
    let mut input = test_input("(a, b, c)");
    let result = pattern(&mut input).unwrap();
    assert!(matches!(result, Pattern::Tuple(pats) if pats.len() == 3));
}

#[test]
fn test_action_ref_symbolic() {
    let mut input = test_input("send_email(\"to\", \"subject\")");
    let result = action_ref(&mut input).unwrap();
    match &result.target {
        crate::surface::OperationalTarget::Symbolic { capability_name } => {
            assert_eq!(capability_name.as_ref(), "send_email");
        }
        _ => panic!("Expected symbolic target"),
    }
    assert_eq!(result.args.len(), 2);
}

#[test]
fn test_action_ref_explicit() {
    // Test explicit provider:action form
    let mut input = test_input("provider:action");

    // Test step by step
    let first = identifier(&mut input).expect("Should parse first identifier");
    assert_eq!(first, "provider");

    skip_whitespace_and_comments(&mut input);
    assert!(input.input.starts_with(":"));

    literal_str(":")
        .parse_next(&mut input)
        .expect("Should parse colon");
    skip_whitespace_and_comments(&mut input);

    let second = identifier(&mut input).expect("Should parse second identifier");
    assert_eq!(second, "action");

    // All input should be consumed
    assert!(input.input.is_empty());
}

#[test]
fn test_action_ref_qualified() {
    // Test module-qualified form: io::fs_read(args)
    let mut input = test_input("io::fs_read(\"file.txt\")");
    let result = action_ref(&mut input).unwrap();

    match &result.target {
        crate::surface::OperationalTarget::Qualified {
            module,
            capability_name,
        } => {
            assert_eq!(module.as_ref(), "io");
            assert_eq!(capability_name.as_ref(), "fs_read");
        }
        _ => panic!("Expected qualified target, got {:?}", result.target),
    }
    assert_eq!(result.args.len(), 1);
}

#[test]
fn test_check_stmt_with_obligation() {
    let mut input = test_input("check admin.is_active");
    let result = check_stmt(&mut input).unwrap();
    assert!(matches!(result, Workflow::Check { .. }));
    match result {
        Workflow::Check { target, .. } => {
            assert!(matches!(target, CheckTarget::Obligation(_)));
        }
        _ => panic!("Expected Check workflow"),
    }
}

#[test]
fn test_check_stmt_rejects_policy_instance() {
    let mut input = test_input("check RateLimit { requests: 100, window_secs: 60 }");
    let result = check_stmt(&mut input);
    assert!(result.is_err());
}

#[test]
fn test_decide_stmt_requires_under_clause() {
    let mut input = test_input("decide { ok } under gate then {}");
    let result = decide_stmt(&mut input).unwrap();
    match result {
        Workflow::Decide {
            policy,
            else_branch,
            ..
        } => {
            assert!(matches!(policy, Some(ref name) if name.as_ref() == "gate"));
            assert!(else_branch.is_none());
        }
        _ => panic!("Expected Decide workflow"),
    }
}

#[test]
fn test_decide_stmt_rejects_missing_policy() {
    let mut input = test_input("decide { ok } then {}");
    let result = decide_stmt(&mut input);
    assert!(result.is_err());
}

#[test]
fn test_set_stmt() {
    let mut input = test_input("set hvac:target = 72");
    let result = parse_stmt(&mut input).unwrap();
    assert!(matches!(result, Workflow::Set { .. }));
    match result {
        Workflow::Set {
            capability,
            channel,
            continuation,
            ..
        } => {
            assert_eq!(capability.as_ref(), "hvac");
            assert_eq!(channel.as_ref(), "target");
            assert!(continuation.is_none());
        }
        _ => panic!("Expected Set"),
    }
}

#[test]
fn test_send_stmt() {
    let mut input = test_input("send kafka:orders order");
    let result = parse_stmt(&mut input).unwrap();
    assert!(matches!(result, Workflow::Send { .. }));
    match result {
        Workflow::Send {
            capability,
            channel,
            continuation,
            ..
        } => {
            assert_eq!(capability.as_ref(), "kafka");
            assert_eq!(channel.as_ref(), "orders");
            assert!(continuation.is_none());
        }
        _ => panic!("Expected Send"),
    }
}

#[test]
fn test_act_stmt_with_then() {
    let mut input = test_input("act provider:action(args) then observe status");
    let result = parse_stmt(&mut input).unwrap();
    match result {
        Workflow::Act {
            action,
            guard,
            result_name,
            continuation,
            ..
        } => {
            assert!(guard.is_none());
            assert!(result_name.is_none());
            assert!(continuation.is_some());
            // Verify continuation is an observe
            let cont = continuation.unwrap();
            assert!(
                matches!(*cont, Workflow::Observe { .. }),
                "Expected Observe continuation, got {:?}",
                cont
            );
            // Verify the action parsed correctly
            assert_eq!(action.args.len(), 1);
        }
        _ => panic!("Expected Act"),
    }
}

#[test]
fn test_act_stmt_with_as() {
    let mut input = test_input("act provider:action(args) as result");
    let result = parse_stmt(&mut input).unwrap();
    match result {
        Workflow::Act {
            action,
            guard,
            result_name,
            continuation,
            ..
        } => {
            assert!(guard.is_none());
            assert_eq!(result_name.as_deref(), Some("result"));
            assert!(continuation.is_none());
            assert_eq!(action.args.len(), 1);
        }
        _ => panic!("Expected Act"),
    }
}

#[test]
fn test_act_stmt_bare_regression() {
    let mut input = test_input("act log_event(\"test\")");
    let result = parse_stmt(&mut input).unwrap();
    match result {
        Workflow::Act {
            guard,
            result_name,
            continuation,
            ..
        } => {
            assert!(guard.is_none());
            assert!(result_name.is_none());
            assert!(continuation.is_none());
        }
        _ => panic!("Expected Act"),
    }
}

#[test]
fn test_let_action_ref_sugar() {
    let mut input = test_input("let result = provider:action(args)");
    let result = parse_stmt(&mut input).unwrap();
    match result {
        Workflow::Act {
            action,
            guard,
            result_name,
            continuation,
            ..
        } => {
            assert!(guard.is_none());
            assert_eq!(result_name.as_deref(), Some("result"));
            assert!(continuation.is_none());
            assert_eq!(action.args.len(), 1);
        }
        _ => panic!("Expected Act (from let sugar), got {:?}", result),
    }
}

#[test]
fn test_let_expr_fallback() {
    let mut input = test_input("let x = 42");
    let result = parse_stmt(&mut input).unwrap();
    // This should still parse as a normal let, not action-ref sugar
    assert!(
        matches!(result, Workflow::Let { .. }),
        "Expected Let, got {:?}",
        result
    );
}

#[test]
fn test_let_builtin_fn_not_desugared_as_action() {
    // `record(...)` followed by a newline must parse as Workflow::Let,
    // not Workflow::Act. Without the builtin-guard, `action_ref` would
    // parse `record(...)` as a Symbolic capability call and the boundary
    // check would fire, producing an incorrect Act statement.
    let mut input = test_input("let r = record(\"a\", 1)\nret r");
    let result = parse_stmt(&mut input).unwrap();
    assert!(
        matches!(result, Workflow::Let { .. }),
        "builtin fn call should not desugar to Act, got {:?}",
        result
    );
}

// ── Dual-context test: workflow-level act vs expression-level act block (TASK-676) ──

#[test]
fn test_workflow_act_unchanged_after_target_act_do_sugar_expression() {
    // Workflow-level `act provider:action(args)` should produce Workflow::Act,
    // NOT expression-level target Act do-sugar. This confirms the two parsing contexts remain distinct.
    let mut input = test_input("act provider:action(args)");
    let result = parse_stmt(&mut input).unwrap();
    match result {
        Workflow::Act {
            action,
            guard,
            result_name,
            continuation,
            ..
        } => {
            // Verify it is a workflow-level Act, not expression-level target Act do-sugar.
            assert!(guard.is_none());
            assert!(result_name.is_none());
            assert!(continuation.is_none());
            // Verify the action parsed with explicit target
            match &action.target {
                crate::surface::OperationalTarget::Explicit {
                    provider,
                    action: action_name,
                } => {
                    assert_eq!(provider.as_ref(), "provider");
                    assert_eq!(action_name.as_ref(), "action");
                }
                _ => panic!("Expected explicit target, got: {:?}", action.target),
            }
            assert_eq!(action.args.len(), 1);
        }
        _ => panic!("Expected Workflow::Act, got: {:?}", result),
    }
}
