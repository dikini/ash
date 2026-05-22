use std::sync::Arc;

use ash_core::{
    Capability, CapabilityAuthorityProvenance, CapabilityBinding, CapabilityBindingDependency,
    CapabilityBindingId, CapabilityImplementationId, CapabilityInterfaceId, Effect, Guard, Pattern,
    Provenance, ResourceId, ResourceInstance, ResourceOwner, ResourceTypeId, Value, Workflow,
    WorkflowId,
};
use ash_interp::{
    ActEnv, MockProvider, PolicyEvaluator, RuntimeState, execute_with_bindings_in_state,
};

fn host_binding(name: &str, provider_name: &str, admitted: Vec<&str>) -> CapabilityBinding {
    host_binding_with_id(CapabilityBindingId::new(), name, provider_name, admitted)
}

fn host_binding_with_id(
    id: CapabilityBindingId,
    name: &str,
    provider_name: &str,
    admitted: Vec<&str>,
) -> CapabilityBinding {
    CapabilityBinding::host_provider(
        id,
        name,
        CapabilityInterfaceId::new("Clock"),
        provider_name,
        admitted.into_iter().map(str::to_string).collect(),
    )
}

#[tokio::test]
async fn runtime_state_admits_and_projects_host_capability_binding_by_id() {
    let runtime_state = RuntimeState::new().with_provider(
        "clock",
        Arc::new(
            MockProvider::new("clock", Effect::Operational)
                .with_execute_result(Ok(Value::String("tick".to_string()))),
        ),
    );
    let binding = host_binding("workflow-clock", "clock", vec!["clock.now"]);
    let binding_id = binding.id;

    runtime_state
        .admit_capability_binding(binding.clone())
        .await
        .expect("host binding admission succeeds for registered provider");

    assert!(runtime_state.has_capability_binding(binding_id).await);
    assert_eq!(
        runtime_state.capability_binding(binding_id).await,
        Some(binding)
    );
    assert_eq!(
        runtime_state
            .capability_binding_by_name("workflow-clock")
            .await
            .map(|binding| binding.id),
        Some(binding_id)
    );
    assert_eq!(runtime_state.capability_binding_count().await, 1);

    let projected = runtime_state
        .create_capability_context_for_bindings(&[binding_id])
        .await
        .expect("projection succeeds for admitted host binding");

    assert_eq!(
        projected.execute("workflow-clock", "now", &[]).await,
        Ok(Value::String("tick".to_string()))
    );
    assert!(
        projected
            .execute("workflow-clock", "later", &[])
            .await
            .is_err()
    );
    assert!(
        projected.execute("clock", "now", &[]).await.is_err(),
        "binding projection is alias-only; provider registry names are not an admitted dispatch alias"
    );
}

#[tokio::test]
async fn runtime_state_projection_rejects_unadmitted_binding_ids_and_does_not_use_ambient_provider_registry()
 {
    let runtime_state = RuntimeState::new().with_provider(
        "clock",
        Arc::new(
            MockProvider::new("clock", Effect::Operational)
                .with_execute_result(Ok(Value::String("ambient".to_string()))),
        ),
    );
    let unadmitted_id = CapabilityBindingId::new();

    let result = runtime_state
        .create_capability_context_for_bindings(&[unadmitted_id])
        .await;
    let err = match result {
        Ok(_) => panic!("unadmitted binding ids must not project ambient providers"),
        Err(err) => err,
    };
    assert!(err.to_string().contains("unadmitted capability binding"));

    let empty_projected = runtime_state
        .create_capability_context_for_bindings(&[])
        .await
        .expect("empty explicit admission set is valid");
    assert!(
        empty_projected
            .execute("workflow-clock", "now", &[])
            .await
            .is_err()
    );
}

