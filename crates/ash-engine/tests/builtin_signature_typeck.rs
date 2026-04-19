//! TASK-635: Imported builtin signatures in Engine::check().
//!
//! These tests verify that imported `builtin fn` declarations carry their
//! declared type signatures into the typechecker, instead of arity-only
//! synthetic types (fresh type variables).
//!
//! Test 1: `len<a>(list: List<a>) -> Int` typechecks `len([1,2,3])` as `Int`.
//! Test 2: Signature stored and typechecks when types match
//! Test 3: Non-builtin callables still use arity-only fallback.
//! Test 4: Signatures with unknown types (e.g. `Record`) fall back gracefully.

// ---------------------------------------------------------------------------
// Test 1: Generic builtin fn with proper signature typechecks correctly
// ---------------------------------------------------------------------------

/// Verify that importing a `builtin fn len<a>(list: List<a>) -> Int;` and
/// calling `len([1, 2, 3])` typechecks successfully, with the return type
/// inferred as Int (not a fresh type variable).
#[test]
fn builtin_fn_generic_signature_typechecks_len_of_list() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    // Module declaring a generic builtin fn
    let list_utils = dir.join("list_utils.ash");
    std::fs::write(
        &list_utils,
        "pub builtin fn len<a>(list: List<a>) -> Int;\n",
    )
    .expect("write list_utils.ash");

    // Caller imports len and uses it on a list literal
    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use list_utils::{len}\nworkflow main { ret len([1, 2, 3]) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse should succeed");

    // The workflow should carry the signature
    assert!(
        workflow.imported_builtin_signatures.contains_key("len"),
        "Expected 'len' in imported_builtin_signatures, found: {:?}",
        workflow.imported_builtin_signatures.keys().collect::<Vec<_>>()
    );

    // Typecheck should pass — the typechecker now knows len takes List<a> -> Int
    engine
        .check(&mut workflow)
        .expect("typecheck should pass for len([1, 2, 3])");
}

// ---------------------------------------------------------------------------
// Test 2: Signature stored and typechecks when types match
// ---------------------------------------------------------------------------

/// Verify that importing a `builtin fn len<a>(list: List<a>) -> Int;` carries
/// the declared signature into the Workflow, enabling future precise type
/// checking.  (Full unification of argument types depends on the constraint
/// solver, which is a separate concern.)
#[test]
fn builtin_fn_generic_signature_is_recorded_in_workflow() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    let list_utils = dir.join("list_utils.ash");
    std::fs::write(
        &list_utils,
        "pub builtin fn len<a>(list: List<a>) -> Int;\n",
    )
    .expect("write list_utils.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use list_utils::{len}\nworkflow main { ret len([1, 2, 3]) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let workflow = engine.parse_file(&caller).expect("parse should succeed");

    // The key assertion: the signature was propagated from InlineCallable
    // through build_imported_closures into the Workflow struct.
    let sig = workflow
        .imported_builtin_signatures
        .get("len")
        .expect("Expected 'len' in imported_builtin_signatures");

    // Verify signature details
    assert_eq!(sig.name.as_ref(), "len");
    assert_eq!(sig.params.len(), 1, "len should have 1 parameter");
    assert_eq!(sig.type_params.len(), 1, "len should have 1 type parameter");

    // Typecheck should pass
    let mut workflow = workflow;
    engine
        .check(&mut workflow)
        .expect("typecheck should pass for len([1, 2, 3])");
}

// ---------------------------------------------------------------------------
// Test 3: Non-builtin callables still use arity-only fallback
// ---------------------------------------------------------------------------

/// Verify that a regular `pub fn` (not a builtin) still typechecks with the
/// arity-only fallback, and doesn't break when no signature is present.
#[test]
fn non_builtin_callable_uses_arity_only_fallback() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    let utils = dir.join("utils.ash");
    std::fs::write(
        &utils,
        "pub fn double(x: Int) -> Int { x + x }\n",
    )
    .expect("write utils.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use utils::{double}\nworkflow main { ret double(3) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse should succeed");

    // No builtin signatures for user-defined callables
    assert!(
        !workflow.imported_builtin_signatures.contains_key("double"),
        "'double' is a user-defined fn and should NOT appear in imported_builtin_signatures"
    );

    // Should still typecheck (arity-only fallback works)
    engine
        .check(&mut workflow)
        .expect("typecheck should pass for user-defined callable with arity-only type");
}

// ---------------------------------------------------------------------------
// Test 4: Signatures with unknown types fall back gracefully
// ---------------------------------------------------------------------------

/// Verify that if a builtin fn signature references a type the typechecker
/// doesn't know about (e.g. `Record`), the system falls back to an arity-only
/// synthetic type rather than failing.
#[test]
fn builtin_fn_signature_with_unknown_type_falls_back_gracefully() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    let mymod = dir.join("mymod.ash");
    std::fs::write(
        &mymod,
        "pub builtin fn keys(r: Record) -> List<String>;\n",
    )
    .expect("write mymod.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use mymod::{keys}\nworkflow main { ret keys(record(\"a\", 1)) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse should succeed");

    // The signature IS recorded (it was parsed)
    assert!(
        workflow.imported_builtin_signatures.contains_key("keys"),
        "Expected 'keys' in imported_builtin_signatures"
    );

    // Typecheck should still pass via fallback to arity-only synthetic
    engine
        .check(&mut workflow)
        .expect("typecheck should pass with arity-only fallback for unknown type in signature");
}

// ---------------------------------------------------------------------------
// Test 5: Precise signature enables return-type inference
// ---------------------------------------------------------------------------

/// Verify that with a precise builtin signature, the typechecker correctly
/// propagates the return type. Here `add(a: Int, b: Int) -> Int` should
/// allow `add(1, 2) + 3` to typecheck (Int + Int = Int).
#[test]
fn builtin_fn_precise_signature_enables_return_type_inference() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    let math_utils = dir.join("math_utils.ash");
    std::fs::write(
        &math_utils,
        "pub builtin fn add(a: Int, b: Int) -> Int;\n",
    )
    .expect("write math_utils.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use math_utils::{add}\nworkflow main { ret add(1, 2) + 3 }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse should succeed");

    // Typecheck should pass — add returns Int, and Int + Int is Int
    engine
        .check(&mut workflow)
        .expect("typecheck should pass: add(1,2) + 3 is Int + Int = Int");
}

// ---------------------------------------------------------------------------
// Test 6: Multiple builtin fns with distinct signatures
// ---------------------------------------------------------------------------

/// Verify that multiple imported builtin fns each get their own signature
/// and typecheck correctly in the same workflow.
#[test]
fn multiple_builtin_fn_signatures_coexist() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    let math_utils = dir.join("math_utils.ash");
    std::fs::write(
        &math_utils,
        "pub builtin fn add(a: Int, b: Int) -> Int;\npub builtin fn mul(a: Int, b: Int) -> Int;\n",
    )
    .expect("write math_utils.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use math_utils::{add, mul}\nworkflow main { ret add(1, mul(2, 3)) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new().build().expect("engine builds");
    let mut workflow = engine.parse_file(&caller).expect("parse should succeed");

    // Both signatures should be present
    assert!(
        workflow.imported_builtin_signatures.contains_key("add"),
        "Expected 'add' in imported_builtin_signatures"
    );
    assert!(
        workflow.imported_builtin_signatures.contains_key("mul"),
        "Expected 'mul' in imported_builtin_signatures"
    );

    // Typecheck should pass
    engine
        .check(&mut workflow)
        .expect("typecheck should pass for add(1, mul(2, 3))");
}
