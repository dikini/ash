use ash_core::runtime::{FailureEntity, ProcessId};
use ash_core::{Expr, ProcessHandle, Value};
use ash_interp::eval::{eval_expr, eval_expr_async};
use ash_interp::{ChildEnvProjection, Context, EvalError, RuntimeState, derive_child_env};
use proptest::prelude::*;

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

fn proc_unit_expr(value: Value) -> Expr {
    Expr::Call {
        func: "unit".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![Expr::Literal(value)],
    }
}

fn proc_par_expr(left: Expr, right: Expr) -> Expr {
    Expr::Call {
        func: "par".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![left, right],
    }
}

fn proc_scatter_expr(items: Vec<Value>, mapper: Expr) -> Expr {
    Expr::Call {
        func: "scatter".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![Expr::Literal(Value::List(Box::new(items))), mapper],
    }
}

fn proc_await_expr(handle: Value) -> Expr {
    Expr::Call {
        func: "await".to_string(),
        module: Some("proc".to_string()),
        arguments: vec![Expr::Literal(handle)],
    }
}

fn immediate_failing_proc(payload: Value) -> Expr {
    Expr::FnDef {
        params: vec![("__proc_env".to_string(), None)],
        return_type: None,
        body: Box::new(Expr::Fail {
            payload: Box::new(Expr::Literal(payload)),
        }),
    }
}

fn echo_proc_mapper() -> Expr {
    Expr::FnDef {
        params: vec![("item".to_string(), None)],
        return_type: None,
        body: Box::new(Expr::Call {
            func: "unit".to_string(),
            module: Some("proc".to_string()),
            arguments: vec![Expr::Variable {
                name: "item".to_string(),
                span: ash_core::ast::Span::default(),
            }],
        }),
    }
}

fn extract_process_handles(value: Value) -> Vec<ProcessHandle> {
    let Value::List(items) = value else {
        panic!("expected handle list/tuple value, got {value:?}");
    };
    items
        .into_iter()
        .map(|item| match item {
            Value::ProcessHandle(handle) => handle,
            other => panic!("expected process handle element, got {other:?}"),
        })
        .collect()
}

