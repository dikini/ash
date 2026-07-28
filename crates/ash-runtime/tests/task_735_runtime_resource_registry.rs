use ash_core::{
    AccessPolicy, ApplicationId, ResourceId, ResourceInstance, ResourceLifecycle, ResourceOwner,
    ResourceProvenance, ResourceRuntimeState, ResourceSplitJoinPolicy, ResourceTypeId,
};
use ash_runtime::RuntimeState;

fn resource(name: &str, lifecycle: ResourceLifecycle) -> ResourceInstance {
    resource_for_owner(
        name,
        lifecycle,
        ResourceOwner::Application(ApplicationId::new()),
    )
}

fn resource_for_owner(
    name: &str,
    lifecycle: ResourceLifecycle,
    owner: ResourceOwner,
) -> ResourceInstance {
    ResourceInstance::new(ResourceId::new(), ResourceTypeId::new(name), owner)
        .with_state(ResourceRuntimeState::opaque(format!("state:{name}")))
        .with_lifecycle(lifecycle)
        .with_access_policy(AccessPolicy::ReadWrite)
        .with_split_join_policy(ResourceSplitJoinPolicy::ReadOnlyShare)
        .with_provenance(ResourceProvenance::internal(format!("allocated {name}")))
}

#[tokio::test]
async fn runtime_state_resource_registry_round_trips_by_identity() {
    let runtime_state = RuntimeState::new();
    let instance = resource("ApplicationKV", ResourceLifecycle::Active);
    let id = instance.id;

    assert!(runtime_state.resource_instance(id).await.is_none());

    runtime_state
        .register_resource_instance(instance.clone())
        .await;

    assert_eq!(
        runtime_state.resource_instance(id).await,
        Some(instance.clone())
    );
    assert!(runtime_state.has_resource_instance(id).await);
    assert_eq!(runtime_state.resource_instance_count().await, 1);
}

#[tokio::test]
async fn runtime_state_resource_registry_replaces_same_identity_without_ambient_lookup() {
    let runtime_state = RuntimeState::new();
    let mut instance = resource("ApplicationKV", ResourceLifecycle::Allocated);
    let id = instance.id;

    runtime_state
        .register_resource_instance(instance.clone())
        .await;

    instance.lifecycle = ResourceLifecycle::Released;
    runtime_state
        .register_resource_instance(instance.clone())
        .await;

    assert_eq!(runtime_state.resource_instance(id).await, Some(instance));
    assert!(
        runtime_state
            .resource_instances_for_owner_by_type(
                ResourceOwner::Application(ApplicationId::new()),
                ResourceTypeId::new("Missing"),
            )
            .await
            .is_empty()
    );
    assert_eq!(runtime_state.resource_instance_count().await, 1);
}

#[tokio::test]
async fn runtime_state_resource_registry_lists_by_owner_and_type_without_ambient_type_lookup() {
    let runtime_state = RuntimeState::new();
    let application_owner = ResourceOwner::Application(ApplicationId::new());
    let other_owner = ResourceOwner::Application(ApplicationId::new());
    let kv_active = resource_for_owner(
        "ApplicationKV",
        ResourceLifecycle::Active,
        application_owner,
    );
    let kv_failed = resource_for_owner(
        "ApplicationKV",
        ResourceLifecycle::Failed,
        application_owner,
    );
    let kv_other_owner =
        resource_for_owner("ApplicationKV", ResourceLifecycle::Active, other_owner);
    let mailbox = resource_for_owner("Mailbox", ResourceLifecycle::Active, application_owner);

    runtime_state
        .register_resource_instance(kv_active.clone())
        .await;
    runtime_state
        .register_resource_instance(kv_failed.clone())
        .await;
    runtime_state
        .register_resource_instance(kv_other_owner)
        .await;
    runtime_state
        .register_resource_instance(mailbox.clone())
        .await;

    let mut kv_ids: Vec<_> = runtime_state
        .resource_instances_for_owner_by_type(
            application_owner,
            ResourceTypeId::new("ApplicationKV"),
        )
        .await
        .into_iter()
        .map(|instance| instance.id)
        .collect();
    kv_ids.sort_by_key(|id| id.0);

    let mut expected = vec![kv_active.id, kv_failed.id];
    expected.sort_by_key(|id| id.0);

    assert_eq!(kv_ids, expected);
    assert_eq!(runtime_state.resource_instance_count().await, 4);
}