#[tokio::test]
async fn workflow_observe_with_admitted_bindings_does_not_fall_back_to_unadmitted_behaviour_provider()
 {
    let runtime_state = RuntimeState::new()
        .with_provider(
            "Args:0",
            Arc::new(
                MockProvider::new("Args:0", Effect::Epistemic)
                    .with_observe_value(Value::String("ambient".to_string())),
            ),
        )
        .with_provider(
            "clock",
            Arc::new(
                MockProvider::new("clock", Effect::Operational)
                    .with_execute_result(Ok(Value::String("tick".to_string()))),
            ),
        );
    runtime_state
        .admit_capability_binding(host_binding("workflow-clock", "clock", vec!["clock.now"]))
        .await
        .expect("unrelated clock binding should be admitted");

    let workflow = Workflow::Observe {
        capability: Capability {
            name: "Args:0".to_string(),
            effect: Effect::Epistemic,
            constraints: vec![],
        },
        pattern: Pattern::Wildcard,
        continuation: Box::new(Workflow::Done),
    };

    let err = execute_with_bindings_in_state(&workflow, &runtime_state, Default::default())
        .await
        .expect_err("unadmitted Args provider must not be reached through behaviour fallback");
    assert!(
        matches!(err, ash_interp::ExecError::CapabilityNotAvailable(_)),
        "expected fail-closed capability error, got {err:?}"
    );
}

#[tokio::test]
async fn runtime_state_admits_implementation_binding_as_dependency_metadata_without_execution_projection()
 {
    let runtime_state = RuntimeState::new().with_provider(
        "clock",
        Arc::new(
            MockProvider::new("clock", Effect::Operational)
                .with_execute_result(Ok(Value::String("tick".to_string()))),
        ),
    );
    let host = host_binding("clock-binding", "clock", vec!["clock.now"]);
    let host_id = host.id;
    runtime_state
        .admit_capability_binding(host)
        .await
        .expect("host binding admitted");

    let resource_id = ResourceId::new();
    let resource = ResourceInstance::new(
        resource_id,
        ResourceTypeId::new("KvStore"),
        ResourceOwner::Workflow(WorkflowId::new()),
    );
    runtime_state.register_resource_instance(resource).await;

    let implementation = CapabilityBinding::implementation(
        CapabilityBindingId::new(),
        "kv-binding",
        CapabilityInterfaceId::new("KeyValue"),
        CapabilityImplementationId::new("KvImpl"),
        vec![
            CapabilityBindingDependency::Resource {
                name: "store".to_string(),
                resource_id,
                type_id: ResourceTypeId::new("KvStore"),
            },
            CapabilityBindingDependency::Capability {
                name: "clock".to_string(),
                binding_id: host_id,
                interface: CapabilityInterfaceId::new("Clock"),
            },
            CapabilityBindingDependency::Config {
                name: "prefix".to_string(),
                value: Value::String("wf".to_string()),
            },
        ],
    );
    let implementation_id = implementation.id;

    runtime_state
        .admit_capability_binding(implementation.clone())
        .await
        .expect("implementation metadata admission succeeds when dependencies are admitted");
    assert_eq!(
        runtime_state.capability_binding(implementation_id).await,
        Some(implementation)
    );

    let projected = runtime_state
        .create_capability_context_for_bindings(&[implementation_id])
        .await
        .expect("implementation binding projection is a metadata-only no-op");
    assert!(projected.execute("kv-binding", "get", &[]).await.is_err());
    assert!(
        projected
            .execute("workflow-clock", "now", &[])
            .await
            .is_err()
    );
}

#[tokio::test]
async fn act_env_can_be_projected_from_explicit_admitted_bindings() {
    let runtime_state = RuntimeState::new().with_provider(
        "clock",
        Arc::new(
            MockProvider::new("clock", Effect::Operational)
                .with_execute_result(Ok(Value::String("tick".to_string()))),
        ),
    );
    let binding = host_binding("workflow-clock", "clock", vec!["clock.now"]);
    let binding_id = binding.id;
    runtime_state
        .admit_capability_binding(binding)
        .await
        .expect("binding admitted");

    let env = ActEnv::from_runtime_state_with_admitted_bindings(
        &runtime_state,
        &[binding_id],
        PolicyEvaluator::new(),
        ash_core::Provenance::default(),
    )
    .await
    .expect("act env projection succeeds");

    assert_eq!(
        env.capability_ctx
            .execute("workflow-clock", "now", &[])
            .await,
        Ok(Value::String("tick".to_string()))
    );
}

