//! Tests for REPL workflow storage

use ash_repl::{EvalResult, Session, Value};

#[tokio::test]
async fn test_workflow_definition_stored() {
    let mut session = Session::new();

    // Define a workflow
    let result = session
        .evaluate(
            r#"
        workflow greet(name) {
            ret "Hello, " + name;
        }
    "#,
        )
        .await;

    assert!(
        matches!(result, Ok(EvalResult::WorkflowDefined { ref name }) if name == "greet"),
        "Expected WorkflowDefined result, got {result:?}"
    );

    // Workflow should be callable
    let result = session.evaluate(r#"greet("World")"#).await;
    assert!(result.is_ok(), "Workflow call failed: {result:?}");
}

#[tokio::test]
async fn test_workflow_redefinition_updates() {
    let mut session = Session::new();

    // First definition
    session
        .evaluate(r#"workflow test { ret "v1"; }"#)
        .await
        .unwrap();

    // Redefinition
    session
        .evaluate(r#"workflow test { ret "v2"; }"#)
        .await
        .unwrap();

    // Should use v2 - call and check result
    let result = session.evaluate("test()").await;
    assert_eq!(
        result.unwrap(),
        EvalResult::Value(Value::String("v2".to_string()))
    );
}

#[tokio::test]
async fn test_undefined_workflow_error() {
    let mut session = Session::new();

    let result = session.evaluate("undefined_workflow()").await;

    assert!(
        matches!(result, Err(ash_repl::ReplError::UnknownWorkflow { .. })),
        "Expected UnknownWorkflow error, got {result:?}"
    );
}

#[tokio::test]
async fn test_workflow_type_checked_at_definition() {
    let mut session = Session::new();

    let result = session
        .evaluate(
            r#"
        workflow bad(x: Int) {
            ret x + "string";
        }
    "#,
        )
        .await;

    // Should fail at definition time
    assert!(
        matches!(result, Err(ash_repl::ReplError::TypeError { .. })),
        "Expected TypeError at definition time, got {result:?}"
    );
}

#[tokio::test]
async fn test_stored_workflow_persists_across_inputs() {
    let mut session = Session::new();

    // Define workflow
    session
        .evaluate(
            r"
        workflow add(a: Int, b: Int) {
            ret a + b;
        }
    ",
        )
        .await
        .unwrap();

    // Use in expression
    let result = session.evaluate("add(2, 3)").await;
    assert_eq!(
        result.unwrap(),
        EvalResult::Value(Value::Int(5)),
        "Workflow should return sum of arguments"
    );
}

#[tokio::test]
async fn test_workflow_with_parameters() {
    let mut session = Session::new();

    // Define a workflow with multiple parameters
    session
        .evaluate(
            r"
        workflow concat(a: String, b: String, sep: String) {
            ret a + sep + b;
        }
    ",
        )
        .await
        .unwrap();

    // Call with multiple arguments
    let result = session.evaluate(r#"concat("Hello", "World", ", ")"#).await;
    assert_eq!(
        result.unwrap(),
        EvalResult::Value(Value::String("Hello, World".to_string()))
    );
}

#[tokio::test]
async fn test_multiple_workflows_stored() {
    let mut session = Session::new();

    // Define multiple workflows
    session
        .evaluate(r"workflow one { ret 1; }")
        .await
        .unwrap();
    session
        .evaluate(r"workflow two { ret 2; }")
        .await
        .unwrap();

    // Both should be callable
    let result1 = session.evaluate("one()").await;
    let result2 = session.evaluate("two()").await;

    assert_eq!(result1.unwrap(), EvalResult::Value(Value::Int(1)));
    assert_eq!(result2.unwrap(), EvalResult::Value(Value::Int(2)));
}

#[tokio::test]
async fn test_simple_workflow_no_params() {
    let mut session = Session::new();

    // Define a simple workflow with no parameters
    let result = session.evaluate(r"workflow simple { ret 42; }").await;
    assert!(
        matches!(result, Ok(EvalResult::WorkflowDefined { ref name }) if name == "simple"),
        "Expected WorkflowDefined result, got {result:?}"
    );

    // Call the workflow
    let result = session.evaluate("simple()").await;
    assert_eq!(
        result.unwrap(),
        EvalResult::Value(Value::Int(42)),
        "Workflow should return 42"
    );
}

#[tokio::test]
async fn test_workflow_has_name() {
    let mut session = Session::new();

    // Define a workflow
    session
        .evaluate(r"workflow named { ret 123; }")
        .await
        .unwrap();

    // Check it's stored with the correct name
    assert!(session.has_workflow("named"));
    let workflow = session.get_workflow("named").unwrap();
    assert_eq!(workflow.name, "named");
}
