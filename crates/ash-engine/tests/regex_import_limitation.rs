//! Historical-file-name regex builtin regression tests.
//!
//! This file keeps the old `regex_import_limitation` integration-test target
//! name so existing verification commands stay stable, but the limitation it
//! originally documented has been removed. These tests now cover the positive
//! imported-builtin path and one complementary runtime error case.

#[test]
fn regex_builtin_declarations_import_at_module_load_boundary() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let ash_file = tmp_dir.path().join("test_regex_import.ash");
    std::fs::write(
        &ash_file,
        "use regex::{find, matches, replace}\nfn test_regex() -> String {\n    {};\n}\n",
    )
    .expect("write temp ash file");

    let loaded = ash_engine::module_loader::load_ordinary_file(&ash_file)
        .expect("regex builtin imports should now resolve at module-load time");

    for (name, expected_params) in [
        ("find", vec!["pattern", "text"]),
        ("matches", vec!["pattern", "text"]),
        ("replace", vec!["pattern", "replacement", "text"]),
    ] {
        let callable = loaded
            .imported_callables
            .get(name)
            .unwrap_or_else(|| panic!("{name} should be imported from regex module"));

        assert!(
            matches!(
                &callable.kind,
                ash_engine::module_loader::CallableKind::Builtin { module }
                    if module == "regex"
            ),
            "Expected regex::{name} to be imported as a builtin callable, got: {:?}",
            callable.kind
        );
        assert_eq!(
            callable.params, expected_params,
            "Expected regex::{name} parameter names to be preserved"
        );
    }
}

#[tokio::test]
async fn regex_builtin_import_reports_invalid_pattern_at_runtime() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let ash_file = tmp_dir.path().join("test_regex_invalid_pattern.ash");
    std::fs::write(
        &ash_file,
        concat!(
            "use regex::{find}\n",
            "fn main() -> Option<String> {\n",
            "    find(\"(\", \"abc\")\n",
            "}\n",
        ),
    )
    .expect("write temp ash file");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&ash_file).expect("parse");
    engine.check(&mut workflow).expect("typecheck");

    let err = engine
        .execute(&workflow)
        .await
        .expect_err("invalid imported regex builtin pattern should surface a runtime error");
    let rendered = err.to_string();

    assert!(
        rendered.contains("Invalid regex pattern") || rendered.contains("regex parse error"),
        "expected invalid-pattern error from imported regex builtin, got: {rendered}"
    );
}
