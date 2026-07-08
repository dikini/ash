use std::sync::Arc;

use ash_core::runtime::{ProcessId, ProcessTerminalState};
use ash_core::{
    AccessPolicy, CapabilityAuthorityProvenance, CapabilityBinding, CapabilityBindingId,
    CapabilityImplementationId, CapabilityInterfaceId, Effect, Expr, ProcessHandle, ResourceId,
    ResourceInstance, ResourceLifecycle, ResourceOwner, ResourceProvenance, ResourceRuntimeState,
    ResourceSplitJoinPolicy, ResourceTypeId, Value, WorkflowId,
};
use ash_interp::eval::{eval_expr, eval_expr_async};
use ash_interp::{
    ChildEnvProjection, Context, EntryOwnedResourceAdmission, EvalError,
    ImplementationBindingAdmission, ImplementationBindingDependencySource, MockProvider,
    RuntimeState, derive_child_env,
};

fn clock_binding(name: &str) -> CapabilityBinding {
    CapabilityBinding::host_provider(
        CapabilityBindingId::new(),
        name,
        CapabilityInterfaceId::new("Clock"),
        "clock",
        vec!["clock.now".to_string()],
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

#[tokio::test]
async fn runtime_admission_integrates_host_internal_and_derived_bindings_with_provenance() {
    let runtime_state = RuntimeState::new().with_provider(
        "clock",
        Arc::new(
            MockProvider::new("clock", Effect::Operational)
                .with_execute_result(Ok(Value::String("tick".to_string()))),
        ),
    );
    let workflow_id = WorkflowId::new();

    let resources = runtime_state
        .admit_entry_owned_resources(
            workflow_id,
            vec![EntryOwnedResourceAdmission::new(
                "store",
                ResourceTypeId::new("KvStore"),
            )],
        )
        .await
        .expect("entry-owned resource admission should succeed");
    let store_id = resources["store"];
    let store = runtime_state
        .resource_instance(store_id)
        .await
        .expect("admitted resource is stored");
    assert_eq!(store.owner, ResourceOwner::Workflow(workflow_id));
    assert_eq!(store.lifecycle, ResourceLifecycle::Admitted);
    assert!(matches!(
        store.provenance,
        ResourceProvenance::InternalAuthority { .. }
    ));

    let clock = clock_binding("entry-clock");
    let clock_id = clock.id;
    runtime_state
        .admit_capability_binding(clock)
        .await
        .expect("host binding admission should succeed");

    let projected = runtime_state
        .create_capability_context_for_bindings(&[clock_id])
        .await
        .expect("host binding should project to capability context");
    assert_eq!(
        projected.execute("entry-clock", "now", &[]).await,
        Ok(Value::String("tick".to_string()))
    );

    runtime_state
        .register_capability_interface_operations(CapabilityInterfaceId::new("KeyValue"), ["get"])
        .await
        .expect("interface surface registration should succeed");

    let derived_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "kv-binding",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("KvImpl"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("KvStore"),
            ))
            .with_dependency(ImplementationBindingDependencySource::capability(
                "clock",
                "entry-clock",
                CapabilityInterfaceId::new("Clock"),
            ))
            .with_requested_operations(["get"]),
            &resources,
        )
        .await
        .expect("derived implementation binding should admit from explicit sources");

    let derived = runtime_state
        .capability_binding(derived_id)
        .await
        .expect("derived binding is stored");
    assert_eq!(derived.name, "kv-binding");
    assert_eq!(derived.dependencies.len(), 2);
    match derived.authority {
        CapabilityAuthorityProvenance::DerivedAuthority {
            dependency_names,
            notes,
        } => {
            assert_eq!(
                dependency_names,
                vec!["store".to_string(), "clock".to_string()]
            );
            let evidence = notes.join("\n");
            assert!(evidence.contains(&store_id.0.to_string()), "{evidence}");
            assert!(evidence.contains(&clock_id.0.to_string()), "{evidence}");
            assert!(
                evidence.contains("operation get derives from store, clock"),
                "{evidence}"
            );
        }
        other => panic!("expected derived authority, got {other:?}"),
    }
}

