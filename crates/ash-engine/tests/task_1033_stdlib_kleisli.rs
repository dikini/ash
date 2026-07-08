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
fn stdlib_kleisli_module_no_longer_exports_concrete_carrier_wrappers() {
    let project = tempfile::tempdir().expect("project");
    let main = project.path().join("main.ash");
    std::fs::write(
        &main,
        r"use algebra::kleisli::{id_option}
fn main() { 0 }
",
    )
    .expect("main");

    let error = ash_engine::module_loader::load_ordinary_file(&main)
        .expect_err("concrete Kleisli wrappers should not resolve from std::algebra");
    let message = error.to_string();
    assert!(message.contains("id_option"), "{message}");
}
