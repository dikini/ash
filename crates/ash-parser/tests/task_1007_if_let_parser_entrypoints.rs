use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::surface::{Definition, Expr, Pattern};

fn parse_expr_complete(source: &str) -> Expr {
    let mut input = new_input(source);
    let parsed = expr(&mut input)
        .unwrap_or_else(|error| panic!("expression should parse: {source}\nerror: {error:?}"));
    assert_eq!(*input.input.as_ref(), "", "parser left trailing input");
    parsed
}

#[test]
fn if_let_parser_entrypoints_accept_raw_expression_and_real_function_context_or_pin_rejection() {
    let raw = parse_expr_complete("if let x = value then { x } else { 0 }");
    let Expr::IfLet { pattern, .. } = raw else {
        panic!("raw expression parser should produce Expr::IfLet, got {raw:?}");
    };
    assert!(matches!(
        pattern,
        Pattern::Variable { ref name, .. } if name.as_ref() == "x"
    ));

    let module = ash_parser::parse_surface_file(
        r#"
        fn unwrap_or_zero(value: Int) -> Int {
            if let x = value then { x } else { 0 }
        }
        "#,
    )
    .expect("if let should parse in a real function body context");

    let function = module
        .definitions
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) => Some(function),
            _ => None,
        })
        .expect("function should parse");
    assert!(matches!(function.body, Expr::Block { .. }));
}

#[test]
fn if_let_without_else_rejected() {
    let mut input = new_input("if let x = value then { x }");
    let result = expr(&mut input);
    assert!(
        result.is_err() || !input.input.as_ref().trim().is_empty(),
        "if let without else must not parse as a complete expression"
    );

    assert!(
        ash_parser::parse_surface_file(
            r#"
            fn bad(maybe: Option<Int>) -> Int {
                if let Some { value: x } = maybe then { x }
            }
            "#,
        )
        .is_err(),
        "if let without else must be rejected in function context"
    );
}
