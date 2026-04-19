//! TASK-620: End-to-end import resolution for `builtin fn`.
//!
//! These tests verify the full pipeline:
//!   1. Module A declares `pub builtin fn add(a: Int, b: Int) -> Int;`
//!   2. Module B imports it via `use math_utils::{add};` and calls it
//!   3. Import resolves at module-load time (no error)
//!   4. Typecheck passes using the declared signature
//!   5. Runtime produces a clear error (no Rust panic) because no
//!      implementation exists yet — Track C will provide actual dispatch.

use std::io::Write;

// ---------------------------------------------------------------------------
// Test 1: Import resolution at module-load time
// ---------------------------------------------------------------------------

/// Verify that a `pub builtin fn` in one module is importable from another
/// via `use module::{name};` and that `load_ordinary_file` succeeds.
#[test]
fn builtin_fn_import_resolves_at_module_load() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let module_dir = tmp_dir.path();

    // Module A: declares a public builtin fn
    let math_utils = module_dir.join("math_utils.ash");
    let mut f = std::fs::File::create(&math_utils).expect("create math_utils.ash");
    writeln!(f, "pub builtin fn add(a: Int, b: Int) -> Int;").expect("write math_utils.ash");

    // Module B: imports and uses the builtin fn in a workflow
    let caller = module_dir.join("caller.ash");
    let mut f = std::fs::File::create(&caller).expect("create caller.ash");
    write!(
        f,
        "use math_utils::{{add}}\nworkflow main {{ ret add(1, 2) }}"
    )
    .expect("write caller.ash");

    let result = ash_engine::module_loader::load_ordinary_file(&caller);
    assert!(
        result.is_ok(),
        "Expected import of builtin fn to succeed, but got error: {:?}",
        result.err()
    );

    let loaded = result.unwrap();

    // Verify the builtin fn was collected as an imported callable
    assert!(
        loaded.imported_callables.contains_key("add"),
        "Expected 'add' in imported_callables, found: {:?}",
        loaded.imported_callables.keys().collect::<Vec<_>>()
    );

    // Verify the callable is marked as Builtin
    let add_callable = loaded.imported_callables.get("add").expect("add callable");
    assert!(
        matches!(
            add_callable.kind,
            ash_engine::module_loader::CallableKind::Builtin
        ),
        "Expected CallableKind::Builtin, got: {:?}",
        add_callable.kind
    );

    // Verify parameter names are preserved
    assert_eq!(
        add_callable.params,
        vec!["a", "b"],
        "Expected params [\"a\", \"b\"], got: {:?}",
        add_callable.params
    );
}

// ---------------------------------------------------------------------------
// Test 2: Full parse + typecheck pipeline
// ---------------------------------------------------------------------------

/// Verify that a workflow importing a `builtin fn` parses and typechecks
/// without errors. The typechecker uses the declared arity to produce a
/// fresh polymorphic function type.
#[test]
fn builtin_fn_import_typechecks_successfully() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let module_dir = tmp_dir.path();

    // Module A: declares a public builtin fn
    let math_utils = module_dir.join("math_utils.ash");
    std::fs::write(&math_utils, "pub builtin fn add(a: Int, b: Int) -> Int;\n")
        .expect("write math_utils.ash");

    // Module B: imports and calls the builtin fn
    let caller = module_dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use math_utils::{add}\nfn double(x: Int) -> Int { add(x, x) }\nworkflow main { ret double(3) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");

    // parse_file internally calls load_ordinary_file then parses the workflow
    let mut workflow = engine
        .parse_file(&caller)
        .expect("parse_file should succeed for builtin fn import");

    // The imported_param_counts should contain the builtin fn
    assert!(
        workflow.imported_param_counts.contains_key("add"),
        "Expected 'add' in imported_param_counts, found: {:?}",
        workflow.imported_param_counts
    );
    assert_eq!(
        workflow.imported_param_counts.get("add"),
        Some(&2),
        "Expected add to have 2 params"
    );

    // Typecheck should pass — the typechecker uses imported_param_counts
    // to create a fresh function type with the correct arity.
    engine
        .check(&mut workflow)
        .expect("typecheck should pass for builtin fn call");
}

// ---------------------------------------------------------------------------
// Test 3: Runtime failure is graceful (not a panic)
// ---------------------------------------------------------------------------

