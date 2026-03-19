//! Integration tests for stdlib parsing
//!
//! These tests verify that the standard library .ash files can be parsed correctly.

use std::fs;
use std::path::PathBuf;

use ash_parser::{input::new_input, parse_type_def::parse_type_def};

/// Get the path to the stdlib source directory
fn stdlib_src_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // ash-parser
    path.pop(); // crates
    path.push("std");
    path.push("src");
    path
}

/// Helper to read and return a stdlib file's content
fn read_stdlib_file(filename: &str) -> String {
    let path = stdlib_src_path().join(filename);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
}

#[test]
fn test_option_file_exists() {
    let path = stdlib_src_path().join("option.ash");
    assert!(path.exists(), "option.ash should exist");
}

#[test]
fn test_result_file_exists() {
    let path = stdlib_src_path().join("result.ash");
    assert!(path.exists(), "result.ash should exist");
}

#[test]
fn test_prelude_file_exists() {
    let path = stdlib_src_path().join("prelude.ash");
    assert!(path.exists(), "prelude.ash should exist");
}

#[test]
fn test_lib_file_exists() {
    let path = stdlib_src_path().join("lib.ash");
    assert!(path.exists(), "lib.ash should exist");
}

#[test]
fn test_option_type_definition_parses() {
    let content = read_stdlib_file("option.ash");

    // Extract the type definition line
    let type_def_line = content
        .lines()
        .find(|l| l.contains("pub type Option"))
        .expect("Should find Option type definition");

    let mut input = new_input(type_def_line);
    let result = parse_type_def(&mut input);

    assert!(
        result.is_ok(),
        "Option type definition should parse: {:?}",
        result
    );

    let type_def = result.unwrap();
    assert_eq!(type_def.name, "Option");
    assert_eq!(type_def.params.len(), 1);
    assert_eq!(type_def.params[0], "T");
}

#[test]
fn test_result_type_definition_parses() {
    let content = read_stdlib_file("result.ash");

    // Extract the type definition line
    let type_def_line = content
        .lines()
        .find(|l| l.contains("pub type Result"))
        .expect("Should find Result type definition");

    let mut input = new_input(type_def_line);
    let result = parse_type_def(&mut input);

    assert!(
        result.is_ok(),
        "Result type definition should parse: {:?}",
        result
    );

    let type_def = result.unwrap();
    assert_eq!(type_def.name, "Result");
    assert_eq!(type_def.params.len(), 2);
    assert_eq!(type_def.params[0], "T");
    assert_eq!(type_def.params[1], "E");
}

#[test]
fn test_option_file_contains_is_some_function() {
    let content = read_stdlib_file("option.ash");
    assert!(
        content.contains("pub fn is_some"),
        "option.ash should contain is_some function"
    );
}

#[test]
fn test_option_file_contains_is_none_function() {
    let content = read_stdlib_file("option.ash");
    assert!(
        content.contains("pub fn is_none"),
        "option.ash should contain is_none function"
    );
}

#[test]
fn test_option_file_contains_unwrap_function() {
    let content = read_stdlib_file("option.ash");
    assert!(
        content.contains("pub fn unwrap"),
        "option.ash should contain unwrap function"
    );
}

#[test]
fn test_option_file_contains_unwrap_or_function() {
    let content = read_stdlib_file("option.ash");
    assert!(
        content.contains("pub fn unwrap_or"),
        "option.ash should contain unwrap_or function"
    );
}

#[test]
fn test_option_file_contains_map_function() {
    let content = read_stdlib_file("option.ash");
    assert!(
        content.contains("pub fn map"),
        "option.ash should contain map function"
    );
}

#[test]
fn test_result_file_contains_is_ok_function() {
    let content = read_stdlib_file("result.ash");
    assert!(
        content.contains("pub fn is_ok"),
        "result.ash should contain is_ok function"
    );
}

#[test]
fn test_result_file_contains_is_err_function() {
    let content = read_stdlib_file("result.ash");
    assert!(
        content.contains("pub fn is_err"),
        "result.ash should contain is_err function"
    );
}

