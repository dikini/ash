use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::surface::{Expr, Literal, MatchArm, Pattern, VariantPatternPayload};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::Type;
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

fn fail_expr() -> Expr {
    Expr::Fail {
        payload: Box::new(Expr::Literal(Literal::String("boom".into()))),
        span: Span::default(),
    }
}

#[test]
fn fail_typechecks_as_bottom_compatible_value() {
    let env = TypeEnv::with_builtin_types();
    let result = check_expr(&env, &parse_expr_source("fail \"boom\""));
    assert!(result.is_ok(), "fail should typecheck: {:?}", result.errors);
    assert!(
        matches!(result.ty, Type::Var(_)),
        "fail should infer bottom-compatible fresh type, got {}",
        result.ty
    );
}

#[test]
fn fail_branch_unifies_with_other_branch() {
    let env = TypeEnv::with_builtin_types();
    let expr = Expr::If {
        condition: Box::new(Expr::Literal(Literal::Bool(true))),
        then_branch: Box::new(Expr::Literal(Literal::Int(1))),
        else_branch: Some(Box::new(fail_expr())),
        span: Span::default(),
    };
    let result = check_expr(&env, &expr);
    assert!(
        result.is_ok(),
        "if/fail should typecheck: {:?}",
        result.errors
    );
    assert_eq!(result.substitution.apply(&result.ty), Type::Int);
}

#[test]
fn with_error_handler_arm_must_match_body_type() {
    let env = TypeEnv::with_builtin_types();
    let result = check_expr(
        &env,
        &parse_expr_source("with_error { 1 } handle { _ => \"wrong\"; }"),
    );
    assert!(!result.is_ok(), "mismatched handler type should fail");
}

#[test]
fn with_error_handler_fail_arm_is_bottom_compatible() {
    let env = TypeEnv::with_builtin_types();
    let result = check_expr(
        &env,
        &parse_expr_source("with_error { 1 } handle { _ => fail \"other\"; }"),
    );
    assert!(
        result.is_ok(),
        "fail handler should be bottom-compatible: {:?}",
        result.errors
    );
    assert_eq!(result.substitution.apply(&result.ty), Type::Int);
}

#[test]
fn with_error_handler_pattern_errors_are_reported() {
    let env = TypeEnv::with_builtin_types();
    let expr = Expr::WithError {
        body: Box::new(Expr::Literal(Literal::Int(1))),
        arms: vec![MatchArm {
            pattern: Pattern::Variant {
                name: "NotAThing".into(),
                fields: None,
                payload: VariantPatternPayload::Unit,
            },
            body: Box::new(Expr::Literal(Literal::Int(1))),
            span: Span::default(),
        }],
        span: Span::default(),
    };

    let result = check_expr(&env, &expr);
    assert!(
        !result.is_ok(),
        "unknown handler pattern variant should be rejected"
    );
    assert!(
        result
            .errors
            .iter()
            .any(|err| format!("{err}").contains("with_error handler pattern type error")),
        "expected handler pattern error, got {:?}",
        result.errors
    );
}
