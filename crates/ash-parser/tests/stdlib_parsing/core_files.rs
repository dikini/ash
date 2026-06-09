use super::support::*;

#[test]
fn test_lib_file_exists() {
    let path = stdlib_src_path().join("lib.ash");
    assert!(path.exists(), "lib.ash should exist");
}

#[test]
fn test_test_file_exists() {
    let path = stdlib_src_path().join("test.ash");
    assert!(path.exists(), "test.ash should exist");
}

#[test]
fn test_test_import_examples_parse_with_canonical_syntax() {
    for source in [
        "use test::assert_true;",
        "use test::{assert_true, assert_false, fail_test};",
        "pub use test::{assert_eq_int, assert_eq_string, assert_eq_bool};",
    ] {
        let mut input = new_input(source);
        let result = parse_use(&mut input);

        assert!(result.is_ok(), "std::test import should parse: {source}");
    }
}

#[test]
fn test_test_public_functions_parse_as_real_fn_definitions() {
    let functions = parse_public_functions("test.ash");
    let names = functions
        .iter()
        .map(|function| function.name.as_ref())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "assert_true",
            "assert_false",
            "assert_eq_int",
            "assert_ne_int",
            "assert_eq_string",
            "assert_eq_bool",
            "fail_test",
        ]
    );

    for function in functions {
        assert!(
            matches!(function.body, Expr::Block { .. }),
            "std::test helper bodies should parse as function blocks"
        );
    }
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

// TASK-494: io module parsing tests
// These tests will fail until the io module and io::path are properly implemented

#[test]
fn test_io_error_type_definition_parses() {
    let content = read_stdlib_file("io/mod.ash");

    // Extract the Error type definition line (not ErrorKind)
    let type_def_line = content
        .lines()
        .find(|l| l.contains("pub type Error ="))
        .expect("Should find Error type definition");

    let mut input = new_input(type_def_line);
    let result = parse_type_def(&mut input);

    assert!(
        result.is_ok(),
        "Error type definition should parse: {:?}",
        result
    );

    let type_def = result.unwrap();
    assert_eq!(type_def.name, "Error");
}
