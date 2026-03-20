//! Runtime-boundary visibility integration tests for the engine.

use ash_core::{Expr, Pattern, Value, Workflow};
use ash_interp::ExecError;

use ash_engine::Engine;

#[tokio::test]
async fn engine_execute_preserves_missing_observe_capability_rejection() {
    let engine = Engine::new().build().expect("engine builds");
    let workflow = engine
        .parse("workflow main { observe read_db as reading; ret reading }")
        .expect("workflow should parse");

    let error = engine
        .execute(&workflow)
        .await
        .expect_err("missing observe provider should reject explicitly");

    assert!(matches!(error, ExecError::CapabilityNotAvailable(name) if name == "read_db"));
}

#[tokio::test]
async fn engine_execute_preserves_missing_stream_context_rejection() {
    let engine = Engine::new().build().expect("engine builds");
    let workflow = engine
        .parse("workflow main { receive { sensor:temp as reading => ret reading } }")
        .expect("workflow should parse");

    let error = engine
        .execute(&workflow)
        .await
        .expect_err("receive without stream context should reject explicitly");

    assert!(
        matches!(error, ExecError::ExecutionFailed(message) if message.contains("Receive requires StreamContext"))
    );
}

#[tokio::test]
async fn engine_execute_preserves_control_authority_across_top_level_runs() {
    let engine = Engine::new().build().expect("engine builds");

    let mut spawn = engine
        .parse("workflow main { ret 0 }")
        .expect("workflow should parse");
    spawn.core = Workflow::Spawn {
        workflow_type: "worker".to_string(),
        init: Expr::Literal(Value::Null),
        pattern: Pattern::Variable("worker".to_string()),
        continuation: Box::new(Workflow::Split {
            expr: Expr::Variable("worker".to_string()),
            pattern: Pattern::Tuple(vec![
                Pattern::Wildcard,
                Pattern::Variable("ctrl".to_string()),
            ]),
            continuation: Box::new(Workflow::Ret {
                expr: Expr::Variable("ctrl".to_string()),
            }),
        }),
    };

    let control = engine
        .execute(&spawn)
        .await
        .expect("spawn should return a control link");

    let mut pause = engine
        .parse("workflow main { ret 0 }")
        .expect("workflow should parse");
    pause.core = Workflow::Let {
        pattern: Pattern::Variable("ctrl".to_string()),
        expr: Expr::Literal(control),
        continuation: Box::new(Workflow::Pause {
            target: "ctrl".to_string(),
            continuation: Box::new(Workflow::CheckHealth {
                target: "ctrl".to_string(),
                continuation: Box::new(Workflow::Resume {
                    target: "ctrl".to_string(),
                    continuation: Box::new(Workflow::Ret {
                        expr: Expr::Literal(Value::Int(1)),
                    }),
                }),
            }),
        }),
    };

    let result = engine
        .execute(&pause)
        .await
        .expect("engine-owned runtime state should preserve control authority");

    assert_eq!(result, Value::Int(1));
}
