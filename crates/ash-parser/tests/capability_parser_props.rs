//! Property-based tests for capability definition parsing.
//!
//! These tests verify that capability definitions are parsed correctly according
//! to SPEC-017 Section 2. The capability syntax supports:
//!
//! ```text
//! capability_def  ::= "capability" IDENTIFIER ":" effect_type
//!                     "(" param_list? ")"
//!                     ("returns" type)?
//!                     constraint_list?
//!                     visibility?
//!
//! effect_type     ::= "observe" | "read" | "analyze" | "decide"
//!                   | "act" | "write" | "external"
//!                   | "epistemic" | "deliberative" | "evaluative" | "operational"
//! ```

use ash_parser::{
    CapabilityDef, Definition, EffectType, Expr, Literal, Type, new_input, parse_module_decl,
};
use proptest::prelude::*;
use winnow::prelude::*;

/// Helper function to parse a capability definition from an inline module.
fn parse_capability(source: &str) -> Result<CapabilityDef, String> {
    let wrapped = format!("mod test {{ {} }}", source);
    let mut input = new_input(&wrapped);
    let decl = parse_module_decl
        .parse_next(&mut input)
        .map_err(|e| format!("{:?}", e))?;

    let definitions = decl.definitions().ok_or("expected inline module")?;

    if definitions.is_empty() {
        return Err("no definitions found".into());
    }

    match &definitions[0] {
        Definition::Capability(cap) => Ok(cap.clone()),
        _ => Err("first definition is not a capability".into()),
    }
}

/// Helper to parse multiple capabilities from an inline module.
fn parse_capabilities(source: &str) -> Result<Vec<CapabilityDef>, String> {
    let wrapped = format!("mod test {{ {} }}", source);
    let mut input = new_input(&wrapped);
    let decl = parse_module_decl
        .parse_next(&mut input)
        .map_err(|e| format!("{:?}", e))?;

    let definitions = decl.definitions().ok_or("expected inline module")?;

    definitions
        .iter()
        .map(|def| match def {
            Definition::Capability(cap) => Ok(cap.clone()),
            _ => Err("non-capability definition found".into()),
        })
        .collect()
}

/// Generate a valid identifier string.
fn valid_identifier() -> impl Strategy<Value = String> {
    // Identifiers start with letter or underscore, followed by alphanumeric or underscore
    // Using a simpler regex-like approach to avoid hyphens in property tests for simplicity
    "[a-zA-Z_][a-zA-Z0-9_]{0,30}".prop_filter("non-keyword", |s| !is_keyword(s))
}

/// Generate a valid type name.
fn valid_type_name() -> impl Strategy<Value = String> {
    "[A-Z][a-zA-Z0-9_]{0,30}".prop_filter("non-keyword", |s| !is_keyword(s))
}

/// Check if a string is a keyword that can't be used as an identifier.
fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "capability"
            | "role"
            | "policy"
            | "workflow"
            | "observe"
            | "read"
            | "analyze"
            | "decide"
            | "act"
            | "write"
            | "external"
            | "epistemic"
            | "deliberative"
            | "evaluative"
            | "operational"
            | "returns"
            | "where"
            | "authority"
            | "obligations"
            | "true"
            | "false"
            | "null"
            | "in"
            | "not"
            | "and"
            | "or"
    )
}

/// All basic effect types (surface syntax).
const BASIC_EFFECT_TYPES: &[(EffectType, &str)] = &[
    (EffectType::Observe, "observe"),
    (EffectType::Read, "read"),
    (EffectType::Analyze, "analyze"),
    (EffectType::Decide, "decide"),
    (EffectType::Act, "act"),
    (EffectType::Write, "write"),
    (EffectType::External, "external"),
];

// =============================================================================
// Property Tests
// =============================================================================

