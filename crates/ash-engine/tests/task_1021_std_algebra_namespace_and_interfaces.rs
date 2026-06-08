//! TASK-1021 RED tests for importing `std::algebra` interfaces through stdlib modules.

use ash_parser::surface::Definition;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn std_src_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../std/src")
        .join(relative)
}

fn assert_stdlib_lib_exports_algebra_namespace() {
    let lib = std::fs::read_to_string(std_src_path("lib.ash")).expect("read std/src/lib.ash");
    assert!(
        lib.lines().any(|line| line.trim() == "pub mod algebra;"),
        "std/src/lib.ash must expose the algebra namespace with `pub mod algebra;`"
    );
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

fn interface_law_names(relative: &str, interface_name: &str) -> BTreeSet<String> {
    let source = std::fs::read_to_string(std_src_path(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"));
    let module = ash_parser::parse_surface_file(&source)
        .unwrap_or_else(|errors| panic!("{relative} should parse: {errors:?}\n{source}"));

    module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Interface(interface) if interface.name.as_ref() == interface_name => Some(
                interface
                    .laws
                    .iter()
                    .map(|law| law.name.to_string())
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{relative} should define interface {interface_name}"))
}

#[test]
fn algebra_interface_stdlib_files_parse_via_engine() {
    assert_stdlib_lib_exports_algebra_namespace();

    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");

    for relative in [
        "algebra/mod.ash",
        "algebra/semigroup.ash",
        "algebra/monoid.ash",
        "algebra/functor.ash",
        "algebra/applicative.ash",
        "algebra/monad.ash",
    ] {
        let path = std_src_path(relative);
        let result = engine
            .check_module_file(&path)
            .unwrap_or_else(|error| panic!("{relative} should parse/check: {error}"));
        assert!(
            result.errors.is_empty(),
            "{relative} should not report module errors: {:?}",
            result.errors
        );
    }
}

#[test]
fn algebra_interfaces_declare_expected_laws_in_stdlib_files() {
    assert_eq!(
        interface_law_names("algebra/semigroup.ash", "Semigroup"),
        BTreeSet::from(["associativity".to_string()])
    );
    assert_eq!(
        interface_law_names("algebra/monoid.ash", "Monoid"),
        BTreeSet::from(["left_identity".to_string(), "right_identity".to_string()])
    );
}

#[test]
fn algebra_interface_explicit_imports_resolve_from_builtin_stdlib() {
    assert_stdlib_lib_exports_algebra_namespace();

    let project = tempfile::tempdir().expect("project");
    let main = project.path().join("main.ash");
    std::fs::write(
        &main,
        r"use algebra::semigroup::{Semigroup}
use algebra::monoid::{Monoid}
use algebra::functor::{Functor}
use algebra::applicative::{Applicative}
use algebra::monad::{Monad}
workflow main { ret 0 }
",
    )
    .expect("main");

    let loaded = ash_engine::module_loader::load_ordinary_file(&main)
        .expect("explicit std::algebra interface imports should resolve");
    let names = imported_interface_names(&loaded);

    for expected in ["Semigroup", "Monoid", "Functor", "Applicative", "Monad"] {
        assert!(
            names.contains(expected),
            "expected imported interface {expected}; imported names: {names:?}"
        );
    }
}