/// Verify that calling an imported `builtin fn` at runtime produces a clear
/// error rather than panicking. The error should indicate the callable is
/// not implemented or not found, since Track C handles actual dispatch.
#[tokio::test]
async fn builtin_fn_runtime_failure_is_graceful() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let module_dir = tmp_dir.path();

    // Module A: declares a public builtin fn
    let math_utils = module_dir.join("math_utils.ash");
    std::fs::write(&math_utils, "pub builtin fn add(a: Int, b: Int) -> Int;\n")
        .expect("write math_utils.ash");

    // Module B: imports and calls the builtin fn directly in the workflow
    let caller = module_dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use math_utils::{add}\nworkflow main { ret add(1, 2) }\n",
    )
    .expect("write caller.ash");

    let engine = ash_engine::Engine::new()
        .build()
        .expect("engine should build");

    let mut workflow = engine
        .parse_file(&caller)
        .expect("parse_file should succeed");

    engine.check(&mut workflow).expect("typecheck should pass");

    // Execute: should fail gracefully, NOT panic.
    // The builtin fn has no closure bound, so the interpreter will report
    // an undefined-variable or unknown-function error.
    let result = engine.execute(&workflow).await;

    assert!(
        result.is_err(),
        "Expected runtime error for unimplemented builtin fn, but execution succeeded: {:?}",
        result.ok()
    );

    let err = result.unwrap_err();
    let err_string = format!("{err}");

    // The error should be informative — either "undefined variable" or
    // "unknown function" referencing the builtin fn name.
    let is_graceful = err_string.contains("undefined")
        || err_string.contains("unknown function")
        || err_string.contains("not found")
        || err_string.contains("builtin")
        || err_string.contains("unimplemented");

    assert!(
        is_graceful,
        "Runtime error should be informative and mention the missing builtin, got: {err_string}"
    );
}

// ---------------------------------------------------------------------------
// Test 4: Glob import of builtin fn
// ---------------------------------------------------------------------------

/// Verify that glob import `use math_utils::*;` also picks up `pub builtin fn`.
#[test]
fn builtin_fn_glob_import_resolves() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let module_dir = tmp_dir.path();

    let math_utils = module_dir.join("math_utils.ash");
    std::fs::write(
        &math_utils,
        "pub builtin fn add(a: Int, b: Int) -> Int;\npub builtin fn sub(a: Int, b: Int) -> Int;\n",
    )
    .expect("write math_utils.ash");

    let caller = module_dir.join("caller.ash");
    std::fs::write(&caller, "use math_utils::*\nworkflow main { ret 42 }\n")
        .expect("write caller.ash");

    let result = ash_engine::module_loader::load_ordinary_file(&caller);
    assert!(
        result.is_ok(),
        "Expected glob import of builtin fns to succeed, but got error: {:?}",
        result.err()
    );

    let loaded = result.unwrap();
    assert!(
        loaded.imported_callables.contains_key("add"),
        "Expected 'add' in imported_callables from glob import"
    );
    assert!(
        loaded.imported_callables.contains_key("sub"),
        "Expected 'sub' in imported_callables from glob import"
    );
}

// ---------------------------------------------------------------------------
// Test 5: Builtin fn alongside regular fn in the same module
// ---------------------------------------------------------------------------

/// Verify that a module can export both `pub fn` and `pub builtin fn`,
/// and the importer correctly distinguishes them.
#[test]
fn builtin_fn_coexists_with_regular_fn() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let module_dir = tmp_dir.path();

    let utils = module_dir.join("utils.ash");
    std::fs::write(
        &utils,
        "pub fn double(x: Int) -> Int { x + x }\npub builtin fn triple(x: Int) -> Int;\n",
    )
    .expect("write utils.ash");

    let caller = module_dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use utils::{double, triple}\nworkflow main { ret double(3) }\n",
    )
    .expect("write caller.ash");

    let result = ash_engine::module_loader::load_ordinary_file(&caller);
    assert!(
        result.is_ok(),
        "Expected mixed import to succeed, but got error: {:?}",
        result.err()
    );

    let loaded = result.unwrap();

    // double should be a User callable (has body)
    let double = loaded
        .imported_callables
        .get("double")
        .expect("'double' should be imported");
    assert!(
        matches!(
            double.kind,
            ash_engine::module_loader::CallableKind::User { .. }
        ),
        "'double' should be User callable, got: {:?}",
        double.kind
    );

    // triple should be a Builtin callable (bodyless)
    let triple = loaded
        .imported_callables
        .get("triple")
        .expect("'triple' should be imported");
    assert!(
        matches!(
            triple.kind,
            ash_engine::module_loader::CallableKind::Builtin
        ),
        "'triple' should be Builtin callable, got: {:?}",
        triple.kind
    );
}
