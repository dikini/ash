//! TASK-1737/TASK-1738 regression tests for expanded-surface engine boundaries.

use ash_engine::Engine;
use ash_engine::module_loader::check_importable_module_file;

fn write_module(source: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("module.ash");
    std::fs::write(&path, source).expect("write module");
    (dir, path)
}

fn write_pair(
    provider_source: &str,
    caller_source: &str,
) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&provider, provider_source).expect("write provider");
    std::fs::write(&caller, caller_source).expect("write caller");
    (dir, caller)
}

#[test]
fn check_module_file_rejects_unresolved_section_in_pub_fn_body() {
    let (_dir, path) = write_module(
        r"
pub fn unresolved_section() -> Int {
    (<*>)
}
",
    );
    let engine = Engine::new().build().expect("engine builds");

    let err = engine
        .check_module_file(&path)
        .expect_err("expanded-surface validation must reject unresolved sections");

    assert!(
        err.to_string().contains("operator section `<*>`"),
        "unexpected error: {err}"
    );
}

#[test]
fn check_module_file_accepts_expanded_builtin_section_in_pub_fn_body() {
    let (_dir, path) = write_module(
        r"
pub fn builtin_section() {
    (+)
}
",
    );
    let engine = Engine::new().build().expect("engine builds");

    engine
        .check_module_file(&path)
        .expect("built-in operator section should expand before module checking");
}

#[test]
fn check_module_file_accepts_expanded_local_notation_section_in_pub_fn_body() {
    let (_dir, path) = write_module(
        r"
infixl 6 <+> = combine;

pub fn combine(x: Int, y: Int) -> Int {
    x + y
}

pub fn local_section() {
    (<+>)
}
",
    );
    let engine = Engine::new().build().expect("engine builds");

    engine
        .check_module_file(&path)
        .expect("local notation section should expand before module checking");
}

#[test]
fn importable_module_rejects_unresolved_section_in_public_callable_body() {
    let (_dir, path) = write_module(
        r"
pub fn unresolved_section() -> Int {
    (<*>)
}
",
    );

    let err = check_importable_module_file(&path)
        .expect_err("importable module validation must reject unresolved public callable sections");

    assert!(
        err.to_string().contains("operator section `<*>`"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn imported_public_callable_uses_expanded_section_body() {
    let (_dir, caller) = write_pair(
        r"
pub fn add_one(x: Int) -> Int {
    let f = (1 +);
    f(x)
}
",
        r"
use provider::{add_one}
fn main() { add_one(2) }
",
    );
    let engine = Engine::new().build().expect("engine builds");

    let value = engine
        .run_file(&caller)
        .await
        .expect("imported callable body should be the expanded body");

    assert_eq!(value.to_string(), "3");
}
