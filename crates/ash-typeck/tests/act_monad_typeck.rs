//! Phase 97 TASK-677/TASK-678: Act monad typeck coverage.
//!
//! These tests pin the next ready substrate for the Act monad work:
//! - `TypeEnv::with_builtin_types()` should register `Act` as a unary type constructor.
//! - `check_expr` should infer `Expr::ActBlock` as `Act<...>` rather than a fresh type variable.

use ash_core::ast::TypeExpr;
use ash_parser::surface::{Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::type_env::{TypeEnv, type_expr_to_type};
use ash_typeck::types::Type;
use ash_typeck::{Kind, QualifiedName};
use std::collections::HashMap;

fn span() -> Span {
    Span::default()
}

fn value_ty() -> Type {
    Type::Constructor {
        name: QualifiedName::root("Value"),
        args: vec![],
        kind: Kind::Type,
    }
}

fn list_value_ty() -> Type {
    Type::List(Box::new(value_ty()))
}

#[test]
fn builtin_types_register_act_as_unary_constructor() {
    let env = TypeEnv::with_builtin_types();

    let act_def = env
        .lookup_type("Act")
        .expect("Act should be registered by with_builtin_types()");
    assert_eq!(act_def.params.len(), 1, "Act should be unary: Act<T>");

    let resolved = type_expr_to_type(
        &TypeExpr::Constructor {
            name: "Act".into(),
            args: vec![TypeExpr::Named("Int".into())],
        },
        &HashMap::new(),
        &env,
    )
    .expect("Act<Int> should resolve through the builtin type environment");

    assert_eq!(
        resolved,
        Type::Constructor {
            name: QualifiedName::root("Act"),
            args: vec![Type::Int],
            kind: Kind::Type,
        },
        "Act<Int> should lower to a unary type constructor application"
    );
}

#[test]
fn invoke_typechecks_as_act_value() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("args", list_value_ty());

    let expr = Expr::Call {
        func: "invoke".into(),
        module: None,
        args: vec![
            Expr::Literal(Literal::String("Fs".into())),
            Expr::Literal(Literal::String("read".into())),
            Expr::Variable {
                name: "args".into(),
                span: span(),
            },
        ],
        span: span(),
    };

    let result = check_expr(&env, &expr);
    assert!(
        result.is_ok(),
        "invoke should typecheck with string provider/action and List<Value> args"
    );
    assert_eq!(
        result.ty,
        Type::Constructor {
            name: QualifiedName::root("Act"),
            args: vec![value_ty()],
            kind: Kind::Type,
        },
        "invoke should infer Act<Value>"
    );
}

#[test]
fn invoke_rejects_non_string_provider_or_action() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("args", list_value_ty());

    let expr = Expr::Call {
        func: "invoke".into(),
        module: None,
        args: vec![
            Expr::Literal(Literal::Int(1)),
            Expr::Literal(Literal::String("read".into())),
            Expr::Variable {
                name: "args".into(),
                span: span(),
            },
        ],
        span: span(),
    };

    let result = check_expr(&env, &expr);
    assert!(
        !result.is_ok(),
        "invoke with a non-string provider should be rejected"
    );
}
