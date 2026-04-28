use ash_core::{
    AccessPolicy, ProcessId, ResourceId, ResourceInstance, ResourceLifecycle, ResourceOwner,
    ResourceProvenance, ResourceRuntimeState, ResourceSplitJoinPolicy, ResourceTypeId, RunId,
    TestId, WorkflowId,
};

#[test]
fn resource_identity_carriers_are_unique_hashable_and_round_trip() {
    let id = ResourceId::new();
    let other_id = ResourceId::new();
    assert_ne!(id, other_id);

    let type_id = ResourceTypeId::new("WorkflowKV");
    let same_type_id = ResourceTypeId::new("WorkflowKV");
    let other_type_id = ResourceTypeId::new("Mailbox");
    assert_eq!(type_id, same_type_id);
    assert_ne!(type_id, other_type_id);
    assert_eq!(type_id.as_str(), "WorkflowKV");

    let encoded_id = serde_json::to_string(&id).expect("ResourceId serializes");
    let decoded_id: ResourceId =
        serde_json::from_str(&encoded_id).expect("ResourceId deserializes");
    assert_eq!(id, decoded_id);

    let encoded_type = serde_json::to_string(&type_id).expect("ResourceTypeId serializes");
    let decoded_type: ResourceTypeId =
        serde_json::from_str(&encoded_type).expect("ResourceTypeId deserializes");
    assert_eq!(type_id, decoded_type);
}

#[test]
fn resource_instance_preserves_metadata_and_terminal_lifecycle_classification() {
    let id = ResourceId::new();
    let type_id = ResourceTypeId::new("WorkflowKV");
    let owner = ResourceOwner::Workflow(WorkflowId::new());

    let instance = ResourceInstance::new(id, type_id.clone(), owner)
        .with_state(ResourceRuntimeState::opaque("in-memory-map"))
        .with_lifecycle(ResourceLifecycle::Active)
        .with_access_policy(AccessPolicy::ReadWrite)
        .with_split_join_policy(ResourceSplitJoinPolicy::Mergeable)
        .with_provenance(ResourceProvenance::internal("workflow owns kv"));

    assert_eq!(instance.id, id);
    assert_eq!(instance.type_id, type_id);
    assert_eq!(instance.owner, owner);
    assert_eq!(instance.lifecycle, ResourceLifecycle::Active);
    assert_eq!(instance.access_policy, AccessPolicy::ReadWrite);
    assert_eq!(
        instance.split_join_policy,
        ResourceSplitJoinPolicy::Mergeable
    );
    assert_eq!(
        instance.provenance,
        ResourceProvenance::internal("workflow owns kv")
    );
    assert_eq!(
        instance.state,
        ResourceRuntimeState::opaque("in-memory-map")
    );

    assert!(!ResourceLifecycle::Allocated.is_terminal());
    assert!(!ResourceLifecycle::Active.is_terminal());
    assert!(ResourceLifecycle::Released.is_terminal());
    assert!(ResourceLifecycle::Failed.is_terminal());
}

#[test]
fn resource_owner_covers_runtime_owner_scopes_without_value_handles() {
    let run = ResourceOwner::Run(RunId::new());
    let workflow = ResourceOwner::Workflow(WorkflowId::new());
    let process = ResourceOwner::Process(ProcessId::new());
    let effect_scope = ResourceOwner::EffectScope(ash_core::EffectScopeId::new());
    let test = ResourceOwner::Test(TestId::new());

    assert_ne!(run, workflow);
    assert_ne!(workflow, process);
    assert_ne!(process, effect_scope);
    assert_ne!(effect_scope, test);

    let encoded = serde_json::to_string(&process).expect("ResourceOwner serializes");
    let decoded: ResourceOwner =
        serde_json::from_str(&encoded).expect("ResourceOwner deserializes");
    assert_eq!(process, decoded);
}

#[test]
fn resource_instance_serializes_complete_carrier_metadata() {
    let instance = ResourceInstance::new(
        ResourceId::new(),
        ResourceTypeId::new("WorkflowKV"),
        ResourceOwner::Test(TestId::new()),
    )
    .with_state(ResourceRuntimeState::opaque("host-state-token"))
    .with_lifecycle(ResourceLifecycle::Admitted)
    .with_access_policy(AccessPolicy::Exclusive)
    .with_split_join_policy(ResourceSplitJoinPolicy::NonShareable)
    .with_provenance(ResourceProvenance::DerivedAuthority {
        sources: vec![ResourceId::new()],
        notes: vec!["derived from input".to_string()],
    });

    let encoded = serde_json::to_string(&instance).expect("ResourceInstance serializes");
    let decoded: ResourceInstance =
        serde_json::from_str(&encoded).expect("ResourceInstance deserializes");
    assert_eq!(instance, decoded);
}
