//! TASK-634: Verify the `signature` field on `InlineCallable`.
//!
//! Tests that:
//! 1. Builtin fn callables preserve the full `BuiltinFnDef` signature
//!    (type params, param types, return type).
//! 2. User-defined callables (`pub fn` with body) have `signature: None`.

use ash_engine::module_loader::CallableSignature;
use ash_parser::surface::Type;
use std::io::Write;

// ---------------------------------------------------------------------------
// Test 1: Builtin fn with generic type params preserves signature
// ---------------------------------------------------------------------------

/// Verify that parsing `pub builtin fn len<a>(list: List<a>) -> Int;` produces
/// an `InlineCallable` whose `signature` is `Some(CallableSignature::Builtin(_))`
/// with the correct `type_params`, parameter types, and return type.
#[test]
fn builtin_fn_signature_preserves_type_params_and_return_type() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    // Module declaring a generic builtin fn
    let module = dir.join("collections.ash");
    writeln!(
        std::fs::File::create(&module).expect("create"),
        "pub builtin fn len<a>(list: List<a>) -> Int;",
    )
    .expect("write collections.ash");

    // Caller imports the builtin fn
    let caller = dir.join("caller.ash");
    writeln!(
        std::fs::File::create(&caller).expect("create"),
        "use collections::{{len}}\nworkflow main {{ ret 0 }}",
    )
    .expect("write caller.ash");

    let loaded = ash_engine::module_loader::load_ordinary_file(&caller)
        .expect("load_ordinary_file should succeed");

    let callable = loaded
        .imported_callables
        .get("len")
        .expect("'len' should be in imported_callables");

    // Verify it's a Builtin kind
    assert!(
        matches!(
            callable.kind,
            ash_engine::module_loader::CallableKind::Builtin { .. }
        ),
        "expected Builtin kind, got: {:?}",
        callable.kind,
    );

    // Verify param names
    assert_eq!(callable.params, vec!["list"]);

    // Verify signature is present
    let sig = match callable
        .signature
        .as_ref()
        .expect("builtin fn callable should have a signature")
    {
        CallableSignature::Builtin(sig) => sig,
        other @ CallableSignature::Function(_) => {
            panic!("expected builtin callable signature, got: {other:?}")
        }
    };

    // Verify type params
    assert_eq!(
        sig.type_params.len(),
        1,
        "expected 1 type param, got: {:?}",
        sig.type_params,
    );
    assert_eq!(
        sig.type_params[0].as_ref(),
        "a",
        "expected type param 'a', got: {:?}",
        sig.type_params[0],
    );

    // Verify parameter types: list param should have type List<a>
    // which is Type::Constructor { name: "List", args: [Type::Name("a")] }
    assert_eq!(
        sig.params.len(),
        1,
        "expected 1 param in signature, got: {:?}",
        sig.params,
    );
    assert_eq!(sig.params[0].name.as_ref(), "list");

    match &sig.params[0].ty {
        Type::Constructor { name, args } => {
            assert_eq!(name.as_ref(), "List", "param type should be List<a>");
            assert_eq!(args.len(), 1, "List should have 1 type arg");
            assert!(
                matches!(&args[0], Type::Name(n) if n.as_ref() == "a"),
                "List arg should be 'a', got: {:?}",
                args[0],
            );
        }
        other => {
            panic!("expected param type to be Constructor(List, [Name(a)]), got: {other:?}");
        }
    }

    // Verify return type is Int
    assert!(
        matches!(&sig.return_type, Type::Name(n) if n.as_ref() == "Int"),
        "expected return type Int, got: {:?}",
        sig.return_type,
    );
}

// ---------------------------------------------------------------------------
// Test 2: User-defined callable (pub fn with body) has signature: None
// ---------------------------------------------------------------------------

/// Verify that `pub fn` callables (with a body) preserve a `CallableSignature::Function`.
#[test]
fn user_defined_callable_has_function_signature() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    let module = dir.join("utils.ash");
    std::fs::write(&module, "pub fn double(x: Int) -> Int { x + x }\n").expect("write utils.ash");

    let caller = dir.join("caller.ash");
    std::fs::write(
        &caller,
        "use utils::{double}\nworkflow main { ret double(3) }\n",
    )
    .expect("write caller.ash");

    let loaded = ash_engine::module_loader::load_ordinary_file(&caller)
        .expect("load_ordinary_file should succeed");

    let callable = loaded
        .imported_callables
        .get("double")
        .expect("'double' should be in imported_callables");

    // Verify it's a User kind
    assert!(
        matches!(
            callable.kind,
            ash_engine::module_loader::CallableKind::User { .. }
        ),
        "expected User kind, got: {:?}",
        callable.kind,
    );

    // Verify signature is preserved for ordinary pub fn callables
    match callable.signature.as_ref() {
        Some(CallableSignature::Function(sig)) => {
            assert_eq!(sig.name.as_ref(), "double");
            assert_eq!(sig.params.len(), 1);
        }
        other => {
            panic!("user-defined callable should have CallableSignature::Function, got: {other:?}")
        }
    }
}

