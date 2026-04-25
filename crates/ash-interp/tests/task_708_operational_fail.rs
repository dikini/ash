use ash_core::runtime::{FailureEntity, TowerLevel};
use ash_core::{Expr, MatchArm, Pattern, Value};
use ash_interp::context::Context;
use ash_interp::error::EvalError;
use ash_interp::eval::eval_expr;

fn fail_expr(payload: Value) -> Expr {
    Expr::Fail {
        payload: Box::new(Expr::Literal(payload)),
    }
}

#[test]
fn eval_fail_returns_operational_failure_not_value() {
    let err = eval_expr(
        &fail_expr(Value::String("boom".to_string())),
        &Context::new(),
    )
    .expect_err("fail must raise operational failure");

    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected operational failure, got {err:?}");
    };

    assert_eq!(failure.payload, Value::String("boom".to_string()));
    assert_eq!(failure.tower, TowerLevel::Pure);
    assert!(matches!(failure.entity, FailureEntity::LexicalFrame(_)));
    assert_eq!(failure.payload_type, "String");
    assert!(failure.cause.is_none());
}

#[test]
fn with_error_catches_operational_failure_payload() {
    let expr = Expr::WithError {
        body: Box::new(fail_expr(Value::String("boom".to_string()))),
        arms: vec![MatchArm {
            pattern: Pattern::Literal(Value::String("boom".to_string())),
            body: Expr::Literal(Value::Int(42)),
        }],
    };

    let result = eval_expr(&expr, &Context::new()).expect("handler should recover");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn with_error_does_not_catch_domain_err_value() {
    let domain_err = Value::variant("Err", vec![("error", Value::String("domain".to_string()))]);
    let expr = Expr::WithError {
        body: Box::new(Expr::Literal(domain_err.clone())),
        arms: vec![MatchArm {
            pattern: Pattern::Wildcard,
            body: Expr::Literal(Value::Int(0)),
        }],
    };

    let result = eval_expr(&expr, &Context::new()).expect("ordinary Err value should complete");
    assert_eq!(result, domain_err);
}

#[test]
fn handler_refail_preserves_original_failure_as_cause() {
    let expr = Expr::WithError {
        body: Box::new(fail_expr(Value::String("lower".to_string()))),
        arms: vec![MatchArm {
            pattern: Pattern::Wildcard,
            body: fail_expr(Value::String("higher".to_string())),
        }],
    };

    let err = eval_expr(&expr, &Context::new()).expect_err("handler re-fail should propagate");
    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected operational failure, got {err:?}");
    };

    assert_eq!(failure.payload, Value::String("higher".to_string()));
    let cause = failure
        .cause
        .as_deref()
        .expect("original failure cause preserved");
    assert_eq!(cause.payload, Value::String("lower".to_string()));
}

#[test]
fn handler_refail_preserves_existing_cause_chain_and_original_failure() {
    let expr = Expr::WithError {
        body: Box::new(fail_expr(Value::String("caught".to_string()))),
        arms: vec![MatchArm {
            pattern: Pattern::Wildcard,
            body: Expr::WithError {
                body: Box::new(fail_expr(Value::String("inner".to_string()))),
                arms: vec![MatchArm {
                    pattern: Pattern::Wildcard,
                    body: fail_expr(Value::String("outer".to_string())),
                }],
            },
        }],
    };

    let err = eval_expr(&expr, &Context::new()).expect_err("nested re-fail should propagate");
    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected operational failure, got {err:?}");
    };

    assert_eq!(failure.payload, Value::String("outer".to_string()));
    let inner = failure.cause.as_deref().expect("inner failure cause kept");
    assert_eq!(inner.payload, Value::String("inner".to_string()));
    let caught = inner
        .cause
        .as_deref()
        .expect("original caught failure appended to cause chain");
    assert_eq!(caught.payload, Value::String("caught".to_string()));
}
