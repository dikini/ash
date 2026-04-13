//! Integration tests for Engine::check_module_file (TASK-541).

use ash_engine::Engine;
use std::path::PathBuf;

/// Helper to build a default engine.
fn make_engine() -> Engine {
    Engine::new().build().expect("engine should build")
}

/// Test 1: Check the real stdlib LLM types file.
///
/// `std/src/llm/types.ash` contains 11 `pub type` definitions.  Two of them
/// reference `Float`, which is not a builtin type in the default `TypeEnv`,
/// so we expect exactly 2 registration errors for those types.
#[test]
fn test_check_module_file_types_ash() {
    let engine = make_engine();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set");
    let types_path = PathBuf::from(manifest_dir)
        .join("../../std/src/llm/types.ash")
        .canonicalize()
        .expect("types.ash should exist");

    let result = engine
        .check_module_file(&types_path)
        .expect("check_module_file should succeed");

    assert_eq!(
        result.type_count, 11,
        "types.ash should have 11 pub type definitions, got {}",
        result.type_count,
    );
    assert_eq!(
        result.fn_count, 0,
        "types.ash should have 0 pub fn definitions, got {}",
        result.fn_count,
    );
    // Float is not a builtin type, so Embedding and CompletionParams fail to register.
    assert_eq!(
        result.errors.len(),
        2,
        "expected 2 registration errors for Float-using types, got {:?}",
        result.errors,
    );
    assert!(
        result.warnings.is_empty(),
        "types.ash should have zero warnings: {:?}",
        result.warnings,
    );
}

/// Test 2: An invalid type definition should produce an error.
#[test]
fn test_check_module_file_invalid_type() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file_path = dir.path().join("bad.ash");
    std::fs::write(
        &file_path,
        "pub type Broken = ;\n", // empty variant list after =
    )
    .expect("write temp file");

    let engine = make_engine();
    let result = engine.check_module_file(&file_path);

    // collect_public_type_defs_from_source should fail to parse the broken type.
    assert!(
        result.is_err(),
        "expected error for invalid type definition, got {:?}",
        result,
    );
}

/// Test 3: An empty file should succeed with 0 types and 0 fns.
#[test]
fn test_check_module_file_empty() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file_path = dir.path().join("empty.ash");
    std::fs::write(&file_path, "").expect("write temp file");

    let engine = make_engine();
    let result = engine
        .check_module_file(&file_path)
        .expect("empty file should succeed");

    assert_eq!(
        result.type_count, 0,
        "empty file should have 0 types, got {}",
        result.type_count,
    );
    assert_eq!(
        result.fn_count, 0,
        "empty file should have 0 fns, got {}",
        result.fn_count,
    );
    assert!(
        result.errors.is_empty(),
        "empty file should have zero errors: {:?}",
        result.errors,
    );
    assert!(
        result.warnings.is_empty(),
        "empty file should have zero warnings: {:?}",
        result.warnings,
    );
}

/// Test 4: A `pub fn` with invalid syntax should produce a warning.
#[test]
fn test_pub_fn_parse_failure_produces_warning() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file_path = dir.path().join("broken_fn.ash");
    std::fs::write(
        &file_path,
        "pub type Role = System | User;\n\npub fn broken( {\n    -- missing closing paren, invalid syntax\n}\n",
    )
    .expect("write temp file");

    let engine = make_engine();
    let result = engine
        .check_module_file(&file_path)
        .expect("check_module_file should succeed even with broken pub fn");

    assert_eq!(
        result.type_count, 1,
        "should have 1 pub type, got {}",
        result.type_count,
    );
    assert_eq!(
        result.fn_count, 0,
        "should have 0 parseable pub fn, got {}",
        result.fn_count,
    );
    assert!(
        !result.warnings.is_empty(),
        "expected at least one warning for broken pub fn, got {:?}",
        result.warnings,
    );
    assert!(
        result.warnings.iter().any(|w| w.contains("broken")),
        "warning should mention the function name 'broken', got {:?}",
        result.warnings,
    );
}

/// Test 5: A valid `pub fn` should produce no warnings.
#[test]
fn test_valid_pub_fn_no_warning() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file_path = dir.path().join("good_fn.ash");
    std::fs::write(
        &file_path,
        "pub fn greet(name: Text) -> Text {\n    \"hello\"\n}\n",
    )
    .expect("write temp file");

    let engine = make_engine();
    let result = engine
        .check_module_file(&file_path)
        .expect("check_module_file should succeed");

    assert_eq!(
        result.fn_count, 1,
        "should have 1 parseable pub fn, got {}",
        result.fn_count,
    );
    assert!(
        result.warnings.is_empty(),
        "valid pub fn should produce zero warnings, got {:?}",
        result.warnings,
    );
}

/// Test 6: `count_pub_fn_snippets` returns correct count and diagnostics for mixed source.
#[test]
fn test_count_pub_fn_snippets_with_diagnostics() {
    use ash_engine::module_loader::count_pub_fn_snippets;

    let source = r#"
pub fn good(x: Int) -> Int {
    x
}

pub fn bad( {
    -- broken syntax
}

pub fn also_good(y: Text) -> Text {
    y
}
"#;

    let (count, diagnostics) = count_pub_fn_snippets(source);

    assert_eq!(
        count, 2,
        "should count 2 valid pub fn snippets, got {}",
        count,
    );
    assert_eq!(
        diagnostics.len(),
        1,
        "should have 1 diagnostic for the broken snippet, got {:?}",
        diagnostics,
    );
    assert_eq!(
        diagnostics[0].name.as_deref(),
        Some("bad"),
        "diagnostic should identify function name 'bad', got {:?}",
        diagnostics[0].name,
    );
}
