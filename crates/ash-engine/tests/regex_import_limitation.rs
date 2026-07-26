//! Historical-file-name regex builtin regression tests.
//!
//! This file keeps the old `regex_import_limitation` integration-test target
//! name so existing verification commands stay stable. These tests cover the
//! imported-builtin parse/load/check path and the strict checked Core/CPS
//! admission boundary. Host regex execution, including invalid-pattern
//! diagnostics, awaits validated typed lowering and authorized async host
//! dispatch.

const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

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
async fn regex_builtin_import_with_invalid_pattern_rejects_at_closed_admission() {
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
    let mut application = engine.parse_file(&ash_file).expect("parse");
    engine.check(&mut application).expect("typecheck");

    let err = engine
        .execute(&application)
        .await
        .expect_err("regex source without validated typed lowering must reject at admission");

    assert_eq!(
        err.to_string(),
        format!("application execution failed: {CLOSED_ADMISSION_ERROR}"),
        "the invalid pattern must not reach legacy host regex dispatch before typed lowering"
    );
}
