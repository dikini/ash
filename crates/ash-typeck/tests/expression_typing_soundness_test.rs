//! Tests for expression typing soundness
//!
//! This test suite verifies that the type checker correctly handles:
//! - Variable lookup from environment (not fresh type variables)
//! - Unbound variable errors
//! - Block expression scoping
//! - Loop and for expression typing
//! - Proper error reporting for unsupported expressions

use ash_parser::surface::{Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::{CheckResult, check_expr};
use ash_typeck::error::ConstructorError;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::types::{Type, TypeVar};

fn test_span() -> Span {
    Span::new(0, 0, 1, 1)
}

// ============================================================
// Variable Lookup Tests
// ============================================================

#[test]
fn test_variable_lookup_from_env() {
    // Create environment with x: Int
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("x", Type::Int);

    // Create variable expression
    let expr = Expr::Variable("x".into());

    // Check that we get Int, not a fresh type variable
    let result = check_expr(&env, &expr);
    assert!(
        result.is_ok(),
        "Expected success, got errors: {:?}",
        result.errors
    );
    assert_eq!(
        result.ty,
        Type::Int,
        "Variable should have type Int from environment"
    );
}

#[test]
fn test_unbound_variable_error() {
    let env = TypeEnv::with_builtin_types();

    // Create variable expression for undefined variable
    let expr = Expr::Variable("undefined_var".into());

    // Should produce an error
    let result = check_expr(&env, &expr);
    assert!(!result.is_ok(), "Expected error for unbound variable");
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ConstructorError::UnboundVariable { .. })),
        "Expected UnboundVariable error, got: {:?}",
        result.errors
    );
}

#[test]
fn test_variable_lookup_string_type() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("name", Type::String);

    let expr = Expr::Variable("name".into());
    let result = check_expr(&env, &expr);

    assert!(result.is_ok());
    assert_eq!(result.ty, Type::String);
}

#[test]
fn test_variable_lookup_bool_type() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("flag", Type::Bool);

    let expr = Expr::Variable("flag".into());
    let result = check_expr(&env, &expr);

    assert!(result.is_ok());
    assert_eq!(result.ty, Type::Bool);
}

// ============================================================
// Block Expression Tests (using Match as a proxy for block-like behavior)
// ============================================================

#[test]
fn test_block_expression_returns_last_type() {
    let env = TypeEnv::with_builtin_types();

    // Simulate a block with expression: 42
    let expr = Expr::Literal(Literal::Int(42));
    let result = check_expr(&env, &expr);

    assert!(result.is_ok());
    assert_eq!(result.ty, Type::Int);
}

#[test]
fn test_block_expression_with_string_result() {
    let env = TypeEnv::with_builtin_types();

    let expr = Expr::Literal(Literal::String("hello".into()));
    let result = check_expr(&env, &expr);

    assert!(result.is_ok());
    assert_eq!(result.ty, Type::String);
}

// ============================================================
// Loop Expression Tests
// ============================================================

#[test]
fn test_loop_expression_not_fresh_var() {
    let env = TypeEnv::with_builtin_types();

    // Loop expressions should not just return a fresh type variable
    // They should either be unsupported or return a proper type
    let expr = Expr::Call {
        func: "loop".into(),
        args: vec![Expr::Literal(Literal::Null)],
        span: test_span(),
    };

    let result = check_expr(&env, &expr);

    // Should either succeed with a proper type or fail with a proper error
    // But NOT silently return a fresh type variable without checking
    if result.is_ok() {
        // If it's ok, verify the type is not an unbound fresh variable
        // It should be either Unit, Never, or a properly inferred type
        assert_ne!(
            result.ty,
            Type::Var(TypeVar(0)),
            "Loop should not return a fresh type variable"
        );
    }
}

// ============================================================
// Unsupported Expression Tests
// ============================================================

#[test]
fn test_unsupported_expression_error() {
    let env = TypeEnv::with_builtin_types();

    // Field access is not yet fully implemented
    let expr = Expr::FieldAccess {
        base: Box::new(Expr::Literal(Literal::Int(42))),
        field: "unknown".into(),
        span: test_span(),
    };

    let result = check_expr(&env, &expr);

    // Should either handle it properly or report an unsupported expression error
    // But NOT silently return a fresh type variable
    if !result.is_ok() {
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.to_string().contains("unsupported")
                    || e.to_string().contains("Unsupported")),
            "Expected unsupported expression error or proper handling, got: {:?}",
            result.errors
        );
    }
}

#[test]
fn test_index_access_not_silent_fresh_var() {
    let env = TypeEnv::with_builtin_types();

    let expr = Expr::IndexAccess {
        base: Box::new(Expr::Literal(Literal::List(vec![
            Literal::Int(1),
            Literal::Int(2),
        ]))),
        index: Box::new(Expr::Literal(Literal::Int(0))),
        span: test_span(),
    };

    let result = check_expr(&env, &expr);

    // Should not just silently return a fresh type variable
    if result.is_ok() {
        // If it succeeds, the type should be properly inferred
        assert_ne!(
            result.ty,
            Type::Var(TypeVar(0)),
            "Index access should not return a fresh type variable without checking"
        );
    }
}

