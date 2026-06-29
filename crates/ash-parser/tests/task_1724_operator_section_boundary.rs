use ash_parser::parse_expr::expr;
use ash_parser::surface::{Expr, OperatorSectionKind};

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
fn parses_bare_binary_operator_section_without_erasing_shape() {
    let parsed = parse_expr_complete("(+)");
    match parsed {
        Expr::OperatorSection { section } => {
            assert_eq!(section.kind, OperatorSectionKind::Bare);
            assert_eq!(section.operator.spelling.as_ref(), "+");
            assert!(section.left.is_none());
            assert!(section.right.is_none());
        }
        other => panic!("expected operator section, got {other:?}"),
    }
}

#[test]
fn parses_left_binary_operator_section_without_desugaring_to_call() {
    let parsed = parse_expr_complete("(value +)");
    match parsed {
        Expr::OperatorSection { section } => {
            assert_eq!(section.kind, OperatorSectionKind::Left);
            assert_eq!(section.operator.spelling.as_ref(), "+");
            assert!(
                matches!(section.left.as_deref(), Some(Expr::Variable { name, .. }) if name.as_ref() == "value")
            );
            assert!(section.right.is_none());
        }
        other => panic!("expected operator section, got {other:?}"),
    }
}

#[test]
fn parses_right_binary_operator_section_without_desugaring_to_call() {
    let parsed = parse_expr_complete("(+ value)");
    match parsed {
        Expr::OperatorSection { section } => {
            assert_eq!(section.kind, OperatorSectionKind::Right);
            assert_eq!(section.operator.spelling.as_ref(), "+");
            assert!(section.left.is_none());
            assert!(
                matches!(section.right.as_deref(), Some(Expr::Variable { name, .. }) if name.as_ref() == "value")
            );
        }
        other => panic!("expected operator section, got {other:?}"),
    }
}

#[test]
fn ordinary_parenthesized_binary_expression_stays_binary_expression() {
    let parsed = parse_expr_complete("(left + right)");
    assert!(
        matches!(parsed, Expr::Binary { .. }),
        "expected binary expression, got {parsed:?}"
    );
}

#[test]
fn generalized_mixfix_section_remains_fail_closed() {
    let mut input = ash_parser::input::new_input("(_ + _)");
    assert!(
        expr(&mut input).is_err(),
        "generalized mixfix section must not parse yet"
    );
}
