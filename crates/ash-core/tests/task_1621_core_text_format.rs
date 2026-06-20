use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("ash-core crate lives under crates/ash-core")
        .to_path_buf()
}

fn fixture_path(name: &str) -> PathBuf {
    repo_root()
        .join("crates/ash-core/tests/fixtures/core")
        .join(name)
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    })
}

#[test]
fn required_core_fixture_files_exist_and_are_small() {
    for file in [
        "let_val_jump.core",
        "let_prim_if.core",
        "call_non_tail.core",
        "raise_handle.core",
        "contract_trap.core",
    ] {
        let path = fixture_path(file);
        let source = read(&path);
        assert!(
            source.lines().count() <= 20,
            "{} should fit on one screen",
            path.display()
        );
        assert!(
            source.trim_start().starts_with('('),
            "{} should be S-expression-like Core text",
            path.display()
        );
        assert!(
            !source.contains("workflow ") && !source.contains("do "),
            "{} must not use surface Ash syntax",
            path.display()
        );
    }
}

#[test]
fn fixtures_cover_expected_top_level_core_forms() {
    let cases = [
        ("let_val_jump.core", "(let-val", "(jump"),
        ("let_prim_if.core", "(let-prim", "(if"),
        ("call_non_tail.core", "(let-val", "(call"),
        ("raise_handle.core", "(handle", "(raise"),
        ("contract_trap.core", "(record-discharge", "(trap"),
    ];

    for (file, top_level, inner_form) in cases {
        let source = read(&fixture_path(file));
        assert!(
            source.trim_start().starts_with(top_level),
            "{file} should start with {top_level}"
        );
        assert!(
            source.contains(inner_form),
            "{file} should contain {inner_form}"
        );
    }
}

#[test]
fn reference_page_freezes_core_text_boundary_and_required_forms() {
    let path = repo_root().join("docs/reference/core-ash-text-format.md");
    let source = read(&path);

    assert!(source.contains(".core is a fixture/debug format"));
    assert!(source.contains("not surface Ash"));

    for form in [
        "let-val",
        "let-prim",
        "if",
        "call",
        "raise",
        "handle",
        "record-discharge",
        "trap",
    ] {
        assert!(
            source.contains(form),
            "reference page should document {form}"
        );
    }
}
