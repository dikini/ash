//! End-to-end integration tests for `Expr::Let` — fn bodies with let-sequencing (TASK-652)
//!
//! These tests verify that fn bodies with multi-statement let-sequencing work
//! through all code paths: inline fn expressions, top-level fn definitions,
//! and imported pub fn.

use ash_engine::Engine;

// ── 1. Inline fn expression with let-sequencing ──────────────────────

#[tokio::test]
async fn task652_inline_fn_let_binding() {
    let engine = Engine::new().build().expect("engine builds");

    let result = engine
        .run(
            r"
            workflow main {
                let add_one = fn(x: Int) -> Int {
                    let y = 1
                    x + y
                }
                ret add_one(5)
            }
        ",
        )
        .await;

    assert!(
        result.is_ok(),
        "inline fn with let should work: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), ash_core::Value::Int(6));
}

#[tokio::test]
async fn task652_inline_fn_nested_let() {
    let engine = Engine::new().build().expect("engine builds");

    let result = engine
        .run(
            r"
            workflow main {
                let compute = fn(x: Int) -> Int {
                    let a = x + 1
                    let b = a * 2
                    b
                }
                ret compute(3)
            }
        ",
        )
        .await;

    assert!(result.is_ok(), "nested let should work: {:?}", result.err());
    // (3 + 1) * 2 = 8
    assert_eq!(result.unwrap(), ash_core::Value::Int(8));
}

// ── 2. Top-level fn definition with let-sequencing ──────────────────

#[tokio::test]
async fn task652_toplevel_fn_let_binding() {
    let engine = Engine::new().build().expect("engine builds");

    let result = engine
        .run(
            r"
            fn double(x: Int) -> Int {
                let result = x + x
                result
            }

            workflow main {
                ret double(7)
            }
        ",
        )
        .await;

    assert!(
        result.is_ok(),
        "top-level fn with let should work: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), ash_core::Value::Int(14));
}

#[tokio::test]
async fn task652_toplevel_fn_multiple_lets() {
    let engine = Engine::new().build().expect("engine builds");

    let result = engine
        .run(
            r"
            fn add_three(a: Int, b: Int, c: Int) -> Int {
                let sum_ab = a + b
                let total = sum_ab + c
                total
            }

            workflow main {
                ret add_three(10, 20, 30)
            }
        ",
        )
        .await;

    assert!(
        result.is_ok(),
        "top-level fn with multiple lets should work: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), ash_core::Value::Int(60));
}

// ── 3. Inline fn with let and closures ──────────────────────────────

#[tokio::test]
async fn task652_fn_let_closure_capture() {
    let engine = Engine::new().build().expect("engine builds");

    let result = engine
        .run(
            r"
            workflow main {
                let x = 10
                let get_x = fn() -> Int {
                    let y = x
                    y
                }
                ret get_x
            }
        ",
        )
        .await;

    assert!(
        result.is_ok(),
        "closure with let should capture: {:?}",
        result.err()
    );
}

// ── 4. Pattern matching in let ──────────────────────────────────────

#[tokio::test]
async fn task652_fn_let_list_pattern() {
    let engine = Engine::new().build().expect("engine builds");

    let result = engine
        .run(
            r"
            workflow main {
                let get_first = fn(items: List) -> Int {
                    let [first, ..rest] = items
                    first
                }
                ret get_first([42, 99, 100])
            }
        ",
        )
        .await;

    assert!(
        result.is_ok(),
        "list pattern in fn let should work: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), ash_core::Value::Int(42));
}

// ── 5. Let binding shadowing in fn body ──────────────────────────────

#[tokio::test]
async fn task652_fn_let_shadowing_in_fn_body() {
    let engine = Engine::new().build().expect("engine builds");

    // Top-level fn where the body shadows a parameter via let.
    // This tests that Expr::Let correctly shadows bindings within fn bodies.
    let result = engine
        .run(
            r"
            fn shadow_test(x: Int) -> Int {
                let x = 999
                x
            }

            workflow main {
                ret shadow_test(1)
            }
        ",
        )
        .await;

    assert!(
        result.is_ok(),
        "fn let shadowing should work: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), ash_core::Value::Int(999));
}

// ── 6. Pattern match failure at runtime in Expr::Let ──────────────────

#[tokio::test]
async fn task652_fn_let_pattern_bind_failure() {
    let engine = Engine::new().build().expect("engine builds");

    // `let [a, b] = 99` — list pattern vs int value → LetPatternBindFailed
    let result = engine
        .run(
            r"
            workflow main {
                let bad = fn() -> Int {
                    let [a, b] = 99
                    a
                }
                ret bad()
            }
        ",
        )
        .await;

    assert!(
        result.is_err(),
        "pattern mismatch in fn let should fail: got {:?}",
        result.ok()
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("pattern bind failed"),
        "error should mention pattern bind failure: {err_msg}"
    );
}

// ── 7. Imported pub fn with let-sequencing (3rd code path) ────────────

#[tokio::test]
async fn task652_imported_pub_fn_let_sequencing() {
    let tmp_dir = tempfile::tempdir().expect("tempdir");
    let dir = tmp_dir.path();

    // Write a file that defines a pub fn with let-sequencing, then uses it.
    // This exercises the module_loader code path for loaded pub fns.
    std::fs::write(
        dir.join("main.ash"),
        r"
pub fn add_and_double(x: Int, y: Int) -> Int {
    let sum = x + y
    let doubled = sum * 2
    doubled
}

workflow main {
    ret add_and_double(3, 4)
}
",
    )
    .expect("write main.ash");

    let engine = Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(dir.join("main.ash")).expect("parse");
    engine.check(&mut workflow).expect("check");
    let result = engine.execute(&workflow).await;

    assert!(
        result.is_ok(),
        "pub fn with let-sequencing should work: {:?}",
        result.err()
    );
    // (3 + 4) * 2 = 14
    assert_eq!(result.unwrap(), ash_core::Value::Int(14));
}
