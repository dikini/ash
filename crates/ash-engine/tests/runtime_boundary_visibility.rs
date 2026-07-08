//! Runtime-boundary visibility integration tests for the engine.

use ash_core::{
    Capability, Effect, Expr, Guard, Pattern, ReceiveArm, ReceiveMode, ReceivePattern, Value,
    Workflow,
};
use ash_interp::ExecError;

use ash_engine::Engine;

// ---- TASK-611: Helper function registration tests ----

#[tokio::test]
async fn engine_parses_helper_function_source_and_registers_function_body() {
    // Source with a helper function (multi-step let expression) followed by the
    // entry definition. The engine should parse this as a program with helper
    // functions, register the helper as a function body, and use `main` as the
    // entry point.
    let engine = Engine::new().build().expect("engine builds");

    // The helper function "compute" has a multi-step expression body.
    // The main function just returns 42.
    let _workflow = engine
        .parse("fn compute() -> Int { let x = 1; x }\nfn main() -> Int { 42 }")
        .expect("helper-function source should parse");
}

#[tokio::test]
async fn engine_helper_function_registered_and_callable_via_core_call() {
    let engine = Engine::new().build().expect("engine builds");

    // Parse the helper-function source. This registers "compute" as a function body.
    let _workflow = engine
        .parse("fn compute() -> Int { let x = 1; x }\nfn main() -> Int { 42 }")
        .expect("helper-function source should parse");

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
        .expect_err("parsed helper source should not implicitly register runtime call targets");

    assert!(
        format!("{result}").contains("not registered"),
        "expected missing runtime registration error, got: {result}"
    );
}

#[tokio::test]
async fn engine_helper_function_unknown_target_rejected() {
    let engine = Engine::new().build().expect("engine builds");

    // Parse multi-fn source(registers "compute" only).
    let _workflow = engine
        .parse("fn compute() -> Int { 1 }\nfn main() -> Int { 42 }")
        .expect("helper-function source should parse");

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
async fn engine_helper_function_arity_mismatch_rejected() {
    let engine = Engine::new().build().expect("engine builds");

    // Helper "compute" takes 0 params.
    let _workflow = engine
        .parse("fn compute() -> Int { 1 }\nfn main() -> Int { 42 }")
        .expect("helper-function source should parse");

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
        format!("{error}").contains("not registered"),
        "expected missing runtime registration error, got: {error}"
    );
}

#[tokio::test]
async fn engine_helper_function_with_params() {
    let engine = Engine::new().build().expect("engine builds");

    // Helper "add" takes one parameter. Its body references the param.
    let _workflow = engine
        .parse("fn add(x: Int) -> Int { x }\nfn main() -> Int { 0 }")
        .expect("helper-function source with parameterized helper should parse");

    // Call with correct arity (1 arg) should resolve the target and bind the param.
    let calling_workflow = Workflow::Call {
        target: "add".to_string(),
        arguments: vec![Expr::Literal(Value::Int(5))],
        continuation: Box::new(Workflow::Done),
    };

    let error = engine
        .execute_core_workflow(&calling_workflow)
        .await
        .expect_err("parsed helper source should not implicitly register runtime call targets");
    assert!(
        format!("{error}").contains("not registered"),
        "expected missing runtime registration error, got: {error}"
    );
}

// ---- Original tests below ----

#[tokio::test]
async fn engine_execute_preserves_missing_observe_capability_rejection() {
    let engine = Engine::new().build().expect("engine builds");
    let mut workflow = engine
        .parse("fn main() -> Int { 0 }")
        .expect("entry source should parse");
    workflow.core = Workflow::Observe {
        capability: Capability {
            name: "read_db".to_string(),
            effect: Effect::Epistemic,
            constraints: vec![],
        },
        pattern: Pattern::Variable {
            name: "reading".to_string(),
            span: ash_core::ast::Span::default(),
        },
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Variable {
                name: "reading".to_string(),
                span: ash_core::ast::Span::default(),
            },
        }),
    };

    let error = engine
        .execute(&workflow)
        .await
        .expect_err("missing observe provider should reject explicitly");

    assert!(matches!(error, ExecError::CapabilityNotAvailable(name) if name == "read_db"));
}

#[tokio::test]
async fn engine_execute_preserves_missing_stream_context_rejection() {
    let engine = Engine::new().build().expect("engine builds");
    let mut workflow = engine
        .parse("fn main() -> Int { 0 }")
        .expect("entry source should parse");
    workflow.core = Workflow::Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![ReceiveArm {
            pattern: ReceivePattern::Stream {
                capability: "sensor".to_string(),
                channel: "temp".to_string(),
                pattern: Pattern::Variable {
                    name: "reading".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            },
            guard: None,
            body: Workflow::Ret {
                expr: Expr::Variable {
                    name: "reading".to_string(),
                    span: ash_core::ast::Span::default(),
                },
            },
        }],
        control: false,
    };

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
        .parse("fn main() -> Int { 0 }")
        .expect("entry source should parse");
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
async fn local_fn_defs_do_not_force_restricted_runtime_when_entry_uses_regular_effects() {
    let engine = Engine::new().build().expect("engine builds");
    let workflow = engine
        .parse("fn helper() -> Int { 1 }\nfn main() -> Int { 0 }")
        .expect("entry source with local fn definitions should parse");
    let mut workflow = workflow;
    workflow.core = Workflow::Observe {
        capability: Capability {
            name: "read_db".to_string(),
            effect: Effect::Epistemic,
            constraints: vec![],
        },
        pattern: Pattern::Variable {
            name: "reading".to_string(),
            span: ash_core::ast::Span::default(),
        },
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::Int(1)),
        }),
    };

    let error = engine
        .execute(&workflow)
        .await
        .expect_err("missing observe provider should still come from the normal entry runtime");

    assert!(matches!(error, ExecError::CapabilityNotAvailable(name) if name == "read_db"));
}

#[tokio::test]
async fn local_fn_calls_in_act_entries_still_use_normal_runtime_failures() {
    let engine = Engine::new().build().expect("engine builds");
    let workflow = engine
        .parse("fn answer() -> Int { 42 }\nfn main() -> Int { 0 }")
        .expect("entry source with local fn call and explicit act should parse");
    let mut workflow = workflow;
    workflow.core = Workflow::Act {
        provider_name: "deploy".to_string(),
        action_name: "noop".to_string(),
        arguments: vec![],
        guard: Guard::Always,
        provenance: ash_core::Provenance::new(),
        result_name: Some("ignored".to_string()),
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::Int(42)),
        }),
    };

    let error = engine
        .execute(&workflow)
        .await
        .expect_err("missing act provider should still be reported by the normal entry runtime");

    assert!(matches!(error, ExecError::CapabilityNotAvailable(name) if name == "deploy"));
}

#[tokio::test]
async fn engine_execute_core_workflow_calls_registered_function_body() {
    let engine = Engine::new().build().expect("engine builds");
    engine
        .register_function_body(
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
        .expect("registered function body should execute through engine runtime state");

    assert_eq!(result, Value::Int(99));
}

#[tokio::test]
async fn engine_execute_core_workflow_rejects_callable_arity_mismatch() {
    let engine = Engine::new().build().expect("engine builds");
    engine
        .register_function_body(
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
        .expect_err("non-zero arity should reject on registered callable entry path");

    assert!(matches!(
        error,
        ExecError::Eval(ash_interp::EvalError::WrongArity {
            expected: 0,
            actual: 1,
            callee: Some(ref name),
        }) if name == "worker"
    ));
}
