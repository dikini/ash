use ash_core::runtime::{
    FailureBoundary, FailureEntity, OperationalFailure, ProcessId, ProcessPropagationBoundary,
    ProcessPropagationOutcome, ProcessTerminalState,
};
use ash_core::{Expr, ProcessHandle, Value};
use ash_interp::eval::{eval_expr, eval_expr_async};
use ash_interp::{ChildEnvProjection, Context, EvalError, RuntimeState, derive_child_env};

fn proc_failure(process_id: ProcessId, message: &str) -> OperationalFailure {
    OperationalFailure::new(
        FailureBoundary::Process,
        FailureEntity::Process(process_id),
        Value::String(message.to_string()),
        "String",
    )
}

fn proc_await_expr(handle: ProcessHandle) -> Expr {
    Expr::Call {
        func: "await".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![Expr::Literal(Value::ProcessHandle(handle))],
    }
}

fn proc_join_expr(left: ProcessHandle, right: ProcessHandle) -> Expr {
    Expr::Call {
        func: "join".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![
            Expr::Literal(Value::ProcessHandle(left)),
            Expr::Literal(Value::ProcessHandle(right)),
        ],
    }
}

async fn force_proc(
    runtime_state: RuntimeState,
    proc_value: Value,
) -> ash_interp::EvalResult<Value> {
    let mut ctx = Context::new().with_runtime_state(runtime_state);
    force_proc_in_context(&mut ctx, proc_value).await
}

async fn force_proc_in_context(
    ctx: &mut Context,
    proc_value: Value,
) -> ash_interp::EvalResult<Value> {
    ctx.set("p".to_string(), proc_value);
    eval_expr_async(
        &Expr::Call {
            func: "p".to_string(),
            module: None,
            arguments: vec![Expr::Literal(Value::Null)],
        },
        ctx,
    )
    .await
}

#[tokio::test]
async fn proc_await_reports_cancellation_distinct_from_ordinary_failure() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();
    let failure = proc_failure(process_id, "cancelled by parent");
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("process registers");
    runtime_state
        .record_process_terminal(
            process_id,
            ProcessTerminalState::Cancelled {
                process_id,
                failure: Box::new(failure.clone()),
            },
        )
        .await
        .expect("cancelled terminal state records");

    let proc_value = eval_expr(
        &proc_await_expr(ProcessHandle::new(process_id, Some("Int".to_string()))),
        &Context::new(),
    )
    .expect("await closure builds");

    let err = force_proc(runtime_state.clone(), proc_value)
        .await
        .expect_err("cancelled child should not surface as ordinary failure");

    assert_eq!(
        err,
        EvalError::ProcessCancelled {
            process_id,
            failure: Box::new(failure.clone()),
        }
    );

    let diagnostics = runtime_state.process_propagation_diagnostics().await;
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].boundary, ProcessPropagationBoundary::Await);
    assert_eq!(diagnostics[0].outcome, ProcessPropagationOutcome::Cancelled);
    assert_eq!(diagnostics[0].observed_process_id, process_id);
    assert_eq!(
        diagnostics[0].payload,
        Some(Value::String("cancelled by parent".to_string()))
    );
}

#[tokio::test]
async fn proc_join_records_supervisor_facing_failure_propagation_for_each_child() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    let failed_process_id = ProcessId::new();
    let succeeded_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process registers");
    runtime_state
        .register_child_process(parent_process_id, failed_process_id, 0)
        .await
        .expect("failed child registers");
    runtime_state
        .register_child_process(parent_process_id, succeeded_process_id, 1)
        .await
        .expect("succeeded child registers");
    runtime_state
        .record_process_terminal(
            failed_process_id,
            ProcessTerminalState::Failed {
                process_id: failed_process_id,
                failure: Box::new(proc_failure(failed_process_id, "boom")),
            },
        )
        .await
        .expect("failed terminal state records");
    runtime_state
        .record_process_terminal(
            succeeded_process_id,
            ProcessTerminalState::Succeeded {
                value: Value::Int(7),
            },
        )
        .await
        .expect("success terminal state records");

    let proc_value = eval_expr(
        &proc_join_expr(
            ProcessHandle::new(failed_process_id, Some("Int".to_string())),
            ProcessHandle::new(succeeded_process_id, Some("Int".to_string())),
        ),
        &Context::new(),
    )
    .expect("join closure builds");

    let mut parent_ctx = derive_child_env(
        &Context::new().with_runtime_state(runtime_state.clone()),
        ChildEnvProjection::new(parent_process_id, 0),
    )
    .expect("parent process context projects");

    let err = force_proc_in_context(&mut parent_ctx, proc_value)
        .await
        .expect_err("join should surface the failed child");
    assert!(matches!(err, EvalError::OperationalFailure(_)));

    let diagnostics = runtime_state.process_propagation_diagnostics().await;
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.boundary == ProcessPropagationBoundary::Join
            && diagnostic.outcome == ProcessPropagationOutcome::Failed
            && diagnostic.observed_process_id == failed_process_id
            && diagnostic.observer_process_id == Some(parent_process_id)
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.boundary == ProcessPropagationBoundary::Join
            && diagnostic.outcome == ProcessPropagationOutcome::Succeeded
            && diagnostic.observed_process_id == succeeded_process_id
            && diagnostic.observer_process_id == Some(parent_process_id)
    }));
}
