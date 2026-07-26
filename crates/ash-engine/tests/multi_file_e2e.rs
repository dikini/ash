//! Multi-file end-to-end integration tests.
//!
//! Tests that exercise import resolution, parsing, and typechecking across multiple Ash source
//! files. TASK-2014 Path B permits execution only after validated typed lowering, so unsupported
//! callable forms also assert their exact closed-admission outcome.

use ash_engine::Engine;
use tempfile::TempDir;

fn write(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dirs");
    }
    std::fs::write(path, contents).expect("write file");
}

fn build_engine() -> Engine {
    Engine::new().build().expect("engine builds")
}

const CLOSED_ADMISSION_ENTRY_RESULT_ERROR: &str = "checked Core/CPS admission rejected: type error: checked Core-to-CPS bridge currently accepts atomic, atom-only binary primitives, atomic-not, variable-let, and boolean-if entry results";

async fn assert_parse_check_then_closed_admission(engine: &Engine, entry: &std::path::Path) {
    let mut application = engine
        .parse_file(entry)
        .expect("cross-file source should parse");
    engine
        .check(&mut application)
        .expect("cross-file source should typecheck");

    let error = engine
        .run_file(entry)
        .await
        .expect_err("callable source without validated typed lowering must reject at admission");
    assert!(
        matches!(
            error,
            ash_interp::ExecError::ExecutionFailed(ref message)
                if message == CLOSED_ADMISSION_ENTRY_RESULT_ERROR
        ),
        "cross-file callable source must expose the exact checked Core/CPS closed-admission error"
    );
}

// ── 1. Cross-file pub fn call ────────────────────────────────────────────

/// `math.ash` defines `pub fn double(x)`, `main.ash` imports and calls it.
#[tokio::test]
async fn cross_file_pub_fn_call() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("math.ash"),
        "\
pub fn double(x: Int) -> Int { x * 2 }
",
    );
    write(
        &dir.join("main.ash"),
        "\
use math::{double}

fn main() -> Int { double(21) }
",
    );

    let engine = build_engine();
    assert_parse_check_then_closed_admission(&engine, &dir.join("main.ash")).await;
}

// ── 2. Cross-file type import and construction ───────────────────────────

/// `types.ash` defines a record type, `main.ash` imports and constructs it.
#[tokio::test]
async fn cross_file_type_import_and_construct() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("types.ash"),
        "\
pub type Point = Point { x: Int, y: Int };
",
    );
    write(
        &dir.join("main.ash"),
        "\
use types::{Point}

fn main() -> Point { Point { x: 10, y: 20 } }
",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "cross-file type import: expected success, got: {:?}",
        result.err()
    );
    let value = result.unwrap();
    assert!(matches!(
        &value,
        ash_core::Value::Variant { name, fields }
            if name == "Point"
            && fields.len() == 2
            && fields[0].0 == "x"
            && fields[1].0 == "y"
    ));
}

// ── 3. Nested directory module with pub fn ───────────────────────────────

/// `lib/mod.ash` declares `pub mod math;`
/// `lib/math.ash` defines `pub fn add()`
/// `main.ash` imports `lib::math::{add}`
#[tokio::test]
async fn nested_directory_module_pub_fn() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("lib").join("math.ash"),
        "\
pub fn add(a: Int, b: Int) -> Int { a + b }
",
    );
    write(&dir.join("lib").join("mod.ash"), "pub mod math;");
    write(
        &dir.join("main.ash"),
        "\
use lib::math::{add}

fn main() -> Int { add(10, 20) }
",
    );

    let engine = build_engine();
    assert_parse_check_then_closed_admission(&engine, &dir.join("main.ash")).await;
}

// ── 4. Local module shadows stdlib name ──────────────────────────────────

