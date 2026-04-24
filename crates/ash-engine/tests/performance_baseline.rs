//! Performance baseline tests (TASK-671).
//!
//! Establishes timing baselines for key engine operations:
//! - Engine build time
//! - Single workflow parse + execute
//! - Stdlib import resolution
//! - Multi-file import chain
//! - Provider creation overhead
//!
//! These are NOT rigorous benchmarks (use criterion for that).
//! They establish that key operations complete within reasonable
//! time bounds, catching gross regressions.

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

// ── 2. Simple workflow execution ─────────────────────────────────────────

#[tokio::test]
async fn baseline_simple_workflow() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(&dir.join("main.ash"), "workflow main() -> Int { ret 42; }");

    let engine = Engine::new().build().expect("engine builds");

    let start = Instant::now();
    let result = engine.run_file(dir.join("main.ash")).await;
    let elapsed = start.elapsed().as_millis();

    assert!(result.is_ok(), "expected success, got: {:?}", result.err());
    assert!(
        elapsed < TIMEOUT_MS,
        "Simple workflow took {elapsed}ms (limit {TIMEOUT_MS}ms)"
    );
    eprintln!("[baseline] simple workflow: {elapsed}ms");
}

// ── 3. Workflow with computation ─────────────────────────────────────────

#[tokio::test]
async fn baseline_computation_workflow() {
    let temp = TempDir::new().expect("tempdir");
    let dir = temp.path();

    write(
        &dir.join("main.ash"),
        "\
workflow main() -> Int {
    let a = 10;
    let b = 20;
    let c = a + b;
    let d = c * 3;
    ret d;
}
",
    );

    let engine = Engine::new().build().expect("engine builds");

    let start = Instant::now();
    let result = engine.run_file(dir.join("main.ash")).await;
    let elapsed = start.elapsed().as_millis();

    assert!(result.is_ok(), "expected success, got: {:?}", result.err());
    assert!(
        elapsed < TIMEOUT_MS,
        "Computation workflow took {elapsed}ms (limit {TIMEOUT_MS}ms)"
    );
    eprintln!("[baseline] computation workflow: {elapsed}ms");
}

// ── 4. Stdlib import resolution ──────────────────────────────────────────

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

workflow main() -> String { ret concat(\"hello\", \" world\"); }
",
    );

    let engine = Engine::new().build().expect("engine builds");

    let start = Instant::now();
    let result = engine.run_file(dir.join("main.ash")).await;
    let elapsed = start.elapsed().as_millis();

    assert!(result.is_ok(), "expected success, got: {:?}", result.err());
    assert!(
        elapsed < TIMEOUT_MS,
        "Stdlib import took {elapsed}ms (limit {TIMEOUT_MS}ms)"
    );
    eprintln!("[baseline] stdlib import: {elapsed}ms");
}

// ── 5. Cross-file import chain ───────────────────────────────────────────

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

workflow main() -> Int { ret double(21); }
",
    );

    let engine = Engine::new().build().expect("engine builds");

    let start = Instant::now();
    let result = engine.run_file(dir.join("main.ash")).await;
    let elapsed = start.elapsed().as_millis();

    assert!(result.is_ok(), "expected success, got: {:?}", result.err());
    assert!(
        elapsed < TIMEOUT_MS,
        "Multi-file import took {elapsed}ms (limit {TIMEOUT_MS}ms)"
    );
    eprintln!("[baseline] multi-file import: {elapsed}ms");
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
