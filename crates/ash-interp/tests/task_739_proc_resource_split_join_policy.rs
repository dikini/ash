use ash_core::runtime::{ProcessId, ProcessTerminalState};
use ash_core::{
    AccessPolicy, Expr, ProcessHandle, ResourceId, ResourceInstance, ResourceLifecycle,
    ResourceOwner, ResourceProvenance, ResourceRuntimeState, ResourceSplitJoinPolicy,
    ResourceTypeId, Value,
};
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
        arguments: vec![Expr::Literal(Value::list_from_vec(items)), mapper],
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

fn process_resource(
    owner: ProcessId,
    resource_type: &str,
    policy: ResourceSplitJoinPolicy,
) -> ResourceInstance {
    ResourceInstance::new(
        ResourceId::new(),
        ResourceTypeId::new(resource_type),
        ResourceOwner::Process(owner),
    )
    .with_state(ResourceRuntimeState::opaque(format!(
        "state:{resource_type}"
    )))
    .with_lifecycle(ResourceLifecycle::Active)
    .with_access_policy(AccessPolicy::ReadWrite)
    .with_split_join_policy(policy)
    .with_provenance(ResourceProvenance::internal(format!(
        "process {owner:?} owns {resource_type}"
    )))
}

fn extract_handles(value: Value) -> Vec<ProcessHandle> {
    let items = value
        .list_to_vec()
        .unwrap_or_else(|| panic!("expected process handle list, got {value:?}"));
    items
        .into_iter()
        .map(|item| match item {
            Value::ProcessHandle(handle) => handle,
            other => panic!("expected process handle, got {other:?}"),
        })
        .collect()
}

#[tokio::test]
async fn proc_par_rejects_non_shareable_process_resources_before_child_admission() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process should register");
    let resource = process_resource(
        parent_process_id,
        "ApplicationKV",
        ResourceSplitJoinPolicy::NonShareable,
    );
    let resource_id = resource.id;
    runtime_state.register_resource_instance(resource).await;
    let process_ctx = process_context(runtime_state.clone(), parent_process_id);

    let proc_value = eval_expr(
        &proc_par_expr(proc_unit_expr(Value::Int(1)), proc_unit_expr(Value::Int(2))),
        &process_ctx,
    )
    .expect("proc::par should build a Proc closure");

    let err = force_proc_in_context(process_ctx, proc_value)
        .await
        .expect_err("non-shareable process resource must block proc::par admission");
    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected operational failure, got {err:?}");
    };
    let rendered = format!("{failure:?}");
    assert!(rendered.contains("NonShareable"));
    assert!(rendered.contains("ApplicationKV"));
    assert!(rendered.contains(&format!("{resource_id:?}")));
    assert_eq!(
        runtime_state.process_children(parent_process_id).await,
        vec![]
    );
}

#[tokio::test]
async fn proc_scatter_rejects_non_shareable_resources_without_partial_child_admission() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process should register");
    runtime_state
        .register_resource_instance(process_resource(
            parent_process_id,
            "Mailbox",
            ResourceSplitJoinPolicy::NonShareable,
        ))
        .await;
    let process_ctx = process_context(runtime_state.clone(), parent_process_id);

    let proc_value = eval_expr(
        &proc_scatter_expr(vec![Value::Int(1), Value::Int(2)], echo_proc_mapper()),
        &process_ctx,
    )
    .expect("proc::scatter should build a Proc closure");

    let err = force_proc_in_context(process_ctx, proc_value)
        .await
        .expect_err("non-shareable process resource must block proc::scatter admission");
    let rendered = format!("{err:?}");
    assert!(rendered.contains("NonShareable"));
    assert!(rendered.contains("proc::scatter"));
    assert!(!rendered.contains("proc::par"));
    assert_eq!(
        runtime_state.process_children(parent_process_id).await,
        vec![]
    );
}

#[tokio::test]
async fn proc_par_allows_read_only_share_resources_and_records_split_lifecycle() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process should register");
    let resource = process_resource(
        parent_process_id,
        "ReadOnlyCache",
        ResourceSplitJoinPolicy::ReadOnlyShare,
    );
    let resource_id = resource.id;
    runtime_state.register_resource_instance(resource).await;
    let process_ctx = process_context(runtime_state.clone(), parent_process_id);

    let proc_value = eval_expr(
        &proc_par_expr(
            proc_unit_expr(Value::Int(11)),
            proc_unit_expr(Value::Int(22)),
        ),
        &process_ctx,
    )
    .expect("proc::par should build a Proc closure");

    let handles = extract_handles(
        force_proc_in_context(process_ctx, proc_value)
            .await
            .expect("shareable resource should permit child admission"),
    );
    assert_eq!(handles.len(), 2);
    assert_eq!(
        runtime_state
            .process_children(parent_process_id)
            .await
            .len(),
        2
    );
    let updated = runtime_state
        .resource_instance(resource_id)
        .await
        .expect("resource should remain registered");
    assert_eq!(updated.lifecycle, ResourceLifecycle::Splitting);
}

