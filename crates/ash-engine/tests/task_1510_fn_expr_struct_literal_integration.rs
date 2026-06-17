//! TASK-1510 integration tests: fn expressions and closures in multi-field struct literals.
//!
//! These tests verify that the full pipeline (parse + typecheck) works for struct literals
//! with fn expressions and closures in multiple fields.

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEST_FILE_ID: AtomicU64 = AtomicU64::new(0);

fn check_module(source: &str) -> Result<(), String> {
    let temp_dir = std::env::temp_dir();
    let file_id = NEXT_TEST_FILE_ID.fetch_add(1, Ordering::Relaxed);
    let path = temp_dir.join(format!("test_1510_{}_{}.ash", std::process::id(), file_id));
    std::fs::write(&path, source).map_err(|e: std::io::Error| e.to_string())?;

    let engine = ash_engine::Engine::new()
        .build()
        .map_err(|e| format!("{e}"))?;
    let result = match engine.check_module_file(&path) {
        Ok(result) => {
            if result.errors.is_empty() {
                Ok(())
            } else {
                Err(result.errors.join("\n"))
            }
        }
        Err(e) => Err(format!("{e}")),
    };
    let _ = std::fs::remove_file(path);
    result
}

// ---------------------------------------------------------------------------
// Single-field struct literals (baseline)
// ---------------------------------------------------------------------------

#[test]
fn single_field_struct_with_fn_expr_checks() {
    let result = check_module(
        "type Box = Box { value: (Int) -> Int };\n\
         fn make() -> Box { Box { value: fn(x: Int) -> Int { 42 } } }",
    );
    assert!(
        result.is_ok(),
        "single-field struct with fn expr should check: {}",
        result.unwrap_err()
    );
}

