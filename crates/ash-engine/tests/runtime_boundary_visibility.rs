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
        pattern: Pattern::Variable {
            name: "ctrl".to_string(),
            span: ash_core::ast::Span::default(),
        },
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

#[tokio::test]
async fn local_fn_defs_do_not_force_restricted_runtime_when_workflow_uses_regular_effects() {
    let engine = Engine::new().build().expect("engine builds");
    let workflow = engine
        .parse(
            "fn helper() -> Int { 1 }\nworkflow main { observe read_db as reading; ret helper() }",
        )
        .expect("workflow with local fn definitions should parse");

    let error = engine
        .execute(&workflow)
        .await
        .expect_err("missing observe provider should still come from the normal workflow runtime");

    assert!(matches!(error, ExecError::CapabilityNotAvailable(name) if name == "read_db"));
}

#[tokio::test]
async fn local_fn_calls_in_act_workflows_still_use_normal_runtime_failures() {
    let engine = Engine::new().build().expect("engine builds");
    let workflow = engine
        .parse(
            "fn answer() -> Int { 42 }\nworkflow main { act deploy:noop() as ignored; ret answer() }",
        )
        .expect("workflow with local fn call and explicit act should parse");

    let error = engine
        .execute(&workflow)
        .await
        .expect_err("missing act provider should still be reported by the normal workflow runtime");

    assert!(matches!(error, ExecError::CapabilityNotAvailable(name) if name == "deploy"));
}
