//! TASK-1769 high-level hygienic binder macro boundary tests.

use ash_engine::Engine;

fn write_one(source: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("main.ash");
    std::fs::write(&path, source).expect("write source");
    (dir, path)
}

#[test]
fn hygienic_fn_binder_macro_checks_through_engine_boundary() {
    let (_dir, path) = write_one(
        r"
macro const_fn(y) => fn(x: Int) -> Int { y };
fn apply(f: (Int) -> Int, n: Int) -> Int { f(n) }
fn use_macro(x: Int) -> Int { apply(const_fn!(x), 0) }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    engine
        .check_module_file(&path)
        .expect("hygienic binder macro should expand before checking");
}

#[test]
fn unsupported_binder_macro_shape_fails_before_engine_acceptance() {
    let (_dir, path) = write_one(
        r"
macro with_block(x) => fn(y: Int) -> Int { let z = x; z };
fn apply(f: (Int) -> Int, n: Int) -> Int { f(n) }
fn use_macro(n: Int) -> Int { apply(with_block!(n), 0) }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let err = engine
        .check_module_file(&path)
        .expect_err("block-bodied binder macro must fail closed");
    assert!(
        err.to_string()
            .contains("macro `with_block` uses unsupported template syntax: block"),
        "unexpected error: {err}"
    );
}
