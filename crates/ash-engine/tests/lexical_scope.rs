//! End-to-end lexical scope tests (TASK-446)
//!
//! These tests verify that ash-engine correctly executes workflows
//! with lexical scoping through the full parsing/typechecking/execution pipeline.

use ash_engine::Engine;

#[tokio::test]
async fn variables_example_scope() {
    // Test that a workflow with multiple let bindings executes correctly
    let engine = Engine::new().build().expect("engine builds");

    let result = engine
        .run(
            r"
            fn main() -> Int {
                let first = 1
                first
            }
        ",
        )
        .await;

    assert!(
        result.is_ok(),
        "workflow should execute: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), ash_core::Value::Int(1));
}

#[tokio::test]
async fn variables_example_nested_bindings() {
    // Test deeply nested let bindings
    let engine = Engine::new().build().expect("engine builds");

    let result = engine
        .run(
            r"
            fn main() -> Int {
                let a = 10
                let b = 20
                let c = 30
                a + b + c
            }
        ",
        )
        .await;

    assert!(
        result.is_ok(),
        "workflow should execute: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), ash_core::Value::Int(60));
}

#[tokio::test]
async fn variables_example_if_scope() {
    // Test that if branches maintain separate scope
    let engine = Engine::new().build().expect("engine builds");
    let mut workflow = engine
        .parse(
            r"
            fn main(flag: Bool) -> Int {
                if flag then {
                    let x = 1
                    x
                } else {
                    let y = 2
                    y
                }
            }
        ",
        )
        .expect("workflow should parse");

    engine
        .check(&mut workflow)
        .expect("workflow should type check");

    // Test true branch
    let mut input = std::collections::HashMap::new();
    input.insert("flag".to_string(), ash_core::Value::Bool(true));
    let result = engine.execute_with_input(&workflow, input).await;
    assert!(
        result.is_ok(),
        "workflow should execute with true: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), ash_core::Value::Int(1));

    // Test false branch
    let mut input = std::collections::HashMap::new();
    input.insert("flag".to_string(), ash_core::Value::Bool(false));
    let result = engine.execute_with_input(&workflow, input).await;
    assert!(
        result.is_ok(),
        "workflow should execute with false: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), ash_core::Value::Int(2));
}

#[tokio::test]
async fn variables_example_refutable_pattern_rejected_before_runtime() {
    // Refutable pattern matching in workflow let is rejected by typechecking.
    let engine = Engine::new().build().expect("engine builds");

    let result = engine
        .run(
            r"
            fn main() -> Int {
                let [first, second] = [1, 2]
                first + second
            }
        ",
        )
        .await;

    assert!(result.is_err(), "refutable workflow let should be rejected");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("non-irrefutable pattern in let"),
        "{err_msg}"
    );
}

#[tokio::test]
async fn variables_example_shadowing_in_block() {
    // Test that later bindings can shadow earlier ones in the same block
    let engine = Engine::new().build().expect("engine builds");

    let result = engine
        .run(
            r"
            fn main() -> Int {
                let x = 1
                let x = x + 1
                x
            }
        ",
        )
        .await;

    assert!(
        result.is_ok(),
        "workflow should execute: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), ash_core::Value::Int(2));
}
