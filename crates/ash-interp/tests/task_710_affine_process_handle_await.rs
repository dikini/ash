use ash_core::runtime::{
    FailureEntity, OperationalFailure, ProcessId, ProcessTerminalState, TowerLevel,
};
use ash_core::{Expr, ProcessHandle, Value, Workflow};
use ash_interp::behaviour::BehaviourContext;
use ash_interp::capability::CapabilityContext;
use ash_interp::eval::{eval_expr, eval_expr_async};
use ash_interp::execute::execute_workflow_with_behaviour_in_state;
use ash_interp::policy::PolicyEvaluator;
use ash_interp::{Context, EvalError, RuntimeState};
use proptest::prelude::*;

fn proc_failure(process_id: ProcessId, message: &str) -> OperationalFailure {
    OperationalFailure::new(
        TowerLevel::Proc,
        FailureEntity::Process(process_id),
        Value::String(message.to_string()),
        "String",
    )
}

fn proc_await_expr(handle: Value) -> Expr {
    Expr::Call {
        func: "await".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![Expr::Literal(handle)],
    }
}

async fn force_proc_with_runtime(
    runtime_state: RuntimeState,
    proc_value: Value,
) -> ash_interp::EvalResult<Value> {
    let mut ctx = Context::new().with_runtime_state(runtime_state);
    ctx.set("p".to_string(), proc_value);
    eval_expr_async(
        &Expr::Call {
            func: "p".to_string(),
            module: None,
            arguments: vec![Expr::Literal(Value::Null)],
        },
        &ctx,
    )
    .await
}

async fn execute_ret_expr(
    workflow_expr: Expr,
    runtime_state: &RuntimeState,
) -> ash_interp::ExecResult<Value> {
    let workflow = Workflow::Ret {
        expr: workflow_expr,
    };
    let ctx = Context::new();
    let cap_ctx = CapabilityContext::new();
    let policy_eval = PolicyEvaluator::new();
    let behaviour_ctx = BehaviourContext::new();
    execute_workflow_with_behaviour_in_state(
        &workflow,
        ctx,
        &cap_ctx,
        &policy_eval,
        &behaviour_ctx,
        runtime_state,
    )
    .await
}

#[tokio::test]
async fn awaiting_fresh_handle_returns_child_success_value() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("process registers");
    runtime_state
        .record_process_terminal(
            process_id,
            ProcessTerminalState::Succeeded {
                value: Value::Int(42),
            },
        )
        .await
        .expect("terminal state records");

    let handle = Value::ProcessHandle(ProcessHandle::new(process_id, Some("Int".to_string())));
    let proc_value = eval_expr(&proc_await_expr(handle), &Context::new())
        .expect("proc::await should build a Proc closure");

    let result = force_proc_with_runtime(runtime_state, proc_value)
        .await
        .expect("await succeeds");
    assert_eq!(result, Value::Int(42));
}

#[tokio::test]
async fn execute_workflow_with_behaviour_in_state_propagates_runtime_state_for_proc_await() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("process registers");
    runtime_state
        .record_process_terminal(
            process_id,
            ProcessTerminalState::Succeeded {
                value: Value::Int(11),
            },
        )
        .await
        .expect("terminal state records");

    let result = execute_ret_expr(
        proc_await_expr(Value::ProcessHandle(ProcessHandle::new(
            process_id,
            Some("Int".to_string()),
        ))),
        &runtime_state,
    )
    .await
    .expect("workflow execution should project runtime state into expression evaluation");

    let Value::Closure { .. } = result else {
        panic!("expected proc::await to evaluate to a Proc closure, got {result:?}");
    };
    let forced = force_proc_with_runtime(runtime_state, result)
        .await
        .unwrap();
    assert_eq!(forced, Value::Int(11));
}

#[tokio::test]
async fn awaiting_failed_child_surfaces_operational_failure_with_child_process_identity() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();
    let failure = proc_failure(process_id, "boom");
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("process registers");
    runtime_state
        .record_process_terminal(
            process_id,
            ProcessTerminalState::Failed {
                process_id,
                failure: Box::new(failure.clone()),
            },
        )
        .await
        .expect("terminal state records");

    let handle = Value::ProcessHandle(ProcessHandle::new(process_id, Some("Int".to_string())));
    let proc_value = eval_expr(&proc_await_expr(handle), &Context::new())
        .expect("proc::await should build a Proc closure");

    let err = force_proc_with_runtime(runtime_state, proc_value)
        .await
        .expect_err("await should surface child failure");
    let EvalError::OperationalFailure(observed) = err else {
        panic!("expected operational failure, got {err:?}");
    };
    assert_eq!(*observed, failure);
    assert_eq!(observed.entity, FailureEntity::Process(process_id));
}

#[tokio::test]
async fn second_await_on_same_handle_fails_with_structured_handle_consumed_error() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("process registers");
    runtime_state
        .record_process_terminal(
            process_id,
            ProcessTerminalState::Succeeded { value: Value::Null },
        )
        .await
        .expect("terminal state records");

    let handle = Value::ProcessHandle(ProcessHandle::new(process_id, Some("Unit".to_string())));
    let proc_value = eval_expr(&proc_await_expr(handle.clone()), &Context::new())
        .expect("proc::await should build a Proc closure");
    let second_proc = eval_expr(&proc_await_expr(handle), &Context::new())
        .expect("proc::await should build a second Proc closure over same handle value");

    force_proc_with_runtime(runtime_state.clone(), proc_value)
        .await
        .expect("first await succeeds");
    let err = force_proc_with_runtime(runtime_state, second_proc)
        .await
        .expect_err("second await must fail");

    assert_eq!(err, EvalError::ProcessHandleConsumed { process_id });
}

#[tokio::test]
async fn awaiting_unregistered_process_fails_honestly_and_deterministically() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();

    let handle = Value::ProcessHandle(ProcessHandle::new(process_id, Some("Int".to_string())));
    let proc_value = eval_expr(&proc_await_expr(handle), &Context::new())
        .expect("proc::await should build a Proc closure");

    let err = force_proc_with_runtime(runtime_state, proc_value)
        .await
        .expect_err("await must reject unregistered process");
    assert_eq!(
        err,
        EvalError::ProcessObservationUnavailable {
            process_id,
            reason: "process is not in a retained terminal state".to_string(),
        }
    );
}

proptest! {
    #[test]
    fn process_handle_linearity_consumes_exactly_one_observation(seed in any::<u64>()) {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async move {
            let runtime_state = RuntimeState::new();
            let process_id = ProcessId::new();
            runtime_state
                .register_root_process(process_id)
                .await
                .expect("process registers");
            runtime_state
                .record_process_terminal(
                    process_id,
                    ProcessTerminalState::Succeeded {
                        value: Value::Int(seed as i64),
                    },
                )
                .await
                .expect("terminal state records");

            let handle = Value::ProcessHandle(ProcessHandle::new(process_id, Some("Int".to_string())));
            let first_proc = eval_expr(&proc_await_expr(handle.clone()), &Context::new())
                .expect("first proc await closure builds");
            let second_proc = eval_expr(&proc_await_expr(handle), &Context::new())
                .expect("second proc await closure builds");

            let first = force_proc_with_runtime(runtime_state.clone(), first_proc).await;
            let second = force_proc_with_runtime(runtime_state, second_proc).await;

            let consumed = EvalError::ProcessHandleConsumed { process_id };
            prop_assert!(
                (first == Ok(Value::Int(seed as i64)) && second == Err(consumed.clone()))
                    || (second == Ok(Value::Int(seed as i64)) && first == Err(consumed))
            );
            Ok(())
        })?;
    }
}
