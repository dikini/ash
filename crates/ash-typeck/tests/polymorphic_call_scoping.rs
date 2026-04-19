//! TASK-636: Regression test for type-variable scoping at polymorphic call sites.
//!
//! Verifies that when a polymorphic builtin (e.g., `len` with type
//! `Fn([List<Var(42)>], Int)`) is called at two different sites with
//! different concrete types, the typechecker handles each call independently
//! without requiring type-variable freshening.
//!
//! Hypothesis (SPEC-044): freshening is NOT needed because:
//! - `instantiate_fn_call` creates a fresh `Substitution` per call
//! - Function types bound in `TypeEnv` are immutable (cloned on lookup)
//! - Each call's `CheckResult` has its own substitution scope

use ash_parser::surface::{Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::{Type, TypeVar};

fn span() -> Span {
    Span::default()
}

/// Build the polymorphic `len` type: `Fn([List<Var(42)>], Int)`
fn len_type() -> Type {
    Type::Fn(
        vec![Type::List(Box::new(Type::Var(TypeVar(42))))],
        Box::new(Type::Int),
    )
}

/// Build an env with `len` bound.
fn env_with_len() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("len", len_type());
    env
}

/// Helper: typecheck an expression, apply the substitution, and return the
/// fully-resolved type on success, or panic with the errors on failure.
fn infer(env: &TypeEnv, expr: &Expr) -> Type {
    let result = check_expr(env, expr);
    assert!(result.is_ok(), "typecheck failed: {:?}", result.errors);
    result.substitution.apply(&result.ty)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn polymorphic_len_with_int_list() {
    let env = env_with_len();

    // len([1, 2, 3])  -- element type should be inferred as Int
    let expr = Expr::Call {
        func: "len".into(),
        module: None,
        args: vec![Expr::Literal(Literal::List(vec![
            Literal::Int(1),
            Literal::Int(2),
            Literal::Int(3),
        ]))],
        span: span(),
    };

    let ty = infer(&env, &expr);
    assert_eq!(ty, Type::Int, "len([1,2,3]) should return Int");
}

#[test]
fn polymorphic_len_with_string_list() {
    let env = env_with_len();

    // len(["a", "b"])  -- element type should be inferred as String
    let expr = Expr::Call {
        func: "len".into(),
        module: None,
        args: vec![Expr::Literal(Literal::List(vec![
            Literal::String("a".into()),
            Literal::String("b".into()),
        ]))],
        span: span(),
    };

    let ty = infer(&env, &expr);
    assert_eq!(ty, Type::Int, "len([\"a\",\"b\"]) should return Int");
}

#[test]
fn polymorphic_len_sequential_calls_independent() {
    // This is the core regression test: calling `len` with Int elements
    // and then calling `len` with String elements must both succeed
    // without any cross-contamination of type-variable bindings.
    let env = env_with_len();

    // --- First call: len([1, 2, 3]) ---
    let expr_int = Expr::Call {
        func: "len".into(),
        module: None,
        args: vec![Expr::Literal(Literal::List(vec![
            Literal::Int(1),
            Literal::Int(2),
            Literal::Int(3),
        ]))],
        span: span(),
    };
    let ty1 = infer(&env, &expr_int);
    assert_eq!(ty1, Type::Int, "first call len([1,2,3]) should return Int");

    // --- Second call: len(["a", "b"]) ---
    let expr_str = Expr::Call {
        func: "len".into(),
        module: None,
        args: vec![Expr::Literal(Literal::List(vec![
            Literal::String("a".into()),
            Literal::String("b".into()),
        ]))],
        span: span(),
    };
    let ty2 = infer(&env, &expr_str);
    assert_eq!(
        ty2, Type::Int,
        "second call len([\"a\",\"b\"]) should return Int"
    );
}

#[test]
fn instantiate_fn_call_produces_fresh_substitution_per_call() {
    // Lower-level test: directly exercise `instantiate_fn_call` on the same
    // type twice with different concrete args, confirming no shared mutation.
    let len_ty = Type::Fn(
        vec![Type::List(Box::new(Type::Var(TypeVar(42))))],
        Box::new(Type::Int),
    );

    // First call with List<Int>
    let result1 = len_ty.instantiate_fn_call(&[Type::List(Box::new(Type::Int))]);
    match result1 {
        Some(Ok(ret)) => assert_eq!(ret, Type::Int),
        other => panic!("first instantiate_fn_call failed: {other:?}"),
    }

    // Second call with List<String> -- must succeed independently
    let result2 = len_ty.instantiate_fn_call(&[Type::List(Box::new(Type::String))]);
    match result2 {
        Some(Ok(ret)) => assert_eq!(ret, Type::Int),
        other => panic!("second instantiate_fn_call failed: {other:?}"),
    }

    // Third call: re-use List<Int> -- should still work
    let result3 = len_ty.instantiate_fn_call(&[Type::List(Box::new(Type::Int))]);
    match result3 {
        Some(Ok(ret)) => assert_eq!(ret, Type::Int),
        other => panic!("third instantiate_fn_call (reusing Int) failed: {other:?}"),
    }
}
