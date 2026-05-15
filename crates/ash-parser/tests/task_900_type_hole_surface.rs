use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::surface::{Definition, Expr, Type, TypePattern};
use winnow::Parser;

fn parse_expr_complete(src: &str) -> Expr {
    let mut input = new_input(src);
    let parsed = expr(&mut input).expect("expression should parse");
    assert_eq!(*input.input.as_ref(), "", "parser left trailing input");
    parsed
}

#[test]
fn parses_do_target_type_argument_hole_with_distinct_span() {
    let src = "do:Result<_, ParseError> { return value }";
    let Expr::DoBlock { target, .. } = parse_expr_complete(src) else {
        panic!("expected generalized do block");
    };

    assert_eq!(target.name.as_ref(), "Result");
    assert_eq!(target.args.len(), 2);

    match &target.args[0] {
        Type::Hole { span } => {
            assert!(span.end > span.start, "hole span should be non-empty");
            assert_eq!(&src[span.start..span.end], "_");
        }
        other => panic!("expected first do target argument to be a type hole, got {other:?}"),
    }

    assert!(matches!(
        &target.args[1],
        Type::Name(name) if name.as_ref() == "ParseError"
    ));
}

#[test]
fn keeps_type_function_pattern_underscore_as_wildcard_not_type_hole() {
    let module = ash_parser::parse_surface_file("type fn F(x: Type) -> Type { case F<_> = Int; }")
        .expect("type function should parse");
    let type_fn = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::TypeFn(definition) => Some(definition),
            _ => None,
        })
        .expect("type function definition should be present");

    assert!(matches!(
        &type_fn.equations[0].patterns[0],
        TypePattern::Wildcard { span } if span.end > span.start
    ));
    assert_eq!(type_fn.equations[0].result, Type::Name("Int".into()));
}

#[test]
fn rejects_type_holes_in_ordinary_workflow_return_types() {
    let mut input = new_input("workflow f() -> Result<_, E> { done }");
    assert!(
        ash_parser::parse_workflow::workflow_def
            .parse_next(&mut input)
            .is_err()
            || !input.input.as_ref().is_empty(),
        "ordinary workflow return types must stay fail-closed for holes"
    );
}

#[test]
fn rejects_type_holes_in_ordinary_type_aliases() {
    assert!(
        ash_parser::parse_surface_file("type Alias = Result<_, E>;").is_err(),
        "ordinary type alias parser must stay fail-closed for holes"
    );
}
