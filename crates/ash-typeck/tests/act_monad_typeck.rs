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

// ── TASK-678: direct ActBlock type inference regression tests ──

#[test]
fn act_block_return_infers_act_type() {
    // act { ret 42; } should type-check as Act<Int>
    let env = TypeEnv::with_builtin_types();
    let expr = Expr::ActBlock {
        stmts: vec![ash_parser::surface::ActStmt::Return {
            value: Box::new(Expr::Literal(Literal::Int(42))),
            span: span(),
        }],
        span: span(),
    };

    let result = check_expr(&env, &expr);
    assert!(result.is_ok(), "single-return act block should type-check");
    assert_eq!(
        result.ty,
        Type::Constructor {
            name: QualifiedName::root("Act"),
            args: vec![Type::Int],
            kind: Kind::Type,
        },
        "act {{ ret 42; }} should infer Act<Int>"
    );
}

#[test]
fn act_block_bind_then_return_infers_inner_type() {
    // act { x = pure_call(); ret x; } where pure_call returns Int
    // Should infer Act<Int>
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("pure_call", Type::Fn(vec![], Box::new(Type::Int)));

    let expr = Expr::ActBlock {
        stmts: vec![
            ash_parser::surface::ActStmt::Bind {
                name: "x".into(),
                value: Box::new(Expr::Call {
                    func: "pure_call".into(),
                    module: None,
                    args: vec![],
                    span: span(),
                }),
                span: span(),
            },
            ash_parser::surface::ActStmt::Return {
                value: Box::new(Expr::Variable {
                    name: "x".into(),
                    span: span(),
                }),
                span: span(),
            },
        ],
        span: span(),
    };

    let result = check_expr(&env, &expr);
    assert!(
        result.is_ok(),
        "bind-then-return act block should type-check"
    );
    assert_eq!(
        result.ty,
        Type::Constructor {
            name: QualifiedName::root("Act"),
            args: vec![Type::Int],
            kind: Kind::Type,
        },
        "act {{ x = pure_call(); ret x; }} should infer Act<Int>"
    );
}

// ── B1: structural contract alignment tests ──

#[test]
fn act_block_empty_is_rejected_by_typeck() {
    // Empty act block should be rejected (aligns with lower_act_block)
    let env = TypeEnv::with_builtin_types();
    let expr = Expr::ActBlock {
        stmts: vec![],
        span: span(),
    };

    let result = check_expr(&env, &expr);
    assert!(
        !result.is_ok(),
        "empty act block should be rejected by typeck"
    );
}

#[test]
fn act_block_bind_without_return_is_rejected_by_typeck() {
    // act { x = 1; } — no return, should be rejected
    let env = TypeEnv::with_builtin_types();
    let expr = Expr::ActBlock {
        stmts: vec![ash_parser::surface::ActStmt::Bind {
            name: "x".into(),
            value: Box::new(Expr::Literal(Literal::Int(1))),
            span: span(),
        }],
        span: span(),
    };

    let result = check_expr(&env, &expr);
    assert!(
        !result.is_ok(),
        "act block without return should be rejected by typeck"
    );
}

#[test]
fn act_block_return_not_last_is_rejected_by_typeck() {
    // act { ret 1; y = 2; } — return not last, should be rejected
    let env = TypeEnv::with_builtin_types();
    let expr = Expr::ActBlock {
        stmts: vec![
            ash_parser::surface::ActStmt::Return {
                value: Box::new(Expr::Literal(Literal::Int(1))),
                span: span(),
            },
            ash_parser::surface::ActStmt::Bind {
                name: "y".into(),
                value: Box::new(Expr::Literal(Literal::Int(2))),
                span: span(),
            },
        ],
        span: span(),
    };

    let result = check_expr(&env, &expr);
    assert!(
        !result.is_ok(),
        "act block with return not last should be rejected by typeck"
    );
}
