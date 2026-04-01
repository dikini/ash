//! Integration tests for stdlib parsing
//!
//! These tests verify that the standard library .ash files can be parsed correctly.

use std::fs;
use std::path::PathBuf;

use ash_parser::{
    Definition, Workflow, input::new_input, parse_module_decl, parse_type_def::parse_type_def,
    parse_use, workflow,
};
use winnow::prelude::*;

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

fn normalize_whitespace(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_capability(source: &str) -> Result<ash_parser::CapabilityDef, String> {
    let normalized = source
        .trim()
        .trim_start_matches("pub ")
        .trim_end_matches(';');
    let wrapped = format!("mod runtime {{ {} }}", normalized);
    let mut input = new_input(&wrapped);
    let decl = parse_module_decl
        .parse_next(&mut input)
        .map_err(|e| format!("{e:?}"))?;

    let definitions = decl.definitions().ok_or("expected inline module")?;

    match &definitions[0] {
        Definition::Capability(cap) => Ok(cap.clone()),
        _ => Err("first definition is not a capability".into()),
    }
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
fn test_runtime_mod_file_exists() {
    let path = stdlib_src_path().join("runtime/mod.ash");
    assert!(path.exists(), "runtime/mod.ash should exist");
}

#[test]
fn test_runtime_error_file_exists() {
    let path = stdlib_src_path().join("runtime/error.ash");
    assert!(path.exists(), "runtime/error.ash should exist");
}

#[test]
fn test_runtime_args_file_exists() {
    let path = stdlib_src_path().join("runtime/args.ash");
    assert!(path.exists(), "runtime/args.ash should exist");
}

#[test]
fn test_runtime_supervisor_file_exists() {
    let path = stdlib_src_path().join("runtime/supervisor.ash");
    assert!(path.exists(), "runtime/supervisor.ash should exist");
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
fn test_runtime_error_type_definition_parses() {
    let content = read_stdlib_file("runtime/error.ash");
    let normalized = normalize_whitespace(&content);

    assert!(
        normalized.contains("pub type RuntimeError = RuntimeError {"),
        "RuntimeError should use the canonical single-variant ADT syntax"
    );
    assert!(
        !normalized.contains("pub type RuntimeError = {"),
        "RuntimeError should reject the legacy plain record-alias syntax"
    );

    let mut input = new_input(&content);
    let result = parse_type_def(&mut input);

    assert!(
        result.is_ok(),
        "RuntimeError type definition should parse: {:?}",
        result
    );

    let type_def = result.unwrap();
    assert_eq!(type_def.name, "RuntimeError");
    assert!(type_def.params.is_empty());

    let variants = match &type_def.body {
        ash_parser::parse_type_def::TypeBody::Enum(variants) => variants,
        other => {
            panic!("RuntimeError body should parse as a single-variant enum ADT, got {other:?}")
        }
    };

    assert_eq!(
        variants.len(),
        1,
        "RuntimeError should have exactly one variant"
    );

    let variant = &variants[0];
    assert_eq!(variant.name, "RuntimeError");
    assert_eq!(
        variant.fields.len(),
        2,
        "RuntimeError variant should expose exactly two fields"
    );
    assert_eq!(variant.fields[0].0, "exit_code");
    assert_eq!(variant.fields[1].0, "message");
}

#[test]
fn test_runtime_args_capability_definition_parses() {
    let content = read_stdlib_file("runtime/args.ash");
    let use_line = content
        .lines()
        .find(|l| l.trim_start().starts_with("use option::Option;"))
        .expect("Should find Option import in runtime/args.ash");
    let mut use_input = new_input(use_line);
    assert!(
        parse_use(&mut use_input).is_ok(),
        "runtime/args.ash should use canonical stdlib import syntax"
    );

    let capability_line = content
        .lines()
        .find(|l| l.contains("pub capability Args"))
        .expect("Should find Args capability definition");

    let capability = parse_capability(capability_line).expect("Args capability should parse");

    assert_eq!(capability.name.as_ref(), "Args");
    assert_eq!(capability.params.len(), 1);
    assert_eq!(capability.params[0].name.as_ref(), "index");
    assert!(capability.return_type.is_some());
}

#[test]
fn test_runtime_supervisor_workflow_definition_parses() {
    let content = read_stdlib_file("runtime/supervisor.ash");
    for use_line in content
        .lines()
        .filter(|line| line.trim_start().starts_with("use "))
    {
        let mut use_input = new_input(use_line.trim());
        assert!(
            parse_use(&mut use_input).is_ok(),
            "system supervisor imports should parse: {use_line}"
        );
    }

    assert!(
        content.contains("pub workflow system_supervisor(args: cap Args) -> Int {"),
        "system_supervisor scaffold should expose the canonical signature"
    );

    let body_source = content
        .lines()
        .find(|line| line.trim_start().starts_with("ret "))
        .expect("system_supervisor should contain a return body")
        .trim();

    let mut input = new_input(body_source);
    let result = workflow(&mut input);

    assert!(
        result.is_ok(),
        "system_supervisor body should parse: {:?}",
        result
    );

    assert!(matches!(result.unwrap(), Workflow::Ret { .. }));
}

#[test]
fn test_runtime_import_examples_parse_with_canonical_syntax() {
    for source in [
        "use runtime::RuntimeError;",
        "use runtime::Args;",
        "use runtime::{RuntimeError, Args};",
    ] {
        let mut input = new_input(source);
        let result = parse_use(&mut input);

        assert!(result.is_ok(), "runtime import should parse: {source}");
    }
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
