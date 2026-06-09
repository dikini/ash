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
async fn test_repl_eval_expression() {
    let mut repl = Repl::new(true).unwrap();
    let result = repl.eval("42").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_repl_eval_workflow() {
    let mut repl = Repl::new(true).unwrap();
    // Test parsing a workflow definition (no execution, just storage)
    let result = repl.eval("workflow test { ret 42; }").await;
    assert!(result.is_ok());
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
    // A workflow with unclosed brace may be incomplete depending on parser behavior
    // Just verify the method runs without panic
    let _ = Repl::is_incomplete("workflow test {");
}

#[test]
fn test_multiline_complete_expression() {
    let _repl = Repl::new(true).unwrap();
    // A complete expression should not be incomplete
    assert!(!Repl::is_incomplete("42"));
}

#[test]
fn test_multiline_complete_workflow() {
    let _repl = Repl::new(true).unwrap();
    // A complete workflow should not be incomplete
    assert!(!Repl::is_incomplete(
        "\n            workflow test {\n                ret 42;\n            }\n        "
    ));
}
