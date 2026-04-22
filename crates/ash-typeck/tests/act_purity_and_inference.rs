//! Phase 97 TASK-682: Comprehensive type-system tests for purity rejection and
//! Act<T> inference.
//!
//! Complements act_monad_typeck.rs with:
//! - Act<T> inference for diverse inner types (String, Bool, chained binds)
//! - Purity rejection via check_expr (fn-level context)
//! - Purity rejection via check_purity (expression-level)
//! - Proptests for Act inference invariants

use ash_parser::surface::{ActStmt, Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::QualifiedName;
use ash_typeck::check_expr::check_expr;
use ash_typeck::purity::{PurityViolation, check_purity};
use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::Type;
use proptest::prelude::*;

fn span() -> Span {
    Span::default()
}

// ── Helpers ──────────────────────────────────────────────────────────

fn act_t(inner: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root("Act"),
        args: vec![inner],
        kind: ash_typeck::Kind::Type,
    }
}

fn int_val(n: i64) -> Expr {
    Expr::Literal(Literal::Int(n))
}

fn bool_val(b: bool) -> Expr {
    Expr::Literal(Literal::Bool(b))
}

fn string_val(s: &str) -> Expr {
    Expr::Literal(Literal::String(s.into()))
}

fn var_ref(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn act_block_return(value: Expr) -> Expr {
    Expr::ActBlock {
        stmts: vec![ActStmt::Return {
            value: Box::new(value),
            span: span(),
        }],
        span: span(),
    }
}

fn act_block_binds_then_return(binds: Vec<(&str, Expr)>, ret_name: &str) -> Expr {
    let mut stmts: Vec<ActStmt> = binds
        .into_iter()
        .map(|(name, expr)| ActStmt::Bind {
            name: name.into(),
            value: Box::new(expr),
            span: span(),
        })
        .collect();
    stmts.push(ActStmt::Return {
        value: Box::new(var_ref(ret_name)),
        span: span(),
    });
    Expr::ActBlock {
        stmts,
        span: span(),
    }
}

fn invoke_expr() -> Expr {
    Expr::Call {
        func: "invoke".into(),
        module: None,
        args: vec![
            string_val("fs"),
            string_val("read"),
            Expr::Literal(Literal::List(vec![])),
        ],
        span: span(),
    }
}

// ── A. Act<T> inference for diverse inner types ─────────────────────

#[test]
fn act_block_return_string_infers_act_string() {
    let env = TypeEnv::with_builtin_types();
    let expr = act_block_return(string_val("hello"));
    let result = check_expr(&env, &expr);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
    assert_eq!(result.ty, act_t(Type::String));
}

#[test]
fn act_block_return_bool_infers_act_bool() {
    let env = TypeEnv::with_builtin_types();
    let expr = act_block_return(bool_val(true));
    let result = check_expr(&env, &expr);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
    assert_eq!(result.ty, act_t(Type::Bool));
}

#[test]
fn act_block_bind_literal_then_return_var_infers_act_int() {
    let env = TypeEnv::with_builtin_types();
    let expr = act_block_binds_then_return(vec![("x", int_val(42))], "x");
    let result = check_expr(&env, &expr);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
    assert_eq!(result.ty, act_t(Type::Int));
}

#[test]
fn act_block_chained_binds_infers_act_int() {
    let env = TypeEnv::with_builtin_types();
    let expr = act_block_binds_then_return(vec![("x", int_val(42)), ("y", int_val(1))], "x");
    let result = check_expr(&env, &expr);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
    assert_eq!(result.ty, act_t(Type::Int));
}

// ── B. Purity rejection via check_expr (fn-level context) ───────────

#[test]
fn pure_fn_containing_act_block_is_rejected() {
    let env = TypeEnv::with_builtin_types();
    let fn_def = Expr::FnDef {
        params: vec![],
        return_type: Some("Int".into()),
        body: Box::new(act_block_return(int_val(1))),
        span: span(),
    };
    let result = check_expr(&env, &fn_def);
    assert!(!result.is_ok(), "pure fn with act block should have errors");
}

#[test]
fn act_returning_fn_containing_act_block_is_ok_in_purity_check() {
    // check_purity uses the return_type name directly (starts-with "Act" → allow effects),
    // so bare "Act" works there.
    let env = TypeEnv::with_builtin_types();
    let fn_def = Expr::FnDef {
        params: vec![],
        return_type: Some("Act".into()),
        body: Box::new(act_block_return(int_val(1))),
        span: span(),
    };
    let result = check_purity(&env, &fn_def, false);
    assert!(
        result.is_ok(),
        "Act-returning fn with act block should be pure-ok via nested context, errors: {:?}",
        result
    );
}

// ── C. Purity rejection via check_purity ────────────────────────────

#[test]
fn purity_rejects_act_block_when_effects_disallowed() {
    let env = TypeEnv::with_builtin_types();
    let expr = act_block_return(int_val(1));
    let result = check_purity(&env, &expr, false);
    assert!(
        result.is_err(),
        "act block should be rejected in pure context"
    );
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| matches!(e.kind, PurityViolation::ActBlockInPureContext)),
        "expected ActBlockInPureContext violation, got: {:?}",
        errors
    );
}

