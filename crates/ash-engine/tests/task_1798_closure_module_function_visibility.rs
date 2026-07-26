//! TASK-1798 closure/module-level function visibility regressions.

const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

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

    let error = engine
        .execute(&application)
        .await
        .expect_err("source without validated typed lowering must reject at admission");
    assert!(
        matches!(error, ash_interp::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR),
        "closure visibility source must expose the exact canonical closed-admission error"
    );
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
    let error = engine
        .execute(&application)
        .await
        .expect_err("source without validated typed lowering must reject at admission");
    assert!(
        matches!(error, ash_interp::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR),
        "imported closure source must expose the exact canonical closed-admission error"
    );

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
    let visibility_error = engine
        .check(&mut application)
        .expect_err("private helper must remain unavailable to the importing module");
    assert!(
        visibility_error.to_string().contains("helper"),
        "visibility diagnostic must name the hidden private helper: {visibility_error}"
    );
    let err = engine
        .execute(&application)
        .await
        .expect_err("source without validated typed lowering must reject at admission");
    assert!(
        matches!(err, ash_interp::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR),
        "unchecked private-helper source must still expose the exact canonical closed-admission error"
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
    let error = engine
        .execute(&application)
        .await
        .expect_err("source without validated typed lowering must reject at admission");
    assert!(
        matches!(error, ash_interp::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR),
        "multi-module closure source must expose the exact canonical closed-admission error"
    );
}
