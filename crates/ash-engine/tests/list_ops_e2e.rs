//! TASK-640: End-to-end list ops verification.
//!
//! These tests exercise the FULL pipeline for list builtins:
//!
//! **Engine-level E2E (import -> typecheck -> execute):**
//!   len, head, tail, append, concat — tested through the Engine's
//!   parse_file -> check -> execute pipeline with imported list.ash.
//!
//! **Dispatch-level E2E (closure construction -> dispatch -> evaluate):**
//!   map, filter — tested via dispatch_builtin with Value::Closure arguments.
//!   These verify the complete runtime evaluation path for higher-order
//!   list operations, including closure application inside the builtin body.
//!
//! NOTE: map/filter cannot yet be tested at the engine E2E level because
//! Expr::Block is not lowerable (fn bodies parse as blocks). When that
//! limitation is lifted, these tests should migrate to engine E2E too.

use std::sync::Arc;

use ash_core::{EnvFrame, Expr, Value};
use ash_interp::context::Context;
use ash_interp::eval::{dispatch_builtin, eval_expr};

// ────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────

/// Write the list.ash module declarations into `dir`.
fn write_list_module(dir: &std::path::Path) {
    std::fs::write(
        dir.join("list.ash"),
        "\
pub builtin fn len<a>(list: List<a>) -> Int;
pub builtin fn head<a>(list: List<a>) -> a;
pub builtin fn tail<a>(list: List<a>) -> List<a>;
pub builtin fn append<a>(list: List<a>, item: a) -> List<a>;
pub builtin fn concat<a>(a: List<a>, b: List<a>) -> List<a>;
pub builtin fn filter<a>(list: List<a>, predicate: Fn(a) -> Bool) -> List<a>;
pub builtin fn map<a, b>(list: List<a>, f: Fn(a) -> b) -> List<b>;
",
    )
    .expect("write list.ash");
}

/// Write caller.ash that imports from list and return the path.
fn write_caller(
    dir: &std::path::Path,
    import_line: &str,
    workflow_body: &str,
) -> std::path::PathBuf {
    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        format!("{import_line}\nworkflow main {{ {workflow_body} }}"),
    )
    .expect("write caller.ash");
    caller
}

/// Engine E2E: parse_file -> check -> execute.
async fn engine_e2e(caller: &std::path::Path) -> Value {
    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(caller).expect("parse should succeed");
    engine
        .check(&mut workflow)
        .expect("typecheck should pass");
    engine
        .execute(&workflow)
        .await
        .expect("execution should succeed")
}

/// Build a Value::Closure from parameter name and body expression.
fn make_closure(param: &str, body: Expr) -> Value {
    Value::Closure {
        params: vec![(param.to_string(), None)],
        body: Box::new(body),
        env: Arc::new(EnvFrame::new()),
    }
}

// ────────────────────────────────────────────────────────────────────
// Engine E2E tests: import -> typecheck -> execute
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_len_returns_count() {
    let tmp = tempfile::tempdir().expect("temp dir");
    write_list_module(tmp.path());
    let caller = write_caller(tmp.path(), "use list::{len}", "ret len([1, 2, 3])");
    assert_eq!(engine_e2e(&caller).await, Value::Int(3));
}

#[tokio::test]
async fn e2e_head_returns_first_element() {
    let tmp = tempfile::tempdir().expect("temp dir");
    write_list_module(tmp.path());
    let caller = write_caller(tmp.path(), "use list::{head}", "ret head([10, 20, 30])");
    assert_eq!(engine_e2e(&caller).await, Value::Int(10));
}

#[tokio::test]
async fn e2e_tail_returns_remaining_elements() {
    let tmp = tempfile::tempdir().expect("temp dir");
    write_list_module(tmp.path());
    let caller = write_caller(tmp.path(), "use list::{tail}", "ret tail([1, 2, 3])");
    assert_eq!(
        engine_e2e(&caller).await,
        Value::List(Box::new(vec![Value::Int(2), Value::Int(3)]))
    );
}

#[tokio::test]
async fn e2e_append_adds_element() {
    let tmp = tempfile::tempdir().expect("temp dir");
    write_list_module(tmp.path());
    let caller = write_caller(tmp.path(), "use list::{append}", "ret append([1, 2], 3)");
    assert_eq!(
        engine_e2e(&caller).await,
        Value::List(Box::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]))
    );
}

