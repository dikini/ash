use std::sync::Arc;

use ash_core::{
    AccessPolicy, CapabilityBinding, CapabilityBindingDependency, CapabilityBindingId,
    CapabilityImplementationId, CapabilityInterfaceId, Effect, ResourceLifecycle, ResourceOwner,
    ResourceProvenance, ResourceSplitJoinPolicy, ResourceTypeId, Value, WorkflowId,
};
use ash_interp::{
    ImplementationBindingAdmission, ImplementationBindingDependencySource, MockProvider,
    RuntimeState, WorkflowOwnedResourceAdmission,
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

#[tokio::test]
async fn admits_workflow_owned_resources_with_conservative_internal_authority_metadata() {
    let runtime_state = RuntimeState::new();
    let workflow_id = WorkflowId::new();

    let allocated = runtime_state
        .admit_workflow_owned_resources(
            workflow_id,
            vec![
                WorkflowOwnedResourceAdmission::new("store", ResourceTypeId::new("KvStore")),
                WorkflowOwnedResourceAdmission::new("audit", ResourceTypeId::new("AuditLog")),
            ],
        )
        .await
        .expect("workflow-owned resources are admitted");

    assert_eq!(allocated.len(), 2);
    assert_ne!(allocated["store"], allocated["audit"]);
    assert_eq!(runtime_state.resource_instance_count().await, 2);

    let store = runtime_state
        .resource_instance(allocated["store"])
        .await
        .expect("allocated store is registered");
    assert_eq!(store.type_id, ResourceTypeId::new("KvStore"));
    assert_eq!(store.owner, ResourceOwner::Workflow(workflow_id));
    assert_eq!(store.lifecycle, ResourceLifecycle::Admitted);
    assert_eq!(store.access_policy, AccessPolicy::Exclusive);
    assert_eq!(
        store.split_join_policy,
        ResourceSplitJoinPolicy::NonShareable
    );
    match store.provenance {
        ResourceProvenance::InternalAuthority { notes } => {
            assert!(
                notes
                    .iter()
                    .any(|note| note.contains("owns store: KvStore"))
            );
        }
        other => panic!("expected internal authority provenance, got {other:?}"),
    }
}

#[tokio::test]
async fn duplicate_workflow_owned_resource_names_are_rejected_without_partial_allocation() {
    let runtime_state = RuntimeState::new();
    let workflow_id = WorkflowId::new();

    let err = runtime_state
        .admit_workflow_owned_resources(
            workflow_id,
            vec![
                WorkflowOwnedResourceAdmission::new("store", ResourceTypeId::new("KvStore")),
                WorkflowOwnedResourceAdmission::new("store", ResourceTypeId::new("OtherStore")),
            ],
        )
        .await
        .expect_err("duplicate owned resource names must be rejected");

    assert!(
        err.to_string()
            .contains("duplicate owned resource name 'store'")
    );
    assert_eq!(runtime_state.resource_instance_count().await, 0);
}

#[tokio::test]
async fn admits_implementation_binding_from_explicit_resource_and_capability_source_names() {
    let runtime_state = RuntimeState::new().with_provider(
        "clock",
        Arc::new(MockProvider::new("clock", Effect::Operational)),
    );
    let workflow_id = WorkflowId::new();
    let resources = runtime_state
        .admit_workflow_owned_resources(
            workflow_id,
            vec![WorkflowOwnedResourceAdmission::new(
                "store",
                ResourceTypeId::new("KvStore"),
            )],
        )
        .await
        .expect("owned resource admitted");

    let clock = clock_binding("workflow-clock");
    let clock_id = clock.id;
    runtime_state
        .admit_capability_binding(clock)
        .await
        .expect("host capability admitted");

    let implementation_id = runtime_state
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
                "workflow-clock",
                CapabilityInterfaceId::new("Clock"),
            ))
            .with_dependency(ImplementationBindingDependencySource::config(
                "prefix",
                Value::String("wf".to_string()),
            )),
            &resources,
        )
        .await
        .expect("implementation binding admitted from explicit source names");

    let binding = runtime_state
        .capability_binding(implementation_id)
        .await
        .expect("implementation binding is stored");
    assert_eq!(binding.name, "kv-binding");
    assert_eq!(binding.dependencies.len(), 3);
    assert!(
        binding
            .dependencies
            .contains(&CapabilityBindingDependency::Resource {
                name: "store".to_string(),
                resource_id: resources["store"],
                type_id: ResourceTypeId::new("KvStore"),
            })
    );
    assert!(
        binding
            .dependencies
            .contains(&CapabilityBindingDependency::Capability {
                name: "clock".to_string(),
                binding_id: clock_id,
                interface: CapabilityInterfaceId::new("Clock"),
            })
    );
}

#[tokio::test]
async fn implementation_binding_admission_rejects_missing_explicit_resource_source_without_ambient_lookup()
 {
    let runtime_state = RuntimeState::new();
    let workflow_id = WorkflowId::new();
    runtime_state
        .admit_workflow_owned_resources(
            workflow_id,
            vec![WorkflowOwnedResourceAdmission::new(
                "store",
                ResourceTypeId::new("KvStore"),
            )],
        )
        .await
        .expect("resource admitted but not exposed in explicit map below");

    let err = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "kv-binding",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("KvImpl"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("KvStore"),
            )),
            &std::collections::HashMap::new(),
        )
        .await
        .expect_err("admission must not discover resources ambiently by name or type");

    assert!(
        err.to_string()
            .contains("missing resource dependency source 'store'")
    );
    assert_eq!(runtime_state.capability_binding_count().await, 0);
}

#[tokio::test]
async fn implementation_binding_admission_rejects_incompatible_resource_source() {
    let runtime_state = RuntimeState::new();
    let workflow_id = WorkflowId::new();
    let resources = runtime_state
        .admit_workflow_owned_resources(
            workflow_id,
            vec![WorkflowOwnedResourceAdmission::new(
                "store",
                ResourceTypeId::new("KvStore"),
            )],
        )
        .await
        .expect("owned resource admitted");

    let err = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "kv-binding",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("KvImpl"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("OtherStore"),
            )),
            &resources,
        )
        .await
        .expect_err("TASK-736 validation must reject mismatched resource types");

    assert!(err.to_string().contains("mismatched type"));
    assert_eq!(runtime_state.capability_binding_count().await, 0);
}

#[tokio::test]
async fn implementation_binding_admission_rejects_missing_capability_source_without_ambient_lookup_by_name()
 {
    let runtime_state = RuntimeState::new().with_provider(
        "clock",
        Arc::new(MockProvider::new("clock", Effect::Operational)),
    );
    runtime_state
        .admit_capability_binding(clock_binding("workflow-clock"))
        .await
        .expect("host capability admitted but source asks for a different name");

    let err = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "kv-binding",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("KvImpl"),
            )
            .with_dependency(ImplementationBindingDependencySource::capability(
                "clock",
                "ambient-clock",
                CapabilityInterfaceId::new("Clock"),
            )),
            &std::collections::HashMap::new(),
        )
        .await
        .expect_err("admission must not silently bind a different capability by interface");

    assert!(
        err.to_string()
            .contains("missing capability dependency source 'ambient-clock'")
    );
    assert_eq!(runtime_state.capability_binding_count().await, 1);
}
