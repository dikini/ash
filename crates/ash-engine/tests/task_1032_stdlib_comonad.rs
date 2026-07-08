//! TASK-1032 final-surface tests for `std::algebra::comonad`.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn std_src_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative)
}

fn imported_interface_names(
    loaded: &ash_engine::module_loader::LoadedOrdinaryFile,
) -> BTreeSet<String> {
    loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.interface_identities.iter())
        .map(|identity| identity.name.clone())
        .collect()
}

#[test]
fn stdlib_comonad_file_parses_and_checks_through_engine() {
    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");

    for relative in ["algebra/comonad.ash", "algebra/mod.ash"] {
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
fn stdlib_comonad_explicit_import_resolves_from_real_stdlib_path() {
    let project = tempfile::tempdir().expect("project");
    let main = project.path().join("main.ash");
    std::fs::write(
        &main,
        r"use algebra::comonad::{Comonad}
fn main() { 0 }
",
    )
    .expect("main");

    let loaded = ash_engine::module_loader::load_ordinary_file(&main)
        .expect("explicit stdlib Comonad import should resolve");
    let names = imported_interface_names(&loaded);

    assert!(
        names.contains("Comonad"),
        "expected imported interface Comonad; imported names: {names:?}"
    );
}
