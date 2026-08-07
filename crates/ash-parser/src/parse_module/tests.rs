//! Tests for `parse_module`.

use super::*;
use crate::input::new_input;
use crate::surface::{Definition, Visibility};

/// Test helper to create a ParseInput for testing
fn test_input(s: &str) -> ParseInput<'_> {
    new_input(s)
}

// ========================================================================
// Phase 156: Parser Blocker Resolution Tests
// ========================================================================

/// TASK-1560: if/else with match in else branch
#[test]
fn test_if_else_with_match_parses() {
    let mut input = test_input("{ if n <= 0 then [] else match list { Nil => [] } }");
    let result = parse_fn_body(&mut input);
    assert!(
        result.is_ok(),
        "if/else with match should parse: {:?}",
        result.err()
    );
}

/// TASK-1562: list literal in expression
#[test]
fn test_list_literal_expr_parses() {
    let mut input = test_input("{ [h] }");
    let result = parse_fn_body(&mut input);
    assert!(
        result.is_ok(),
        "list literal expr should parse: {:?}",
        result.err()
    );
}

/// TASK-1562: list literal in match arm body
#[test]
fn test_list_literal_in_match_arm_parses() {
    let mut input = test_input("{ match list { Nil => [] } }");
    let result = parse_fn_body(&mut input);
    assert!(
        result.is_ok(),
        "list literal in match arm should parse: {:?}",
        result.err()
    );
}

/// TASK-1561: variant pattern with record payload (baseline)
#[test]
fn test_variant_record_pattern_parses() {
    let mut input = test_input("{ match list { Cons { head: h, tail: rest } => h } }");
    let result = parse_fn_body(&mut input);
    assert!(
        result.is_ok(),
        "variant record pattern should parse: {:?}",
        result.err()
    );
}

/// TASK-1561: variant pattern with record payload and list body
#[test]
fn test_variant_record_pattern_list_body_parses() {
    let mut input = test_input("{ match list { Cons { head: h, tail: rest } => [h] } }");
    let result = parse_fn_body(&mut input);
    assert!(
        result.is_ok(),
        "variant record pattern with list body should parse: {:?}",
        result.err()
    );
}

/// TASK-1561: variant pattern with shorthand (NOT SUPPORTED - requires parser change)
#[test]
fn test_variant_shorthand_pattern_not_supported() {
    let mut input = test_input("{ match list { Cons { head, tail } => [head] } }");
    let result = parse_fn_body(&mut input);
    // Shorthand patterns are not currently supported - they should fail
    assert!(
        result.is_err(),
        "variant shorthand pattern should NOT parse (not supported)"
    );
}

/// TASK-1562: list pattern
#[test]
fn test_list_pattern_parses() {
    let mut input = test_input("{ match list { [h, ..rest] => [h] } }");
    let result = parse_fn_body(&mut input);
    assert!(
        result.is_ok(),
        "list pattern should parse: {:?}",
        result.err()
    );
}

/// TASK-1562: empty list pattern
#[test]
fn test_empty_list_pattern_parses() {
    let mut input = test_input("{ match list { [] => [] } }");
    let result = parse_fn_body(&mut input);
    assert!(
        result.is_ok(),
        "empty list pattern should parse: {:?}",
        result.err()
    );
}

/// TASK-1560: match alone (baseline)
#[test]
fn test_match_alone_parses() {
    let mut input = test_input("{ match list { Nil => [] } }");
    let result = parse_fn_body(&mut input);
    assert!(
        result.is_ok(),
        "match alone should parse: {:?}",
        result.err()
    );
}

/// TASK-1560: Simple if/else (baseline)
#[test]
fn test_simple_if_else_parses() {
    let mut input = test_input("{ if n == 0 then 1 else 2 }");
    let result = parse_fn_body(&mut input);
    assert!(
        result.is_ok(),
        "simple if/else should parse: {:?}",
        result.err()
    );
}

/// TASK-1560: if with match in then branch
#[test]
fn test_if_then_match_parses() {
    let mut input = test_input("{ if n == 0 then match list { Nil => [] } else [] }");
    let result = parse_fn_body(&mut input);
    assert!(
        result.is_ok(),
        "if with match in then branch should parse: {:?}",
        result.err()
    );
}

fn inline_module_with_unknown_item(body_after_unknown: &str) -> String {
    format!("mod governance {{ extension custom {{ enabled: true }} {body_after_unknown} }}")
}

fn assert_inline_module_rejects_after_unknown_item(
    body_after_unknown: &str,
    item_description: &str,
) {
    let source = inline_module_with_unknown_item(body_after_unknown);
    let mut input = test_input(&source);

    let result = parse_module_decl(&mut input);

    match result {
        Err(_) => {}
        Ok(decl) => panic!(
            "Expected parse to fail instead of silently skipping an unsupported {item_description} after unknown-item recovery, but parsed definitions: {:?}",
            decl.definitions()
        ),
    }
}

// ========================================================================
// File-based Module Tests
// ========================================================================

