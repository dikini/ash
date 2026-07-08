use ash_core::runtime::{
    FailureBoundary, FailureEntity, OperationalFailure, ProcessId, ProcessTerminalState,
};
use ash_core::{Expr, ProcessHandle, Value};
use ash_interp::eval::{eval_expr, eval_expr_async};
use ash_interp::{ChildEnvProjection, Context, EvalError, RuntimeState, derive_child_env};
use proptest::prelude::*;
use tokio::time::{Duration, sleep};

fn proc_failure(process_id: ProcessId, message: &str) -> OperationalFailure {
    OperationalFailure::new(
        FailureBoundary::Process,
        FailureEntity::Process(process_id),
        Value::String(message.to_string()),
        "String",
    )
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

fn proc_gather_expr(handles: Vec<ProcessHandle>) -> Expr {
    Expr::Call {
        func: "gather".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![Expr::Literal(Value::list_from_vec(
            handles.into_iter().map(Value::ProcessHandle).collect(),
        ))],
    }
}

fn extract_list(value: Value) -> Vec<Value> {
    value
        .list_to_vec()
        .unwrap_or_else(|| panic!("expected list value, got {value:?}"))
}

#[tokio::test]
async fn proc_join_waits_for_all_children_even_when_one_fails_early() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    let left_process_id = ProcessId::new();
    let right_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process should register");
    runtime_state
        .register_child_process(parent_process_id, left_process_id, 0)
        .await
        .expect("left child should register");
    runtime_state
        .register_child_process(parent_process_id, right_process_id, 1)
        .await
        .expect("right child should register");
    runtime_state
        .record_process_terminal(
            left_process_id,
            ProcessTerminalState::Failed {
                process_id: left_process_id,
                failure: Box::new(proc_failure(left_process_id, "left boom")),
            },
        )
        .await
        .expect("left child failure should record");

    let delayed_runtime = runtime_state.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(25)).await;
        delayed_runtime
            .record_process_terminal(
                right_process_id,
                ProcessTerminalState::Succeeded {
                    value: Value::Int(41),
                },
            )
            .await
            .expect("right child success should record");
    });

    let proc_value = eval_expr(
        &proc_join_expr(
            ProcessHandle::new(left_process_id, Some("Int".to_string())),
            ProcessHandle::new(right_process_id, Some("Int".to_string())),
        ),
        &Context::new(),
    )
    .expect("proc::join should build a Proc closure");

    let err = force_proc_in_context(
        process_context(runtime_state.clone(), parent_process_id),
        proc_value,
    )
    .await
    .expect_err("join should surface failure only after both children become terminal");

    assert!(matches!(err, EvalError::OperationalFailure(_)));
    assert!(
        runtime_state
            .process_terminal_state(right_process_id)
            .await
            .is_some(),
        "join must not fail fast before the still-running sibling reaches terminal state"
    );
}

#[tokio::test]
async fn proc_join_surfaces_single_child_failure_unchanged() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    let left_process_id = ProcessId::new();
    let right_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process should register");
    runtime_state
        .register_child_process(parent_process_id, left_process_id, 0)
        .await
        .expect("left child should register");
    runtime_state
        .register_child_process(parent_process_id, right_process_id, 1)
        .await
        .expect("right child should register");
    runtime_state
        .record_process_terminal(
            left_process_id,
            ProcessTerminalState::Failed {
                process_id: left_process_id,
                failure: Box::new(proc_failure(left_process_id, "left boom")),
            },
        )
        .await
        .expect("left child failure should record");
    runtime_state
        .record_process_terminal(
            right_process_id,
            ProcessTerminalState::Succeeded {
                value: Value::Int(41),
            },
        )
        .await
        .expect("right child success should record");

    let proc_value = eval_expr(
        &proc_join_expr(
            ProcessHandle::new(left_process_id, Some("Int".to_string())),
            ProcessHandle::new(right_process_id, Some("Int".to_string())),
        ),
        &Context::new(),
    )
    .expect("proc::join should build a Proc closure");

    let err = force_proc_in_context(
        process_context(runtime_state, parent_process_id),
        proc_value,
    )
    .await
    .expect_err("join should surface the single child failure directly");

    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected operational failure, got {err:?}");
    };
    assert_eq!(failure.entity, FailureEntity::Process(left_process_id));
    assert_eq!(failure.payload, Value::String("left boom".to_string()));
    assert!(
        failure.cause.is_none(),
        "single-child failure should not be wrapped in an aggregate cause chain"
    );
}