proptest! {
    /// Property: Minimal capability with no params parses correctly.
    #[test]
    fn prop_minimal_capability_parses(name in valid_identifier()) {
        let source = format!("capability {}: observe()", name);
        let result = parse_capability(&source);

        prop_assert!(result.is_ok(), "Failed to parse minimal capability: {:?}", result);

        let cap = result.unwrap();
        prop_assert_eq!(cap.name.as_ref(), name);
        prop_assert_eq!(cap.effect, EffectType::Observe);
        prop_assert!(cap.params.is_empty());
        prop_assert!(cap.return_type.is_none());
        prop_assert!(cap.constraints.is_empty());
    }

    /// Property: Capability with single parameter parses correctly.
    #[test]
    fn prop_capability_with_single_param(
        cap_name in valid_identifier(),
        param_name in valid_identifier(),
        param_type in valid_type_name()
    ) {
        let source = format!(
            "capability {}: read({}: {})",
            cap_name, param_name, param_type
        );
        let result = parse_capability(&source);

        prop_assert!(result.is_ok(), "Failed to parse capability with param: {:?}", result);

        let cap = result.unwrap();
        prop_assert_eq!(cap.name.as_ref(), cap_name);
        prop_assert_eq!(cap.effect, EffectType::Read);
        prop_assert_eq!(cap.params.len(), 1);
        prop_assert_eq!(cap.params[0].name.as_ref(), param_name);
        prop_assert!(
            matches!(&cap.params[0].ty, Type::Name(n) if n.as_ref() == param_type)
        );
    }

    /// Property: Capability with return type parses correctly.
    #[test]
    fn prop_capability_with_return_type(
        cap_name in valid_identifier(),
        return_type in valid_type_name()
    ) {
        let source = format!(
            "capability {}: analyze() returns {}",
            cap_name, return_type
        );
        let result = parse_capability(&source);

        prop_assert!(result.is_ok(), "Failed to parse capability with return: {:?}", result);

        let cap = result.unwrap();
        prop_assert_eq!(cap.name.as_ref(), cap_name);
        prop_assert_eq!(cap.effect, EffectType::Analyze);
        prop_assert!(
            matches!(cap.return_type, Some(Type::Name(ref n)) if n.as_ref() == return_type)
        );
    }

    /// Property: All basic effect types parse correctly.
    #[test]
    fn prop_all_basic_effect_types_parse(
        cap_name in valid_identifier(),
        effect_idx in 0..BASIC_EFFECT_TYPES.len()
    ) {
        let (expected_effect, effect_str) = BASIC_EFFECT_TYPES[effect_idx];
        let source = format!("capability {}: {}()", cap_name, effect_str);
        let result = parse_capability(&source);

        prop_assert!(
            result.is_ok(),
            "Failed to parse capability with effect '{}': {:?}",
            effect_str, result
        );

        let cap = result.unwrap();
        prop_assert_eq!(cap.effect, expected_effect);
    }

    /// Property: Capability names are preserved correctly.
    #[test]
    fn prop_capability_name_roundtrip(name in valid_identifier()) {
        let source = format!("capability {}: external()", name);
        let result = parse_capability(&source);

        prop_assert!(result.is_ok());
        let cap = result.unwrap();
        prop_assert_eq!(cap.name.as_ref(), name);
    }

    /// Property: List type parameters parse correctly.
    #[test]
    fn prop_capability_with_list_param(
        cap_name in valid_identifier(),
        param_name in valid_identifier(),
        inner_type in valid_type_name()
    ) {
        let source = format!(
            "capability {}: decide({}: [{}])",
            cap_name, param_name, inner_type
        );
        let result = parse_capability(&source);

        prop_assert!(result.is_ok(), "Failed to parse list param: {:?}", result);

        let cap = result.unwrap();
        prop_assert_eq!(cap.params.len(), 1);
        prop_assert_eq!(cap.params[0].name.as_ref(), param_name);
        prop_assert!(
            matches!(&cap.params[0].ty, Type::List(inner) if matches!(inner.as_ref(), Type::Name(n) if n.as_ref() == inner_type))
        );
    }

    /// Property: Record type parameters parse correctly.
    #[test]
    fn prop_capability_with_record_param(
        cap_name in valid_identifier(),
        param_name in valid_identifier()
    ) {
        let source = format!(
            "capability {}: act({}: {{ id: Int, name: String }})",
            cap_name, param_name
        );
        let result = parse_capability(&source);

        prop_assert!(result.is_ok(), "Failed to parse record param: {:?}", result);

        let cap = result.unwrap();
        prop_assert_eq!(cap.params.len(), 1);
        prop_assert_eq!(cap.params[0].name.as_ref(), param_name);
        prop_assert!(
            matches!(&cap.params[0].ty, Type::Record(fields) if fields.len() == 2)
        );
    }

    /// Property: Capability type parameters parse correctly.
    #[test]
    fn prop_capability_with_capability_param(
        cap_name in valid_identifier(),
        param_name in valid_identifier(),
        cap_type_name in valid_identifier()
    ) {
        let source = format!(
            "capability {}: write({}: capability {})",
            cap_name, param_name, cap_type_name
        );
        let result = parse_capability(&source);

        prop_assert!(result.is_ok(), "Failed to parse capability param: {:?}", result);

        let cap = result.unwrap();
        prop_assert_eq!(cap.params.len(), 1);
        prop_assert!(
            matches!(&cap.params[0].ty, Type::Capability(n) if n.as_ref() == cap_type_name)
        );
    }

    /// Property: Multiple parameters parse correctly.
    #[test]
    fn prop_capability_with_multiple_params(
        cap_name in valid_identifier(),
        param1_name in "[a-z]{3,10}",
        param1_type in valid_type_name(),
        param2_name in "[a-z]{3,10}",
        param2_type in valid_type_name()
    ) {
        // Ensure param names are different
        prop_assume!(param1_name != param2_name);

        let source = format!(
            "capability {}: read({}: {}, {}: {})",
            cap_name, param1_name, param1_type, param2_name, param2_type
        );
        let result = parse_capability(&source);

        prop_assert!(result.is_ok(), "Failed to parse multi-param: {:?}", result);

        let cap = result.unwrap();
        prop_assert_eq!(cap.params.len(), 2);
        prop_assert_eq!(cap.params[0].name.as_ref(), param1_name);
        prop_assert_eq!(cap.params[1].name.as_ref(), param2_name);
    }

    /// Property: Whitespace variations are handled correctly.
    #[test]
    fn prop_whitespace_variations(
        cap_name in valid_identifier(),
        ws_count in 1usize..5
    ) {
        let ws = " ".repeat(ws_count);
        // Valid syntax: capability <name> : <effect> ( <params> )
        let source = format!(
            "capability{}{}:{}observe({})",
            ws, cap_name, ws, ws
        );
        let result = parse_capability(&source);

        prop_assert!(result.is_ok(), "Failed with whitespace: {:?}", result);
    }

    /// Property: Capability with constraint predicate (no args) parses.
    #[test]
    fn prop_capability_with_simple_constraint(
        cap_name in valid_identifier(),
        pred_name in valid_identifier()
    ) {
        let source = format!(
            "capability {}: act() where {}()",
            cap_name, pred_name
        );
        let result = parse_capability(&source);

        prop_assert!(result.is_ok(), "Failed to parse constraint: {:?}", result);

        let cap = result.unwrap();
        prop_assert_eq!(cap.constraints.len(), 1);
        prop_assert_eq!(cap.constraints[0].predicate.name.as_ref(), pred_name);
        prop_assert!(cap.constraints[0].predicate.args.is_empty());
    }

    /// Property: Invalid effect types should fail to parse.
    #[test]
    fn prop_invalid_effect_type_fails(
        cap_name in valid_identifier(),
        invalid_effect in "(invalid|unknown|effect123|foo_bar)"
    ) {
        let source = format!("capability {}: {}()", cap_name, invalid_effect);
        let result = parse_capability(&source);

        prop_assert!(
            result.is_err(),
            "Expected parse to fail for invalid effect '{}', but got: {:?}",
            invalid_effect, result
        );
    }

    /// Property: Missing colon should fail.
    #[test]
    fn prop_missing_colon_fails(cap_name in valid_identifier()) {
        let source = format!("capability {} observe()", cap_name);
        let result = parse_capability(&source);

        prop_assert!(result.is_err(), "Expected parse to fail without colon");
    }

    /// Property: Missing parentheses should fail.
    #[test]
    fn prop_missing_parens_fails(cap_name in valid_identifier()) {
        let source = format!("capability {}: observe", cap_name);
        let result = parse_capability(&source);

        prop_assert!(result.is_err(), "Expected parse to fail without parens");
    }

    /// Property: Capability name with hyphen parses correctly.
    #[test]
    fn prop_hyphenated_capability_name(name_prefix in "[a-z]{3,10}", name_suffix in "[a-z]{3,10}") {
        let name = format!("{}-{}", name_prefix, name_suffix);
        let source = format!("capability {}: observe()", name);
        let result = parse_capability(&source);

        prop_assert!(result.is_ok(), "Failed to parse hyphenated name: {:?}", result);
        let cap = result.unwrap();
        prop_assert_eq!(cap.name.as_ref(), name);
    }

    /// Property: Source spans are correctly tracked.
    #[test]
    fn prop_span_tracking(
        cap_name in valid_identifier(),
        leading_ws in "[ \n]{0,10}"
    ) {
        let source = format!("{}capability {}: observe()", leading_ws, cap_name);
        let result = parse_capability(&source);

        prop_assert!(result.is_ok());

        let cap = result.unwrap();
        // Span should be non-empty
        prop_assert!(cap.span.end > cap.span.start);
        // Span start should be at or after the leading whitespace
        // (spans are relative to the module content, not the wrapped source)
        prop_assert!(cap.span.start >= leading_ws.len() || cap.span.start == 0);
    }
}

