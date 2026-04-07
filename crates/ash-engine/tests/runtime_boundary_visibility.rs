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

    let mut pause = engine
        .parse("workflow main { ret 0 }")
        .expect("workflow should parse");
    pause.core = Workflow::Let {
        pattern: Pattern::Variable("ctrl".to_string()),
        expr: Expr::Literal(Value::ControlLink(ash_core::ControlLink {
            instance_id: ash_core::WorkflowId::new(),
        })),
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

    let error = engine
        .execute(&pause)
        .await
        .expect_err("engine-owned runtime state should not silently discard control authority");

    assert!(matches!(
        error,
        ExecError::InvalidRuntimeState(message)
            if message.contains("control link") || message.contains("not found")
    ));
}
