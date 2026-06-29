use ash_parser::lower::lower_expr;
use ash_parser::parse_expr::expr;
use ash_parser::surface::{BinaryOp, Expr};

fn parse_expr_complete(source: &str) -> Expr {
    let mut input = ash_parser::input::new_input(source);
    let parsed = expr(&mut input).expect("expression should parse");
    ash_parser::parse_utils::skip_whitespace_and_comments(&mut input);
    assert!(
        input.input.is_empty(),
        "unconsumed input: {:?}",
        input.input
    );
    parsed
}

#[test]
fn builtin_binary_expression_preserves_raw_operator_token() {
    let parsed = parse_expr_complete("left + right");
    match parsed {
        Expr::Binary {
            op,
            raw_operator: Some(raw),
            ..
        } => {
            assert_eq!(op, BinaryOp::Add);
            assert_eq!(raw.spelling.as_ref(), "+");
            assert!(raw.span.start < raw.span.end);
        }
        other => panic!("expected binary expression with raw operator, got {other:?}"),
    }
}

#[test]
fn comparison_expression_preserves_source_spelling_and_lowers_unchanged() {
    let parsed = parse_expr_complete("left <= right");
    match &parsed {
        Expr::Binary {
            op,
            raw_operator: Some(raw),
            ..
        } => {
            assert_eq!(*op, BinaryOp::Leq);
            assert_eq!(raw.spelling.as_ref(), "<=");
        }
        other => panic!("expected comparison with raw operator, got {other:?}"),
    }
    let lowered = lower_expr(&parsed).expect("semantic binary expression still lowers");
    assert!(matches!(lowered, ash_core::Expr::Binary { .. }));
}

#[test]
fn builtin_binary_parser_does_not_steal_prefix_of_longer_symbolic_operator() {
    let mut input = ash_parser::input::new_input("left ++ right");
    let parsed = expr(&mut input).expect("left variable parses");
    assert!(matches!(parsed, Expr::Variable { ref name, .. } if name.as_ref() == "left"));
    ash_parser::parse_utils::skip_whitespace_and_comments(&mut input);
    assert!(
        input.input.as_ref().starts_with("++"),
        "custom symbolic operator prefix should remain unconsumed, got {:?}",
        input.input
    );
}
