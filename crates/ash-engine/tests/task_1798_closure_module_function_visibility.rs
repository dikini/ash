//! TASK-1798 closure/module-level function visibility regressions.

use ash_core::Value;

#[tokio::test]
async fn local_closure_can_call_sibling_module_pure_helper() {
    let source = r"
fn helper(x: Int) -> Int {
    x + 1
}

fn apply(f: (Int) -> Int, n: Int) -> Int {
    f(n)
}

fn run_with_closure(n: Int) -> Int {
    apply(fn(x: Int) -> Int { helper(x) }, n)
}

fn main() {
    run_with_closure(41)
}
";

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse(source).expect("parse");
    engine.check(&mut application).expect("typecheck");

    let result = engine.execute(&application).await.expect("execute");
    assert_eq!(result, Value::Int(42));
}

#[tokio::test]
async fn imported_private_helper_used_by_nested_closure_does_not_leak_to_caller() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");

    std::fs::write(
        &provider,
        r"
fn helper(x: Int) -> Int {
    x + 1
}

fn apply(f: (Int) -> Int, n: Int) -> Int {
    f(n)
}

pub fn run_with_closure(n: Int) -> Int {
    apply(fn(x: Int) -> Int { helper(x) }, n)
}
",
    )
    .expect("write provider");

    std::fs::write(
        &caller,
        r"use provider::{run_with_closure}
fn main() {
    run_with_closure(41)
}
",
    )
    .expect("write caller");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(&caller).expect("parse caller");
    engine.check(&mut application).expect("typecheck caller");
    let result = engine.execute(&application).await.expect("execute caller");
    assert_eq!(result, Value::Int(42));

    std::fs::write(
        &caller,
        r"use provider::{run_with_closure}
fn main() {
    helper(41)
}
",
    )
    .expect("write leakage caller");

    let mut application = engine.parse_file(&caller).expect("parse leakage caller");
    let _ = engine.check(&mut application);
    let err = engine
        .execute(&application)
        .await
        .expect_err("private helper must not leak into caller runtime bindings");
    assert!(
        err.to_string().contains("Undefined variable") || err.to_string().contains("helper"),
        "expected unknown private helper diagnostic, got: {err}"
    );
}
#[tokio::test]
async fn imported_private_helpers_with_same_name_stay_module_local() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_a = dir.path().join("provider_a.ash");
    let provider_b = dir.path().join("provider_b.ash");
    let caller = dir.path().join("caller.ash");

    std::fs::write(
        &provider_a,
        r"
fn helper(x: Int) -> Int {
    x + 1
}

fn apply(f: (Int) -> Int, n: Int) -> Int {
    f(n)
}

pub fn run_a(n: Int) -> Int {
    apply(fn(x: Int) -> Int { helper(x) }, n)
}
",
    )
    .expect("write provider a");

    std::fs::write(
        &provider_b,
        r"
fn helper(x: Int) -> Int {
    x + 100
}

fn apply(f: (Int) -> Int, n: Int) -> Int {
    f(n)
}

pub fn run_b(n: Int) -> Int {
    apply(fn(x: Int) -> Int { helper(x) }, n)
}
",
    )
    .expect("write provider b");

    std::fs::write(
        &caller,
        r"use provider_a::{run_a}
use provider_b::{run_b}

fn main() {
    run_a(1) + run_b(1)
}
",
    )
    .expect("write caller");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut application = engine.parse_file(&caller).expect("parse caller");
    engine.check(&mut application).expect("typecheck caller");
    let result = engine.execute(&application).await.expect("execute caller");

    assert_eq!(result, Value::Int(103));
}
