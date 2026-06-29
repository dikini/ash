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
fn operator_section_preserves_raw_operator_token_and_span() {
    let parsed = parse_expr_complete("(+ value)");
    match parsed {
        Expr::OperatorSection { section } => {
            assert_eq!(section.kind, OperatorSectionKind::Right);
            assert_eq!(section.operator.spelling.as_ref(), "+");
            assert!(section.operator.span.start < section.operator.span.end);
            assert!(section.span.start <= section.operator.span.start);
            assert!(section.operator.span.end <= section.span.end);
        }
        other => panic!("expected operator section, got {other:?}"),
    }
}

#[test]
fn existing_builtin_infix_shape_still_parses_as_binary_until_notation_resolution_exists() {
    let parsed = parse_expr_complete("left + right");
    match parsed {
        Expr::Binary { left, right, .. } => {
            assert!(
                matches!(left.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "left")
            );
            assert!(
                matches!(right.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "right")
            );
        }
        other => panic!("expected existing binary expression, got {other:?}"),
    }
}

#[test]
fn parenthesized_underscore_identifier_still_parses_as_ordinary_expression() {
    let parsed = parse_expr_complete("(_foo)");
    assert!(
        matches!(parsed, Expr::Variable { name, .. } if name.as_ref() == "_foo"),
        "underscore-leading identifiers are ordinary names, not operator-section holes"
    );
}

#[test]
fn underscore_identifier_left_section_is_not_confused_with_placeholder_section() {
    let parsed = parse_expr_complete("(_foo +)");
    match parsed {
        Expr::OperatorSection { section } => {
            assert_eq!(section.kind, OperatorSectionKind::Left);
            assert_eq!(section.operator.spelling.as_ref(), "+");
            assert!(
                matches!(section.left.as_deref(), Some(Expr::Variable { name, .. }) if name.as_ref() == "_foo")
            );
        }
        other => panic!("expected left operator section, got {other:?}"),
    }
}

#[test]
fn unsupported_symbolic_notation_fails_before_silent_erasure() {
    let mut input = ash_parser::input::new_input("left ⊕ right");
    let parsed = expr(&mut input).expect("left variable should parse before unsupported token");
    assert!(matches!(parsed, Expr::Variable { .. }));
    assert!(
        !input.input.trim_start().is_empty(),
        "unsupported notation token must not be silently consumed"
    );
}
