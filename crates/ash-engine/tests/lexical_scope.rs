//! Lexical scope parser/typechecker tests (TASK-446).
//!
//! The bounded TASK-2014 checked Core/CPS entry path executes its validated `PureAnf` subset,
//! including distinct lexical bindings and nested integer primitives. These fixtures retain
//! lexical-scope parser/typechecker evidence and assert closed admission for forms whose checked
//! Core representation remains invalid; they must not revive direct evaluation.

use ash_engine::Engine;

#[tokio::test]
async fn variables_example_scope() {
    // Test that a application with multiple let bindings executes correctly
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
        "application should execute: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), ash_core::Value::Int(1));
}

#[tokio::test]
async fn variables_example_nested_bindings() {
    // PureAnf normalizes the nested integer primitive tree under distinct lexical bindings, so
    // this must run through the checked Core/CPS owner rather than a direct evaluator fallback.
    let engine = Engine::new().build().expect("engine builds");
    let source = r"
        fn main() -> Int {
            let a = 10
            let b = 20
            let c = 30
            a + b + c
        }
    ";
    let result = engine
        .run(source)
        .await
        .expect("nested lexical integer bindings run through checked Core/CPS");
    assert_eq!(result, ash_core::Value::Int(60));
}

#[tokio::test]
async fn variables_example_if_scope() {
    // Test that if branches maintain separate scope
    let engine = Engine::new().build().expect("engine builds");
    let mut application = engine
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
        .expect("application should parse");

    engine
        .check(&mut application)
        .expect("application should type check");

    // Input-bearing execution has no validated production typed lowering yet. Both values must
    // close at admission rather than falling back to the former direct evaluator.
    for flag in [true, false] {
        let mut input = std::collections::HashMap::new();
        input.insert("flag".to_string(), ash_core::Value::Bool(flag));
        let error = engine
            .execute_with_input(&application, input)
            .await
            .expect_err("input-bearing conditional lowering must reject at admission");
        assert_eq!(
            error.to_string(),
            "application execution failed: checked Core/CPS admission rejected: no validated production typed lowering is available"
        );
    }
}

#[tokio::test]
async fn variables_example_refutable_pattern_rejected_before_runtime() {
    // Refutable pattern matching in application let is rejected by typechecking.
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

    assert!(
        result.is_err(),
        "refutable application let should be rejected"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("non-irrefutable pattern in let"),
        "{err_msg}"
    );
}

#[tokio::test]
async fn variables_example_shadowing_in_block() {
    // Later bindings may shadow earlier ones in the source scope. The lexical-scope program still
    // type checks, but its current checked-Core representation rejects duplicate value binders.
    let engine = Engine::new().build().expect("engine builds");
    let source = r"
        fn main() -> Int {
            let x = 1
            let x = x + 1
            x
        }
    ";
    let mut application = engine.parse(source).expect("application should parse");
    engine
        .check(&mut application)
        .expect("shadowing should type check");

    let error = engine
        .run(source)
        .await
        .expect_err("unsupported shadowing lowering must reject at admission");

    assert!(
        error
            .to_string()
            .contains("checked Core validation failed: duplicate value binding `x`"),
        "shadowing must remain closed at the checked-Core validation boundary: {error}"
    );
}
