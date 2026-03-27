//! Tests for crate root and dependency syntax parsing.
//!
//! This module tests the parsing of crate metadata declarations including:
//! - `crate <name>;` declarations
//! - `dependency <alias> from "<path>";` declarations
//! - Error cases for invalid syntax

use ash_parser::input::new_input;
use ash_parser::parse_crate_root::parse_crate_root_metadata;
use winnow::prelude::*;

// =========================================================================
// RED Phase: Tests should FAIL initially (TDD)
// =========================================================================

#[test]
fn test_parse_crate_root_name() {
    let input_str = "crate app;";
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let metadata = result.unwrap();
    assert_eq!(metadata.crate_name.as_ref(), "app");
    assert!(metadata.dependencies.is_empty());
}

#[test]
fn test_parse_crate_root_with_dependencies() {
    let input_str = r#"crate app;

dependency util from "../util/main.ash";
dependency policy from "../policy/main.ash";
"#;
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let metadata = result.unwrap();
    assert_eq!(metadata.crate_name.as_ref(), "app");
    assert_eq!(metadata.dependencies.len(), 2);

    // Check first dependency
    assert_eq!(metadata.dependencies[0].alias.as_ref(), "util");
    assert_eq!(
        metadata.dependencies[0].root_path.as_ref(),
        "../util/main.ash"
    );

    // Check second dependency
    assert_eq!(metadata.dependencies[1].alias.as_ref(), "policy");
    assert_eq!(
        metadata.dependencies[1].root_path.as_ref(),
        "../policy/main.ash"
    );
}

#[test]
fn test_parse_dependency_requires_quoted_path() {
    // This should fail - path must be quoted
    let input_str = "crate app;\ndependency util from ../util/main.ash;";
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(result.is_err(), "Expected parse to fail for unquoted path");
}

#[test]
fn test_parse_dependency_without_crate_decl_rejected() {
    // Dependencies must come after crate declaration
    let input_str = r#"dependency util from "../util/main.ash";
crate app;
"#;
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(
        result.is_err(),
        "Expected parse to fail when dependency comes before crate"
    );
}

// =========================================================================
// Additional Edge Case Tests
// =========================================================================

#[test]
fn test_parse_crate_root_with_single_dependency() {
    let input_str = r#"crate myapp;

dependency core from "./core/main.ash";
"#;
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let metadata = result.unwrap();
    assert_eq!(metadata.crate_name.as_ref(), "myapp");
    assert_eq!(metadata.dependencies.len(), 1);
    assert_eq!(metadata.dependencies[0].alias.as_ref(), "core");
    assert_eq!(
        metadata.dependencies[0].root_path.as_ref(),
        "./core/main.ash"
    );
}

#[test]
fn test_parse_crate_root_with_multiple_dependencies() {
    let input_str = r#"crate server;

dependency api from "../api/main.ash";
dependency db from "../db/main.ash";
dependency auth from "../auth/main.ash";
"#;
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let metadata = result.unwrap();
    assert_eq!(metadata.crate_name.as_ref(), "server");
    assert_eq!(metadata.dependencies.len(), 3);
}

#[test]
fn test_parse_crate_name_with_underscores() {
    let input_str = "crate my_crate_name;";
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let metadata = result.unwrap();
    assert_eq!(metadata.crate_name.as_ref(), "my_crate_name");
}

#[test]
fn test_parse_crate_with_empty_dependencies() {
    let input_str = "crate standalone;";
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let metadata = result.unwrap();
    assert_eq!(metadata.crate_name.as_ref(), "standalone");
    assert!(metadata.dependencies.is_empty());
}

#[test]
fn test_parse_dependency_with_absolute_path() {
    let input_str = r#"crate app;

dependency std from "/usr/lib/ash/std/main.ash";
"#;
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let metadata = result.unwrap();
    assert_eq!(metadata.dependencies.len(), 1);
    assert_eq!(
        metadata.dependencies[0].root_path.as_ref(),
        "/usr/lib/ash/std/main.ash"
    );
}

#[test]
fn test_parse_crate_with_whitespace_variations() {
    // Test with extra whitespace between tokens
    let input_str = "crate   spaced   ;\n\n  dependency   util   from   \"../util.ash\"   ;";
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse with extra whitespace, got: {:?}",
        result
    );
    let metadata = result.unwrap();
    assert_eq!(metadata.crate_name.as_ref(), "spaced");
    assert_eq!(metadata.dependencies.len(), 1);
}

#[test]
fn test_parse_missing_semicolon_after_crate() {
    let input_str = "crate app\ndependency util from \"../util.ash\";";
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(
        result.is_err(),
        "Expected parse to fail without semicolon after crate name"
    );
}

#[test]
fn test_parse_missing_semicolon_after_dependency() {
    let input_str = r#"crate app;

dependency util from "../util.ash"
dependency other from "../other.ash";
"#;
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(
        result.is_err(),
        "Expected parse to fail without semicolon after dependency"
    );
}

#[test]
fn test_parse_empty_string_path_rejected() {
    let input_str = r#"crate app;

dependency util from "";
"#;
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(result.is_err(), "Expected parse to fail with empty path");
}

#[test]
fn test_parse_dependency_alias_with_numbers() {
    let input_str = r#"crate app;

dependency util2 from "../util2.ash";
"#;
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse with numbers in alias, got: {:?}",
        result
    );
    let metadata = result.unwrap();
    assert_eq!(metadata.dependencies[0].alias.as_ref(), "util2");
}

#[test]
fn test_span_information_preserved() {
    let input_str = "crate app;";
    let mut input = new_input(input_str);
    let result = parse_crate_root_metadata.parse_next(&mut input);

    assert!(result.is_ok());
    let metadata = result.unwrap();
    // Span should capture the entire declaration
    assert_eq!(metadata.span.start, 0);
    assert!(metadata.span.end > 0);
    assert_eq!(metadata.span.line, 1);
    assert_eq!(metadata.span.column, 1);
}
