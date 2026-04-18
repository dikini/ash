//! Runtime-boundary visibility integration tests for the engine.

use ash_core::{Expr, Pattern, Value, Workflow};
use ash_interp::ExecError;

use ash_engine::Engine;

// ---- TASK-611: Helper workflow registration tests ----

#[tokio::test]
async fn engine_parses_multi_workflow_source_and_registers_helper_as_callable() {
    // Source with a helper workflow (multi-step: let + ret) followed by the
    // entry workflow. The engine should parse this as a Program with
    // helper_workflows, register the helper as a callable target, and
    // the main workflow should be the entry point.
    let engine = Engine::new().build().expect("engine builds");

    // The helper workflow "compute" has a multi-step body (let + ret).
    // The main workflow just returns 42.
    let _workflow = engine
        .parse("workflow compute { let x = 1; ret x }\nworkflow main { ret 42 }")
        .expect("multi-workflow source should parse");
}

#[tokio::test]
async fn engine_multi_workflow_helper_registered_and_callable_via_core_workflow_call() {
    let engine = Engine::new().build().expect("engine builds");

    // Parse the multi-workflow source. This registers "compute" as a callable.
    let _workflow = engine
        .parse("workflow compute { let x = 1; ret x }\nworkflow main { ret 42 }")
        .expect("multi-workflow source should parse");

    // Now construct a core Workflow::Call targeting the registered "compute" helper.
    let calling_workflow = Workflow::Call {
        target: "compute".to_string(),
        arguments: vec![],
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::Int(99)),
        }),
    };

    let result = engine
        .execute_core_workflow(&calling_workflow)
        .await
        .expect("Workflow::Call to registered helper 'compute' should execute");

    // The continuation after the call returns 99.
    assert_eq!(result, Value::Int(99));
}

#[tokio::test]
async fn engine_multi_workflow_unknown_target_rejected() {
    let engine = Engine::new().build().expect("engine builds");

    // Parse multi-workflow source (registers "compute" only).
    let _workflow = engine
        .parse("workflow compute { ret 1 }\nworkflow main { ret 42 }")
        .expect("multi-workflow source should parse");

    // Call a target that doesn't exist.
    let calling_workflow = Workflow::Call {
        target: "nonexistent".to_string(),
        arguments: vec![],
        continuation: Box::new(Workflow::Done),
    };

    let error = engine
        .execute_core_workflow(&calling_workflow)
        .await
        .expect_err("calling unregistered target should fail");

    assert!(
        format!("{error}").contains("not registered"),
        "expected 'not registered' error, got: {error}"
    );
}

#[tokio::test]
async fn engine_multi_workflow_arity_mismatch_rejected() {
    let engine = Engine::new().build().expect("engine builds");

    // Helper "compute" takes 0 params.
    let _workflow = engine
        .parse("workflow compute { ret 1 }\nworkflow main { ret 42 }")
        .expect("multi-workflow source should parse");

    // Call with wrong arity.
    let calling_workflow = Workflow::Call {
        target: "compute".to_string(),
        arguments: vec![Expr::Literal(Value::Int(1))],
        continuation: Box::new(Workflow::Done),
    };

    let error = engine
        .execute_core_workflow(&calling_workflow)
        .await
        .expect_err("wrong arity should be rejected");

    assert!(
        format!("{error}").contains("arity") || format!("{error}").contains("expected 0"),
        "expected arity mismatch error, got: {error}"
    );
}

#[tokio::test]
async fn engine_multi_workflow_helper_with_params() {
    let engine = Engine::new().build().expect("engine builds");

    // Helper "add" takes one parameter. Its body references the param.
    let _workflow = engine
        .parse("workflow add(x: Int) { ret x }\nworkflow main { ret 0 }")
        .expect("multi-workflow source with parameterized helper should parse");

    // Call with correct arity (1 arg) should resolve the target and bind the param.
    let calling_workflow = Workflow::Call {
        target: "add".to_string(),
        arguments: vec![Expr::Literal(Value::Int(5))],
        continuation: Box::new(Workflow::Done),
    };

    let result = engine.execute_core_workflow(&calling_workflow).await;
    // The child workflow 'add(x: Int) { ret x }' must resolve 'x' successfully.
    // If parameter binding were broken, 'ret x' would fail with "variable not found".
    // The child result is discarded by Workflow::Call; the continuation (Done) returns Null.
    let value = result.expect("calling 'add' with argument should succeed");
    assert_eq!(value, Value::Null);
}

// ---- Original tests below ----

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

#[tokio::test]
async fn engine_execute_core_workflow_calls_registered_callable_workflow() {
    let engine = Engine::new().build().expect("engine builds");
    engine
        .register_callable_workflow(
            "worker",
            Workflow::Ret {
                expr: Expr::Literal(Value::Int(7)),
            },
            0,
        )
        .await;

    let workflow = Workflow::Call {
        target: "worker".to_string(),
        arguments: vec![],
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::Int(99)),
        }),
    };

    let result = engine
        .execute_core_workflow(&workflow)
        .await
        .expect("registered callable workflow should execute through engine runtime state");

    assert_eq!(result, Value::Int(99));
}

#[tokio::test]
async fn engine_execute_core_workflow_rejects_callable_arity_mismatch() {
    let engine = Engine::new().build().expect("engine builds");
    engine
        .register_callable_workflow(
            "worker",
            Workflow::Ret {
                expr: Expr::Literal(Value::Int(7)),
            },
            0,
        )
        .await;

    let workflow = Workflow::Call {
        target: "worker".to_string(),
        arguments: vec![Expr::Literal(Value::Int(1))],
        continuation: Box::new(Workflow::Done),
    };

    let error = engine
        .execute_core_workflow(&workflow)
        .await
        .expect_err("non-zero arity should reject on registered callable workflow path");

    assert!(matches!(
        error,
        ExecError::Eval(ash_interp::EvalError::WrongArity {
            expected: 0,
            actual: 1,
            callee: Some(ref name),
        }) if name == "worker"
    ));
}
