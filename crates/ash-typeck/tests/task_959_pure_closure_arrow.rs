//! TASK-959 typechecker coverage for preferred pure closure arrow syntax.

use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::surface::Expr;
use ash_typeck::check_expr::check_expr;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::Type;

fn parse_expr_complete(source: &str) -> Expr {
    let mut input = new_input(source);
    let parsed = expr(&mut input).expect("expression should parse");
    assert!(
        input.input.trim().is_empty(),
        "parser left trailing input: {:?}",
        input.input
    );
    parsed
}

#[test]
fn pure_closure_arrow_typechecks_as_type_fn_in_pure_context() {
    let env = TypeEnv::with_builtin_types();
    let parsed = parse_expr_complete("|x: Int| -> x + 1");

    let result = check_expr(&env, &parsed);

    assert!(
        result.is_ok(),
        "pure closure arrow should typecheck, got {:?}",
        result.errors
    );
    assert_eq!(result.ty, Type::Fn(vec![Type::Int], Box::new(Type::Int)));
}

#[test]
fn pure_closure_arrow_in_ambient_profile_context_keeps_pure_boundary() {
    let mut env = TypeEnv::with_builtin_types();
    env.set_ambient_effect(ash_core::Effect::Operational);
    let parsed = parse_expr_complete("|x: Int| -> x + 1");

    let result = check_expr(&env, &parsed);

    assert!(
        result.is_ok(),
        "ambient profile closure boundary should typecheck, got {:?}",
        result.errors
    );
    assert_eq!(
        result.ty,
        Type::Fn(vec![Type::Int], Box::new(Type::Int)),
        "pure closure arrow must remain Pure-stratum even in ambient profile contexts"
    );
}
