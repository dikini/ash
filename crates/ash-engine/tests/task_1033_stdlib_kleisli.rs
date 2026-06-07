//! TASK-1033 final-surface tests for `std::algebra::kleisli`.

use std::path::PathBuf;

fn std_src_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative)
}

#[test]
fn stdlib_kleisli_file_parses_and_checks_through_engine() {
    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");

    for relative in ["algebra/kleisli.ash", "algebra/mod.ash"] {
        let result = engine
            .check_module_file(&std_src_path(relative))
            .unwrap_or_else(|error| panic!("{relative} should parse/check: {error}"));
        assert!(
            result.errors.is_empty(),
            "{relative} should not report module errors: {:?}",
            result.errors
        );
    }
}

#[test]
fn stdlib_kleisli_explicit_helper_imports_resolve_from_real_stdlib_path() {
    let project = tempfile::tempdir().expect("project");
    let main = project.path().join("main.ash");
    std::fs::write(
        &main,
        r"use algebra::kleisli::{id_option, compose_option, id_result, compose_result}
workflow main { ret 0 }
",
    )
    .expect("main");

    ash_engine::module_loader::load_ordinary_file(&main)
        .expect("explicit stdlib Kleisli helper imports should resolve");
}