#[test]
fn single_field_struct_with_closure_checks() {
    let result = check_module(
        "type Box = Box { value: (Int) -> Int };\n\
         fn make() -> Box { Box { value: |x: Int| -> 42 } }",
    );
    assert!(
        result.is_ok(),
        "single-field struct with closure should check: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// Multi-field struct literals with fn expressions (the bug)
// ---------------------------------------------------------------------------

#[test]
fn two_field_struct_with_fn_expr_checks() {
    let result = check_module(
        "type Pair = Pair { first: (Int) -> Int, second: (Int) -> Int };\n\
         fn make() -> Pair { \
             Pair { \
                 first: fn(x: Int) -> Int { 42 }, \
                 second: fn(x: Int) -> Int { 43 } \
             } \
         }",
    );
    assert!(
        result.is_ok(),
        "two-field struct with fn expr should check: {}",
        result.unwrap_err()
    );
}

#[test]
fn two_field_struct_with_closure_checks() {
    let result = check_module(
        "type Pair = Pair { first: (Int) -> Int, second: (Int) -> Int };\n\
         fn make() -> Pair { \
             Pair { \
                 first: |x: Int| -> 42, \
                 second: |x: Int| -> 43 \
             } \
         }",
    );
    assert!(
        result.is_ok(),
        "two-field struct with closure should check: {}",
        result.unwrap_err()
    );
}

#[test]
fn three_field_struct_with_fn_expr_checks() {
    let result = check_module(
        "type Triple = Triple { a: (Int) -> Int, b: (Int) -> Int, c: (Int) -> Int };\n\
         fn make() -> Triple { \
             Triple { \
                 a: fn(x: Int) -> Int { 1 }, \
                 b: fn(x: Int) -> Int { 2 }, \
                 c: fn(x: Int) -> Int { 3 } \
             } \
         }",
    );
    assert!(
        result.is_ok(),
        "three-field struct with fn expr should check: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// Generic struct with fn expressions (Strategy-like)
// ---------------------------------------------------------------------------

#[test]
fn generic_struct_with_fn_expr_checks() {
    let result = check_module(
        "type Strategy<T> = Strategy { gen: (Int) -> T, shrink: (T) -> List<T> };\n\
         fn make() -> Strategy<Int> { \
             Strategy { \
                 gen: fn(x: Int) -> Int { 42 }, \
                 shrink: fn(x: Int) -> List<Int> { [] } \
             } \
         }",
    );
    assert!(
        result.is_ok(),
        "generic struct with fn expr should check: {}",
        result.unwrap_err()
    );
}

#[test]
fn generic_struct_with_closure_checks() {
    let result = check_module(
        "type Strategy<T> = Strategy { gen: (Int) -> T, shrink: (T) -> List<T> };\n\
         fn make() -> Strategy<Int> { \
             Strategy { \
                 gen: |x: Int| -> 42, \
                 shrink: |x: Int| -> [] \
             } \
         }",
    );
    assert!(
        result.is_ok(),
        "generic struct with closure should check: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// Mixed field types
// ---------------------------------------------------------------------------

#[test]
fn mixed_field_types_with_fn_expr_checks() {
    let result = check_module(
        "type Mixed = Mixed { name: String, f: (Int) -> Int };\n\
         fn make() -> Mixed { \
             Mixed { \
                 name: \"hello\", \
                 f: fn(x: Int) -> Int { 42 } \
             } \
         }",
    );
    assert!(
        result.is_ok(),
        "mixed field types with fn expr should check: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// Trailing comma cases
// ---------------------------------------------------------------------------

#[test]
fn two_field_struct_with_trailing_comma_and_fn_expr_checks() {
    let result = check_module(
        "type Pair = Pair { first: (Int) -> Int, second: (Int) -> Int };\n\
         fn make() -> Pair { \
             Pair { \
                 first: fn(x: Int) -> Int { 42 }, \
                 second: fn(x: Int) -> Int { 43 }, \
             } \
         }",
    );
    assert!(
        result.is_ok(),
        "trailing comma with fn expr should check: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// Nested struct literals
// ---------------------------------------------------------------------------

#[test]
fn nested_struct_with_fn_expr_checks() {
    let result = check_module(
        "type Inner = Inner { f: (Int) -> Int };\n\
         type Outer = Outer { inner: Inner };\n\
         fn make() -> Outer { \
             Outer { \
                 inner: Inner { f: fn(x: Int) -> Int { 42 } } \
             } \
         }",
    );
    assert!(
        result.is_ok(),
        "nested struct with fn expr should check: {}",
        result.unwrap_err()
    );
}

// ---------------------------------------------------------------------------
// Map-like combinator pattern (the motivating use case)
// ---------------------------------------------------------------------------

#[test]
fn map_combinator_pattern_checks() {
    let result = check_module(
        "type GenContext = GenContext { seed: Int, size: Int };\n\
         type Strategy<T> = Strategy { gen: (GenContext) -> T, shrink: (T) -> List<T> };\n\
         fn map<A, B>(s: Strategy<A>, f: (A) -> B) -> Strategy<B> { \
             Strategy { \
                 gen: fn(ctx: GenContext) -> B { f(s.gen(ctx)) }, \
                 shrink: fn(b: B) -> List<B> { [] } \
             } \
         }",
    );
    assert!(
        result.is_ok(),
        "map combinator pattern should check: {}",
        result.unwrap_err()
    );
}

#[test]
fn with_shrink_combinator_pattern_checks() {
    let result = check_module(
        "type GenContext = GenContext { seed: Int, size: Int };\n\
         type Strategy<T> = Strategy { gen: (GenContext) -> T, shrink: (T) -> List<T> };\n\
         fn with_shrink<T>(s: Strategy<T>, new_shrink: (T) -> List<T>) -> Strategy<T> { \
             Strategy { \
                 gen: s.gen, \
                 shrink: new_shrink \
             } \
         }",
    );
    assert!(
        result.is_ok(),
        "with_shrink combinator pattern should check: {}",
        result.unwrap_err()
    );
}

#[test]
fn append_shrink_combinator_pattern_checks() {
    let result = check_module(
        "type GenContext = GenContext { seed: Int, size: Int };\n\
         type Strategy<T> = Strategy { gen: (GenContext) -> T, shrink: (T) -> List<T> };\n\
         fn append_shrink<T>(s: Strategy<T>, extra: List<T>) -> Strategy<T> { \
             Strategy { \
                 gen: s.gen, \
                 shrink: fn(t: T) -> List<T> { concat(s.shrink(t), extra) } \
             } \
         }",
    );
    assert!(
        result.is_ok(),
        "append_shrink combinator pattern should check: {}",
        result.unwrap_err()
    );
}

#[test]
fn prepend_shrink_combinator_pattern_checks() {
    let result = check_module(
        "type GenContext = GenContext { seed: Int, size: Int };\n\
         type Strategy<T> = Strategy { gen: (GenContext) -> T, shrink: (T) -> List<T> };\n\
         fn prepend_shrink<T>(s: Strategy<T>, extra: List<T>) -> Strategy<T> { \
             Strategy { \
                 gen: s.gen, \
                 shrink: fn(t: T) -> List<T> { concat(extra, s.shrink(t)) } \
             } \
         }",
    );
    assert!(
        result.is_ok(),
        "prepend_shrink combinator pattern should check: {}",
        result.unwrap_err()
    );
}

#[test]
fn one_of_combinator_pattern_checks() {
    let result = check_module(
        "type GenContext = GenContext { seed: Int, size: Int };\n\
         type Strategy<T> = Strategy { gen: (GenContext) -> T, shrink: (T) -> List<T> };\n\
         fn one_of<T>(choices: List<Strategy<T>>) -> Strategy<T> { \
             Strategy { \
                 gen: fn(ctx: GenContext) -> T { choices[0].gen(ctx) }, \
                 shrink: fn(t: T) -> List<T> { [] } \
             } \
         }",
    );
    assert!(
        result.is_ok(),
        "one_of combinator pattern should check: {}",
        result.unwrap_err()
    );
}
