//! Tests for `parse_module`.

use super::*;
use crate::input::new_input;
use crate::surface::{Constraint, Definition, EffectType, Expr, Literal, Predicate, Visibility};

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
    assert!(result.is_ok(), "if/else with match should parse: {:?}", result.err());
}

/// TASK-1562: list literal in expression
#[test]
fn test_list_literal_expr_parses() {
    let mut input = test_input("{ [h] }");
    let result = parse_fn_body(&mut input);
    assert!(result.is_ok(), "list literal expr should parse: {:?}", result.err());
}

/// TASK-1562: list literal in match arm body
#[test]
fn test_list_literal_in_match_arm_parses() {
    let mut input = test_input("{ match list { Nil => [] } }");
    let result = parse_fn_body(&mut input);
    assert!(result.is_ok(), "list literal in match arm should parse: {:?}", result.err());
}

/// TASK-1561: variant pattern with record payload (baseline)
#[test]
fn test_variant_record_pattern_parses() {
    let mut input = test_input("{ match list { Cons { head: h, tail: rest } => h } }");
    let result = parse_fn_body(&mut input);
    assert!(result.is_ok(), "variant record pattern should parse: {:?}", result.err());
}

/// TASK-1561: variant pattern with record payload and list body
#[test]
fn test_variant_record_pattern_list_body_parses() {
    let mut input = test_input("{ match list { Cons { head: h, tail: rest } => [h] } }");
    let result = parse_fn_body(&mut input);
    assert!(result.is_ok(), "variant record pattern with list body should parse: {:?}", result.err());
}

/// TASK-1561: variant pattern with shorthand (NOT SUPPORTED - requires parser change)
#[test]
fn test_variant_shorthand_pattern_not_supported() {
    let mut input = test_input("{ match list { Cons { head, tail } => [head] } }");
    let result = parse_fn_body(&mut input);
    // Shorthand patterns are not currently supported - they should fail
    assert!(result.is_err(), "variant shorthand pattern should NOT parse (not supported)");
}

/// TASK-1562: list pattern
#[test]
fn test_list_pattern_parses() {
    let mut input = test_input("{ match list { [h, ..rest] => [h] } }");
    let result = parse_fn_body(&mut input);
    assert!(result.is_ok(), "list pattern should parse: {:?}", result.err());
}

/// TASK-1562: empty list pattern
#[test]
fn test_empty_list_pattern_parses() {
    let mut input = test_input("{ match list { [] => [] } }");
    let result = parse_fn_body(&mut input);
    assert!(result.is_ok(), "empty list pattern should parse: {:?}", result.err());
}

/// TASK-1560: match alone (baseline)
#[test]
fn test_match_alone_parses() {
    let mut input = test_input("{ match list { Nil => [] } }");
    let result = parse_fn_body(&mut input);
    assert!(result.is_ok(), "match alone should parse: {:?}", result.err());
}

/// TASK-1560: Simple if/else (baseline)
#[test]
fn test_simple_if_else_parses() {
    let mut input = test_input("{ if n == 0 then 1 else 2 }");
    let result = parse_fn_body(&mut input);
    assert!(result.is_ok(), "simple if/else should parse: {:?}", result.err());
}

