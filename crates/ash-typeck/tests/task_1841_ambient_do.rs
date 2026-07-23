//! TASK-1841 regression tests for target ambient `do { ... }` sequencing.

use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::surface::Expr;
use ash_typeck::Type;
use ash_typeck::check_expr::check_expr;
use ash_typeck::type_env::TypeEnv;
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

#[test]
fn target_ambient_do_typechecks_as_return_expression_type() {
    let expr = parse_expr_source("do { let x = 1; return x }");
    let checked = check_expr(&TypeEnv::with_builtin_types(), &expr);

    assert!(checked.is_ok(), "ambient do failed: {checked:?}");
    assert_eq!(checked.ty, Type::Int);
}

#[test]
fn target_ambient_bind_sequence_does_not_require_named_computation_constructor() {
    let expr = parse_expr_source("do { x <- 1; return x }");
    let checked = check_expr(&TypeEnv::with_builtin_types(), &expr);

    assert!(checked.is_ok(), "ambient bind failed: {checked:?}");
    assert_eq!(checked.ty, Type::Int);
}

#[test]
fn target_ambient_do_rejects_legacy_contract_statements() {
    assert!(
        expr.parse_next(&mut new_input("do { requires: true; return 1 }"))
            .is_err(),
        "legacy contract statements must be rejected before target type checking"
    );
}
