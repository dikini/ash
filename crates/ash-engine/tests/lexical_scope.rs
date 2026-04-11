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
            workflow main() {
                let items = [1, 2, 3]
                let first = items[0]
                ret first
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
            workflow main() {
                let a = 10
                let b = 20
                let c = 30
                ret a + b + c
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
    let workflow = engine
        .parse(
            r"
            workflow main(flag: Bool) {
                if flag then {
                    let x = 1
                    ret x
                } else {
                    let y = 2
                    ret y
                }
            }
        ",
        )
        .expect("workflow should parse");

    engine.check(&workflow).expect("workflow should type check");

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
async fn variables_example_pattern_matching() {
    // Test that pattern matching introduces bindings
    let engine = Engine::new().build().expect("engine builds");

    let result = engine
        .run(
            r"
            workflow main() {
                let [first, second] = [1, 2]
                ret first + second
            }
        ",
        )
        .await;

    assert!(
        result.is_ok(),
        "workflow should execute: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), ash_core::Value::Int(3));
}

#[tokio::test]
async fn variables_example_shadowing_in_block() {
    // Test that later bindings can shadow earlier ones in the same block
    let engine = Engine::new().build().expect("engine builds");

    let result = engine
        .run(
            r"
            workflow main() {
                let x = 1
                let x = x + 1
                ret x
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
