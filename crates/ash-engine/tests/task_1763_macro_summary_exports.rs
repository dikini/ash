//! TASK-1763 engine regressions for macro summary export collection.

use ash_engine::module_loader::{check_importable_module_file, load_ordinary_file};

fn write_pair(
    provider_source: &str,
    caller_source: &str,
) -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    std::fs::write(&provider, provider_source).expect("write provider");
    std::fs::write(&caller, caller_source).expect("write caller");
    (dir, provider, caller)
}

#[test]
fn public_macro_import_collects_summary_without_callable_leakage() {
    let (_dir, _provider, caller) = write_pair(
        r"
pub macro inc(x) => add(x, 1);
pub fn add(x: Int, y: Int) -> Int { x + y }
",
        r"
use provider::{inc, add}

fn use_add(n: Int) -> Int { add(n, 1) }
",
    );

    let loaded = load_ordinary_file(&caller).expect("caller imports public macro summary");
    assert!(loaded.imported_callables.contains_key("add"));
    assert!(
        !loaded.imported_callables.contains_key("inc"),
        "macro summary must not be transported as an imported callable"
    );
    assert_eq!(loaded.imported_macro_summaries.len(), 1);
    let summary = &loaded.imported_macro_summaries[0];
    assert_eq!(summary.name.as_ref(), "inc");
    assert_eq!(summary.params, vec!["x".into()]);
}

#[test]
fn malformed_public_macro_summary_rejects_importable_module() {
    let (_dir, provider, _caller) = write_pair(
        r"
pub macro bad(x) => y;
pub fn add(x: Int, y: Int) -> Int { x + y }
",
        "fn main(n: Int) -> Int { n }",
    );

    let err = check_importable_module_file(&provider)
        .expect_err("malformed public macro summary must fail closed");
    assert!(
        err.to_string().contains("invalid public macro summary")
            && err.to_string().contains("free variable"),
        "unexpected error: {err}"
    );
}