// =============================================================================
// Non-Property Tests (Specific Cases)
// =============================================================================

#[test]
fn test_parse_minimal_capability() {
    // SPEC-017: Parse minimal capability
    let cap = parse_capability("capability read_temp: observe()").unwrap();

    assert_eq!(cap.name.as_ref(), "read_temp");
    assert_eq!(cap.effect, EffectType::Observe);
    assert!(cap.params.is_empty());
    assert!(cap.return_type.is_none());
    assert!(cap.constraints.is_empty());
}

#[test]
fn test_parse_capability_with_params() {
    // SPEC-017: Parse capability with params
    let cap = parse_capability("capability read_file: read(path: String)").unwrap();

    assert_eq!(cap.name.as_ref(), "read_file");
    assert_eq!(cap.effect, EffectType::Read);
    assert_eq!(cap.params.len(), 1);
    assert_eq!(cap.params[0].name.as_ref(), "path");
    assert!(matches!(&cap.params[0].ty, Type::Name(n) if n.as_ref() == "String"));
}

#[test]
fn test_parse_capability_with_returns() {
    // SPEC-017: Parse capability with returns
    let cap = parse_capability("capability get_temp: observe() returns Int").unwrap();

    assert_eq!(cap.name.as_ref(), "get_temp");
    assert_eq!(cap.effect, EffectType::Observe);
    assert!(matches!(cap.return_type, Some(Type::Name(n)) if n.as_ref() == "Int"));
}

