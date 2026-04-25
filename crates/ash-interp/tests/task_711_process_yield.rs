use ash_core::runtime::{FailureEntity, ProcessId, ProcessTerminalState, TowerLevel};
use ash_core::{Expr, ProcessHandle, Value, Workflow};
use ash_interp::behaviour::BehaviourContext;
use ash_interp::capability::CapabilityContext;
use ash_interp::eval::{eval_expr, eval_expr_async};
use ash_interp::execute::execute_workflow_with_behaviour_in_state;
use ash_interp::policy::PolicyEvaluator;
use ash_interp::{ChildEnvProjection, Context, EvalError, RuntimeState, derive_child_env};
use proptest::prelude::*;

fn proc_yield_expr() -> Expr {
    Expr::Call {
        func: "yield".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![],
    }
}

fn proc_await_expr(handle: Value) -> Expr {
    Expr::Call {
        func: "await".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![Expr::Literal(handle)],
    }
}

fn fail_expr(payload: Value) -> Expr {
    Expr::Fail {
        payload: Box::new(Expr::Literal(payload)),
    }
}

fn proc_fail_continuation(payload: Value) -> Expr {
    Expr::FnDef {
        params: vec![("unit".to_string(), None)],
        return_type: None,
        body: Box::new(Expr::FnDef {
            params: vec![("__proc_env".to_string(), None)],
            return_type: None,
            body: Box::new(fail_expr(payload)),
        }),
    }
}

fn process_context(runtime_state: RuntimeState, process_id: ProcessId) -> Context {
    derive_child_env(
        &Context::new().with_runtime_state(runtime_state),
        ChildEnvProjection::new(process_id, 0),
    )
    .expect("process context projection should succeed")
}

async fn force_proc_in_context(ctx: Context, proc_value: Value) -> ash_interp::EvalResult<Value> {
    let mut call_ctx = ctx;
    call_ctx.set("p".to_string(), proc_value);
    eval_expr_async(
        &Expr::Call {
            func: "p".to_string(),
            module: None,
            arguments: vec![Expr::Literal(Value::Null)],
        },
        &call_ctx,
    )
    .await
}

fn force_proc_in_context_sync(ctx: Context, proc_value: Value) -> ash_interp::EvalResult<Value> {
    let mut call_ctx = ctx;
    call_ctx.set("p".to_string(), proc_value);
    eval_expr(
        &Expr::Call {
            func: "p".to_string(),
            module: None,
            arguments: vec![Expr::Literal(Value::Null)],
        },
        &call_ctx,
    )
}

async fn execute_ret_expr_in_context(
    workflow_expr: Expr,
    ctx: Context,
    runtime_state: &RuntimeState,
) -> ash_interp::ExecResult<Value> {
    let workflow = Workflow::Ret {
        expr: workflow_expr,
    };
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
async fn proc_yield_builds_proc_that_forces_to_null_unit() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("root process should register");
    let process_ctx = process_context(runtime_state.clone(), process_id);

    let proc_value = eval_expr(&proc_yield_expr(), &process_ctx)
        .expect("proc::yield should evaluate to a Proc closure");
    let Value::Closure { .. } = proc_value else {
        panic!("expected proc::yield to build a Proc closure, got {proc_value:?}");
    };

    let forced = force_proc_in_context(process_ctx, proc_value)
        .await
        .expect("forcing proc::yield should return the Unit value");
    assert_eq!(forced, Value::Null);
}

#[test]
fn proc_yield_sync_force_rejects_non_scheduler_thread_fallback() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    runtime
        .block_on(runtime_state.register_root_process(process_id))
        .expect("root process should register");
    let process_ctx = process_context(runtime_state, process_id);

    let proc_value = eval_expr(&proc_yield_expr(), &process_ctx)
        .expect("proc::yield should evaluate to a Proc closure");
    let err = force_proc_in_context_sync(process_ctx, proc_value)
        .expect_err("sync forcing proc::yield must reject non-scheduler fallback");

    assert_eq!(err, EvalError::ProcYieldRequiresAsyncRuntime);
}

