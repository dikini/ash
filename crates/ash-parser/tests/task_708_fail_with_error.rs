use ash_parser::input::new_input;
use ash_parser::lower::lower_expr;
use ash_parser::parse_expr::expr;
use ash_parser::parse_module::parse_fn_definition;
use ash_parser::surface::{Expr, Literal, Pattern};
use winnow::prelude::*;

fn parse_expr_source(source: &str) -> Expr {
    let mut input = new_input(source);
    let parsed = expr.parse_next(&mut input).expect("expression parses");
    assert!(
        input.input.is_empty(),
        "parser left trailing input: {:?}",
        input.input
    );
    parsed
}

fn parse_fn_definition_fails(source: &str) {
    let mut input = new_input(source);
    let result = parse_fn_definition(&mut input);
    assert!(
        result.is_err(),
        "expected fn definition to fail parsing: {source}"
    );
}

#[test]
fn parses_fail_expression_payload() {
    let parsed = parse_expr_source("fail \"boom\"");
    let Expr::Fail { payload, .. } = parsed else {
        panic!("expected fail expression");
    };
    assert!(matches!(*payload, Expr::Literal(Literal::String(ref s)) if s.as_ref() == "boom"));
}

#[test]
fn parses_with_error_wildcard_handler() {
    let parsed = parse_expr_source("with_error { fail \"boom\" } handle { _ => 1; }");
    let Expr::WithError { body, arms, .. } = parsed else {
        panic!("expected with_error expression");
    };
    assert!(matches!(*body, Expr::Block { .. }));
    assert_eq!(arms.len(), 1);
    assert!(matches!(arms[0].pattern, Pattern::Wildcard));
    assert!(matches!(*arms[0].body, Expr::Literal(Literal::Int(1))));
}

#[test]
fn parses_with_error_block_body_with_let_and_tail() {
    let parsed = parse_expr_source("with_error { let x = fail \"boom\"; x } handle { _ => 1; }");
    let Expr::WithError { body, .. } = parsed else {
        panic!("expected with_error expression");
    };
    let Expr::Block {
        statements,
        tail_expr,
        ..
    } = *body
    else {
        panic!("expected block body");
    };
    assert_eq!(statements.len(), 1);
    assert!(tail_expr.is_some());
}

#[test]
fn with_error_participates_in_pipe_precedence() {
    let parsed = parse_expr_source("with_error { fail \"boom\" } handle { _ => 1; } |> f");
    let Expr::Call { func, args, .. } = parsed else {
        panic!("expected pipe to desugar into call");
    };
    assert_eq!(func.as_ref(), "f");
    assert_eq!(args.len(), 1);
    assert!(matches!(args[0], Expr::WithError { .. }));
}

#[test]
fn fail_keyword_does_not_capture_identifier_prefixes() {
    for source in ["fail_count", "fail-count"] {
        let parsed = parse_expr_source(source);
        assert!(
            matches!(parsed, Expr::Variable { ref name, .. } if name.as_ref() == source),
            "expected {source:?} to parse as a variable, got {parsed:?}"
        );
    }
}

#[test]
fn with_error_keyword_does_not_capture_identifier_prefixes() {
    for source in ["with_error_handler", "with_error-handler"] {
        let parsed = parse_expr_source(source);
        assert!(
            matches!(parsed, Expr::Variable { ref name, .. } if name.as_ref() == source),
            "expected {source:?} to parse as a variable, got {parsed:?}"
        );
    }
}

#[test]
fn fail_and_with_error_are_reserved_exact_identifiers() {
    for source in [
        "{ let fail = 1; fail }",
        "{ let with_error = 1; with_error }",
    ] {
        let mut input = new_input(source);
        let parsed = expr.parse_next(&mut input);
        assert!(
            parsed.is_err(),
            "expected exact contextual keyword identifier to be rejected: {source}"
        );
    }

    parse_fn_definition_fails(r#"fn fail() -> Int { 1 }"#);
    parse_fn_definition_fails(r#"fn with_error() -> Int { 1 }"#);
}

#[test]
fn lowers_fail_and_with_error_to_core_carriers() {
    let parsed = parse_expr_source("with_error { fail \"boom\" } handle { _ => 1; }");
    let lowered = lower_expr(&parsed).expect("with_error lowers");
    let ash_core::Expr::WithError { body, arms } = lowered else {
        panic!("expected core with_error expression");
    };
    assert!(matches!(
        *body,
        ash_core::Expr::Fail { .. } | ash_core::Expr::Let { .. }
    ));
    assert_eq!(arms.len(), 1);
    assert!(matches!(arms[0].pattern, ash_core::Pattern::Wildcard));
}
