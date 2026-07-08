//! TASK-1572: Property tests for list algebraic laws.
//!
//! These tests verify that the pure Ash list operations in std/src/list.ash
//! satisfy the algebraic laws for Functor, Semigroup, and Monoid.
//!
//! The tests use the Ash engine to parse, typecheck, and execute Ash code
//! that verifies these laws with concrete examples.

#![allow(clippy::needless_raw_string_hashes)]

/// Build an engine with the stdlib available.
fn build_engine() -> ash_engine::Engine {
    ash_engine::Engine::new()
        .build()
        .expect("engine should build")
}

// ────────────────────────────────────────────────────────────────────
// Functor Laws
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_functor_identity_law() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let source_path = tmp_dir.path().join("main.ash");
    std::fs::write(
        &source_path,
        r#"
use list::{map}

fn identity(x: Int) -> Int { x }

fn main() -> Bool {
    let list = [1, 2, 3, 4, 5]
    let mapped = map(list, identity)
    mapped == list
}
"#,
    )
    .expect("write main.ash");

    let engine = build_engine();
    let result = engine.run_file(&source_path).await;
    assert_eq!(
        result,
        Ok(ash_core::Value::Bool(true)),
        "Functor identity: map(id) == id"
    );
}

// Note: Functor composition test is deferred due to language limitations:
// - nested function calls across module-level functions are not yet supported
// (e.g., add_one(mul_two(x)) inside compose doesn't resolve add_one)
// This will be re-enabled when the language supports these patterns.

// ────────────────────────────────────────────────────────────────────
// Semigroup Laws (via concat)
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_semigroup_associativity_law() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let source_path = tmp_dir.path().join("main.ash");
    std::fs::write(
        &source_path,
        r#"
use list::{concat}

fn main() -> Bool {
    let a = [1, 2]
    let b = [3, 4]
    let c = [5, 6]
    let lhs = concat(concat(a, b), c)
    let rhs = concat(a, concat(b, c))
    lhs == rhs
}
"#,
    )
    .expect("write main.ash");

    let engine = build_engine();
    let result = engine.run_file(&source_path).await;
    assert_eq!(
        result,
        Ok(ash_core::Value::Bool(true)),
        "Semigroup associativity: concat(concat(a,b),c) == concat(a,concat(b,c))"
    );
}

// ────────────────────────────────────────────────────────────────────
// Monoid Laws (via concat and empty list)
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_monoid_left_identity_law() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let source_path = tmp_dir.path().join("main.ash");
    std::fs::write(
        &source_path,
        r#"
use list::{concat}

fn main() -> Bool {
    let list = [1, 2, 3, 4, 5]
    let lhs = concat([], list)
    lhs == list
}
"#,
    )
    .expect("write main.ash");

    let engine = build_engine();
    let result = engine.run_file(&source_path).await;
    assert_eq!(
        result,
        Ok(ash_core::Value::Bool(true)),
        "Monoid left identity: concat([], list) == list"
    );
}

#[tokio::test]
async fn list_monoid_right_identity_law() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let source_path = tmp_dir.path().join("main.ash");
    std::fs::write(
        &source_path,
        r#"
use list::{concat}

fn main() -> Bool {
    let list = [1, 2, 3, 4, 5]
    let lhs = concat(list, [])
    lhs == list
}
"#,
    )
    .expect("write main.ash");

    let engine = build_engine();
    let result = engine.run_file(&source_path).await;
    assert_eq!(
        result,
        Ok(ash_core::Value::Bool(true)),
        "Monoid right identity: concat(list, []) == list"
    );
}

// ────────────────────────────────────────────────────────────────────
// Additional List Operation Tests
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_len_empty_is_zero() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let source_path = tmp_dir.path().join("main.ash");
    std::fs::write(
        &source_path,
        r#"
use list::{len}

fn main() -> Bool {
    len([]) == 0
}
"#,
    )
    .expect("write main.ash");

    let engine = build_engine();
    let result = engine.run_file(&source_path).await;
    assert_eq!(result, Ok(ash_core::Value::Bool(true)), "len([]) == 0");
}

#[tokio::test]
async fn list_len_non_empty() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let source_path = tmp_dir.path().join("main.ash");
    std::fs::write(
        &source_path,
        r#"
use list::{len}

fn main() -> Bool {
    len([1, 2, 3, 4, 5]) == 5
}
"#,
    )
    .expect("write main.ash");

    let engine = build_engine();
    let result = engine.run_file(&source_path).await;
    assert_eq!(
        result,
        Ok(ash_core::Value::Bool(true)),
        "len([1,2,3,4,5]) == 5"
    );
}

#[tokio::test]
async fn list_append_increases_length() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let source_path = tmp_dir.path().join("main.ash");
    std::fs::write(
        &source_path,
        r#"
use list::{len, append}

fn main() -> Bool {
    let list = [1, 2, 3]
    let new_list = append(list, 4)
    len(new_list) == 4
}
"#,
    )
    .expect("write main.ash");

    let engine = build_engine();
    let result = engine.run_file(&source_path).await;
    assert_eq!(
        result,
        Ok(ash_core::Value::Bool(true)),
        "len(append([1,2,3], 4)) == 4"
    );
}

#[tokio::test]
async fn list_take_drop_identity() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let source_path = tmp_dir.path().join("main.ash");
    std::fs::write(
        &source_path,
        r#"
use list::{concat, take, drop}

fn main() -> Bool {
    let list = [1, 2, 3, 4, 5]
    let n = 2
    let lhs = concat(take(n, list), drop(n, list))
    lhs == list
}
"#,
    )
    .expect("write main.ash");

    let engine = build_engine();
    let result = engine.run_file(&source_path).await;
    assert_eq!(
        result,
        Ok(ash_core::Value::Bool(true)),
        "concat(take(n, list), drop(n, list)) == list"
    );
}

// Note: reverse and map composition tests are deferred due to language limitations:
// - reverse is treated as a symbolic capability by the engine
// - nested function calls across module-level functions are not yet supported
// These will be re-enabled when the language supports these patterns.
