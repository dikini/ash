//! TASK-1758 high-level Phase 172 macro execution boundary tests.

use ash_engine::Engine;
use ash_engine::module_loader::load_ordinary_file;

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

fn write_one(source: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("main.ash");
    std::fs::write(&path, source).expect("write source");
    (dir, path)
}

#[test]
fn local_supported_macro_expands_through_check_module_file() {
    let (_dir, path) = write_one(
        r"
macro inc(x) => add(x, 1);
fn add(x: Int, y: Int) -> Int { x + y }
fn use_macro(n: Int) -> Int { inc!(n) }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    engine
        .check_module_file(&path)
        .expect("local supported expression macro should expand before checking");
}

#[test]
fn exported_callable_using_local_macro_loads_as_ordinary_callable_body() {
    let (_dir, caller) = write_pair(
        r"
macro inc(x) => add(x, 1);
pub fn add(x: Int, y: Int) -> Int { x + y }
pub fn exported(n: Int) -> Int { inc!(n) }
",
        r"
use provider::{exported}

fn use_exported(n: Int) -> Int { exported(n) }
",
    );

    let loaded = load_ordinary_file(&caller).expect("caller loads provider exports");
    let callable = loaded
        .imported_callables
        .get("exported")
        .expect("exported callable is imported");
    assert_eq!(callable.exported_name, "exported");
    assert!(
        !loaded.imported_callables.contains_key("inc"),
        "macro declaration must not be imported as a callable"
    );
}

#[test]
fn imported_macro_declaration_still_does_not_activate_in_caller() {
    let (_dir, caller) = write_pair(
        r"
pub macro inc(x) => add(x, 1);
pub fn add(x: Int, y: Int) -> Int { x + y }
",
        r"
use provider::{add}

fn use_macro(n: Int) -> Int { inc!(n) }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let err = engine
        .check_module_file(&caller)
        .expect_err("imported macro declarations must not activate in caller");
    assert!(
        err.to_string()
            .contains("unknown local macro invocation `inc!`"),
        "unexpected error: {err}"
    );
}

#[test]
fn unsupported_macro_syntax_rejects_before_high_level_acceptance() {
    let (_dir, path) = write_one(
        r"
macro inc(x) => add(x, 1);
fn add(x: Int, y: Int) -> Int { x + y }
fn use_macro(n: Int) -> Int { inc![n] }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let err = engine
        .check_module_file(&path)
        .expect_err("unsupported macro syntax must fail closed");
    assert!(
        err.to_string()
            .contains("macro invocation `inc!` uses unsupported Phase 172 MVP syntax"),
        "unexpected error: {err}"
    );
}
