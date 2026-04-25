use ash_core::runtime::{FailureEntity, ProcessId, TowerLevel};
use ash_core::{Expr, MatchArm, Pattern, Value};
use ash_interp::act_env::ActEnv;
use ash_interp::context::Context;
use ash_interp::error::EvalError;
use ash_interp::eval::{eval_expr, eval_expr_async};
use ash_interp::{ChildEnvProjection, derive_child_env};

fn fail_expr(payload: Value) -> Expr {
    Expr::Fail {
        payload: Box::new(Expr::Literal(payload)),
    }
}

fn force_hidden_act_fail(payload: Value) -> ash_interp::EvalResult<Value> {
    eval_expr(
        &Expr::FnApply {
            func: Box::new(Expr::FnDef {
                params: vec![("__act_env".to_string(), None)],
                return_type: None,
                body: Box::new(fail_expr(payload)),
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
        &Context::new().with_act_env(ActEnv::default()),
    )
}

fn invoke_closure_in_context(body: Expr, ctx: &Context) -> ash_interp::EvalResult<Value> {
    eval_expr(
        &Expr::FnApply {
            func: Box::new(Expr::FnDef {
                params: vec![("x".to_string(), None)],
                return_type: None,
                body: Box::new(body),
            }),
            args: vec![Expr::Literal(Value::Int(1))],
        },
        ctx,
    )
}

#[test]
fn fail_stays_pure_even_when_hidden_act_env_is_attached() {
    let err = eval_expr(
        &fail_expr(Value::String("boom".to_string())),
        &Context::new().with_act_env(ActEnv::default()),
    )
    .expect_err("fail must still attribute pure expressions to lexical frames");

    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected operational failure, got {err:?}");
    };

    assert_eq!(failure.tower, TowerLevel::Pure);
    assert!(matches!(failure.entity, FailureEntity::LexicalFrame(_)));
}

#[test]
fn forced_act_fail_is_attributed_to_effect_scope() {
    let err = force_hidden_act_fail(Value::String("boom".to_string()))
        .expect_err("forcing an Act closure that fails should raise an operational failure");

    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected operational failure, got {err:?}");
    };

    assert_eq!(failure.tower, TowerLevel::Effectful);
    assert!(matches!(failure.entity, FailureEntity::EffectScope(_)));
}

#[test]
fn proc_context_fail_through_closure_keeps_process_identity() {
    let child_process_id = ProcessId::new();
    let child_ctx = derive_child_env(
        &Context::new(),
        ChildEnvProjection::new(child_process_id, 0),
    )
    .expect("child process env projection should succeed");

    let err = invoke_closure_in_context(fail_expr(Value::String("boom".to_string())), &child_ctx)
        .expect_err("proc-context failure should propagate as operational failure");

    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected operational failure, got {err:?}");
    };

    assert_eq!(failure.tower, TowerLevel::Proc);
    assert_eq!(failure.entity, FailureEntity::Process(child_process_id));
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

#[tokio::test]
async fn async_with_error_catches_operational_failure_payload() {
    let expr = Expr::WithError {
        body: Box::new(fail_expr(Value::String("boom".to_string()))),
        arms: vec![MatchArm {
            pattern: Pattern::Literal(Value::String("boom".to_string())),
            body: Expr::Literal(Value::Int(42)),
        }],
    };

    let result = eval_expr_async(&expr, &Context::new())
        .await
        .expect("async handler should recover");
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