#[tokio::test]
async fn runtime_state_rejects_duplicate_binding_ids_and_names() {
    let runtime_state = RuntimeState::new().with_provider(
        "clock",
        Arc::new(MockProvider::new("clock", Effect::Operational)),
    );
    let id = CapabilityBindingId::new();

    runtime_state
        .admit_capability_binding(host_binding_with_id(
            id,
            "workflow-clock",
            "clock",
            vec!["clock.now"],
        ))
        .await
        .expect("first binding admission succeeds");

    let duplicate_id = runtime_state
        .admit_capability_binding(host_binding_with_id(
            id,
            "workflow-clock-2",
            "clock",
            vec!["clock.now"],
        ))
        .await
        .expect_err("duplicate binding id must not overwrite authority records");
    assert!(
        duplicate_id
            .to_string()
            .contains("duplicate capability binding id")
    );

    let duplicate_name = runtime_state
        .admit_capability_binding(host_binding("workflow-clock", "clock", vec!["clock.now"]))
        .await
        .expect_err("duplicate binding name must be rejected before name lookup is ambiguous");
    assert!(
        duplicate_name
            .to_string()
            .contains("duplicate capability binding name")
    );
    assert_eq!(runtime_state.capability_binding_count().await, 1);
}

#[tokio::test]
async fn runtime_state_rejects_kind_authority_inconsistent_bindings() {
    let runtime_state = RuntimeState::new().with_provider(
        "clock",
        Arc::new(MockProvider::new("clock", Effect::Operational)),
    );

    let mut host = host_binding("workflow-clock", "clock", vec!["clock.now"]);
    host.authority = CapabilityAuthorityProvenance::DerivedAuthority {
        dependency_names: vec!["fake".to_string()],
        notes: vec![],
    };
    let host_err = runtime_state
        .admit_capability_binding(host)
        .await
        .expect_err("host binding with derived authority must be rejected");
    assert!(host_err.to_string().contains("host capability binding"));

    let mut implementation = CapabilityBinding::implementation(
        CapabilityBindingId::new(),
        "kv-binding",
        CapabilityInterfaceId::new("KeyValue"),
        CapabilityImplementationId::new("KvImpl"),
        vec![],
    );
    implementation.authority = CapabilityAuthorityProvenance::HostAuthority { notes: vec![] };
    let implementation_err = runtime_state
        .admit_capability_binding(implementation)
        .await
        .expect_err("implementation binding with host authority must be rejected");
    assert!(
        implementation_err
            .to_string()
            .contains("implementation capability binding")
    );
}

#[tokio::test]
async fn implementation_binding_admission_rejects_mismatched_dependencies() {
    let runtime_state = RuntimeState::new().with_provider(
        "clock",
        Arc::new(MockProvider::new("clock", Effect::Operational)),
    );
    let host = host_binding("clock-binding", "clock", vec!["clock.now"]);
    let host_id = host.id;
    runtime_state
        .admit_capability_binding(host)
        .await
        .expect("host binding admitted");

    let resource_id = ResourceId::new();
    runtime_state
        .register_resource_instance(ResourceInstance::new(
            resource_id,
            ResourceTypeId::new("KvStore"),
            ResourceOwner::Workflow(WorkflowId::new()),
        ))
        .await;

    let wrong_resource_type = CapabilityBinding::implementation(
        CapabilityBindingId::new(),
        "kv-resource-mismatch",
        CapabilityInterfaceId::new("KeyValue"),
        CapabilityImplementationId::new("KvImpl"),
        vec![CapabilityBindingDependency::Resource {
            name: "store".to_string(),
            resource_id,
            type_id: ResourceTypeId::new("OtherStore"),
        }],
    );
    let resource_err = runtime_state
        .admit_capability_binding(wrong_resource_type)
        .await
        .expect_err("resource dependency type mismatch must be rejected");
    assert!(resource_err.to_string().contains("mismatched type"));

    let wrong_cap_interface = CapabilityBinding::implementation(
        CapabilityBindingId::new(),
        "kv-cap-mismatch",
        CapabilityInterfaceId::new("KeyValue"),
        CapabilityImplementationId::new("KvImpl"),
        vec![CapabilityBindingDependency::Capability {
            name: "clock".to_string(),
            binding_id: host_id,
            interface: CapabilityInterfaceId::new("Timer"),
        }],
    );
    let capability_err = runtime_state
        .admit_capability_binding(wrong_cap_interface)
        .await
        .expect_err("capability dependency interface mismatch must be rejected");
    assert!(capability_err.to_string().contains("mismatched interface"));
}

