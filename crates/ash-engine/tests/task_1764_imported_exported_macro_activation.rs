//! TASK-1764 regressions for bounded imported/exported macro activation.

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

#[test]
fn imported_public_macro_expands_at_authorized_named_import_site() {
    let (_dir, caller) = write_pair(
        r"
pub macro inc(x) => add(x, 1);
pub fn add(x: Int, y: Int) -> Int { x + y }
",
        r"
use provider::{inc, add}

fn use_macro(n: Int) -> Int { inc!(n) }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    engine
        .check_module_file(&caller)
        .expect("named public macro import expands before checking");

    let loaded = load_ordinary_file(&caller).expect("caller loads provider metadata");
    assert_eq!(loaded.imported_macro_summaries.len(), 1);
    assert!(
        !loaded.imported_callables.contains_key("inc"),
        "imported macro must not create a callable binding"
    );
}

#[test]
fn imported_public_macro_alias_expands_under_alias_only() {
    let (_dir, caller) = write_pair(
        r"
pub macro inc(x) => add(x, 1);
pub fn add(x: Int, y: Int) -> Int { x + y }
",
        r"
use provider::{inc as bump, add}

fn use_macro(n: Int) -> Int { bump!(n) }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    engine
        .check_module_file(&caller)
        .expect("aliased public macro import expands under the alias");
}

#[test]
fn private_macro_remains_inaccessible_across_module_boundary() {
    let (_dir, caller) = write_pair(
        r"
macro inc(x) => add(x, 1);
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
        .expect_err("private macro must not be visible to caller");
    assert!(
        err.to_string()
            .contains("unknown local macro invocation `inc!`"),
        "unexpected error: {err}"
    );
}

#[test]
fn callable_import_with_macro_name_does_not_activate_macro_syntax() {
    let (_dir, caller) = write_pair(
        r"
pub fn inc(x: Int) -> Int { x + 1 }
",
        r"
use provider::{inc}

fn use_macro(n: Int) -> Int { inc!(n) }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let err = engine
        .check_module_file(&caller)
        .expect_err("callable import must not activate macro syntax");
    assert!(
        err.to_string()
            .contains("unknown local macro invocation `inc!`"),
        "unexpected error: {err}"
    );
}
