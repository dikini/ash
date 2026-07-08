//! TASK-641: Integration tests for the predicate builtin module.
//!
//! Verifies that std/src/predicate.ash parses correctly and its declarations
//! are importable from other modules via the module loader.

use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Test 1: predicate.ash parses via check_module_file
// ---------------------------------------------------------------------------

/// Verify that `std/src/predicate.ash` parses successfully using
/// `Engine::check_module_file`. This confirms the file is syntactically valid.
#[test]
fn predicate_ash_parses_via_check_module_file() {
    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set");
    let predicate_path = PathBuf::from(manifest_dir)
        .join("../../std/src/predicate.ash")
        .canonicalize()
        .expect("predicate.ash should exist");

    let result = engine
        .check_module_file(&predicate_path)
        .expect("check_module_file should succeed for predicate.ash");

    // check_module_file counts pub fn and pub type definitions; builtin fn
    // declarations are not counted separately but the file must parse cleanly.
    assert_eq!(
        result.type_count, 0,
        "predicate.ash should have 0 type definitions, got {}",
        result.type_count
    );
    assert!(
        result.errors.is_empty(),
        "predicate.ash should have no errors: {:?}",
        result.errors
    );
}

// ---------------------------------------------------------------------------
// Test 2: predicate builtins are importable from another module
// ---------------------------------------------------------------------------

/// Verify that a caller module can `use predicate::{is_int}` and that the
/// import resolves as a `CallableKind::Builtin` entry at module-load time.
#[test]
fn predicate_builtin_import_resolves_at_module_load() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    // Write a local predicate.ash that mirrors std/src/predicate.ash. The
    // generic parameter is part of the public signature and must not be treated
    // as an unresolved ordinary type by import/export validation.
    std::fs::write(
        dir.join("predicate.ash"),
        "\
-- Type predicate functions
pub builtin fn is_int<a>(value: a) -> Bool;
pub builtin fn is_string<a>(value: a) -> Bool;
pub builtin fn is_bool<a>(value: a) -> Bool;
pub builtin fn is_list<a>(value: a) -> Bool;
pub builtin fn is_record<a>(value: a) -> Bool;
pub builtin fn is_null<a>(value: a) -> Bool;
",
    )
    .expect("write predicate.ash");

    // Write a caller that imports is_int from the predicate module
    let caller = dir.join("caller.ash");
    std::fs::write(&caller, "use predicate::{is_int}\nfn main() { {}; }\n")
        .expect("write caller.ash");

    let result = ash_engine::module_loader::load_ordinary_file(&caller);
    assert!(
        result.is_ok(),
        "Expected import from predicate module to succeed, but got error: {:?}",
        result.err()
    );

    let loaded = result.unwrap();

    // Verify is_int was imported as a builtin callable
    let is_int_callable = loaded
        .imported_callables
        .get("is_int")
        .expect("is_int should be imported from predicate module");

    assert!(
        matches!(
            &is_int_callable.kind,
            ash_engine::module_loader::CallableKind::Builtin { module }
                if module == "predicate"
        ),
        "Expected is_int to be imported as CallableKind::Builtin from module 'predicate', got: {:?}",
        is_int_callable.kind
    );
    assert_eq!(
        is_int_callable.params,
        vec!["value"],
        "Expected is_int parameter name 'value' to be preserved"
    );
}

// ---------------------------------------------------------------------------
// Test 3: All six predicate builtins import via glob
// ---------------------------------------------------------------------------

/// Verify that glob import `use predicate::*` picks up all six builtin
/// declarations.
#[test]
fn predicate_all_builtins_import_via_glob() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("predicate.ash"),
        "\
pub builtin fn is_int<a>(value: a) -> Bool;
pub builtin fn is_string<a>(value: a) -> Bool;
pub builtin fn is_bool<a>(value: a) -> Bool;
pub builtin fn is_list<a>(value: a) -> Bool;
pub builtin fn is_record<a>(value: a) -> Bool;
pub builtin fn is_null<a>(value: a) -> Bool;
",
    )
    .expect("write predicate.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(&caller, "use predicate::*\nfn main() { {}; }\n").expect("write caller.ash");

    let result = ash_engine::module_loader::load_ordinary_file(&caller);
    assert!(
        result.is_ok(),
        "Expected glob import from predicate to succeed, but got error: {:?}",
        result.err()
    );

    let loaded = result.unwrap();
    for name in &[
        "is_int",
        "is_string",
        "is_bool",
        "is_list",
        "is_record",
        "is_null",
    ] {
        assert!(
            loaded.imported_callables.contains_key(*name),
            "Expected '{name}' in imported_callables from glob import of predicate module"
        );
    }
}