#[tokio::test]
async fn mixed_projection_projects_only_host_bindings_and_keeps_implementation_metadata_inert() {
    let runtime_state = RuntimeState::new().with_provider(
        "clock",
        Arc::new(
            MockProvider::new("clock", Effect::Operational)
                .with_execute_result(Ok(Value::String("tick".to_string()))),
        ),
    );
    let host = host_binding("clock-binding", "clock", vec!["clock.now"]);
    let host_id = host.id;
    runtime_state
        .admit_capability_binding(host)
        .await
        .expect("host binding admitted");

    let implementation = CapabilityBinding::implementation(
        CapabilityBindingId::new(),
        "kv-binding",
        CapabilityInterfaceId::new("KeyValue"),
        CapabilityImplementationId::new("KvImpl"),
        vec![CapabilityBindingDependency::Capability {
            name: "clock".to_string(),
            binding_id: host_id,
            interface: CapabilityInterfaceId::new("Clock"),
        }],
    );
    let implementation_id = implementation.id;
    runtime_state
        .admit_capability_binding(implementation)
        .await
        .expect("implementation binding admitted as metadata");

    let projected = runtime_state
        .create_capability_context_for_bindings(&[implementation_id, host_id])
        .await
        .expect("mixed projection succeeds");
    assert_eq!(
        projected.execute("clock-binding", "now", &[]).await,
        Ok(Value::String("tick".to_string()))
    );
    assert!(projected.execute("kv-binding", "get", &[]).await.is_err());
}

#[tokio::test]
async fn workflow_and_proc_contexts_can_carry_admitted_binding_projection_metadata() {
    let runtime_state = RuntimeState::new().with_provider(
        "clock",
        Arc::new(MockProvider::new("clock", Effect::Operational)),
    );
    let binding = host_binding("workflow-clock", "clock", vec!["clock.now"]);
    let binding_id = binding.id;
    runtime_state
        .admit_capability_binding(binding)
        .await
        .expect("binding admitted");

    let workflow_context =
        ash_core::WorkflowAdmissionContext::default().with_admitted_capability_binding(binding_id);
    assert_eq!(
        workflow_context.admitted_capability_bindings,
        vec![binding_id]
    );

    let parent = ash_interp::Context::new()
        .with_runtime_state(runtime_state)
        .with_admitted_capability_bindings(vec![binding_id]);
    assert_eq!(parent.admitted_capability_bindings(), &[binding_id]);

    let child = ash_interp::derive_child_env(
        &parent,
        ash_interp::ChildEnvProjection::new(ash_core::ProcessId::new(), 0),
    )
    .expect("proc child projection preserves admitted binding metadata");
    assert_eq!(child.admitted_capability_bindings(), &[binding_id]);
}

#[tokio::test]
async fn implementation_binding_admission_rejects_missing_dependencies() {
    let runtime_state = RuntimeState::new();
    let binding = CapabilityBinding::implementation(
        CapabilityBindingId::new(),
        "kv-binding",
        CapabilityInterfaceId::new("KeyValue"),
        CapabilityImplementationId::new("KvImpl"),
        vec![CapabilityBindingDependency::Resource {
            name: "store".to_string(),
            resource_id: ResourceId::new(),
            type_id: ResourceTypeId::new("KvStore"),
        }],
    );

    let err = runtime_state
        .admit_capability_binding(binding)
        .await
        .expect_err("missing resources must not be manufactured by Ash-defined code");
    assert!(err.to_string().contains("missing resource dependency"));
}