/// A local `option.ash` should be preferred over the stdlib `option` module.
#[tokio::test]
async fn local_module_shadows_stdlib() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("option.ash"),
        "\
pub type MyOption = MySome { v: Int } | MyNone;
",
    );
    write(
        &dir.join("main.ash"),
        "\
use option::{MyOption, MySome}

fn main() -> MyOption { MySome { v: 99 } }
",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "local shadow stdlib: expected success, got: {:?}",
        result.err()
    );
    assert!(matches!(
        result.unwrap(),
        ash_core::Value::Variant { name, fields } if name == "MySome" && fields.len() == 1
    ));
}

// ── 5. Two files sharing a type definition ───────────────────────────────

/// `shared.ash` defines a type, `main.ash` imports and uses it in a application.
/// Note: `FieldAccess` on imported types not yet supported in typeck.
/// This test verifies type construction only.
#[tokio::test]
async fn shared_type_import_and_construct() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("shared.ash"),
        "\
pub type Config = Config { name: String, value: Int };
",
    );
    write(
        &dir.join("main.ash"),
        "\
use shared::{Config}

fn main() -> Config { Config { name: \"test\", value: 42 } }
",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "shared type import: expected success, got: {:?}",
        result.err()
    );
    assert!(matches!(
        result.unwrap(),
        ash_core::Value::Variant { name, fields }
            if name == "Config" && fields.len() == 2
    ));
}

// ── 6. Multi-file type-only imports with construction ────────────────────

/// `shapes.ash` defines multiple types, `main.ash` imports subset and uses.
#[tokio::test]
async fn multi_type_import_from_single_file() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("shapes.ash"),
        "\
pub type Circle = Circle { radius: Int };
pub type Square = Square { side: Int };
",
    );
    write(
        &dir.join("main.ash"),
        "\
use shapes::{Circle, Square}

fn main() -> Circle { Circle { radius: 5 } }
",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    assert!(
        result.is_ok(),
        "multi-type import: expected success, got: {:?}",
        result.err()
    );
    assert!(matches!(
        result.unwrap(),
        ash_core::Value::Variant { name, fields }
            if name == "Circle" && fields.len() == 1
    ));
}

// ── 7. Stdlib type import with cross-file pub fn ─────────────────────────

/// Combines stdlib import with local module pub fn.
#[tokio::test]
async fn stdlib_type_with_local_pub_fn() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("helpers.ash"),
        "\
pub fn make_label(x: Int) -> String { \"value\" }
",
    );
    write(
        &dir.join("main.ash"),
        "\
use helpers::{make_label}
use option::{Option, Some, None}

fn main() -> String { make_label(42) }
",
    );

    let engine = build_engine();
    assert_parse_check_then_closed_admission(&engine, &dir.join("main.ash")).await;
}

// ── 8. Gap documentation: cross-file fn calling cross-file fn ─────────────

/// NOTE: This test documents a known limitation.
/// `b.ash` defines `pub fn double_then_inc` that calls `inc` from `a.ash`.
/// Currently the evaluator does not resolve cross-file fn calls inside
/// imported pub fn bodies. This is tracked as a future improvement.
/// When fixed, this test should pass.
#[tokio::test]
async fn cross_file_fn_calling_cross_file_fn_currently_fails() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("a.ash"),
        "\
pub fn inc(x: Int) -> Int { x + 1 }
",
    );
    write(
        &dir.join("b.ash"),
        "\
use a::{inc}

pub fn double_then_inc(x: Int) -> Int { inc(x * 2) }
",
    );
    write(
        &dir.join("main.ash"),
        "\
use b::{double_then_inc}

fn main() -> Int { double_then_inc(5) }
",
    );

    let engine = build_engine();
    let result = engine.run_file(dir.join("main.ash")).await;

    // Document current limitation: this fails with UndefinedVariable("inc")
    // because the evaluator doesn't resolve cross-file fn refs inside imported fn bodies
    assert!(
        result.is_err(),
        "cross-file fn calling cross-file fn: currently expected to fail, got: {:?}",
        result.ok()
    );
}