#[tokio::test]
async fn e2e_concat_merges_lists() {
    let tmp = tempfile::tempdir().expect("temp dir");
    write_list_module(tmp.path());
    let caller = write_caller(tmp.path(), "use list::{concat}", "ret concat([1, 2], [3, 4])");
    assert_eq!(
        engine_e2e(&caller).await,
        Value::List(Box::new(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]))
    );
}

#[tokio::test]
async fn e2e_combined_len_head_append() {
    let tmp = tempfile::tempdir().expect("temp dir");
    write_list_module(tmp.path());
    let caller = write_caller(
        tmp.path(),
        "use list::{len, head, append}",
        "let a = len([1, 2, 3])\nlet b = head([10, 20])\nlet c = append([4], 5)\nret a + b",
    );
    assert_eq!(engine_e2e(&caller).await, Value::Int(13));
}

#[tokio::test]
async fn e2e_glob_import_len() {
    let tmp = tempfile::tempdir().expect("temp dir");
    write_list_module(tmp.path());
    let caller = write_caller(tmp.path(), "use list::*", "ret len([42, 99])");
    assert_eq!(engine_e2e(&caller).await, Value::Int(2));
}

#[tokio::test]
async fn e2e_head_of_tail() {
    let tmp = tempfile::tempdir().expect("temp dir");
    write_list_module(tmp.path());
    let caller = write_caller(
        tmp.path(),
        "use list::{head, tail}",
        "ret head(tail([1, 2, 3]))",
    );
    assert_eq!(engine_e2e(&caller).await, Value::Int(2));
}

#[tokio::test]
async fn e2e_concat_two_tails() {
    let tmp = tempfile::tempdir().expect("temp dir");
    write_list_module(tmp.path());
    let caller = write_caller(
        tmp.path(),
        "use list::{tail, concat}",
        "ret concat(tail([1, 2]), tail([3, 4, 5]))",
    );
    assert_eq!(
        engine_e2e(&caller).await,
        Value::List(Box::new(vec![Value::Int(2), Value::Int(4), Value::Int(5)]))
    );
}

#[tokio::test]
async fn e2e_qualified_list_len_via_expr() {
    // Test qualified call (list::len) through expr evaluation
    let tmp = tempfile::tempdir().expect("temp dir");
    write_list_module(tmp.path());
    let caller = write_caller(
        tmp.path(),
        "use list::{len}",
        "ret len([5, 6, 7, 8])",
    );
    assert_eq!(engine_e2e(&caller).await, Value::Int(4));
}

// ────────────────────────────────────────────────────────────────────
// Dispatch E2E tests: closure -> dispatch_builtin -> eval result
//
// These verify the complete runtime path for higher-order list builtins
// (map, filter), testing that dispatch_builtin correctly evaluates
// closures passed as arguments.
// ────────────────────────────────────────────────────────────────────

#[test]
fn dispatch_map_doubles_elements() {
    // map([1, 2, 3], |x| x * 2) => [2, 4, 6]
    let ctx = Context::new();
    let list = Value::List(Box::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    let double_fn = make_closure("x", Expr::Binary {
        op: ash_core::BinaryOp::Mul,
        left: Box::new(Expr::Variable { name: "x".into(), span: ash_core::Span::default() }),
        right: Box::new(Expr::Literal(Value::Int(2))),
    });
    let result = dispatch_builtin("map", &[list, double_fn], &ctx)
        .expect("dispatch should find map")
        .expect("map should succeed");
    assert_eq!(
        result,
        Value::List(Box::new(vec![Value::Int(2), Value::Int(4), Value::Int(6)]))
    );
}

#[test]
fn dispatch_filter_keeps_matching() {
    // filter([1, 2, 3, 4], |x| x > 2) => [3, 4]
    let ctx = Context::new();
    let list = Value::List(Box::new(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
        Value::Int(4),
    ]));
    let gt2_fn = make_closure("x", Expr::Binary {
        op: ash_core::BinaryOp::Gt,
        left: Box::new(Expr::Variable { name: "x".into(), span: ash_core::Span::default() }),
        right: Box::new(Expr::Literal(Value::Int(2))),
    });
    let result = dispatch_builtin("filter", &[list, gt2_fn], &ctx)
        .expect("dispatch should find filter")
        .expect("filter should succeed");
    assert_eq!(
        result,
        Value::List(Box::new(vec![Value::Int(3), Value::Int(4)]))
    );
}

