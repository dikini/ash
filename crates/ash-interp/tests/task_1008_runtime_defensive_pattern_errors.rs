#![allow(non_snake_case)]

use ash_core::ast::{MatchArm, Span};
use ash_core::{Expr, Pattern, Value, Workflow};
use ash_interp::{Context, EvalError, ExecError, eval_expr_async, interpret};
use ash_parser::surface::{
    Expr as SurfaceExpr, Literal as SurfaceLiteral, Pattern as SurfacePattern,
    Workflow as SurfaceWorkflow,
};
use ash_parser::token::Span as SurfaceSpan;
use ash_typeck::{TypeCheckError, type_check_workflow};

fn span() -> Span {
    Span::default()
}

fn surface_span() -> SurfaceSpan {
    SurfaceSpan::default()
}

fn literal_int_pattern(value: i64) -> Pattern {
    Pattern::Literal(Value::Int(value))
}

#[tokio::test]
async fn runtime_defensive_expr_let_still_yields_LetPatternBindFailed_for_unchecked_ir() {
    let expr = Expr::Let {
        pattern: literal_int_pattern(0),
        expr: Box::new(Expr::Literal(Value::Int(1))),
        body: Box::new(Expr::Literal(Value::Int(9))),
        span: span(),
    };

    let err = eval_expr_async(&expr, &Context::new())
        .await
        .expect_err("unchecked IR let mismatch should remain a defensive runtime error");

    match err {
        EvalError::LetPatternBindFailed { pattern, value } => {
            assert!(pattern.contains("Int(0)"), "{pattern}");
            assert!(value.contains("Int(1)"), "{value}");
        }
        other => panic!("expected EvalError::LetPatternBindFailed, got {other:?}"),
    }
}

#[tokio::test]
async fn runtime_defensive_workflow_binder_still_yields_PatternMatchFailed_for_unchecked_ir() {
    let workflow = Workflow::Let {
        pattern: literal_int_pattern(0),
        expr: Expr::Literal(Value::Int(1)),
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::Int(9)),
        }),
    };

    let err = interpret(&workflow)
        .await
        .expect_err("unchecked workflow binder mismatch should remain a defensive runtime error");

    match err {
        ExecError::PatternMatchFailed { pattern, value } => {
            assert!(pattern.contains("Int(0)"), "{pattern}");
            assert_eq!(*value, Value::Int(1));
        }
        other => panic!("expected ExecError::PatternMatchFailed, got {other:?}"),
    }
}

#[tokio::test]
async fn runtime_defensive_match_still_yields_NonExhaustiveMatch_for_unchecked_ir() {
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Literal(Value::Int(1))),
        arms: vec![MatchArm {
            pattern: literal_int_pattern(0),
            body: Expr::Literal(Value::Int(9)),
        }],
    };

    let err = eval_expr_async(&expr, &Context::new())
        .await
        .expect_err("unchecked non-exhaustive match should remain a defensive runtime error");

    match err {
        EvalError::NonExhaustiveMatch { value } => {
            assert!(value.contains("Int(1)"), "{value}");
        }
        other => panic!("expected EvalError::NonExhaustiveMatch, got {other:?}"),
    }
}

#[test]
fn checked_source_refutable_binders_fail_in_typeck_not_runtime() {
    let checked = type_check_workflow(
        &SurfaceWorkflow::Let {
            pattern: SurfacePattern::Literal(SurfaceLiteral::Int(0)),
            expr: SurfaceExpr::Literal(SurfaceLiteral::Int(1)),
            continuation: Some(Box::new(SurfaceWorkflow::Done {
                span: surface_span(),
            })),
            span: surface_span(),
        },
        None,
    );

    let err = checked.expect_err("checked source refutable workflow binders must fail in typeck");
    let TypeCheckError::TypeError(message) = err else {
        panic!("expected TypeCheckError::TypeError, got {err:?}");
    };

    assert!(message.contains("workflow let"), "{message}");
    assert!(message.contains("irrefutable"), "{message}");
    assert!(message.contains("use match or if let"), "{message}");
}