// ---------------------------------------------------------------------------
// Test 3: Builtin fn without type params still has a signature
// ---------------------------------------------------------------------------

/// Verify that a non-generic builtin fn like `pub builtin fn add(x: Int, y: Int) -> Int;`
/// still gets a signature with empty `type_params`.
#[test]
fn builtin_fn_without_type_params_has_signature() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    let module = dir.join("math.ash");
    writeln!(
        std::fs::File::create(&module).expect("create"),
        "pub builtin fn add(x: Int, y: Int) -> Int;",
    )
    .expect("write math.ash");

    let caller = dir.join("caller.ash");
    writeln!(
        std::fs::File::create(&caller).expect("create"),
        "use math::{{add}}\nworkflow main {{ ret 0 }}",
    )
    .expect("write caller.ash");

    let loaded = ash_engine::module_loader::load_ordinary_file(&caller)
        .expect("load_ordinary_file should succeed");

    let callable = loaded
        .imported_callables
        .get("add")
        .expect("'add' should be in imported_callables");

    let sig = match callable
        .signature
        .as_ref()
        .expect("builtin fn should have a signature")
    {
        CallableSignature::Builtin(sig) => sig,
        other @ CallableSignature::Function(_) => {
            panic!("expected builtin callable signature, got: {other:?}")
        }
    };

    // No type params
    assert!(
        sig.type_params.is_empty(),
        "expected no type params, got: {:?}",
        sig.type_params,
    );

    // Two params with Int types
    assert_eq!(sig.params.len(), 2);
    assert_eq!(sig.params[0].name.as_ref(), "x");
    assert_eq!(sig.params[1].name.as_ref(), "y");

    assert!(matches!(&sig.params[0].ty, Type::Name(n) if n.as_ref() == "Int"));
    assert!(matches!(&sig.params[1].ty, Type::Name(n) if n.as_ref() == "Int"));

    // Return type Int
    assert!(
        matches!(&sig.return_type, Type::Name(n) if n.as_ref() == "Int"),
        "expected return type Int, got: {:?}",
        sig.return_type,
    );
}

// ---------------------------------------------------------------------------
// Test 4: Glob import still preserves signature
// ---------------------------------------------------------------------------

/// Verify that glob-imported builtin fns also preserve their signatures.
#[test]
fn glob_imported_builtin_fn_preserves_signature() {
    let tmp_dir = tempfile::tempdir().expect("temp dir created");
    let dir = tmp_dir.path();

    let module = dir.join("collections.ash");
    writeln!(
        std::fs::File::create(&module).expect("create"),
        "pub builtin fn map<a, b>(f: Fn(a) -> b, xs: List<a>) -> List<b>;\n",
    )
    .expect("write collections.ash");

    let caller = dir.join("caller.ash");
    writeln!(
        std::fs::File::create(&caller).expect("create"),
        "use collections::*\nworkflow main {{ ret 0 }}",
    )
    .expect("write caller.ash");

    let loaded = ash_engine::module_loader::load_ordinary_file(&caller)
        .expect("load_ordinary_file should succeed");

    let callable = loaded
        .imported_callables
        .get("map")
        .expect("'map' should be in imported_callables");

    let sig = match callable
        .signature
        .as_ref()
        .expect("glob-imported builtin fn should have a signature")
    {
        CallableSignature::Builtin(sig) => sig,
        other @ CallableSignature::Function(_) => {
            panic!("expected builtin callable signature, got: {other:?}")
        }
    };

    // Two type params
    assert_eq!(sig.type_params.len(), 2);
    assert_eq!(sig.type_params[0].as_ref(), "a");
    assert_eq!(sig.type_params[1].as_ref(), "b");

    // Two params
    assert_eq!(sig.params.len(), 2);
    assert_eq!(sig.params[0].name.as_ref(), "f");
    assert_eq!(sig.params[1].name.as_ref(), "xs");
}