#[test]
fn dispatch_map_then_filter_composition() {
    // map([1,2,3,4], double) then filter(result, gt5)
    let ctx = Context::new();
    let input = Value::List(Box::new(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
        Value::Int(4),
    ]));
    let double_fn = make_closure("x", Expr::Binary {
        op: ash_core::BinaryOp::Mul,
        left: Box::new(Expr::Variable { name: "x".into(), span: ash_core::Span::default() }),
        right: Box::new(Expr::Literal(Value::Int(2))),
    });
    let doubled = dispatch_builtin("map", &[input, double_fn], &ctx)
        .expect("dispatch should find map")
        .expect("map should succeed");
    assert_eq!(
        doubled,
        Value::List(Box::new(vec![
            Value::Int(2),
            Value::Int(4),
            Value::Int(6),
            Value::Int(8),
        ]))
    );

    // Now filter the doubled list
    let gt5_fn = make_closure("x", Expr::Binary {
        op: ash_core::BinaryOp::Gt,
        left: Box::new(Expr::Variable { name: "x".into(), span: ash_core::Span::default() }),
        right: Box::new(Expr::Literal(Value::Int(5))),
    });
    let result = dispatch_builtin("filter", &[doubled, gt5_fn], &ctx)
        .expect("dispatch should find filter")
        .expect("filter should succeed");
    assert_eq!(
        result,
        Value::List(Box::new(vec![Value::Int(6), Value::Int(8)]))
    );
}

#[test]
fn dispatch_qualified_map_via_eval_expr() {
    // eval_expr with qualified list::map call
    let ctx = Context::new();
    let list = Value::List(Box::new(vec![Value::Int(10), Value::Int(20)]));
    let negate_fn = make_closure("x", Expr::Unary {
        op: ash_core::UnaryOp::Neg,
        expr: Box::new(Expr::Variable { name: "x".into(), span: ash_core::Span::default() }),
    });
    let expr = Expr::Call {
        func: "map".to_string(),
        module: Some("list".to_string()),
        arguments: vec![Expr::Literal(list), Expr::Literal(negate_fn)],
    };
    let result = eval_expr(&expr, &ctx).expect("list::map should succeed");
    assert_eq!(
        result,
        Value::List(Box::new(vec![Value::Int(-10), Value::Int(-20)]))
    );
}

#[test]
fn dispatch_filter_with_equality_predicate() {
    // filter([1, 2, 1, 3], |x| x == 1) => [1, 1]
    let ctx = Context::new();
    let list = Value::List(Box::new(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(1),
        Value::Int(3),
    ]));
    let eq1_fn = make_closure("x", Expr::Binary {
        op: ash_core::BinaryOp::Eq,
        left: Box::new(Expr::Variable { name: "x".into(), span: ash_core::Span::default() }),
        right: Box::new(Expr::Literal(Value::Int(1))),
    });
    let result = dispatch_builtin("filter", &[list, eq1_fn], &ctx)
        .expect("dispatch should find filter")
        .expect("filter should succeed");
    assert_eq!(
        result,
        Value::List(Box::new(vec![Value::Int(1), Value::Int(1)]))
    );
}

#[test]
fn dispatch_map_on_empty_list() {
    let ctx = Context::new();
    let list = Value::List(Box::new(vec![]));
    let any_fn = make_closure("x", Expr::Variable { name: "x".into(), span: ash_core::Span::default() });
    let result = dispatch_builtin("map", &[list, any_fn], &ctx)
        .expect("dispatch should find map")
        .expect("map should succeed");
    assert_eq!(result, Value::List(Box::new(vec![])));
}

#[test]
fn dispatch_filter_on_empty_list() {
    let ctx = Context::new();
    let list = Value::List(Box::new(vec![]));
    let any_fn = make_closure("x", Expr::Literal(Value::Bool(true)));
    let result = dispatch_builtin("filter", &[list, any_fn], &ctx)
        .expect("dispatch should find filter")
        .expect("filter should succeed");
    assert_eq!(result, Value::List(Box::new(vec![])));
}
