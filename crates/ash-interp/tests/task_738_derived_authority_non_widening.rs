use std::sync::Arc;

use ash_core::{
    CapabilityAuthorityProvenance, CapabilityBinding, CapabilityBindingId,
    CapabilityImplementationId, CapabilityInterfaceId, Effect, ResourceTypeId, Value, WorkflowId,
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
async fn implementation_binding_rejects_zero_dependency_external_authority_claims() {
    let runtime_state = RuntimeState::new();

    let err = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "derived-store",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("KvImpl"),
            ),
            &std::collections::HashMap::new(),
        )
        .await
        .expect_err("derived implementation bindings must have authority-bearing dependencies");

    assert!(
        err.to_string()
            .contains("implementation capability binding must derive authority from at least one resource or capability dependency"),
        "unexpected error: {err}"
    );
    assert_eq!(runtime_state.capability_binding_count().await, 0);
}

#[tokio::test]
async fn implementation_binding_rejects_config_only_external_authority_claims() {
    let runtime_state = RuntimeState::new();

    let err = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "derived-store",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("KvImpl"),
            )
            .with_dependency(ImplementationBindingDependencySource::config(
                "prefix",
                Value::String("tenant-a".to_string()),
            )),
            &std::collections::HashMap::new(),
        )
        .await
        .expect_err("config dependencies alone do not carry external authority");

    assert!(
        err.to_string()
            .contains("implementation capability binding must derive authority from at least one resource or capability dependency"),
        "unexpected error: {err}"
    );
    assert_eq!(runtime_state.capability_binding_count().await, 0);
}

#[tokio::test]
async fn implementation_binding_rejects_requested_operations_outside_allowed_interface_surface() {
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

    runtime_state
        .register_capability_interface_operations(
            CapabilityInterfaceId::new("KeyValue"),
            ["get", "put"],
        )
        .await
        .expect("interface surface registered");

    let err = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "derived-store",
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
        .expect_err(
            "caller-supplied operation surfaces must not widen registered interface metadata",
        );

    assert!(
        err.to_string()
            .contains("requested operation 'delete' is outside registered interface KeyValue"),
        "unexpected error: {err}"
    );
    assert_eq!(runtime_state.capability_binding_count().await, 0);
}

#[tokio::test]
async fn implementation_binding_rejects_requested_operations_when_interface_surface_is_unregistered()
 {
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
                "derived-store",
                CapabilityInterfaceId::new("KeyValue"),
                CapabilityImplementationId::new("KvImpl"),
            )
            .with_dependency(ImplementationBindingDependencySource::resource(
                "store",
                ResourceTypeId::new("KvStore"),
            ))
            .with_requested_operations(["get"]),
            &resources,
        )
        .await
        .expect_err("requested operation surfaces need registered interface metadata");

    assert!(
        err.to_string()
            .contains("cannot validate requested operations for unregistered interface KeyValue"),
        "unexpected error: {err}"
    );
    assert_eq!(runtime_state.capability_binding_count().await, 0);
}

#[tokio::test]
async fn implementation_binding_records_authority_provenance_chain_for_resource_and_capability_dependencies()
 {
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

    runtime_state
        .register_capability_interface_operations(
            CapabilityInterfaceId::new("KeyValue"),
            ["get", "put"],
        )
        .await
        .expect("interface surface registered");

    let implementation_id = runtime_state
        .admit_implementation_binding(
            ImplementationBindingAdmission::new(
                "derived-store",
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
            .with_requested_operations(["get"]),
            &resources,
        )
        .await
        .expect("non-widening implementation binding is admitted");

    let binding = runtime_state
        .capability_binding(implementation_id)
        .await
        .expect("implementation binding is stored");

    match binding.authority {
        CapabilityAuthorityProvenance::DerivedAuthority {
            dependency_names,
            notes,
        } => {
            assert_eq!(
                dependency_names,
                vec!["store".to_string(), "clock".to_string()]
            );
            let notes_text = notes.join("\n");
            assert!(
                notes_text.contains("resource store:"),
                "notes: {notes_text}"
            );
            assert!(
                notes_text.contains(&resources["store"].0.to_string()),
                "notes: {notes_text}"
            );
            assert!(notes_text.contains("type=KvStore"), "notes: {notes_text}");
            assert!(
                notes_text.contains("capability clock:"),
                "notes: {notes_text}"
            );
            assert!(
                notes_text.contains(&clock_id.0.to_string()),
                "notes: {notes_text}"
            );
            assert!(
                notes_text.contains("interface=Clock"),
                "notes: {notes_text}"
            );
            assert!(
                notes_text.contains("operation get derives from store, clock"),
                "notes: {notes_text}"
            );
        }
        other => panic!("expected derived authority provenance, got {other:?}"),
    }
}