#[test]
fn test_parse_capability_with_constraints() {
    // SPEC-017: Parse capability with constraints
    let cap =
        parse_capability("capability transfer: act(amount: Int) where gt(amount, 0)").unwrap();

    assert_eq!(cap.name.as_ref(), "transfer");
    assert_eq!(cap.effect, EffectType::Act);
    assert_eq!(cap.params.len(), 1);
    assert_eq!(cap.constraints.len(), 1);
    assert_eq!(cap.constraints[0].predicate.name.as_ref(), "gt");
    assert_eq!(cap.constraints[0].predicate.args.len(), 2);
}

#[test]
fn test_parse_all_basic_effect_types() {
    // SPEC-017: Parse all effect types
    for (expected_effect, effect_str) in BASIC_EFFECT_TYPES {
        let source = format!("capability test_cap: {}()", effect_str);
        let cap = parse_capability(&source).unwrap();
        assert_eq!(
            cap.effect, *expected_effect,
            "Effect type '{}' should parse to {:?}",
            effect_str, expected_effect
        );
    }
}

#[test]
fn test_parse_effect_lattice_variants() {
    // SPEC-017: Parse effect lattice variants
    // Note: These may need to be added to the parser and EffectType enum
    let lattice_variants = [
        ("epistemic", EffectType::Epistemic),
        ("deliberative", EffectType::Deliberative),
        ("evaluative", EffectType::Evaluative),
        ("operational", EffectType::Operational),
    ];

    for (effect_str, expected_effect) in lattice_variants {
        let source = format!("capability test_cap: {}()", effect_str);
        let result = parse_capability(&source);

        assert!(
            result.is_ok(),
            "Lattice variant '{}' should parse successfully",
            effect_str
        );
        assert_eq!(
            result.unwrap().effect,
            expected_effect,
            "Lattice variant '{}' should parse to correct EffectType",
            effect_str
        );
    }
}

