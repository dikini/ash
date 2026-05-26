//! TASK-958 coverage for preferred pure callable rendering and exact-arity checking.

use ash_parser::surface::{Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::{CheckResult, check_expr};
use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::Type;

fn span() -> Span {
    Span::default()
}

fn int(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

fn var(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn error_text(result: &CheckResult) -> String {
    result
        .errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

fn env_with_binary_add() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable(
        "add",
        Type::Fn(vec![Type::Int, Type::Int], Box::new(Type::Int)),
    );
    env
}

#[test]
fn type_display_prefers_parenthesized_callable_domain() {
    let unary = Type::Fn(vec![Type::Int], Box::new(Type::String));
    let n_ary = Type::Fn(vec![Type::Int, Type::String], Box::new(Type::Bool));
    let callable_param = Type::Fn(vec![n_ary.clone()], Box::new(Type::Bool));

    assert_eq!(unary.to_string(), "(Int) -> String");
    assert_eq!(n_ary.to_string(), "(Int, String) -> Bool");
    assert_eq!(
        callable_param.to_string(),
        "((Int, String) -> Bool) -> Bool"
    );
}

#[test]
fn nested_return_callable_renders_right_associative() {
    let nested_return = Type::Fn(
        vec![Type::Int],
        Box::new(Type::Fn(vec![Type::String], Box::new(Type::Bool))),
    );

    assert_eq!(nested_return.to_string(), "(Int) -> (String) -> Bool");
}

#[test]
fn callable_application_requires_exact_arity() {
    let env = env_with_binary_add();
    let named_call = Expr::Call {
        func: "add".into(),
        module: None,
        args: vec![int(1), int(2)],
        span: span(),
    };
    let fn_apply = Expr::FnApply {
        func: Box::new(var("add")),
        args: vec![int(1), int(2)],
        span: span(),
    };

    let named_result = check_expr(&env, &named_call);
    assert!(
        named_result.is_ok(),
        "exact-arity named call should typecheck, got {:?}",
        named_result.errors
    );
    assert_eq!(named_result.ty, Type::Int);

    let apply_result = check_expr(&env, &fn_apply);
    assert!(
        apply_result.is_ok(),
        "exact-arity function application should typecheck, got {:?}",
        apply_result.errors
    );
    assert_eq!(apply_result.ty, Type::Int);
}

#[test]
fn too_few_arguments_are_not_partial_application() {
    let env = env_with_binary_add();
    let expr = Expr::FnApply {
        func: Box::new(var("add")),
        args: vec![int(1)],
        span: span(),
    };

    let result = check_expr(&env, &expr);

    assert!(
        !result.is_ok(),
        "too few args must fail instead of returning a partial callable type: {:?}",
        result.ty
    );
    let text = error_text(&result);
    assert!(
        text.contains("expected exactly 2 args, got 1"),
        "expected exact-arity diagnostic, got {text}"
    );
}

#[test]
fn too_many_arguments_report_exact_arity() {
    let env = env_with_binary_add();
    let expr = Expr::Call {
        func: "add".into(),
        module: None,
        args: vec![int(1), int(2), int(3)],
        span: span(),
    };

    let result = check_expr(&env, &expr);

    assert!(!result.is_ok(), "too many args must fail");
    let text = error_text(&result);
    assert!(
        text.contains("expected exactly 2 args, got 3"),
        "expected exact-arity diagnostic, got {text}"
    );
    assert!(
        !text.contains("expected at most"),
        "diagnostic must not describe partial-application-compatible arity: {text}"
    );
}
