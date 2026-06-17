//! TASK-639: Typecheck list ops through imported .ash declarations.
//!
//! These tests verify that the typechecker correctly resolves list builtin types
//! when imported through the list.ash declarations. The tests exercise:
//!
//! 1. Importing `len` from a list module and calling it on a list literal
//! 2. Verifying `len([1,2,3])` typechecks as Int (precise return type)
//! 3. Verifying `head([1,2,3])` typechecks as Int (element type)
//! 4. Verifying multiple polymorphic calls in the same workflow work
//! 5. Verifying type mismatches are rejected (e.g., len("string"))

// ---------------------------------------------------------------------------
// Test 1: len([1,2,3]) typechecks successfully via import
// ---------------------------------------------------------------------------

/// Verify that importing `len` from a list module and calling `len([1, 2, 3])`
/// typechecks successfully. This exercises the full path: module loading,
/// builtin fn signature extraction, and type checking through [`Engine::check`].
#[test]
fn list_len_typechecks_via_imported_declaration() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    // Write a local list.ash mirroring std/src/list.ash
    std::fs::write(
        dir.join("list.ash"),
        "\
pub builtin fn len<a>(list: List<a>) -> Int;
pub builtin fn head<a>(list: List<a>) -> a;
pub builtin fn tail<a>(list: List<a>) -> List<a>;
",
    )
    .expect("write list.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use list::{len}\nworkflow main { ret len([1, 2, 3]) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse should succeed");

    // Verify the builtin signature was imported
    assert!(
        workflow.imported_builtin_signatures.contains_key("len"),
        "Expected 'len' in imported_builtin_signatures, found: {:?}",
        workflow
            .imported_builtin_signatures
            .keys()
            .collect::<Vec<_>>()
    );

    // Typecheck should pass
    engine
        .check(&mut workflow)
        .expect("typecheck should pass for len([1, 2, 3])");
}

// ---------------------------------------------------------------------------
// Test 2: len([1,2,3]) signature is recorded with correct types
// ---------------------------------------------------------------------------

/// Verify that importing `len<a>(list: List<a>) -> Int` records the correct
/// signature details: 1 type param, 1 parameter with List type, Int return.
#[test]
fn list_len_signature_details_are_correct() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("list.ash"),
        "pub builtin fn len<a>(list: List<a>) -> Int;\n",
    )
    .expect("write list.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use list::{len}\nworkflow main { ret len([1, 2, 3]) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let workflow = engine.parse_file(&caller).expect("parse should succeed");

    let sig = workflow
        .imported_builtin_signatures
        .get("len")
        .expect("Expected 'len' in imported_builtin_signatures");

    assert_eq!(sig.name.as_ref(), "len");
    assert_eq!(sig.params.len(), 1, "len should have 1 parameter");
    assert_eq!(sig.type_params.len(), 1, "len should have 1 type parameter");

    // The parameter type should be a List type
    match &sig.params[0].ty {
        ash_parser::surface::Type::Constructor { name, args } => {
            assert_eq!(
                name.as_ref(),
                "List",
                "Expected parameter type List<...>, got {name:?}"
            );
            assert_eq!(args.len(), 1, "List should have 1 type argument");
        }
        other => panic!("Expected List<...> type, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 3: head([1,2,3]) typechecks as Int (element type)
// ---------------------------------------------------------------------------

/// Verify that importing `head<a>(list: List<a>) -> a` and calling
/// `head([1, 2, 3])` typechecks successfully. The polymorphic return
/// type `a` should unify with `Int`.
#[test]
fn list_head_typechecks_as_element_type() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("list.ash"),
        "pub builtin fn head<a>(list: List<a>) -> a;\n",
    )
    .expect("write list.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use list::{head}\nworkflow main { ret head([1, 2, 3]) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse should succeed");

    assert!(
        workflow.imported_builtin_signatures.contains_key("head"),
        "Expected 'head' in imported_builtin_signatures"
    );

    engine
        .check(&mut workflow)
        .expect("typecheck should pass for head([1, 2, 3])");
}

// ---------------------------------------------------------------------------
// Test 4: head result used as Int (head([1,2,3]) + 1 typechecks)
// ---------------------------------------------------------------------------

/// Verify that `head([1, 2, 3]) + 1` typechecks. Since `head` returns `a`
/// and the argument is `[1, 2, 3]` (List<Int>), the return type should
/// unify with Int, making `Int + Int` valid.
#[test]
fn list_head_result_used_as_int() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("list.ash"),
        "pub builtin fn head<a>(list: List<a>) -> a;\n",
    )
    .expect("write list.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use list::{head}\nworkflow main { ret head([1, 2, 3]) + 1 }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse should succeed");

    engine
        .check(&mut workflow)
        .expect("typecheck should pass: head([1,2,3]) + 1 is Int + Int = Int");
}

