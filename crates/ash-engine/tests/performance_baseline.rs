//! Performance baseline tests (TASK-671).
//!
//! Establishes timing baselines for key engine operations:
//! - Engine build time
//! - Single target-Ash entry parse + checked Core/CPS admission
//! - Stdlib import resolution
//! - Multi-file import chain
//! - Provider creation overhead
//!
//! These are NOT rigorous benchmarks (use criterion for that).
//! They establish that key operations complete within reasonable
//! time bounds, catching gross regressions. The selected supported pure
//! Core/CPS admission measures Engine terminalization; intentionally
//! unimplemented call routes retain their exact closed-admission controls and
//! never fall back to a direct evaluator.

use ash_core::Value;
use ash_engine::Engine;
use std::time::Instant;
use tempfile::TempDir;

fn write(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dirs");
    }
    std::fs::write(path, contents).expect("write file");
}

const TIMEOUT_MS: u128 = 5000; // 5s generous timeout
const CLOSED_ADMISSION_ENTRY_RESULT_ERROR: &str = "checked Core/CPS admission rejected: type error: checked Core-to-CPS bridge currently accepts pure typed atoms, approved integer binary primitives, recursive Boolean Not, variable-let, and boolean-if entry results";

/// Run one selected file through the Engine and retain its terminal value as
/// the timing baseline's semantic control.
async fn assert_engine_terminal(engine: &Engine, entry: &std::path::Path, expected: Value) {
    let terminal = engine
        .run_file(entry)
        .await
        .expect("selected baseline source must terminalize through Engine admission");
    assert_eq!(
        terminal, expected,
        "the timing baseline must retain its established Engine terminal value"
    );
}

/// Prove an intentionally unimplemented call route reaches parse/check, then
/// closes at checked Core/CPS admission rather than selecting another executor.
async fn assert_parse_check_then_closed_admission(engine: &Engine, entry: &std::path::Path) {
    let mut application = engine
        .parse_file(entry)
        .expect("baseline source should parse before admission");
    engine
        .check(&mut application)
        .expect("baseline source should typecheck before admission");

    let error = engine
        .run_file(entry)
        .await
        .expect_err("an unimplemented call route must close at checked Core/CPS admission");
    assert!(
        matches!(
            error,
            ash_interp::ExecError::ExecutionFailed(ref message)
                if message == CLOSED_ADMISSION_ENTRY_RESULT_ERROR
        ),
        "the call-route baseline must retain its exact checked Core/CPS closed-admission error"
    );
}

// ── 1. Engine build ──────────────────────────────────────────────────────

#[tokio::test]
async fn baseline_engine_build() {
    let start = Instant::now();
    let _engine = Engine::new().build().expect("engine builds");
    let elapsed = start.elapsed().as_millis();

    assert!(
        elapsed < TIMEOUT_MS,
        "Engine build took {elapsed}ms (limit {TIMEOUT_MS}ms)"
    );
    // Log for baseline tracking
    eprintln!("[baseline] engine build: {elapsed}ms");
}

// ── 2. Simple target entry execution (atomic positive control) ────────────

#[tokio::test]
async fn baseline_simple_application() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(&dir.join("main.ash"), "fn main() -> Int { 42 }");

    let engine = Engine::new().build().expect("engine builds");

    let start = Instant::now();
    let result = engine.run_file(dir.join("main.ash")).await;
    let elapsed = start.elapsed().as_millis();

    assert!(result.is_ok(), "expected success, got: {:?}", result.err());
    assert!(
        elapsed < TIMEOUT_MS,
        "Simple fn took() {elapsed}ms (limit {TIMEOUT_MS}ms)"
    );
    eprintln!("[baseline] simple target entry: {elapsed}ms");
}

// ── 3. Target entry computation ──────────────────────────────────────────

#[tokio::test]
async fn baseline_computation_application() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
fn main() -> Int {
    let a = 10;
    let b = 20;
    let c = a + b;
    let d = c * 3;
    d
}
",
    );

    let engine = Engine::new().build().expect("engine builds");

    let start = Instant::now();
    assert_engine_terminal(&engine, &dir.join("main.ash"), Value::Int(90)).await;
    let elapsed = start.elapsed().as_millis();

    assert!(
        elapsed < TIMEOUT_MS,
        "Computation fn took() {elapsed}ms (limit {TIMEOUT_MS}ms)"
    );
    eprintln!("[baseline] computation target entry: {elapsed}ms");
}

// ── 4. Stdlib import resolution + closed admission ───────────────────────

#[tokio::test]
async fn baseline_stdlib_import() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
use option::{Option, Some, None}
use result::{Result, Ok, Err}
use string::{concat}

fn main() -> String { concat(\"hello\", \" world\") }
",
    );

    let engine = Engine::new().build().expect("engine builds");

    let start = Instant::now();
    assert_parse_check_then_closed_admission(&engine, &dir.join("main.ash")).await;
    let elapsed = start.elapsed().as_millis();

    assert!(
        elapsed < TIMEOUT_MS,
        "Stdlib import took {elapsed}ms (limit {TIMEOUT_MS}ms)"
    );
    eprintln!("[baseline] stdlib import parse/check + closed admission: {elapsed}ms");
}

// ── 5. Cross-file import chain + closed admission ─────────────────────────

#[tokio::test]
async fn baseline_multi_file_import() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("math.ash"),
        "pub fn double(x: Int) -> Int { x * 2 }",
    );
    write(
        &dir.join("main.ash"),
        "\
use math::{double}

fn main() -> Int { double(21) }
",
    );

    let engine = Engine::new().build().expect("engine builds");

    let start = Instant::now();
    assert_parse_check_then_closed_admission(&engine, &dir.join("main.ash")).await;
    let elapsed = start.elapsed().as_millis();

    assert!(
        elapsed < TIMEOUT_MS,
        "Multi-file import took {elapsed}ms (limit {TIMEOUT_MS}ms)"
    );
    eprintln!("[baseline] multi-file import parse/check + closed admission: {elapsed}ms");
}

// ── 6. Provider creation overhead ────────────────────────────────────────

#[test]
fn baseline_provider_creation() {
    use ash_engine::providers::{
        FsProvider, HttpProvider, ProcessProvider, StdioProvider, TimeProvider,
    };

    let start = Instant::now();
    for _ in 0..100 {
        let _ = StdioProvider::new();
        let _ = FsProvider::new();
        let _ = HttpProvider::new();
        let _ = TimeProvider::new();
        let _ = ProcessProvider::new();
    }
    let elapsed = start.elapsed().as_micros();

    // 500 provider creations: reqwest client init is expensive (~30ms each)
    // so we set a generous threshold
    assert!(
        elapsed < 5_000_000,
        "500 provider creations took {elapsed}us (limit 5s)"
    );
    eprintln!("[baseline] 500 provider creations: {}ms", elapsed / 1000);
}
