//! Phase 154 regressions for imported type visibility in type annotations.

use ash_engine::Engine;

fn write(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash source");
}

#[test]
fn imported_type_is_available_in_local_type_definition_and_pub_fn_return() {
    let dir = tempfile::tempdir().expect("tempdir");
    let module_a = dir.path().join("module_a.ash");
    let module_b = dir.path().join("module_b.ash");

    write(&module_a, "pub type Point = Point { x: Int, y: Int };\n");
    write(
        &module_b,
        r"
use module_a::{Point}
pub type Line = Line { start: Point, end: Point };
pub fn origin() -> Point { Point { x: 0, y: 0 } }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .check_module_file(&module_b)
        .expect("module with imported annotation type checks");
    assert!(
        result.errors.is_empty(),
        "unexpected errors: {:?}",
        result.errors
    );
}

#[test]
fn imported_type_is_available_in_strategy_like_record_field() {
    let dir = tempfile::tempdir().expect("tempdir");
    let context = dir.path().join("context.ash");
    let strategy = dir.path().join("strategy.ash");

    write(
        &context,
        "pub type GenContext = GenContext { seed: Int, size: Int };\n",
    );
    write(
        &strategy,
        r"
use context::{GenContext}
pub type Strategy<T> = Strategy { gen: (GenContext) -> T, shrink: (T) -> List<T> };
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .check_module_file(&strategy)
        .expect("strategy-like module checks");
    assert!(
        result.errors.is_empty(),
        "unexpected errors: {:?}",
        result.errors
    );
}

#[test]
fn public_callable_makes_private_type_nameable_but_not_constructible_downstream() {
    let dir = tempfile::tempdir().expect("tempdir");
    let internal = dir.path().join("internal.ash");
    let public_good = dir.path().join("public_good.ash");
    let public_bad = dir.path().join("public_bad.ash");

    write(
        &internal,
        r"
type Secret = Secret { value: Int };
pub fn make_secret(v: Int) -> Secret { Secret { value: v } }
pub fn get_value(s: Secret) -> Int { s.value }
",
    );
    write(
        &public_good,
        r"
use internal::{make_secret, get_value}
pub fn double(v: Int) -> Secret { make_secret(get_value(make_secret(v)) * 2) }
",
    );
    write(
        &public_bad,
        r"
use internal::{make_secret, get_value}
pub fn bad() -> Secret { Secret { value: 42 } }
",
    );

    let public_glob = dir.path().join("public_glob.ash");
    write(
        &public_glob,
        r"
use internal::*
pub fn double(v: Int) -> Secret { make_secret(get_value(make_secret(v)) * 2) }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .check_module_file(&public_good)
        .expect("smart-constructor consumer checks");
    assert!(
        result.errors.is_empty(),
        "unexpected errors: {:?}",
        result.errors
    );

    let glob = engine
        .check_module_file(&public_glob)
        .expect("glob smart-constructor consumer checks");
    assert!(
        glob.errors.is_empty(),
        "glob import should keep smart-constructor private types opaque/nameable, got {:?}",
        glob.errors
    );

    let bad = engine
        .check_module_file(&public_bad)
        .expect("constructor misuse parses but reports errors");
    assert!(
        bad.errors
            .iter()
            .any(|error| error.contains("Secret") && error.contains("constructor")),
        "expected opaque constructor diagnostic, got {:?}",
        bad.errors
    );
}

#[test]
fn unimported_inferred_type_in_public_signature_reports_import_hint() {
    let dir = tempfile::tempdir().expect("tempdir");
    let internal = dir.path().join("internal.ash");
    let public_bad = dir.path().join("public_bad.ash");

    write(&internal, "pub type Secret = Secret { value: Int };\n");
    write(
        &public_bad,
        r"
pub fn bad(s: Int) -> Secret { Secret { value: s } }
",
    );

    let engine = Engine::new().build().expect("engine builds");
    let result = engine
        .check_module_file(&public_bad)
        .expect("module parses but reports signature leakage");
    let message = result.errors.join("; ");
    assert!(
        message.contains("Secret") && message.contains("use internal::{Secret}"),
        "expected missing import hint for Secret, got {message}"
    );
}