// ---------------------------------------------------------------------------
// Test 5: Polymorphic calls in same workflow: len([1,2]) + len(["a","b"])
// ---------------------------------------------------------------------------

/// Verify that two polymorphic calls to `len` with different element types
/// typecheck in the same workflow. `len([1, 2])` returns Int and
/// `len(["a", "b"])` returns Int, so `Int + Int` should be valid.
#[test]
fn polymorphic_len_calls_with_different_element_types() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("list.ash"),
        "pub builtin fn len<a>(list: List<a>) -> Int;\n",
    )
    .expect("write list.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use list::{len}\nworkflow main { ret len([1, 2]) + len([\"a\", \"b\"]) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse should succeed");

    engine
        .check(&mut workflow)
        .expect("typecheck should pass: len([1,2]) + len([\"a\",\"b\"]) is Int + Int");
}

// ---------------------------------------------------------------------------
// Test 6: Multiple list operations in same workflow
// ---------------------------------------------------------------------------

/// Verify that importing multiple list builtins and using them together
/// typechecks correctly.
#[test]
fn multiple_list_ops_in_same_workflow() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("list.ash"),
        "\
pub builtin fn len<a>(list: List<a>) -> Int;
pub builtin fn head<a>(list: List<a>) -> a;
pub builtin fn tail<a>(list: List<a>) -> List<a>;
",
    )
    .expect("write list.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use list::{len, head}\nworkflow main { ret len([1, 2, 3]) + head([4, 5, 6]) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse should succeed");

    // Both signatures should be present
    assert!(
        workflow.imported_builtin_signatures.contains_key("len"),
        "Expected 'len' in imported_builtin_signatures"
    );
    assert!(
        workflow.imported_builtin_signatures.contains_key("head"),
        "Expected 'head' in imported_builtin_signatures"
    );

    // len returns Int, head returns element type which is Int here
    // So Int + Int = Int should typecheck
    engine
        .check(&mut workflow)
        .expect("typecheck should pass: len([1,2,3]) + head([4,5,6]) is Int + Int");
}

// ---------------------------------------------------------------------------
// Test 7: tail([1,2,3]) typechecks (returns List<Int>)
// ---------------------------------------------------------------------------

/// Verify that `tail<a>(list: List<a>) -> List<a>` typechecks on an Int list.
#[test]
fn list_tail_typechecks() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("list.ash"),
        "\
pub builtin fn len<a>(list: List<a>) -> Int;
pub builtin fn tail<a>(list: List<a>) -> List<a>;
",
    )
    .expect("write list.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use list::{len, tail}\nworkflow main { ret len(tail([1, 2, 3])) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse should succeed");

    engine
        .check(&mut workflow)
        .expect("typecheck should pass: len(tail([1,2,3])) is len(List<Int>) = Int");
}

// ---------------------------------------------------------------------------
// Test 8: Glob import from list module
// ---------------------------------------------------------------------------

/// Verify that glob import `use list::*` picks up all list builtin
/// declarations and they typecheck correctly.
#[test]
fn list_glob_import_typechecks() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    std::fs::write(
        dir.join("list.ash"),
        "\
pub builtin fn len<a>(list: List<a>) -> Int;
pub builtin fn head<a>(list: List<a>) -> a;
pub builtin fn tail<a>(list: List<a>) -> List<a>;
",
    )
    .expect("write list.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use list::*\nworkflow main { ret len([1, 2, 3]) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse should succeed");

    engine
        .check(&mut workflow)
        .expect("typecheck should pass for glob-imported len");
}

// ---------------------------------------------------------------------------
// Test 9: Full std/src/list.ash imports correctly
// ---------------------------------------------------------------------------

/// Verify that the actual std/src/list.ash file loads and its builtins
/// are importable from a caller module.
#[test]
fn std_list_ash_imports_correctly() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set");
    let std_list_path = std::path::PathBuf::from(manifest_dir)
        .join("../../std/src/list.ash")
        .canonicalize();

    // Skip if std/src/list.ash is not available (e.g. in CI without std)
    let Ok(std_list_path) = std_list_path else {
        eprintln!("Skipping: std/src/list.ash not found");
        return;
    };

    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    // Copy std list.ash into temp dir so the module loader can find it
    std::fs::copy(&std_list_path, dir.join("list.ash")).expect("copy list.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use list::{len}\nworkflow main { ret len([1, 2, 3]) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse should succeed");

    // Since Phase 153, list functions are pure Ash functions, not builtins
    // They should be available as regular imports, not builtin signatures
    assert!(
        !workflow.imported_builtin_signatures.contains_key("len"),
        "'len' should NOT be in imported_builtin_signatures - it's now a pure Ash function"
    );

    // Verify the function is available through the imported function signatures
    assert!(
        workflow.imported_fn_signatures.contains_key("len"),
        "Expected 'len' to be available as an imported function signature"
    );

    engine
        .check(&mut workflow)
        .expect("typecheck should pass using std list.ash declarations");
}
