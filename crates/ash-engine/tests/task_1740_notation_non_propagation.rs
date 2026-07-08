//! TASK-1740 non-propagation tests for notation import/export scope.

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

#[tokio::test]
async fn imported_pub_notation_is_not_active_in_caller_scope() {
    let (_dir, caller) = write_pair(
        r"
pub infixl 6 <+> = combine;

pub fn combine(x: Int, y: Int) -> Int {
    x + y
}
",
        r"
use provider::{combine}
fn main() { (<+>) }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let err = engine
        .run_file(&caller)
        .await
        .expect_err("importing combine must not import provider's notation declaration");

    assert!(
        err.to_string().contains("unsupported feature")
            || err.to_string().contains("operator section"),
        "unexpected error: {err}"
    );
    assert!(
        err.to_string().contains("<+>"),
        "error should name the inactive imported operator: {err}"
    );
}

#[test]
fn callable_target_import_remains_usable_when_notation_alias_does_not_propagate() {
    let (_dir, caller) = write_pair(
        r"
pub infixl 6 <+> = combine;

pub fn combine(x: Int, y: Int) -> Int {
    x + y
}
",
        r"
use provider::{combine}
fn main() { combine(1, 2) }
",
    );

    load_ordinary_file(&caller).expect("ordinary callable import remains usable by direct call");
}