#[test]
fn rust_capability_provider_trait_remains_compatible() {
    fn accepts_provider(_provider: Arc<dyn ash_core::capability::CapabilityProvider>) {}

    accepts_provider(Arc::new(MockProvider::new("clock", Effect::Operational)));
}

#[tokio::test]
async fn existing_runtime_state_capability_context_remains_provider_registry_compatible() {
    let runtime_state = RuntimeState::new().with_provider(
        "clock",
        Arc::new(
            MockProvider::new("clock", Effect::Operational)
                .with_execute_result(Ok(Value::String("legacy".to_string()))),
        ),
    );

    let legacy_context = runtime_state.create_capability_context().await;
    assert_eq!(
        legacy_context.execute("clock", "any", &[]).await,
        Ok(Value::String("legacy".to_string()))
    );

    let explicitly_empty = runtime_state
        .create_capability_context_for_bindings(&[])
        .await
        .expect("empty explicit admission set remains valid");
    assert!(explicitly_empty.execute("clock", "any", &[]).await.is_err());
}

#[tokio::test]
async fn alpha_policy_profile_act_execution_uses_projected_action_grants() {
    let runtime_state = RuntimeState::new().with_provider(
        "deploy",
        Arc::new(
            MockProvider::new("deploy", Effect::Operational)
                .with_execute_result(Ok(Value::String("leaked".to_string()))),
        ),
    );
    runtime_state
        .admit_capability_binding(host_binding(
            "workflow-deploy",
            "deploy",
            vec!["deploy.plan"],
        ))
        .await
        .expect("policy profile admits only the plan action");

    let workflow = Workflow::Act {
        provider_name: "workflow-deploy".to_string(),
        action_name: "apply".to_string(),
        arguments: vec![],
        guard: Guard::Always,
        provenance: Provenance::new(),
        result_name: None,
        continuation: Box::new(Workflow::Done),
    };

    let error = execute_with_bindings_in_state(&workflow, &runtime_state, Default::default())
        .await
        .expect_err("ungranted action must fail closed even when provider is registered");

    assert!(
        error.to_string().contains("deploy.apply")
            || error
                .to_string()
                .contains("capability not available: deploy")
            || error
                .to_string()
                .contains("capability not available: workflow-deploy"),
        "diagnostic should identify the ungranted action/provider boundary: {error}"
    );
}

#[tokio::test]
async fn alpha_policy_profile_records_projected_grant_facts_in_execution_record() {
    let runtime_state = RuntimeState::new().with_provider(
        "deploy",
        Arc::new(
            MockProvider::new("deploy", Effect::Operational)
                .with_execute_result(Ok(Value::String("planned".to_string()))),
        ),
    );
    let binding = host_binding("workflow-deploy", "deploy", vec!["deploy.plan"]);
    let binding_id = binding.id;
    runtime_state
        .admit_capability_binding(binding)
        .await
        .expect("policy profile admits the plan action");

    let workflow = Workflow::Act {
        provider_name: "workflow-deploy".to_string(),
        action_name: "plan".to_string(),
        arguments: vec![],
        guard: Guard::Always,
        provenance: Provenance::new(),
        result_name: None,
        continuation: Box::new(Workflow::Done),
    };

    let result = execute_with_bindings_in_state(&workflow, &runtime_state, Default::default())
        .await
        .expect("granted action should execute");
    assert_eq!(result, Value::String("planned".to_string()));

    let record = runtime_state
        .last_execution_record()
        .await
        .expect("workflow execution should persist an execution record");
    let record_debug = format!("{record:?}");
    assert!(
        record_debug.contains(&format!("{binding_id:?}")),
        "execution record should include admitted binding identity: {record_debug}"
    );
    assert!(
        record_debug.contains("deploy.plan"),
        "execution record should include projected action grant: {record_debug}"
    );
}