#[test]
fn test_parse_mod_foo_semicolon() {
    // Test: `mod foo;` → file-based module
    let mut input = test_input("mod foo;");
    let result = parse_module_decl(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );

    let decl = result.unwrap();
    assert_eq!(decl.name.as_ref(), "foo");
    assert_eq!(decl.visibility, Visibility::Inherited);
    assert!(decl.is_file_based());
    assert!(!decl.is_inline());
    assert!(matches!(decl.source, ModuleSource::File));
}

#[test]
fn test_parse_pub_mod_foo_semicolon() {
    // Test: `pub mod foo;` → public file-based module
    let mut input = test_input("pub mod foo;");
    let result = parse_module_decl(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );

    let decl = result.unwrap();
    assert_eq!(decl.name.as_ref(), "foo");
    assert_eq!(decl.visibility, Visibility::Public);
    assert!(decl.is_file_based());
    assert!(!decl.is_inline());
}

#[test]
fn test_parse_pub_crate_mod_foo_semicolon() {
    // Test: `pub(crate) mod foo;` → crate-visible file-based module
    let mut input = test_input("pub(crate) mod foo;");
    let result = parse_module_decl(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );

    let decl = result.unwrap();
    assert_eq!(decl.name.as_ref(), "foo");
    assert_eq!(decl.visibility, Visibility::Crate);
    assert!(decl.is_file_based());
}

// ========================================================================
// Inline Module Tests
// ========================================================================

#[test]
fn test_parse_inline_module_empty() {
    // Test: `mod foo {}` → empty inline module
    let mut input = test_input("mod foo {}");
    let result = parse_module_decl(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );

    let decl = result.unwrap();
    assert_eq!(decl.name.as_ref(), "foo");
    assert_eq!(decl.visibility, Visibility::Inherited);
    assert!(!decl.is_file_based());
    assert!(decl.is_inline());

    let defs = decl
        .definitions()
        .expect("inline module should have definitions");
    assert!(defs.is_empty());
}

#[test]
fn test_parse_inline_module_rejects_invalid_constraint_predicate_identifier() {
    let mut input = test_input("mod foo { capability approve: decide() where 1requires_mfa(); }");

    let result = parse_module_decl(&mut input);

    assert!(
        result.is_err(),
        "Expected parse to fail for a non-canonical predicate identifier"
    );
}

#[test]
fn test_parse_inline_module_rejects_unsupported_workflow_after_unknown_item() {
    assert_inline_module_rejects_after_unknown_item("fn main() { {} }", "workflow");
}

#[test]
fn test_parse_inline_module_rejects_unsupported_datatype_after_unknown_item() {
    assert_inline_module_rejects_after_unknown_item(
        "datatype review_state = Pending | Approved; legacy_metadata reviewer { approve }",
        "datatype",
    );
}

#[test]
fn test_parse_inline_module_rejects_unsupported_canonical_datatype_definition() {
    let mut input = test_input(
        "mod governance { datatype review_state = Pending | Approved; legacy_metadata reviewer { approve } }",
    );

    let result = parse_module_decl(&mut input);

    assert!(
        result.is_err(),
        "Expected inline modules to reject unsupported canonical datatype definitions explicitly"
    );
}

#[test]
fn test_parse_pub_inline_module() {
    // Test: `pub mod foo {}` → public inline module
    let mut input = test_input("pub mod foo {}");
    let result = parse_module_decl(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );

    let decl = result.unwrap();
    assert_eq!(decl.name.as_ref(), "foo");
    assert_eq!(decl.visibility, Visibility::Public);
    assert!(decl.is_inline());
}

// ========================================================================
// Whitespace and Formatting Tests
// ========================================================================

#[test]
fn test_parse_mod_with_whitespace() {
    // Test parsing with extra whitespace
    let mut input = test_input("  mod   foo   ;  ");
    let result = parse_module_decl(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );

    let decl = result.unwrap();
    assert_eq!(decl.name.as_ref(), "foo");
    assert!(decl.is_file_based());
}

#[test]
fn test_parse_inline_mod_with_whitespace() {
    // Test parsing inline module with extra whitespace
    let mut input = test_input("  mod   foo   {   }  ");
    let result = parse_module_decl(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );

    let decl = result.unwrap();
    assert_eq!(decl.name.as_ref(), "foo");
    assert!(decl.is_inline());
}

#[test]
fn test_parse_inline_module_definition_spans_track_comments_and_indentation() {
    let mut input =
        test_input("mod foo {\n  -- comment before function\n  fn approve() -> Bool { true }\n}");

    let decl = parse_module_decl(&mut input).expect("inline module should parse");
    let definitions = decl
        .definitions()
        .expect("inline module should expose parsed definitions");

    let Definition::Function(function) = &definitions[0] else {
        panic!("expected first definition to be a function: {definitions:?}");
    };

    assert_eq!(function.span.line, 3);
    assert_eq!(function.span.column, 3);
}

#[test]
fn removed_act_do_sugar_does_not_parse_in_function_body() {
    for source in [
        "{ act { return 42 } }",
        "{ act { x <- act::unit(42); return x } }",
        "{ act { result <- read_file(path); return result } }",
        "{ act {} }",
    ] {
        let mut input = test_input(source);
        assert!(
            parse_fn_body(&mut input).is_err(),
            "removed act block parsed in function body: {source}"
        );
    }
}