/// TASK-1560: if with match in then branch
#[test]
fn test_if_then_match_parses() {
    let mut input = test_input("{ if n == 0 then match list { Nil => [] } else [] }");
    let result = parse_fn_body(&mut input);
    assert!(result.is_ok(), "if with match in then branch should parse: {:?}", result.err());
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
fn test_parse_inline_module_with_capability() {
    let mut input = test_input("mod foo { capability approve: decide() where requires_mfa(); }");
    let result = parse_module_decl(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );

    let decl = result.unwrap();
    assert_eq!(decl.name.as_ref(), "foo");
    assert!(decl.is_inline());

    let definitions = decl
        .definitions()
        .expect("inline module should expose parsed definitions");

    assert_eq!(definitions.len(), 1);

    let Definition::Capability(capability) = &definitions[0] else {
        panic!("expected first definition to be a capability: {definitions:?}");
    };

    assert_eq!(capability.name.as_ref(), "approve");
    assert_eq!(capability.effect, EffectType::Decide);
    assert!(matches!(
        &capability.constraints[..],
        [Constraint {
            predicate: Predicate { name, args }
        }] if name.as_ref() == "requires_mfa" && args.is_empty()
    ));
}

#[test]
fn test_parse_inline_module_with_capability_constraint_arguments() {
    let mut input =
        test_input("mod foo { capability approve: decide() where requires_region(\"EU\"); }");
    let result = parse_module_decl(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );

    let decl = result.unwrap();
    let definitions = decl
        .definitions()
        .expect("inline module should expose parsed definitions");

    assert_eq!(definitions.len(), 1);

    let Definition::Capability(capability) = &definitions[0] else {
        panic!("expected first definition to be a capability: {definitions:?}");
    };

    assert!(matches!(
        &capability.constraints[..],
        [Constraint {
            predicate: Predicate { name, args }
        }] if name.as_ref() == "requires_region"
            && matches!(&args[..], [Expr::Literal(Literal::String(region))] if region.as_ref() == "EU")
    ));
}

#[test]
fn test_parse_inline_module_preserves_capability_signature_metadata() {
    let mut input = test_input(
        "mod foo { capability approve: decide(user: User, scopes: [Scope]) returns Bool where requires_mfa(); }",
    );
    let result = parse_module_decl(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );

    let decl = result.unwrap();
    let definitions = decl
        .definitions()
        .expect("inline module should expose parsed definitions");

    let Definition::Capability(capability) = &definitions[0] else {
        panic!("expected first definition to be a capability: {definitions:?}");
    };

    assert_eq!(capability.params.len(), 2);
    assert!(matches!(
        &capability.params[..],
        [
            Param { name: user_name, ty: Type::Name(user_type) },
            Param { name: scopes_name, ty: Type::List(inner) }
        ] if user_name.as_ref() == "user"
            && user_type.as_ref() == "User"
            && scopes_name.as_ref() == "scopes"
            && matches!(inner.as_ref(), Type::Name(scope_type) if scope_type.as_ref() == "Scope")
    ));
    assert!(matches!(
        capability.return_type.as_ref(),
        Some(Type::Name(name)) if name.as_ref() == "Bool"
    ));
}

#[test]
fn test_parse_inline_module_with_capability_returns_and_constraint_arguments() {
    let mut input = test_input(
        "mod foo { capability approve: decide() returns Bool where requires_region(\"EU\"); }",
    );
    let result = parse_module_decl(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );

    let decl = result.unwrap();
    let definitions = decl
        .definitions()
        .expect("inline module should expose parsed definitions");

    assert_eq!(definitions.len(), 1);

    let Definition::Capability(capability) = &definitions[0] else {
        panic!("expected first definition to be a capability: {definitions:?}");
    };

    assert!(matches!(
        &capability.constraints[..],
        [Constraint {
            predicate: Predicate { name, args }
        }] if name.as_ref() == "requires_region"
            && matches!(&args[..], [Expr::Literal(Literal::String(region))] if region.as_ref() == "EU")
    ));
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
fn test_parse_inline_module_with_role_definition() {
    let mut input = test_input(
        "mod governance { role reviewer { capabilities: [approve, review], obligations: [check_tests, audit_log] } }",
    );

    let result = parse_module_decl(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );

    let decl = result.unwrap();
    let definitions = decl
        .definitions()
        .expect("inline module should expose parsed definitions");

    assert_eq!(definitions.len(), 1);

    let Definition::Role(role) = &definitions[0] else {
        panic!("expected first definition to be a role: {definitions:?}");
    };

    assert_eq!(role.name.as_ref(), "reviewer");
    assert_eq!(role.capabilities.len(), 2);
    assert_eq!(role.capabilities[0].capability.as_ref(), "approve");
    assert_eq!(role.capabilities[1].capability.as_ref(), "review");
    assert_eq!(role.obligations.len(), 2);
    assert_eq!(role.obligations[0].as_ref(), "check_tests");
    assert_eq!(role.obligations[1].as_ref(), "audit_log");
}

#[test]
fn test_parse_inline_module_rejects_unsupported_inline_workflow_before_role() {
    let mut input = test_input(
        "mod governance { workflow main { done } role reviewer { capabilities: [approve] } }",
    );

    let result = parse_module_decl(&mut input);

    assert!(
        result.is_err(),
        "Expected parse to fail instead of silently skipping unsupported inline workflow items"
    );
}

#[test]
fn test_parse_inline_module_rejects_unsupported_inline_workflow_before_capability_and_role() {
    let mut input = test_input(
        "mod governance { workflow main { done } capability approve: decide() where requires_mfa(); role reviewer { capabilities: [approve] } }",
    );

    let result = parse_module_decl(&mut input);

    assert!(
        result.is_err(),
        "Expected parse to fail instead of silently skipping unsupported inline workflow items"
    );
}

#[test]
fn test_parse_inline_module_rejects_unsupported_workflow_after_unknown_item() {
    assert_inline_module_rejects_after_unknown_item(
        "workflow main { done } role reviewer { capabilities: [approve] }",
        "workflow",
    );
}

#[test]
fn test_parse_inline_module_rejects_unsupported_policy_after_unknown_item() {
    assert_inline_module_rejects_after_unknown_item(
        "policy approval: when true then permit role reviewer { capabilities: [approve] }",
        "policy",
    );
}

#[test]
fn test_parse_inline_module_rejects_unsupported_datatype_after_unknown_item() {
    assert_inline_module_rejects_after_unknown_item(
        "datatype review_state = Pending | Approved; role reviewer { capabilities: [approve] }",
        "datatype",
    );
}

#[test]
fn test_parse_inline_module_preserves_visibility_qualified_item_after_unknown_item() {
    let source = inline_module_with_unknown_item(
        "pub capability approve: decide() role reviewer { capabilities: [approve] }",
    );
    let mut input = test_input(&source);

    let result = parse_module_decl(&mut input);

    let decl = result.expect("visibility-qualified capability should parse after recovery");
    let definitions = decl
        .definitions()
        .expect("inline module definitions should be present");
    match &definitions[0] {
        Definition::Capability(capability) => {
            assert_eq!(capability.name.as_ref(), "approve");
            assert_eq!(capability.visibility, Visibility::Public);
        }
        other => panic!("expected capability definition, got {other:?}"),
    }
}

#[test]
fn test_parse_inline_module_rejects_unsupported_canonical_datatype_definition() {
    let mut input = test_input(
        "mod governance { datatype review_state = Pending | Approved; role reviewer { capabilities: [approve] } }",
    );

    let result = parse_module_decl(&mut input);

    assert!(
        result.is_err(),
        "Expected inline modules to reject unsupported canonical datatype definitions explicitly"
    );
}

#[test]
fn test_parse_inline_module_preserves_visibility_qualified_capabilities() {
    let mut input = test_input("mod governance { pub capability approve: decide() }");

    let result = parse_module_decl(&mut input);

    let decl = result.expect("visibility-qualified capability should parse");
    let definitions = decl
        .definitions()
        .expect("inline module definitions should be present");
    match &definitions[0] {
        Definition::Capability(capability) => {
            assert_eq!(capability.name.as_ref(), "approve");
            assert_eq!(capability.visibility, Visibility::Public);
        }
        other => panic!("expected capability definition, got {other:?}"),
    }
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
        test_input("mod foo {\n  -- comment before capability\n  capability approve: decide()\n}");

    let decl = parse_module_decl(&mut input).expect("inline module should parse");
    let definitions = decl
        .definitions()
        .expect("inline module should expose parsed definitions");

    let Definition::Capability(capability) = &definitions[0] else {
        panic!("expected first definition to be a capability: {definitions:?}");
    };

    assert_eq!(capability.span.line, 3);
    assert_eq!(capability.span.column, 3);
}

// =========================================================================
// TASK-674: act block expression parsing tests
// =========================================================================

#[test]
fn test_parse_act_block_simple_return() {
    let mut input = test_input("{ act { ret 42; } }");
    let result = parse_fn_body(&mut input);
    assert!(result.is_ok(), "parse failed: {:?}", result);
    let expr = result.unwrap();
    assert!(
        matches!(expr, Expr::Block { ref tail_expr, .. } if tail_expr.is_some()),
        "expected a block with a tail expression, got: {:?}",
        expr
    );
}

#[test]
fn test_parse_act_block_bind_and_return() {
    let mut input = test_input("{ act { x = 42; ret x; } }");
    let result = parse_fn_body(&mut input);
    assert!(result.is_ok(), "parse failed: {:?}", result);
}

#[test]
fn test_parse_act_block_nested_calls() {
    let mut input = test_input("{ act { result = read_file(path); ret result; } }");
    let result = parse_fn_body(&mut input);
    assert!(result.is_ok(), "parse failed: {:?}", result);
}

#[test]
fn test_parse_act_block_empty() {
    let mut input = test_input("{ act {} }");
    let result = parse_fn_body(&mut input);
    assert!(result.is_ok(), "parse failed: {:?}", result);
}
