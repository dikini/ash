//! TASK-747 regression tests for generalized do-block parser/typechecker boundaries.

use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::surface::Expr;
use ash_typeck::check_expr::check_expr;
use ash_typeck::error::ConstructorError;
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
fn do_block_typecheck_is_explicitly_unsupported_until_typed_elaboration() {
    let expr = parse_expr_source("do:Act { return 1 }");
    let result = check_expr(&TypeEnv::with_builtin_types(), &expr);

    assert!(
        result.errors.iter().any(|err| matches!(
            err,
            ConstructorError::UnsupportedExpression { kind, .. }
                if kind.contains("generalized do-block type checking")
        )),
        "expected explicit unsupported do-block typecheck error, got {:?}",
        result.errors
    );
}