#[test]
fn test_result_file_contains_map_err_function() {
    let content = read_stdlib_file("result.ash");
    assert!(
        content.contains("pub fn map_err"),
        "result.ash should contain map_err function"
    );
}

#[test]
fn test_result_file_contains_and_then_function() {
    let content = read_stdlib_file("result.ash");
    assert!(
        content.contains("pub fn and_then"),
        "result.ash should contain and_then function"
    );
}

#[test]
fn test_prelude_contains_use_declarations() {
    let content = read_stdlib_file("prelude.ash");
    assert!(
        content.contains("use option::"),
        "prelude.ash should import from option"
    );
    assert!(
        content.contains("use result::"),
        "prelude.ash should import from result"
    );
}

#[test]
fn test_prelude_contains_re_exports() {
    let content = read_stdlib_file("prelude.ash");
    assert!(
        content.contains("pub use option::"),
        "prelude.ash should re-export from option"
    );
    assert!(
        content.contains("pub use result::"),
        "prelude.ash should re-export from result"
    );
}

#[test]
fn test_lib_contains_all_re_exports() {
    let content = read_stdlib_file("lib.ash");

    // Check for Option and Result types
    assert!(content.contains("Option"), "lib.ash should export Option");
    assert!(content.contains("Result"), "lib.ash should export Result");

    // Check for Some, None, Ok, Err constructors
    assert!(content.contains("Some"), "lib.ash should export Some");
    assert!(content.contains("None"), "lib.ash should export None");
    assert!(content.contains("Ok"), "lib.ash should export Ok");
    assert!(content.contains("Err"), "lib.ash should export Err");
}

#[test]
fn test_option_has_documentation_comments() {
    let content = read_stdlib_file("option.ash");
    // Check for module-level doc comment
    assert!(
        content.contains("-- Option type"),
        "option.ash should have module documentation"
    );
    // Check for function-level doc comments
    assert!(
        content.contains("-- Returns true"),
        "option.ash functions should have documentation"
    );
}

#[test]
fn test_result_has_documentation_comments() {
    let content = read_stdlib_file("result.ash");
    // Check for module-level doc comment
    assert!(
        content.contains("-- Result type"),
        "result.ash should have module documentation"
    );
    // Check for function-level doc comments
    assert!(
        content.contains("-- Returns true"),
        "result.ash functions should have documentation"
    );
}

#[test]
fn test_option_has_all_required_functions() {
    let content = read_stdlib_file("option.ash");

    let required_functions = [
        "is_some",
        "is_none",
        "unwrap",
        "unwrap_or",
        "map",
        "and",
        "or",
        "ok_or",
    ];

    for func in &required_functions {
        assert!(
            content.contains(&format!("pub fn {}", func)),
            "option.ash should contain {} function",
            func
        );
    }
}

#[test]
fn test_result_has_all_required_functions() {
    let content = read_stdlib_file("result.ash");

    let required_functions = [
        "is_ok",
        "is_err",
        "unwrap",
        "unwrap_err",
        "unwrap_or",
        "map",
        "map_err",
        "and_then",
        "ok",
        "err",
    ];

    for func in &required_functions {
        assert!(
            content.contains(&format!("pub fn {}", func)),
            "result.ash should contain {} function",
            func
        );
    }
}

#[test]
fn test_stdlib_readme_exists() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // ash-parser
    path.pop(); // crates
    path.push("std");
    path.push("README.md");

    assert!(path.exists(), "std/README.md should exist");

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("# Ash Standard Library"),
        "README should have title"
    );
    assert!(content.contains("Option"), "README should document Option");
    assert!(content.contains("Result"), "README should document Result");
}

#[test]
fn test_stdlib_cargo_toml_exists() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // ash-parser
    path.pop(); // crates
    path.push("std");
    path.push("Cargo.toml");

    assert!(path.exists(), "std/Cargo.toml should exist");

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("name = \"ash-std\""),
        "Cargo.toml should have correct name"
    );
}
