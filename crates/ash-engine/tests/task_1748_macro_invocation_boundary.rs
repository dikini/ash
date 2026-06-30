//! TASK-1748 high-level macro invocation boundary tests.

use ash_engine::Engine;
use ash_engine::module_loader::check_importable_module_file;

fn write_module(source: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("module.ash");
    std::fs::write(&path, source).expect("write module");
    (dir, path)
}

#[test]
fn check_module_file_rejects_macro_invocation_in_public_callable_body() {
    let (_dir, path) = write_module(
        r"
pub fn use_macro() -> Int {
    make_id!(1)
}
",
    );
    let engine = Engine::new().build().expect("engine builds");

    let err = engine
        .check_module_file(&path)
        .expect_err("macro invocation must not be accepted by module validation");

    assert!(
        err.to_string()
            .contains("unknown local macro invocation `make_id!`"),
        "unexpected error: {err}"
    );
}

#[test]
fn importable_module_rejects_macro_invocation_before_export_acceptance() {
    let (_dir, path) = write_module(
        r"
pub fn use_macro() -> Int {
    make_id!(1)
}
",
    );

    let err = check_importable_module_file(&path)
        .expect_err("public callable with macro invocation must not become importable");

    assert!(
        err.to_string()
            .contains("unknown local macro invocation `make_id!`"),
        "unexpected error: {err}"
    );
}
