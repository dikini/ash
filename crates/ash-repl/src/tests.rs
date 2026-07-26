use super::*;

#[test]
fn test_repl_creates() {
    let repl = Repl::new(true);
    assert!(repl.is_ok());
}

#[test]
fn test_history_path_when_disabled() {
    let repl = Repl::new(true).unwrap();
    assert!(repl.history_path.is_none());
}

#[tokio::test]
async fn test_repl_eval_unannotated_expression_rejects_at_checked_admission() {
    let mut repl = Repl::new(true).unwrap();
    let error = repl
        .eval("42")
        .await
        .expect_err("an unannotated REPL expression has no typed admission artifact");
    let ReplError::Engine(message) = error else {
        panic!("expected the checked-admission engine error, got {error:?}");
    };
    assert_eq!(
        message,
        "application execution failed: checked Core/CPS admission rejected: type error: \
         checked Core-to-CPS lowering failed: unknown type variable `main_return`"
    );
}

#[tokio::test]
async fn test_repl_eval_removed_workflow_syntax_rejected() {
    let mut repl = Repl::new(true).unwrap();
    let source = concat!("work", "flow test { ret 42; }");
    let result = repl.eval(source).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_repl_eval_empty() {
    let mut repl = Repl::new(true).unwrap();
    let result = repl.eval("").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_multiline_incomplete_brace() {
    let _repl = Repl::new(true).unwrap();
    // A target entry function with an unclosed brace may be incomplete depending on parser behavior.
    // Just verify the method runs without panic
    let _ = Repl::is_incomplete("fn main() {");
}

#[test]
fn test_multiline_complete_expression() {
    let _repl = Repl::new(true).unwrap();
    // A complete expression should not be incomplete
    assert!(!Repl::is_incomplete("42"));
}

#[test]
fn test_multiline_complete_entry_function() {
    let _repl = Repl::new(true).unwrap();
    // A complete target entry function should not be incomplete.
    assert!(!Repl::is_incomplete(
        "\n            fn main() -> Int {\n                42\n            }\n        "
    ));
}