async fn wait_for_terminal_children(runtime_state: &RuntimeState, handles: &[ProcessHandle]) {
    tokio::time::timeout(std::time::Duration::from_secs(1), async {
        loop {
            let mut all_ready = true;
            for handle in handles {
                if runtime_state
                    .process_terminal_state(handle.process_id)
                    .await
                    .is_none()
                {
                    all_ready = false;
                    break;
                }
            }
            if all_ready {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("child processes did not reach terminal state within bounded wall-clock time");
}

#[tokio::test]
async fn proc_par_returns_ordered_child_handles_and_registers_matching_children() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process should register");
    let process_ctx = process_context(runtime_state.clone(), parent_process_id);

    let proc_value = eval_expr(
        &proc_par_expr(
            proc_unit_expr(Value::Int(11)),
            proc_unit_expr(Value::Int(22)),
        ),
        &process_ctx,
    )
    .expect("proc::par should evaluate to a Proc closure");

    let forced_handles = force_proc_in_context(process_ctx.clone(), proc_value)
        .await
        .expect("forcing proc::par should return ordered child handles");
    let handles = extract_process_handles(forced_handles.clone());
    assert_eq!(
        handles.len(),
        2,
        "proc::par should return exactly two handles"
    );

    let left_handle = eval_expr(
        &Expr::FieldAccess {
            expr: Box::new(Expr::Literal(forced_handles.clone())),
            field: "0".into(),
        },
        &Context::new(),
    )
    .expect("tuple-style .0 access should project the first proc::par handle");
    let right_handle = eval_expr(
        &Expr::FieldAccess {
            expr: Box::new(Expr::Literal(forced_handles)),
            field: "1".into(),
        },
        &Context::new(),
    )
    .expect("tuple-style .1 access should project the second proc::par handle");
    assert_eq!(left_handle, Value::ProcessHandle(handles[0].clone()));
    assert_eq!(right_handle, Value::ProcessHandle(handles[1].clone()));

    let children = runtime_state.process_children(parent_process_id).await;
    assert_eq!(children.len(), 2, "proc::par should register two children");
    assert_eq!(children[0], handles[0].process_id);
    assert_eq!(children[1], handles[1].process_id);

    for (expected_index, handle) in handles.iter().enumerate() {
        let record = runtime_state
            .process_record(handle.process_id)
            .await
            .expect("child process record should exist");
        assert_eq!(record.parent_process_id, Some(parent_process_id));
        assert_eq!(record.child_index, Some(expected_index));
    }
    wait_for_terminal_children(&runtime_state, &handles).await;

    let observed_left = force_proc_in_context(
        process_ctx.clone(),
        eval_expr(
            &proc_await_expr(Value::ProcessHandle(handles[0].clone())),
            &Context::new(),
        )
        .expect("proc::await should build left observer"),
    )
    .await
    .expect("left child should complete successfully");
    let observed_right = force_proc_in_context(
        process_ctx,
        eval_expr(
            &proc_await_expr(Value::ProcessHandle(handles[1].clone())),
            &Context::new(),
        )
        .expect("proc::await should build right observer"),
    )
    .await
    .expect("right child should complete successfully");

    assert_eq!(observed_left, Value::Int(11));
    assert_eq!(observed_right, Value::Int(22));
}

#[test]
fn proc_admission_sync_force_rejects_non_runtime_fallback() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    runtime
        .block_on(runtime_state.register_root_process(parent_process_id))
        .expect("parent process should register");
    let process_ctx = process_context(runtime_state, parent_process_id);

    let proc_value = eval_expr(
        &proc_par_expr(proc_unit_expr(Value::Int(1)), proc_unit_expr(Value::Int(2))),
        &process_ctx,
    )
    .expect("proc::par should evaluate to a Proc closure");
    let err = force_proc_in_context_sync(process_ctx, proc_value)
        .expect_err("sync forcing proc child admission must reject non-runtime fallback");

    assert_eq!(err, EvalError::ProcAdmissionRequiresAsyncRuntime);
}

#[tokio::test]
async fn proc_par_child_failure_is_observed_through_await_after_handle_return() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process should register");
    let process_ctx = process_context(runtime_state.clone(), parent_process_id);

    let proc_value = eval_expr(
        &proc_par_expr(
            proc_unit_expr(Value::Int(7)),
            immediate_failing_proc(Value::String("boom".to_string())),
        ),
        &process_ctx,
    )
    .expect("proc::par should evaluate to a Proc closure");

    let handles = extract_process_handles(
        force_proc_in_context(process_ctx.clone(), proc_value)
            .await
            .expect("proc::par admission should return handles before child failure is observed"),
    );
    assert_eq!(
        runtime_state
            .process_children(parent_process_id)
            .await
            .len(),
        2
    );
    wait_for_terminal_children(&runtime_state, &handles).await;

    let success = force_proc_in_context(
        process_ctx.clone(),
        eval_expr(
            &proc_await_expr(Value::ProcessHandle(handles[0].clone())),
            &Context::new(),
        )
        .expect("proc::await should build success observer"),
    )
    .await
    .expect("successful child should still be awaitable");
    assert_eq!(success, Value::Int(7));

    let err = force_proc_in_context(
        process_ctx,
        eval_expr(
            &proc_await_expr(Value::ProcessHandle(handles[1].clone())),
            &Context::new(),
        )
        .expect("proc::await should build failure observer"),
    )
    .await
    .expect_err("failing child should surface only when observed later");
    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected operational failure from awaited child, got {err:?}");
    };
    assert_eq!(
        failure.entity,
        FailureEntity::Process(handles[1].process_id)
    );
}