#[test]
fn purity_allows_act_block_when_effects_allowed() {
    let env = TypeEnv::with_builtin_types();
    let expr = act_block_return(int_val(1));
    let result = check_purity(&env, &expr, true);
    assert!(
        result.is_ok(),
        "act block should be allowed in effectful context"
    );
}

#[test]
fn purity_rejects_invoke_when_effects_disallowed() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("invoke", Type::Fn(vec![], Box::new(Type::Int)));
    let expr = invoke_expr();
    let result = check_purity(&env, &expr, false);
    assert!(result.is_err(), "invoke should be rejected in pure context");
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| matches!(e.kind, PurityViolation::InvokeInPureContext)),
        "expected InvokeInPureContext violation, got: {:?}",
        errors
    );
}

#[test]
fn purity_allows_invoke_when_effects_allowed() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("invoke", Type::Fn(vec![], Box::new(Type::Int)));
    let expr = invoke_expr();
    let result = check_purity(&env, &expr, true);
    assert!(
        result.is_ok(),
        "invoke should be allowed in effectful context"
    );
}

// ── D. Proptests ────────────────────────────────────────────────────

proptest! {
    /// For any integer literal, act { ret <int>; } infers Act<Int>.
    #[test]
    fn prop_act_return_int_infers_act_int(n in -1000..1000i64) {
        let env = TypeEnv::with_builtin_types();
        let expr = act_block_return(int_val(n));
        let result = check_expr(&env, &expr);
        prop_assert!(result.is_ok(), "errors: {:?}", result.errors);
        prop_assert_eq!(result.ty, act_t(Type::Int));
    }

    /// For any string literal, act { ret <str>; } infers Act<String>.
    #[test]
    fn prop_act_return_string_infers_act_string(s in "[a-z]{1,10}") {
        let env = TypeEnv::with_builtin_types();
        let expr = act_block_return(string_val(&s));
        let result = check_expr(&env, &expr);
        prop_assert!(result.is_ok(), "errors: {:?}", result.errors);
        prop_assert_eq!(result.ty, act_t(Type::String));
    }

    /// act { x = <int>; ret x; } always infers Act<Int>.
    #[test]
    fn prop_act_bind_int_return_infers_act_int(n in -1000..1000i64) {
        let env = TypeEnv::with_builtin_types();
        let expr = act_block_binds_then_return(vec![("x", int_val(n))], "x");
        let result = check_expr(&env, &expr);
        prop_assert!(result.is_ok(), "errors: {:?}", result.errors);
        prop_assert_eq!(result.ty, act_t(Type::Int));
    }
}