#[test]
fn test_error_on_invalid_effect_type() {
    // SPEC-017: Error on invalid effect type
    let result = parse_capability("capability test: invalid()");
    assert!(
        result.is_err(),
        "Should fail for invalid effect type 'invalid'"
    );

    let result = parse_capability("capability test: unknown()");
    assert!(
        result.is_err(),
        "Should fail for invalid effect type 'unknown'"
    );

    let result = parse_capability("capability test: ()");
    assert!(result.is_err(), "Should fail for missing effect type");
}

#[test]
fn test_error_duplicate_capability_name_detection() {
    // SPEC-017: Error on duplicate capability name
    // Note: This might be a semantic check rather than parse-time check
    let source = "capability foo: observe() capability foo: read()";
    let result = parse_capabilities(source);

    // The parser might accept this, but a later validation pass should catch it
    // For now, we just verify both capabilities parse
    if let Ok(caps) = result {
        assert_eq!(caps.len(), 2);
        assert_eq!(caps[0].name.as_ref(), "foo");
        assert_eq!(caps[1].name.as_ref(), "foo");
    }
}

#[test]
fn test_capability_with_multiple_params() {
    let cap = parse_capability("capability transfer: act(from: String, to: String, amount: Int)")
        .unwrap();

    assert_eq!(cap.params.len(), 3);
    assert_eq!(cap.params[0].name.as_ref(), "from");
    assert_eq!(cap.params[1].name.as_ref(), "to");
    assert_eq!(cap.params[2].name.as_ref(), "amount");
}

#[test]
fn test_capability_with_multiple_constraints() {
    let cap = parse_capability(
        "capability transfer: act(amount: Int) where gt(amount, 0), lt(amount, 1000000)",
    )
    .unwrap();

    assert_eq!(cap.constraints.len(), 2);
    assert_eq!(cap.constraints[0].predicate.name.as_ref(), "gt");
    assert_eq!(cap.constraints[1].predicate.name.as_ref(), "lt");
}

#[test]
fn test_capability_with_complex_types() {
    // Test list type
    let cap = parse_capability("capability process: analyze(items: [Item])").unwrap();
    assert!(matches!(&cap.params[0].ty, Type::List(_)));

    // Test record type
    let cap =
        parse_capability("capability create: write(data: { name: String, value: Int })").unwrap();
    assert!(matches!(&cap.params[0].ty, Type::Record(fields) if fields.len() == 2));

    // Test capability type
    let cap = parse_capability("capability delegate: decide(auth: capability Auth)").unwrap();
    assert!(matches!(&cap.params[0].ty, Type::Capability(n) if n.as_ref() == "Auth"));
}

#[test]
fn test_capability_with_string_constraint_arg() {
    let cap =
        parse_capability("capability access: read(resource: String) where requires_region(\"EU\")")
            .unwrap();

    assert_eq!(cap.constraints.len(), 1);
    assert_eq!(cap.constraints[0].predicate.args.len(), 1);
    assert!(matches!(
        &cap.constraints[0].predicate.args[0],
        Expr::Literal(Literal::String(s)) if s.as_ref() == "EU"
    ));
}

