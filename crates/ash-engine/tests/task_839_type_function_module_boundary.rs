//! TASK-839: engine/module boundary and semantic-summary non-interference for type functions.

use ash_engine::Engine;
use ash_engine::module_loader::{check_importable_module_file, load_ordinary_file};

#[test]
fn check_module_file_preserves_same_module_type_functions_for_public_alias_validation() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("local.ash");
    std::fs::write(
        &module,
        r"pub sealed type domain TypeList { Nil; Cons<head: Type, tail: TypeList>; }

type fn Head(xs: TypeList) -> Type {
    case Head<Cons<h, t>> = h;
}

type LocalHead = Head<Cons<Int, Nil>>;
",
    )
    .expect("write module");

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .check_module_file(&module)
        .expect("module with local type function parses and checks");

    assert!(
        result.errors.is_empty(),
        "private same-module aliases may use local type functions without public-boundary leakage: {:?}",
        result.errors
    );
}

#[test]
fn check_module_file_rejects_public_alias_leaking_local_type_function_head() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("leaky.ash");
    std::fs::write(
        &module,
        r"pub sealed type domain TypeList { Nil; Cons<head: Type, tail: TypeList>; }

type fn Head(xs: TypeList) -> Type {
    case Head<Cons<h, t>> = h;
}

pub type Leaky = Head<Cons<Int, Nil>>;
",
    )
    .expect("write module");

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .check_module_file(&module)
        .expect("module parses but reports public type-function leak");

    assert!(
        result.errors.iter().any(|error| error.contains("Leaky")
            && error.contains("Head")
            && error.contains("type function")),
        "public alias should reject local type-function computation head leakage: {:?}",
        result.errors
    );

    let err = check_importable_module_file(&module)
        .expect_err("importable module check should reject public type-function leak");
    let msg = err.to_string();
    assert!(
        msg.contains("Leaky") && msg.contains("Head") && msg.contains("type function"),
        "importable module diagnostic should mention Leaky and Head: {msg}"
    );
}

#[test]
fn public_callable_signatures_reject_local_type_function_heads() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("leaky_sig.ash");
    std::fs::write(
        &module,
        r"pub sealed type domain TypeList { Nil; Cons<head: Type, tail: TypeList>; }

type fn Head(xs: TypeList) -> Type {
    case Head<Cons<h, t>> = h;
}

pub fn leak(x: Head<Cons<Int, Nil>>) -> Int { 0 }
pub fn leak_flow(x: Head<Cons<Int, Nil>>) -> Int { 0 }
",
    )
    .expect("write module");

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .check_module_file(&module)
        .expect("module parses but reports public signature type-function leak");

    assert!(
        result.errors.iter().any(|error| error.contains("leak")
            && error.contains("Head")
            && error.contains("type function")),
        "pub fn signature should reject local type-function head leakage: {:?}",
        result.errors
    );
    assert!(
        result.errors.iter().any(|error| error.contains("leak_flow")
            && error.contains("Head")
            && error.contains("type function")),
        "pub fn signature should reject local type-function head leakage: {:?}",
        result.errors
    );
}

#[test]
fn imported_semantic_summaries_do_not_serialize_type_function_equations() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    std::fs::write(
        &module,
        r"pub sealed type domain TypeList { Nil; Cons<head: Type, tail: TypeList>; }

type fn Head(xs: TypeList) -> Type {
    case Head<Cons<h, t>> = h;
}

pub type Boxed = Boxed { value: Int };
",
    )
    .expect("write provider");
    std::fs::write(
        &caller,
        r"use provider::{Boxed}
fn main() { 0 }
",
    )
    .expect("write caller");

    let loaded = load_ordinary_file(&caller).expect("caller imports ordinary public type");
    assert!(
        loaded
            .imported_type_defs
            .iter()
            .any(|ty| ty.name == "Boxed"),
        "ordinary type summary transport should remain intact"
    );
    for summary in &loaded.imported_semantic_summaries {
        assert!(
            summary
                .exported_types
                .iter()
                .all(|ty| ty.exported_name.as_str() != "Head"),
            "type-function heads must not appear as exported ordinary types before SPEC-F: {summary:?}"
        );
        assert!(
            summary
                .exported_constructors
                .iter()
                .all(|constructor| constructor.exported_name.as_str() != "Head"),
            "type-function heads must not appear as exported constructors before SPEC-F: {summary:?}"
        );
        assert!(
            summary
                .exported_sealed_domains
                .iter()
                .flat_map(|domain| domain.constructors.iter())
                .all(|constructor| constructor.exported_name.as_str() != "Head"),
            "type-function heads must not appear as sealed-domain marker constructors before SPEC-F: {summary:?}"
        );
    }
}