#[tokio::test]
async fn proc_join_aggregates_multiple_failures_and_preserves_both_source_process_ids() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    let left_process_id = ProcessId::new();
    let right_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process should register");
    runtime_state
        .register_child_process(parent_process_id, left_process_id, 0)
        .await
        .expect("left child should register");
    runtime_state
        .register_child_process(parent_process_id, right_process_id, 1)
        .await
        .expect("right child should register");
    runtime_state
        .record_process_terminal(
            left_process_id,
            ProcessTerminalState::Failed {
                process_id: left_process_id,
                failure: Box::new(proc_failure(left_process_id, "left boom")),
            },
        )
        .await
        .expect("left child failure should record");
    runtime_state
        .record_process_terminal(
            right_process_id,
            ProcessTerminalState::Failed {
                process_id: right_process_id,
                failure: Box::new(proc_failure(right_process_id, "right boom")),
            },
        )
        .await
        .expect("right child failure should record");

    let proc_value = eval_expr(
        &proc_join_expr(
            ProcessHandle::new(left_process_id, Some("Int".to_string())),
            ProcessHandle::new(right_process_id, Some("Int".to_string())),
        ),
        &Context::new(),
    )
    .expect("proc::join should build a Proc closure");

    let err = force_proc_in_context(
        process_context(runtime_state, parent_process_id),
        proc_value,
    )
    .await
    .expect_err("join should aggregate multiple child failures");
    let rendered = format!("{err:?}");
    assert!(rendered.contains(&format!("{left_process_id:?}")));
    assert!(rendered.contains(&format!("{right_process_id:?}")));
}

#[tokio::test]
async fn proc_gather_aggregates_multiple_failures_and_preserves_all_source_process_ids() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    let first_process_id = ProcessId::new();
    let second_process_id = ProcessId::new();
    let third_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process should register");
    for (index, child_process_id) in [first_process_id, second_process_id, third_process_id]
        .into_iter()
        .enumerate()
    {
        runtime_state
            .register_child_process(parent_process_id, child_process_id, index)
            .await
            .expect("child should register");
    }
    runtime_state
        .record_process_terminal(
            first_process_id,
            ProcessTerminalState::Failed {
                process_id: first_process_id,
                failure: Box::new(proc_failure(first_process_id, "first boom")),
            },
        )
        .await
        .expect("first child failure should record");
    runtime_state
        .record_process_terminal(
            second_process_id,
            ProcessTerminalState::Succeeded {
                value: Value::Int(22),
            },
        )
        .await
        .expect("second child success should record");
    runtime_state
        .record_process_terminal(
            third_process_id,
            ProcessTerminalState::Failed {
                process_id: third_process_id,
                failure: Box::new(proc_failure(third_process_id, "third boom")),
            },
        )
        .await
        .expect("third child failure should record");

    let proc_value = eval_expr(
        &proc_gather_expr(vec![
            ProcessHandle::new(first_process_id, Some("Int".to_string())),
            ProcessHandle::new(second_process_id, Some("Int".to_string())),
            ProcessHandle::new(third_process_id, Some("Int".to_string())),
        ]),
        &Context::new(),
    )
    .expect("proc::gather should build a Proc closure");

    let err = force_proc_in_context(
        process_context(runtime_state, parent_process_id),
        proc_value,
    )
    .await
    .expect_err("gather should aggregate multiple child failures");
    let rendered = format!("{err:?}");
    assert!(rendered.contains(&format!("{first_process_id:?}")));
    assert!(rendered.contains(&format!("{third_process_id:?}")));
}

proptest! {
    #![proptest_config(proptest::test_runner::Config {
        failure_persistence: None,
        ..proptest::test_runner::Config::default()
    })]

    #[test]
    fn proc_gather_preserves_input_order_for_successes(values in proptest::collection::vec(any::<i16>(), 0..6)) {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async move {
            let runtime_state = RuntimeState::new();
            let parent_process_id = ProcessId::new();
            runtime_state
                .register_root_process(parent_process_id)
                .await
                .expect("parent process should register");

            let mut handles = Vec::with_capacity(values.len());
            for (index, value) in values.iter().enumerate() {
                let child_process_id = ProcessId::new();
                runtime_state
                    .register_child_process(parent_process_id, child_process_id, index)
                    .await
                    .expect("child should register");
                handles.push(ProcessHandle::new(child_process_id, Some("Int".to_string())));
                runtime_state
                    .record_process_terminal(
                        child_process_id,
                        ProcessTerminalState::Succeeded {
                            value: Value::Int(i64::from(*value)),
                        },
                    )
                    .await
                    .expect("child success should record");
            }

            let proc_value = eval_expr(&proc_gather_expr(handles), &Context::new())
                .expect("proc::gather should build a Proc closure");
            let observed = force_proc_in_context(process_context(runtime_state, parent_process_id), proc_value)
                .await
                .expect("gather should return ordered child successes");

            prop_assert_eq!(
                extract_list(observed),
                values
                    .into_iter()
                    .map(|value| Value::Int(i64::from(value)))
                    .collect::<Vec<_>>()
            );
            Ok(())
        })?;
    }
}
