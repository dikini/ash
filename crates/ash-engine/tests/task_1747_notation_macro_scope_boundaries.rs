//! TASK-1747 regression tests for conservative notation and macro scope boundaries.

use ash_engine::Engine;
use ash_engine::module_loader::load_ordinary_file;

fn write_three(
    provider_source: &str,
    facade_source: &str,
    caller_source: &str,
) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let facade = dir.path().join("facade.ash");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&provider, provider_source).expect("write provider");
    std::fs::write(&facade, facade_source).expect("write facade");
    std::fs::write(&caller, caller_source).expect("write caller");
    (dir, caller)
}

#[test]
fn reexported_callable_target_remains_callable_without_notation_activation() {
    let (_dir, caller) = write_three(
        r"
pub infixl 6 <+> = combine;

pub fn combine(x: Int, y: Int) -> Int {
    x + y
}
",
        "pub use provider::{combine};\n",
        r"
use facade::{combine}
fn main() { combine(1, 2) }
",
    );

    load_ordinary_file(&caller).expect("re-exported callable target remains directly callable");
}

#[tokio::test]
async fn reexported_pub_notation_is_not_active_transitively() {
    let (_dir, caller) = write_three(
        r"
pub infixl 6 <+> = combine;

pub fn combine(x: Int, y: Int) -> Int {
    x + y
}
",
        "pub use provider::{combine};\n",
        r"
use facade::{combine}
fn main() { (<+>) }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let err = engine
        .run_file(&caller)
        .await
        .expect_err("re-exported callable must not activate provider notation");

    assert!(
        err.to_string().contains("<+>")
            && (err.to_string().contains("operator section")
                || err.to_string().contains("unsupported feature")),
        "unexpected error: {err}"
    );
}

#[test]
fn macro_scope_placeholder_syntax_does_not_activate_at_module_boundary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("module.ash");
    std::fs::write(
        &module,
        r"
pub macro make_id { }

pub fn use_macro() -> Int {
    make_id!(1)
}
",
    )
    .expect("write module");

    let engine = Engine::new().build().expect("engine builds");
    let err = engine
        .check_module_file(&module)
        .expect_err("macro-like syntax must remain fail-closed before Core");

    assert!(
        err.to_string().contains("parse") || err.to_string().contains("expanded-surface"),
        "unexpected error: {err}"
    );
}