#[tokio::test]
async fn runtime_integration_rejects_missing_resources_and_authority_widening_without_partial_binding()
 {
    let runtime_state = RuntimeState::new().with_provider(
        "clock",
        Arc::new(MockProvider::new("clock", Effect::Operational)),
    );
    let clock = clock_binding("entry-clock");
    runtime_state
        .admit_capability_binding(clock)
        .await
        .expect("host binding admission should succeed");
    runtime_state
        .register_capability_interface_operations(CapabilityInterfaceId::new("KeyValue"), ["get"])
        .await
        .expect("interface surface registration should succeed");

    let missing_resource = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "kv-missing-resource",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("KvImpl"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("KvStore"),
            ))
            .with_dependency(ImplementationBindingDependencySource::capability(
                "clock",
                "entry-clock",
                CapabilityInterfaceId::new("Clock"),
            ))
            .with_requested_operations(["get"]),
            &std::collections::HashMap::new(),
        )
        .await
        .expect_err("resource dependencies must come from explicit owned-resource map");
    assert!(
        missing_resource
            .to_string()
            .contains("missing resource dependency source 'store'"),
        "unexpected error: {missing_resource}"
    );

    let workflow_id = WorkflowId::new();
    let resources = runtime_state
        .admit_entry_owned_resources(
            workflow_id,
            vec![EntryOwnedResourceAdmission::new(
                "store",
                ResourceTypeId::new("KvStore"),
            )],
        )
        .await
        .expect("resource admission should succeed");

    let widening = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "kv-widening",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("KvImpl"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("KvStore"),
            ))
            .with_requested_operations(["delete"]),
            &resources,
        )
        .await
        .expect_err("requested operations outside registered surface must be rejected");
    assert!(
        widening
            .to_string()
            .contains("requested operation 'delete' is outside registered interface KeyValue"),
        "unexpected error: {widening}"
    );
    assert_eq!(
        runtime_state
            .capability_binding_by_name("kv-missing-resource")
            .await,
        None
    );
    assert_eq!(
        runtime_state
            .capability_binding_by_name("kv-widening")
            .await,
        None
    );
    assert_eq!(runtime_state.capability_binding_count().await, 1);
}

#[tokio::test]
async fn proc_split_join_integration_rejects_nonshareable_and_preserves_resource_evidence() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process registration should succeed");
    let blocked = process_resource(
        parent_process_id,
        "ExclusiveMailbox",
        ResourceSplitJoinPolicy::NonShareable,
    );
    let blocked_id = blocked.id;
    runtime_state.register_resource_instance(blocked).await;
    let process_ctx = process_context(runtime_state.clone(), parent_process_id);

    let proc_value = eval_expr(
        &proc_par_expr(proc_unit_expr(Value::Int(1)), proc_unit_expr(Value::Int(2))),
        &process_ctx,
    )
    .expect("proc::par should build a Proc closure");

    let err = force_proc_in_context(process_ctx, proc_value)
        .await
        .expect_err("non-shareable resource should reject process split");
    let EvalError::OperationalFailure(failure) = err else {
        panic!("expected operational failure, got {err:?}");
    };
    let rendered = format!("{failure:?}");
    assert!(rendered.contains("ExclusiveMailbox"), "{rendered}");
    assert!(rendered.contains("NonShareable"), "{rendered}");
    assert!(rendered.contains(&blocked_id.0.to_string()), "{rendered}");
    assert!(rendered.contains("proc::par"), "{rendered}");
    assert_eq!(
        runtime_state.process_children(parent_process_id).await,
        vec![]
    );
}

