//! TASK-1755 engine regressions for macro registry scope boundaries.

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
fn pub_macro_is_not_imported_as_callable_export() {
    let (_dir, caller) = write_pair(
        r"
pub macro inc(x) => add(x, 1);

pub fn add(x: Int, y: Int) -> Int {
    x + y
}
",
        r"
use provider::{add}

fn use_add(n: Int) -> Int {
    add(n, 1)
}
",
    );

    let loaded = load_ordinary_file(&caller).expect("ordinary callable import loads");
    assert!(loaded.imported_callables.contains_key("add"));
    assert!(
        !loaded.imported_callables.contains_key("inc"),
        "pub macro must not be transported as an imported callable"
    );
}

#[test]
fn imported_macro_declaration_does_not_activate_in_caller() {
    let (_dir, caller) = write_pair(
        r"
pub macro inc(x) => add(x, 1);

pub fn add(x: Int, y: Int) -> Int {
    x + y
}
",
        r"
use provider::{add}

fn use_macro(n: Int) -> Int {
    inc!(n)
}
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let err = engine
        .check_module_file(&caller)
        .expect_err("provider macro must not activate in caller");
    assert!(
        err.to_string()
            .contains("unknown local macro invocation `inc!`"),
        "unexpected error: {err}"
    );
}