#[tokio::test]
async fn proc_scatter_returns_handles_in_input_order() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process should register");
    let process_ctx = process_context(runtime_state.clone(), parent_process_id);

    let proc_value = eval_expr(
        &proc_scatter_expr(
            vec![Value::Int(3), Value::Int(5), Value::Int(8)],
            echo_proc_mapper(),
        ),
        &process_ctx,
    )
    .expect("proc::scatter should evaluate to a Proc closure");

    let handles = extract_process_handles(
        force_proc_in_context(process_ctx.clone(), proc_value)
            .await
            .expect("forcing proc::scatter should return ordered child handles"),
    );
    assert_eq!(handles.len(), 3);

    let children = runtime_state.process_children(parent_process_id).await;
    assert_eq!(children.len(), 3);
    assert_eq!(
        children,
        handles
            .iter()
            .map(|handle| handle.process_id)
            .collect::<Vec<_>>()
    );
    wait_for_terminal_children(&runtime_state, &handles).await;

    for (idx, expected) in [3_i64, 5, 8].into_iter().enumerate() {
        let observed = force_proc_in_context(
            process_ctx.clone(),
            eval_expr(
                &proc_await_expr(Value::ProcessHandle(handles[idx].clone())),
                &Context::new(),
            )
            .expect("proc::await should build scatter observer"),
        )
        .await
        .expect("scatter child should complete successfully");
        assert_eq!(observed, Value::Int(expected));
    }
}

#[tokio::test]
async fn proc_par_admission_failure_leaves_no_partially_registered_children() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process should register");
    runtime_state
        .record_process_terminal(
            parent_process_id,
            ash_core::runtime::ProcessTerminalState::Succeeded { value: Value::Null },
        )
        .await
        .expect("parent terminal state should record");
    let process_ctx = process_context(runtime_state.clone(), parent_process_id);

    let proc_value = eval_expr(
        &proc_par_expr(proc_unit_expr(Value::Int(1)), proc_unit_expr(Value::Int(2))),
        &process_ctx,
    )
    .expect("proc::par should still build a Proc closure before forcing");

    let err = force_proc_in_context(process_ctx, proc_value)
        .await
        .expect_err("terminal parent should reject child admission");
    assert!(
        matches!(err, EvalError::ExecutionFailed(ref message) if message.contains("register") || message.contains("terminal")),
        "expected honest admission failure, got {err:?}"
    );
    assert!(
        runtime_state
            .process_children(parent_process_id)
            .await
            .is_empty(),
        "failed admission must not leave partially registered children"
    );
}

proptest! {
    #![proptest_config(proptest::test_runner::Config {
        failure_persistence: None,
        ..proptest::test_runner::Config::default()
    })]

    #[test]
    fn proc_scatter_preserves_input_order(values in proptest::collection::vec(any::<i16>(), 0..6)) {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async move {
            let runtime_state = RuntimeState::new();
            let parent_process_id = ProcessId::new();
            runtime_state
                .register_root_process(parent_process_id)
                .await
                .expect("parent process should register");
            let process_ctx = process_context(runtime_state.clone(), parent_process_id);
            let items = values.iter().map(|v| Value::Int(i64::from(*v))).collect::<Vec<_>>();

            let proc_value = eval_expr(&proc_scatter_expr(items, echo_proc_mapper()), &process_ctx)
                .expect("proc::scatter should evaluate to a Proc closure");
            let handles = extract_process_handles(
                force_proc_in_context(process_ctx.clone(), proc_value)
                    .await
                    .expect("forcing proc::scatter should return handles"),
            );
            let children = runtime_state.process_children(parent_process_id).await;

            prop_assert_eq!(children, handles.iter().map(|handle| handle.process_id).collect::<Vec<_>>());
            wait_for_terminal_children(&runtime_state, &handles).await;

            for (idx, expected) in values.iter().enumerate() {
                let observed = force_proc_in_context(
                    process_ctx.clone(),
                    eval_expr(
                        &proc_await_expr(Value::ProcessHandle(handles[idx].clone())),
                        &Context::new(),
                    )
                    .expect("proc::await should build scatter observer"),
                )
                .await
                .expect("scatter child should complete successfully");
                prop_assert_eq!(observed, Value::Int(i64::from(*expected)));
            }

            Ok(())
        })?;
    }
}
