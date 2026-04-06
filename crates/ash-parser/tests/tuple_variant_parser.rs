use ash_core::ast::{Expr as CoreExpr, Pattern as CorePattern};
use ash_parser::input::new_input;
use ash_parser::lower::{lower_expr, lower_pattern};
use ash_parser::parse_expr::expr;
use ash_parser::parse_pattern::pattern;
use ash_parser::parse_type_def::{TypeBody, VariantPayload, parse_type_def};
use ash_parser::surface::{ConstructorPayload, Expr, Literal, Pattern, VariantPatternPayload};
use winnow::Parser;

#[test]
fn parses_tuple_variant_declaration_shape() {
    let mut input = new_input("type RuntimeError = RuntimeError(Int, String);");
    let type_def = parse_type_def(&mut input).expect("tuple variant declaration should parse");

    match type_def.body {
        TypeBody::Enum(variants) => {
            assert_eq!(variants.len(), 1);
            assert_eq!(variants[0].name, "RuntimeError");
            assert!(matches!(
                &variants[0].payload,
                VariantPayload::Tuple(items)
                    if matches!(items.as_slice(), [
                        ash_parser::parse_type_def::TypeExpr::Named(first),
                        ash_parser::parse_type_def::TypeExpr::Named(second)
                    ] if first == "Int" && second == "String")
            ));
        }
        other => panic!("expected enum body, got {other:?}"),
    }
}

#[test]
fn parses_tuple_constructor_expression_shape() {
    let mut input = new_input("RuntimeError(2, \"missing config\")");
    let parsed = expr
        .parse_next(&mut input)
        .expect("tuple constructor should parse");

    match parsed {
        Expr::Constructor { name, payload, .. } => {
            assert_eq!(name.as_ref(), "RuntimeError");
            assert!(matches!(
                payload,
                ConstructorPayload::Tuple(items)
                    if matches!(items.as_slice(), [
                        Expr::Literal(Literal::Int(2)),
                        Expr::Literal(Literal::String(message))
                    ] if message.as_ref() == "missing config")
            ));
        }
        other => panic!("expected constructor expression, got {other:?}"),
    }
}

#[test]
fn parses_empty_tuple_constructor_expression_shape() {
    let mut input = new_input("Box()");
    let parsed = expr
        .parse_next(&mut input)
        .expect("empty tuple constructor should parse");

    match parsed {
        Expr::Constructor { name, payload, .. } => {
            assert_eq!(name.as_ref(), "Box");
            assert!(matches!(payload, ConstructorPayload::Tuple(items) if items.is_empty()));
        }
        other => panic!("expected constructor expression, got {other:?}"),
    }
}

#[test]
fn parses_tuple_variant_pattern_shape() {
    let mut input = new_input("RuntimeError(code, msg)");
    let parsed = pattern(&mut input).expect("tuple variant pattern should parse");

    match parsed {
        Pattern::Variant { name, payload, .. } => {
            assert_eq!(name.as_ref(), "RuntimeError");
            assert!(matches!(
                payload,
                VariantPatternPayload::Tuple(items)
                    if matches!(items.as_slice(), [
                        Pattern::Variable(code),
                        Pattern::Variable(msg)
                    ] if code.as_ref() == "code" && msg.as_ref() == "msg")
            ));
        }
        other => panic!("expected variant pattern, got {other:?}"),
    }
}

#[test]
fn parses_nested_tuple_variant_pattern_shape() {
    let mut input = new_input("RuntimeError(code, (line, column))");
    let parsed = pattern(&mut input).expect("nested tuple variant pattern should parse");

    match parsed {
        Pattern::Variant { name, payload, .. } => {
            assert_eq!(name.as_ref(), "RuntimeError");
            assert!(matches!(
                payload,
                VariantPatternPayload::Tuple(items)
                    if matches!(items.as_slice(), [
                        Pattern::Variable(code),
                        Pattern::Tuple(nested)
                    ] if code.as_ref() == "code"
                        && matches!(nested.as_slice(), [
                            Pattern::Variable(line),
                            Pattern::Variable(column)
                        ] if line.as_ref() == "line" && column.as_ref() == "column"))
            ));
        }
        other => panic!("expected nested variant pattern, got {other:?}"),
    }
}

#[test]
fn rejects_malformed_tuple_variant_declaration_payload() {
    let mut input = new_input("type RuntimeError = RuntimeError(code: Int, String);");
    let result = parse_type_def(&mut input);
    assert!(
        result.is_err(),
        "named fields inside tuple payload must be rejected"
    );
}

#[test]
fn rejects_malformed_tuple_constructor_payload() {
    let mut input = new_input("RuntimeError(code: 2, \"missing config\")");
    let result = expr.parse_next(&mut input);
    assert!(
        result.is_err(),
        "named fields inside tuple constructor payload must be rejected"
    );
}

#[test]
fn rejects_malformed_tuple_variant_pattern_payload() {
    let mut input = new_input("RuntimeError(code: x, msg)");
    let result = pattern(&mut input);
    assert!(
        result.is_err(),
        "named fields inside tuple variant pattern payload must be rejected"
    );
}

#[test]
fn lowers_tuple_constructor_with_stable_positional_fields() {
    let mut input = new_input("RuntimeError(2, \"missing config\")");
    let parsed = expr
        .parse_next(&mut input)
        .expect("tuple constructor should parse before lowering");

    let lowered = lower_expr(&parsed).expect("tuple constructor should lower");
    assert_eq!(
        lowered,
        CoreExpr::Constructor {
            name: "RuntimeError".into(),
            fields: vec![
                ("_0".into(), CoreExpr::Literal(ash_core::Value::Int(2))),
                (
                    "_1".into(),
                    CoreExpr::Literal(ash_core::Value::String("missing config".into())),
                ),
            ],
        }
    );
}

#[test]
fn lowers_tuple_variant_pattern_with_stable_positional_fields() {
    let mut input = new_input("RuntimeError(code, msg)");
    let parsed = pattern(&mut input).expect("tuple variant pattern should parse before lowering");

    let lowered = lower_pattern(&parsed).expect("tuple variant pattern should lower");
    assert_eq!(
        lowered,
        CorePattern::Variant {
            name: "RuntimeError".into(),
            fields: Some(vec![
                ("_0".into(), CorePattern::Variable("code".into())),
                ("_1".into(), CorePattern::Variable("msg".into())),
            ]),
        }
    );
}