#[tokio::test]
async fn execute_path_preserves_process_identity_across_proc_yield() {
    let runtime_state = RuntimeState::new();
    let process_id = ProcessId::new();
    runtime_state
        .register_root_process(process_id)
        .await
        .expect("root process should register");
    let process_ctx = process_context(runtime_state.clone(), process_id);

    let yielded_then_failed = Expr::Call {
        func: "bind".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![
            proc_yield_expr(),
            proc_fail_continuation(Value::String("after-yield".to_string())),
        ],
    };

    let proc_value =
        execute_ret_expr_in_context(yielded_then_failed, process_ctx.clone(), &runtime_state)
            .await
            .expect("workflow execute path should return the Proc built around proc::yield");
    let Value::Closure { .. } = proc_value else {
        panic!(
            "expected workflow Ret expression to evaluate to a Proc closure, got {proc_value:?}"
        );
    };

    let err = force_proc_in_context(process_ctx, proc_value)
        .await
        .expect_err(
            "forcing the yielded Proc should preserve process identity into later proc steps",
        );
    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected operational failure after proc::yield continuation, got {err:?}");
    };
    assert_eq!(failure.tower, TowerLevel::Proc);
    assert_eq!(failure.entity, FailureEntity::Process(process_id));
}

#[tokio::test]
async fn proc_yield_does_not_create_child_processes_or_consume_existing_handles() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    let child_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process should register");
    runtime_state
        .register_child_process(parent_process_id, child_process_id, 0)
        .await
        .expect("child process should register");
    runtime_state
        .record_process_terminal(
            child_process_id,
            ProcessTerminalState::Succeeded { value: Value::Null },
        )
        .await
        .expect("child terminal state should record");
    let process_ctx = process_context(runtime_state.clone(), parent_process_id);

    let before_children = runtime_state.process_children(parent_process_id).await;
    assert_eq!(before_children, vec![child_process_id]);

    let yielded = eval_expr(&proc_yield_expr(), &process_ctx)
        .expect("proc::yield should evaluate to a Proc closure");
    let forced = force_proc_in_context(process_ctx.clone(), yielded)
        .await
        .expect("forcing proc::yield should succeed");
    assert_eq!(forced, Value::Null);

    let after_children = runtime_state.process_children(parent_process_id).await;
    assert_eq!(
        after_children, before_children,
        "proc::yield must not register child processes or terminal child side effects"
    );

    let await_proc = eval_expr(
        &proc_await_expr(Value::ProcessHandle(ProcessHandle::new(
            child_process_id,
            Some("Null".to_string()),
        ))),
        &Context::new(),
    )
    .expect("proc::await should still build a Proc closure after proc::yield");
    let observed = force_proc_in_context(process_ctx, await_proc)
        .await
        .expect("proc::yield should not consume unrelated process handles");
    assert_eq!(observed, Value::Null);
}

proptest! {
    #![proptest_config(proptest::test_runner::Config {
        failure_persistence: None,
        ..proptest::test_runner::Config::default()
    })]

    #[test]
    fn proc_yield_preserves_registered_children_and_returns_null(seed in any::<u64>()) {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async move {
            let runtime_state = RuntimeState::new();
            let parent_process_id = ProcessId::new();
            let child_process_id = ProcessId::new();
            runtime_state
                .register_root_process(parent_process_id)
                .await
                .expect("parent process should register");
            runtime_state
                .register_child_process(parent_process_id, child_process_id, 0)
                .await
                .expect("child process should register");
            runtime_state
                .record_process_terminal(
                    child_process_id,
                    ProcessTerminalState::Succeeded {
                        value: Value::Int(seed as i64),
                    },
                )
                .await
                .expect("child terminal state should record");

            let process_ctx = process_context(runtime_state.clone(), parent_process_id);
            let before_children = runtime_state.process_children(parent_process_id).await;
            let yielded = eval_expr(&proc_yield_expr(), &process_ctx)
                .expect("proc::yield should evaluate to a Proc closure");
            let forced = force_proc_in_context(process_ctx.clone(), yielded)
                .await
                .expect("forcing proc::yield should succeed");
            let after_children = runtime_state.process_children(parent_process_id).await;

            prop_assert_eq!(forced, Value::Null);
            prop_assert_eq!(after_children, before_children);

            let await_proc = eval_expr(
                &proc_await_expr(Value::ProcessHandle(ProcessHandle::new(
                    child_process_id,
                    Some("Int".to_string()),
                ))),
                &Context::new(),
            )
            .expect("proc::await should still build a Proc closure after proc::yield");
            let observed = force_proc_in_context(process_ctx, await_proc)
                .await
                .expect("proc::yield should not consume unrelated process handles");
            prop_assert_eq!(observed, Value::Int(seed as i64));

            Ok(())
        })?;
    }
}