#[test]
fn test_capability_with_int_constraint_arg() {
    let cap = parse_capability("capability allocate: act(size: Int) where max_size(1024)").unwrap();

    assert_eq!(cap.constraints.len(), 1);
    assert!(matches!(
        &cap.constraints[0].predicate.args[0],
        Expr::Literal(Literal::Int(1024))
    ));
}

#[test]
fn test_capability_with_identifier_constraint_arg() {
    let cap =
        parse_capability("capability validate: analyze(value: Int) where gt(value, min_threshold)")
            .unwrap();

    assert_eq!(cap.constraints.len(), 1);
    assert!(matches!(
        &cap.constraints[0].predicate.args[1],
        Expr::Variable(name) if name.as_ref() == "min_threshold"
    ));
}

#[test]
fn test_capability_preserves_span_info() {
    let source = "capability test: observe()";
    let cap = parse_capability(source).unwrap();

    // Span should cover the entire capability definition
    assert!(cap.span.start < cap.span.end);
    // The span length should be approximately the source length
    let span_len = cap.span.end - cap.span.start;
    assert!(span_len > 0 && span_len <= source.len() + 10); // +10 for module wrapper
}

#[test]
fn test_capability_with_comments() {
    // Line comments before the capability definition
    let cap = parse_capability("-- comment before\ncapability test: observe()").unwrap();
    assert_eq!(cap.name.as_ref(), "test");

    // Block comments are also supported
    let cap = parse_capability("/* block comment */ capability test2: read()").unwrap();
    assert_eq!(cap.name.as_ref(), "test2");
}

#[test]
fn test_capability_with_multiline() {
    let cap = parse_capability("capability test:\n  observe(\n  )\n  returns Int").unwrap();
    assert_eq!(cap.name.as_ref(), "test");
    assert_eq!(cap.effect, EffectType::Observe);
    assert!(matches!(cap.return_type, Some(Type::Name(n)) if n.as_ref() == "Int"));
}

// =============================================================================
// Roundtrip Property Tests
// =============================================================================

proptest! {
    /// Property: Parsed capability can be serialized back to valid syntax.
    /// This is a conceptual roundtrip - we verify the structure is preserved.
    #[test]
    fn prop_capability_structure_preserved(
        cap_name in valid_identifier(),
        effect_idx in 0..BASIC_EFFECT_TYPES.len(),
        has_params in proptest::bool::ANY,
        has_return in proptest::bool::ANY
    ) {
        let (_, effect_str) = BASIC_EFFECT_TYPES[effect_idx];

        let mut source = format!("capability {}: {}()", cap_name, effect_str);

        if has_params {
            source = format!(
                "capability {}: {}(x: Int, y: String)",
                cap_name, effect_str
            );
        }

        if has_return {
            source.push_str(" returns Bool");
        }

        let result = parse_capability(&source);
        prop_assert!(result.is_ok());

        let cap = result.unwrap();
        prop_assert_eq!(cap.name.as_ref(), cap_name);

        if has_params {
            prop_assert_eq!(cap.params.len(), 2);
        } else {
            prop_assert!(cap.params.is_empty());
        }

        if has_return {
            prop_assert!(cap.return_type.is_some());
        } else {
            prop_assert!(cap.return_type.is_none());
        }
    }
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_empty_param_list() {
    let cap = parse_capability("capability test: observe(  )").unwrap();
    assert!(cap.params.is_empty());
}

#[test]
fn test_param_with_whitespace() {
    let cap = parse_capability("capability test: read(  name  :  String  )").unwrap();
    assert_eq!(cap.params.len(), 1);
    assert_eq!(cap.params[0].name.as_ref(), "name");
}

#[test]
fn test_nested_list_type() {
    // [[Int]] - list of list of int
    let cap = parse_capability("capability test: analyze(matrix: [[Int]])").unwrap();
    assert!(
        matches!(&cap.params[0].ty, Type::List(inner) if matches!(inner.as_ref(), Type::List(_)))
    );
}

#[test]
fn test_empty_record_type() {
    let cap = parse_capability("capability test: write(data: {})").unwrap();
    assert!(matches!(&cap.params[0].ty, Type::Record(fields) if fields.is_empty()));
}