#[tokio::test]
async fn proc_join_applies_merge_policy_and_records_joined_lifecycle() {
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
    let mut resource = process_resource(
        parent_process_id,
        "MergeableKV",
        ResourceSplitJoinPolicy::Mergeable,
    );
    resource.lifecycle = ResourceLifecycle::Splitting;
    let resource_id = resource.id;
    runtime_state.register_resource_instance(resource).await;
    runtime_state
        .record_process_terminal(
            left_process_id,
            ProcessTerminalState::Succeeded {
                value: Value::Int(1),
            },
        )
        .await
        .expect("left child success should record");
    runtime_state
        .record_process_terminal(
            right_process_id,
            ProcessTerminalState::Succeeded {
                value: Value::Int(2),
            },
        )
        .await
        .expect("right child success should record");
    let process_ctx = process_context(runtime_state.clone(), parent_process_id);

    let proc_value = eval_expr(
        &proc_join_expr(
            ProcessHandle::new(left_process_id, Some("Int".to_string())),
            ProcessHandle::new(right_process_id, Some("Int".to_string())),
        ),
        &Context::new(),
    )
    .expect("proc::join should build a Proc closure");

    let joined = force_proc_in_context(process_ctx, proc_value)
        .await
        .expect("mergeable split resource should join successfully");
    assert!(matches!(joined, Value::Record(_)));
    let updated = runtime_state
        .resource_instance(resource_id)
        .await
        .expect("resource should remain registered");
    assert_eq!(updated.lifecycle, ResourceLifecycle::Joined);
}

#[tokio::test]
async fn proc_join_preserves_resource_policy_failure_evidence() {
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
    let mut resource = process_resource(
        parent_process_id,
        "UnmergeableClone",
        ResourceSplitJoinPolicy::BranchLocalClone,
    );
    resource.lifecycle = ResourceLifecycle::Splitting;
    let resource_id = resource.id;
    runtime_state.register_resource_instance(resource).await;
    runtime_state
        .record_process_terminal(
            left_process_id,
            ProcessTerminalState::Succeeded {
                value: Value::Int(1),
            },
        )
        .await
        .expect("left child success should record");
    runtime_state
        .record_process_terminal(
            right_process_id,
            ProcessTerminalState::Succeeded {
                value: Value::Int(2),
            },
        )
        .await
        .expect("right child success should record");
    let process_ctx = process_context(runtime_state, parent_process_id);

    let proc_value = eval_expr(
        &proc_join_expr(
            ProcessHandle::new(left_process_id, Some("Int".to_string())),
            ProcessHandle::new(right_process_id, Some("Int".to_string())),
        ),
        &Context::new(),
    )
    .expect("proc::join should build a Proc closure");

    let err = force_proc_in_context(process_ctx, proc_value)
        .await
        .expect_err("unmergeable split resource should fail join/gather merge policy");
    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected operational failure, got {err:?}");
    };
    assert!(format!("{failure:?}").contains(&format!("{resource_id:?}")));
    assert!(format!("{failure:?}").contains("BranchLocalClone"));
    assert!(
        failure
            .evidence
            .notes
            .iter()
            .any(|note| note.contains("UnmergeableClone"))
    );
}

proptest! {
    #[test]
    fn split_policy_property_rejects_only_non_crossable_mvp_policies(policy in prop_oneof![
        Just(ResourceSplitJoinPolicy::ReadOnlyShare),
        Just(ResourceSplitJoinPolicy::CommunicationOnly),
        Just(ResourceSplitJoinPolicy::Mergeable),
        Just(ResourceSplitJoinPolicy::NonShareable),
        Just(ResourceSplitJoinPolicy::BranchLocalClone),
        Just(ResourceSplitJoinPolicy::LinearMove),
    ]) {
        let can_cross_mvp = matches!(
            policy,
            ResourceSplitJoinPolicy::ReadOnlyShare
                | ResourceSplitJoinPolicy::CommunicationOnly
                | ResourceSplitJoinPolicy::Mergeable
        );
        prop_assert_eq!(can_cross_mvp, !matches!(
            policy,
            ResourceSplitJoinPolicy::NonShareable
                | ResourceSplitJoinPolicy::BranchLocalClone
                | ResourceSplitJoinPolicy::LinearMove
        ));
    }
}