#[tokio::test]
async fn proc_split_join_integration_allows_read_only_share_and_mergeable_join() {
    let runtime_state = RuntimeState::new();
    let parent_process_id = ProcessId::new();
    runtime_state
        .register_root_process(parent_process_id)
        .await
        .expect("parent process registration should succeed");

    let readonly = process_resource(
        parent_process_id,
        "ReadOnlyCache",
        ResourceSplitJoinPolicy::ReadOnlyShare,
    );
    let readonly_id = readonly.id;
    runtime_state.register_resource_instance(readonly).await;
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
        force_proc_in_context(process_ctx.clone(), proc_value)
            .await
            .expect("read-only share resource should allow process split"),
    );
    assert_eq!(handles.len(), 2);
    assert_eq!(
        runtime_state
            .resource_instance(readonly_id)
            .await
            .expect("read-only resource remains registered")
            .lifecycle,
        ResourceLifecycle::Splitting
    );

    runtime_state
        .record_process_terminal(
            handles[0].process_id,
            ProcessTerminalState::Succeeded {
                value: Value::Int(11),
            },
        )
        .await
        .expect("left terminal state should be recorded");
    runtime_state
        .record_process_terminal(
            handles[1].process_id,
            ProcessTerminalState::Succeeded {
                value: Value::Int(22),
            },
        )
        .await
        .expect("right terminal state should be recorded");
    let join_proc = eval_expr(
        &proc_join_expr(handles[0].clone(), handles[1].clone()),
        &Context::new(),
    )
    .expect("proc::join should build a Proc closure");
    let joined = force_proc_in_context(process_ctx.clone(), join_proc)
        .await
        .expect("join should observe both children");
    assert!(matches!(joined, Value::Record(_)));
    assert_eq!(
        runtime_state
            .resource_instance(readonly_id)
            .await
            .expect("read-only resource remains registered")
            .lifecycle,
        ResourceLifecycle::Active,
        "read-only shared resources should return to active after join observation"
    );

    let mergeable = process_resource(
        parent_process_id,
        "MergeableKV",
        ResourceSplitJoinPolicy::Mergeable,
    );
    let mergeable_id = mergeable.id;
    runtime_state.register_resource_instance(mergeable).await;
    runtime_state
        .apply_process_resource_split(parent_process_id, 2, "integration-mergeable-split")
        .await
        .expect("mergeable resource should allow explicit split tracking");
    let left_process_id = ProcessId::new();
    let right_process_id = ProcessId::new();
    runtime_state
        .register_child_process(parent_process_id, left_process_id, 2)
        .await
        .expect("left merge child registration should succeed");
    runtime_state
        .register_child_process(parent_process_id, right_process_id, 3)
        .await
        .expect("right merge child registration should succeed");
    runtime_state
        .record_process_terminal(
            left_process_id,
            ProcessTerminalState::Succeeded {
                value: Value::Int(101),
            },
        )
        .await
        .expect("left merge child terminal state should be recorded");
    runtime_state
        .record_process_terminal(
            right_process_id,
            ProcessTerminalState::Succeeded {
                value: Value::Int(202),
            },
        )
        .await
        .expect("right merge child terminal state should be recorded");
    let merge_join_proc = eval_expr(
        &proc_join_expr(
            ProcessHandle::new(left_process_id, Some("Int".to_string())),
            ProcessHandle::new(right_process_id, Some("Int".to_string())),
        ),
        &Context::new(),
    )
    .expect("proc::join should build a merge Proc closure");
    let merge_join = force_proc_in_context(process_ctx.clone(), merge_join_proc)
        .await
        .expect("mergeable join should succeed");
    assert!(matches!(merge_join, Value::Record(_)));
    let mergeable_after = runtime_state
        .resource_instance(mergeable_id)
        .await
        .expect("mergeable resource remains registered");
    assert_eq!(mergeable_after.lifecycle, ResourceLifecycle::Joined);
    let ResourceProvenance::InternalAuthority { notes } = mergeable_after.provenance else {
        panic!("expected internal provenance after mergeable join");
    };
    assert!(
        notes.iter().any(|note| note.contains("MergeableKV")),
        "original internal provenance should remain preserved after join: {notes:?}"
    );
}