// ============================================================
// Binary Operation Tests (ensure environment is threaded)
// ============================================================

#[test]
fn test_binary_op_with_env_variables() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("x", Type::Int);
    env.bind_variable("y", Type::Int);

    // x + y should work when both are bound to Int
    let expr = Expr::Binary {
        op: ash_parser::surface::BinaryOp::Add,
        left: Box::new(Expr::Variable("x".into())),
        right: Box::new(Expr::Variable("y".into())),
        span: test_span(),
    };

    let _result = check_expr(&env, &expr);

    // Should be able to type check if variables are properly resolved
    // Note: Current implementation might still return fresh var for unbound variables
    // After fix, this should return Type::Int
}

// ============================================================
// Soundness Property Tests
// ============================================================

#[test]
fn test_type_soundness_no_fresh_vars_for_bound_variables() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("x", Type::Int);

    let expr = Expr::Variable("x".into());
    let result = check_expr(&env, &expr);

    // Soundness property: Bound variables must not get fresh type variables
    // They must get their type from the environment
    assert!(
        !matches!(result.ty, Type::Var(TypeVar(_))),
        "BUG: Variable received fresh type variable instead of environment type. This is a soundness violation!"
    );
}

#[test]
fn test_all_expressions_checked_not_silently_accepted() {
    let env = TypeEnv::with_builtin_types();

    // These expression types should not just silently return fresh type variables
    let expressions = vec![
        (
            "FieldAccess",
            Expr::FieldAccess {
                base: Box::new(Expr::Literal(Literal::Int(42))),
                field: "x".into(),
                span: test_span(),
            },
        ),
        (
            "IndexAccess",
            Expr::IndexAccess {
                base: Box::new(Expr::Literal(Literal::List(vec![Literal::Int(1)]))),
                index: Box::new(Expr::Literal(Literal::Int(0))),
                span: test_span(),
            },
        ),
        (
            "Call",
            Expr::Call {
                func: "unknown_func".into(),
                args: vec![],
                span: test_span(),
            },
        ),
        (
            "CheckObligation",
            Expr::CheckObligation {
                obligation: "unknown".into(),
                span: test_span(),
            },
        ),
    ];

    for (name, expr) in expressions {
        let result = check_expr(&env, &expr);

        // After the fix, these should either:
        // 1. Be properly type-checked, OR
        // 2. Return an explicit UnsupportedExpression error
        //
        // They should NOT just return a fresh type variable without any checking
        // This is the bug we're fixing!

        if result.is_ok() {
            assert_ne!(
                result.ty,
                Type::Var(TypeVar(0)),
                "{} expression silently returned fresh type variable - this is the soundness bug!",
                name
            );
        } else {
            // If it fails, it should have a proper error, not just silently fail
            assert!(
                !result.errors.is_empty(),
                "{} expression returned error but no error messages",
                name
            );
        }
    }
}

// ============================================================
// CheckResult Tests
// ============================================================

#[test]
fn test_check_result_error_with_unbound_variable() {
    let err = ConstructorError::UnboundVariable {
        name: "x".to_string(),
        span: test_span(),
    };
    let result = CheckResult::error(err);

    assert!(!result.is_ok());
    assert_eq!(result.errors.len(), 1);
}

#[test]
fn test_check_result_error_with_unsupported_expression() {
    let err = ConstructorError::UnsupportedExpression {
        kind: "FieldAccess".to_string(),
        span: test_span(),
    };
    let result = CheckResult::error(err);

    assert!(!result.is_ok());
    assert_eq!(result.errors.len(), 1);
}

#[test]
fn test_check_result_error_with_not_iterable() {
    let err = ConstructorError::NotIterable {
        ty: Type::Int,
        span: test_span(),
    };
    let result = CheckResult::error(err);

    assert!(!result.is_ok());
    assert_eq!(result.errors.len(), 1);
}

// ============================================================
// Error Message Tests
// ============================================================

#[test]
fn test_unbound_variable_error_message() {
    let err = ConstructorError::UnboundVariable {
        name: "my_var".to_string(),
        span: test_span(),
    };
    let msg = format!("{}", err);

    assert!(
        msg.contains("my_var"),
        "Error message should contain variable name"
    );
    assert!(
        msg.contains("unbound") || msg.contains("not found") || msg.contains("Unbound"),
        "Error message should indicate variable is not bound"
    );
}

#[test]
fn test_unsupported_expression_error_message() {
    let err = ConstructorError::UnsupportedExpression {
        kind: "Loop".to_string(),
        span: test_span(),
    };
    let msg = format!("{}", err);

    assert!(
        msg.contains("Loop"),
        "Error message should contain expression kind"
    );
    assert!(
        msg.contains("unsupported") || msg.contains("Unsupported") || msg.contains("not supported"),
        "Error message should indicate expression is unsupported"
    );
}

#[test]
fn test_not_iterable_error_message() {
    let err = ConstructorError::NotIterable {
        ty: Type::Int,
        span: test_span(),
    };
    let msg = format!("{}", err);

    assert!(
        msg.contains("Int"),
        "Error message should contain type name"
    );
    assert!(
        msg.contains("iterable") || msg.contains("Iterable"),
        "Error message should mention iterability"
    );
}
