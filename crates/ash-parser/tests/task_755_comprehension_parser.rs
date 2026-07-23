use ash_parser::input::new_input;
use ash_parser::lower::{LoweringError, lower_expr};
use ash_parser::parse_expr::expr;
use ash_parser::surface::{ComprehensionQualifier, Expr};

fn parse_expr(src: &str) -> Expr {
    let mut input = new_input(src);
    let parsed = expr(&mut input).expect("expression should parse");
    assert_eq!(*input.input.as_ref(), "", "parser left trailing input");
    parsed
}

fn try_parse_expr(src: &str) -> Result<Expr, String> {
    let mut input = new_input(src);
    expr(&mut input)
        .map_err(|err| format!("{err:?}"))
        .and_then(|parsed| {
            if input.input.as_ref().is_empty() {
                Ok(parsed)
            } else {
                Err(format!("leftover input: {:?}", input.input.as_ref()))
            }
        })
}

#[test]
fn parses_explicit_target_comprehension() {
    let parsed = parse_expr("[f(x) | x <- xs]: List");

    let Expr::Comprehension {
        result,
        qualifiers,
        target,
        span,
    } = parsed
    else {
        panic!("expected comprehension expression");
    };

    assert!(span.start < span.end);
    match result.as_ref() {
        Expr::Call { func, args, .. } => {
            assert_eq!(func.as_ref(), "f");
            assert_eq!(args.len(), 1);
        }
        other => panic!("expected call result expression, got {other:?}"),
    }
    assert_eq!(qualifiers.len(), 1);
    match &qualifiers[0] {
        ComprehensionQualifier::Bind { name, value, span } => {
            assert_eq!(name.as_ref(), "x");
            assert!(span.start < span.end);
            assert!(matches!(value.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "xs"));
        }
        other => panic!("expected bind qualifier, got {other:?}"),
    }
    let target = target.expect("target should be parsed");
    assert_eq!(target.name.as_ref(), "List");
    assert!(target.args.is_empty());
}

#[test]
fn parses_explicit_process_target_comprehension() {
    let parsed = parse_expr("[x | x <- proc_value]: process");

    let Expr::Comprehension {
        result,
        qualifiers,
        target,
        span,
    } = parsed
    else {
        panic!("expected comprehension expression");
    };

    assert!(span.start < span.end);
    assert!(matches!(result.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "x"));
    assert_eq!(qualifiers.len(), 1);
    match &qualifiers[0] {
        ComprehensionQualifier::Bind { name, value, span } => {
            assert_eq!(name.as_ref(), "x");
            assert!(span.start < span.end);
            assert!(
                matches!(value.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "proc_value")
            );
        }
        other => panic!("expected bind qualifier, got {other:?}"),
    }
    let target = target.expect("target should be parsed");
    assert_eq!(target.name.as_ref(), "process");
    assert!(target.args.is_empty());
}

#[test]
fn rejects_removed_proc_target_comprehension() {
    assert!(try_parse_expr("[x | x <- proc_value]: Proc").is_err());
}

#[test]
fn parses_multiple_mixed_qualifiers_and_target_args() {
    let parsed = parse_expr(
        "[result | raw <- read(path), let parsed = parse(raw), _ <- guard(parsed)]: Result<ParseError>",
    );

    let Expr::Comprehension {
        result,
        qualifiers,
        target,
        ..
    } = parsed
    else {
        panic!("expected comprehension expression");
    };

    assert!(matches!(result.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "result"));
    assert_eq!(qualifiers.len(), 3);
    assert!(
        matches!(&qualifiers[0], ComprehensionQualifier::Bind { name, .. } if name.as_ref() == "raw")
    );
    assert!(
        matches!(&qualifiers[1], ComprehensionQualifier::Let { name, .. } if name.as_ref() == "parsed")
    );
    assert!(matches!(
        &qualifiers[2],
        ComprehensionQualifier::DiscardBind { .. }
    ));
    let target = target.expect("target should be parsed");
    assert_eq!(target.name.as_ref(), "Result");
    assert_eq!(target.args.len(), 1);
}

#[test]
fn parses_only_bare_underscore_as_discard_bind() {
    let parsed = parse_expr("[_x | _x <- xs]: List");

    let Expr::Comprehension { qualifiers, .. } = parsed else {
        panic!("expected comprehension expression");
    };

    assert_eq!(qualifiers.len(), 1);
    assert!(
        matches!(&qualifiers[0], ComprehensionQualifier::Bind { name, .. } if name.as_ref() == "_x")
    );
}

#[test]
fn parses_unannotated_comprehension_with_no_target() {
    let parsed = parse_expr("[x | x <- xs]");

    let Expr::Comprehension {
        qualifiers, target, ..
    } = parsed
    else {
        panic!("expected comprehension expression");
    };

    assert_eq!(qualifiers.len(), 1);
    assert!(target.is_none());
}

#[test]
fn rejects_empty_qualifier_list_and_trailing_separator() {
    assert!(try_parse_expr("[x | ]: List").is_err());
    assert!(try_parse_expr("[x | x <- xs, ]: List").is_err());
}

#[test]
fn rejects_bare_boolean_qualifier_shape() {
    assert!(try_parse_expr("[x | x <- xs, x > 0]: List").is_err());
}

#[test]
fn rejects_malformed_target_annotation() {
    assert!(try_parse_expr("[x | x <- xs]:").is_err());
}

#[test]
fn preserves_list_literal_and_index_access_parsing() {
    let list = parse_expr("[1, 2, 3]");
    assert!(matches!(list, Expr::List { items, .. } if items.len() == 3));

    let index = parse_expr("xs[0]");
    assert!(matches!(index, Expr::IndexAccess { .. }));
}

#[test]
fn malformed_comprehension_attempt_does_not_corrupt_subsequent_parse() {
    assert!(try_parse_expr("[x | ]: List").is_err());

    let parsed = parse_expr("[1, 2]");
    assert!(matches!(parsed, Expr::List { items, .. } if items.len() == 2));
}

#[test]
fn parser_only_lowering_rejects_comprehension() {
    let parsed = parse_expr("[x | x <- xs]: List");
    let err = lower_expr(&parsed).expect_err("comprehension lowering must be deferred");

    assert!(matches!(
        err,
        LoweringError::ExprNotLowerable { kind }
            if kind.contains("comprehension requires typed do elaboration")
    ));
}
