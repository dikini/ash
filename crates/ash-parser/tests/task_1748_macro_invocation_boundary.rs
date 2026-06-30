use ash_parser::lower::{LoweringError, lower_expr};
use ash_parser::surface::{Expr, MacroDelimiter, expand_surface_module};

fn first_function_body(module: &ash_parser::surface::ModuleFile) -> &Expr {
    let ash_parser::surface::Definition::Function(def) = &module.definitions[0] else {
        panic!("expected function definition")
    };
    match &def.body {
        Expr::Block {
            tail_expr: Some(tail),
            ..
        } => tail.as_ref(),
        other => other,
    }
}

#[test]
fn macro_invocation_shape_is_parsed_and_preserved() {
    let module = ash_parser::parse_surface_file("fn use_macro() -> Int { make_id!(1, two) }")
        .expect("macro invocation carrier parses");

    let Expr::MacroInvocation { invocation } = first_function_body(&module) else {
        panic!("expected macro invocation carrier")
    };

    assert_eq!(invocation.name.as_ref(), "make_id");
    assert_eq!(invocation.delimiter, MacroDelimiter::Paren);
    assert_eq!(invocation.raw_body.as_ref(), "1, two");
    assert!(invocation.span.start < invocation.span.end);
}

#[test]
fn macro_invocation_fails_expanded_surface_boundary_before_core() {
    let module = ash_parser::parse_surface_file("fn use_macro() -> Int { make_id![1, two] }")
        .expect("macro invocation carrier parses");

    let err = expand_surface_module(module).expect_err("macro invocation must not cross expansion");

    assert!(
        err.to_string()
            .contains("unknown local macro invocation `make_id!`"),
        "unexpected error: {err}"
    );
}

#[test]
fn direct_core_lowering_rejects_macro_invocation_carrier() {
    let module = ash_parser::parse_surface_file("fn use_macro() -> Int { make_id!{x + y} }")
        .expect("macro invocation carrier parses");
    let expr = first_function_body(&module);

    let err = lower_expr(expr).expect_err("macro invocation must not lower to Core");

    assert!(matches!(err, LoweringError::UnsupportedFeature(message)
        if message.contains("unexpanded macro invocation carrier `make_id!` reached lowering")));
}

#[test]
fn ordinary_calls_and_unary_bang_still_parse_unchanged() {
    let call_module = ash_parser::parse_surface_file("fn call() -> Int { make_id(1) }")
        .expect("ordinary call parses");
    assert!(
        matches!(first_function_body(&call_module), Expr::Call { func, .. } if func.as_ref() == "make_id")
    );

    let bang_module = ash_parser::parse_surface_file("fn invert(x: Bool) -> Bool { !x }")
        .expect("unary bang parses");
    assert!(matches!(
        first_function_body(&bang_module),
        Expr::Unary { .. }
    ));
}

#[test]
fn qualified_macro_like_invocation_is_not_claimed_by_the_carrier() {
    let err = ash_parser::parse_surface_file("fn use_macro() -> Int { macros::make_id!(1) }")
        .expect_err("qualified macro paths are not part of the Phase 171 carrier");

    assert!(!err.is_empty(), "expected at least one parse error");
}
