//! TASK-1572: List algebraic-law source fixtures under strict closed admission.
//!
//! The fixtures preserve concrete Functor, Semigroup, Monoid, and list-operation source examples
//! so imports, parsing, and typechecking remain covered. TASK-2014 Path B intentionally rejects
//! each at the checked Core/CPS admission boundary until the required typed lowering exists.
//! They are not evidence that the algebraic laws execute in production.

#![allow(clippy::needless_raw_string_hashes)]

/// Build an engine with the stdlib available.
fn build_engine() -> ash_engine::Engine {
    ash_engine::Engine::new()
        .build()
        .expect("engine should build")
}

const CLOSED_ADMISSION_PREFIX: &str =
    "application execution failed: checked Core/CPS admission rejected: type error: ";
const ATOMIC_LET_LOWERING_ERROR: &str = "checked Core-to-CPS bridge accepts only atomic let values";
const ENTRY_RESULT_LOWERING_ERROR: &str = "checked Core-to-CPS bridge currently accepts atomic, atomic-add, atomic-not, variable-let, and boolean-if entry results";

async fn assert_list_law_source_rejects_without_typed_lowering(
    source: &str,
    expected_lowering_error: &str,
) {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let source_path = tmp_dir.path().join("main.ash");
    std::fs::write(&source_path, source).expect("write main.ash");

    let engine = build_engine();
    let mut application = engine
        .parse_file(&source_path)
        .expect("list-law source should parse");
    engine
        .check(&mut application)
        .expect("list-law source should typecheck");

    let error = engine
        .run_file(&source_path)
        .await
        .expect_err("source without validated typed lowering must reject at admission");
    assert_eq!(
        error.to_string(),
        format!("{CLOSED_ADMISSION_PREFIX}{expected_lowering_error}"),
        "list source must expose its exact checked Core/CPS admission diagnostic"
    );
}

// ────────────────────────────────────────────────────────────────────
// Functor Laws
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_functor_identity_source_typechecks_and_rejects_without_typed_lowering() {
    assert_list_law_source_rejects_without_typed_lowering(
        r#"
use list::{map}

fn identity(x: Int) -> Int { x }

fn main() -> Bool {
    let list = [1, 2, 3, 4, 5]
    let mapped = map(list, identity)
    mapped == list
}
"#,
        ATOMIC_LET_LOWERING_ERROR,
    )
    .await;
}

// Note: Functor composition test is deferred due to language limitations:
// - nested function calls across module-level functions are not yet supported
// (e.g., add_one(mul_two(x)) inside compose doesn't resolve add_one)
// This will be re-enabled when the language supports these patterns.

// ────────────────────────────────────────────────────────────────────
// Semigroup Laws (via concat)
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_semigroup_associativity_source_typechecks_and_rejects_without_typed_lowering() {
    assert_list_law_source_rejects_without_typed_lowering(
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
        ATOMIC_LET_LOWERING_ERROR,
    )
    .await;
}

// ────────────────────────────────────────────────────────────────────
// Monoid Laws (via concat and empty list)
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_monoid_left_identity_source_typechecks_and_rejects_without_typed_lowering() {
    assert_list_law_source_rejects_without_typed_lowering(
        r#"
use list::{concat}

fn main() -> Bool {
    let list = [1, 2, 3, 4, 5]
    let lhs = concat([], list)
    lhs == list
}
"#,
        ATOMIC_LET_LOWERING_ERROR,
    )
    .await;
}

#[tokio::test]
async fn list_monoid_right_identity_source_typechecks_and_rejects_without_typed_lowering() {
    assert_list_law_source_rejects_without_typed_lowering(
        r#"
use list::{concat}

fn main() -> Bool {
    let list = [1, 2, 3, 4, 5]
    let lhs = concat(list, [])
    lhs == list
}
"#,
        ATOMIC_LET_LOWERING_ERROR,
    )
    .await;
}

// ────────────────────────────────────────────────────────────────────
// Additional List Operation Tests
// ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_len_empty_source_typechecks_and_rejects_without_typed_lowering() {
    assert_list_law_source_rejects_without_typed_lowering(
        r#"
use list::{len}

fn main() -> Bool {
    len([]) == 0
}
"#,
        ENTRY_RESULT_LOWERING_ERROR,
    )
    .await;
}

#[tokio::test]
async fn list_len_non_empty_source_typechecks_and_rejects_without_typed_lowering() {
    assert_list_law_source_rejects_without_typed_lowering(
        r#"
use list::{len}

fn main() -> Bool {
    len([1, 2, 3, 4, 5]) == 5
}
"#,
        ENTRY_RESULT_LOWERING_ERROR,
    )
    .await;
}

#[tokio::test]
async fn list_append_source_typechecks_and_rejects_without_typed_lowering() {
    assert_list_law_source_rejects_without_typed_lowering(
        r#"
use list::{len, append}

fn main() -> Bool {
    let list = [1, 2, 3]
    let new_list = append(list, 4)
    len(new_list) == 4
}
"#,
        ATOMIC_LET_LOWERING_ERROR,
    )
    .await;
}

#[tokio::test]
async fn list_take_drop_source_typechecks_and_rejects_without_typed_lowering() {
    assert_list_law_source_rejects_without_typed_lowering(
        r#"
use list::{concat, take, drop}

fn main() -> Bool {
    let list = [1, 2, 3, 4, 5]
    let n = 2
    let lhs = concat(take(n, list), drop(n, list))
    lhs == list
}
"#,
        ATOMIC_LET_LOWERING_ERROR,
    )
    .await;
}

// Note: reverse and map composition tests are deferred due to language limitations:
// - reverse is treated as a symbolic capability by the engine
// - nested function calls across module-level functions are not yet supported
// These will be re-enabled when the language supports these patterns.
