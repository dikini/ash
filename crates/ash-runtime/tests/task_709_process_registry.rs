use ash_core::runtime::{
    FailureBoundary, FailureEntity, OperationalFailure, ProcessId, ProcessLifecycleState,
    ProcessTerminalState,
};
use ash_core::{ApplicationId, Capability, Effect, Role, RoleObligationRef, Value};
use ash_runtime::role_context::RoleContext;
use ash_runtime::{ChildEnvProjection, Context, ProcessRegistry, RuntimeState, derive_child_env};
use proptest::prelude::*;
use std::collections::BTreeSet;

fn proc_failure(process_id: ProcessId, message: &str) -> OperationalFailure {
    OperationalFailure::new(
        FailureBoundary::Process,
        FailureEntity::Process(process_id),
        Value::String(message.to_string()),
        "String",
    )
}

fn role_with_authority(names: &[&str]) -> Role {
    Role {
        name: "runner".to_string(),
        authority: names
            .iter()
            .map(|name| Capability {
                name: (*name).to_string(),
                effect: Effect::Operational,
                constraints: vec![],
            })
            .collect(),
        obligations: vec![RoleObligationRef {
            name: "audit".to_string(),
        }],
    }
}

fn role_with_owned_authority(names: &[String]) -> Role {
    Role {
        name: "runner".to_string(),
        authority: names
            .iter()
            .map(|name| Capability {
                name: name.clone(),
                effect: Effect::Operational,
                constraints: vec![],
            })
            .collect(),
        obligations: vec![RoleObligationRef {
            name: "audit".to_string(),
        }],
    }
}

fn capability(name: impl Into<String>) -> Capability {
    Capability {
        name: name.into(),
        effect: Effect::Operational,
        constraints: vec![],
    }
}

#[test]
fn process_registry_records_parent_child_identity_and_order() {
    let mut registry = ProcessRegistry::new();
    let parent = ProcessId::new();
    let first_child = ProcessId::new();
    let second_child = ProcessId::new();

    registry.register_root(parent).expect("root registers");
    registry
        .register_child(parent, first_child, 0)
        .expect("first child registers");
    registry
        .register_child(parent, second_child, 1)
        .expect("second child registers");

    let parent_record = registry.record(parent).expect("parent record exists");
    assert_eq!(parent_record.process_id, parent);
    assert_eq!(parent_record.parent_process_id, None);
    assert_eq!(
        parent_record.lifecycle_state,
        ProcessLifecycleState::Admitting
    );

    registry
        .mark_running(parent)
        .expect("admitted process transitions to running");
    assert_eq!(
        registry
            .record(parent)
            .expect("parent record exists")
            .lifecycle_state,
        ProcessLifecycleState::Running
    );

    let first_record = registry
        .record(first_child)
        .expect("first child record exists");
    assert_eq!(first_record.parent_process_id, Some(parent));
    assert_eq!(first_record.child_index, Some(0));

    assert_eq!(
        registry.children_of(parent),
        vec![first_child, second_child]
    );
}

#[test]
fn process_registry_terminal_state_is_write_once_and_preserves_failure_identity() {
    let mut registry = ProcessRegistry::new();
    let process_id = ProcessId::new();
    registry.register_root(process_id).expect("root registers");

    let failure = proc_failure(process_id, "boom");
    registry
        .record_terminal(
            process_id,
            ProcessTerminalState::Failed {
                process_id,
                failure: Box::new(failure.clone()),
            },
        )
        .expect("first terminal record succeeds");

    let record = registry.record(process_id).expect("record exists");
    assert_eq!(
        record.lifecycle_state,
        ProcessLifecycleState::Failed {
            process_id,
            failure: Box::new(failure.clone()),
        }
    );
    assert_eq!(
        record.terminal_state,
        Some(ProcessTerminalState::Failed {
            process_id,
            failure: Box::new(failure),
        })
    );

    let second = registry.record_terminal(
        process_id,
        ProcessTerminalState::Succeeded { value: Value::Null },
    );
    assert!(second.is_err(), "terminal state must be write-once");
}

#[test]
fn process_registry_rejects_children_under_terminal_parent() {
    let mut registry = ProcessRegistry::new();
    let parent = ProcessId::new();
    let child = ProcessId::new();
    registry.register_root(parent).expect("root registers");
    registry
        .record_terminal(
            parent,
            ProcessTerminalState::Succeeded { value: Value::Null },
        )
        .expect("parent terminal state records");

    assert!(
        registry.register_child(parent, child, 0).is_err(),
        "terminal parents must not admit children"
    );
}

#[test]
fn process_registry_rejects_duplicate_child_indices() {
    let mut registry = ProcessRegistry::new();
    let parent = ProcessId::new();
    let first = ProcessId::new();
    let duplicate = ProcessId::new();
    registry.register_root(parent).expect("root registers");
    registry
        .register_child(parent, first, 0)
        .expect("first child registers");

    assert!(
        registry.register_child(parent, duplicate, 0).is_err(),
        "one parent must not have duplicate child indices"
    );
}

#[tokio::test]
async fn runtime_state_owns_process_registry_without_replacing_control_links() {
    let state = RuntimeState::new();
    let process_id = ProcessId::new();
    let application_id = ApplicationId::new();

    state
        .register_root_process(process_id)
        .await
        .expect("root process registers");
    state.register_spawned_control_link(application_id).await;

    assert_eq!(
        state
            .process_record(process_id)
            .await
            .expect("process exists")
            .process_id,
        process_id
    );
    assert_eq!(
        state.control_link_state(application_id).await,
        Some(ash_runtime::control_link::LinkState::Running)
    );
}

#[test]
fn derive_child_env_preserves_lexical_bindings_but_allocates_child_local_state() {
    let mut parent = Context::new();
    parent.set("x".to_string(), Value::Int(42));
    parent.add_obligation("parent_only".to_string());

    let child = derive_child_env(&parent, ChildEnvProjection::new(ProcessId::new(), 0))
        .expect("projection succeeds");

    assert_eq!(child.get("x"), Some(&Value::Int(42)));
    assert!(child.local_pending_obligations().is_empty());
    assert!(child.visible_pending_obligations().is_empty());

    let mut mutated_child = child;
    mutated_child.set("y".to_string(), Value::Int(7));
    assert_eq!(mutated_child.get("y"), Some(&Value::Int(7)));
    assert_eq!(parent.get("y"), None);
    assert!(parent.has_obligation("parent_only"));
}

#[test]
fn derive_child_env_allows_equal_or_narrower_role_authority_only() {
    let parent_role = RoleContext::new(role_with_authority(&["sensor", "actuator"]));
    let sensor_capability = Capability {
        name: "sensor".to_string(),
        effect: Effect::Operational,
        constraints: vec![],
    };
    let actuator_capability = Capability {
        name: "actuator".to_string(),
        effect: Effect::Operational,
        constraints: vec![],
    };
    let mut parent = Context::new().with_role_context(parent_role);
    parent.set("x".to_string(), Value::Int(1));

    let child = derive_child_env(
        &parent,
        ChildEnvProjection::new(ProcessId::new(), 0)
            .with_role_authority(vec![sensor_capability.clone()]),
    )
    .expect("narrower projection succeeds");

    let role_context = child.role_context().expect("projected role exists");
    assert!(role_context.can_access(&sensor_capability));
    assert!(!role_context.can_access(&actuator_capability));

    let wider = derive_child_env(
        &parent,
        ChildEnvProjection::new(ProcessId::new(), 1).with_role_authority(vec![
            sensor_capability,
            actuator_capability,
            Capability {
                name: "network".to_string(),
                effect: Effect::Operational,
                constraints: vec![],
            },
        ]),
    );
    assert!(wider.is_err(), "wider authority projection must fail");
}

#[test]
fn derive_child_env_rejects_same_name_with_stronger_effect() {
    let parent = Context::new().with_role_context(RoleContext::new(Role {
        name: "runner".to_string(),
        authority: vec![Capability {
            name: "sensor".to_string(),
            effect: Effect::Epistemic,
            constraints: vec![],
        }],
        obligations: vec![],
    }));

    let result = derive_child_env(
        &parent,
        ChildEnvProjection::new(ProcessId::new(), 0).with_role_authority(vec![Capability {
            name: "sensor".to_string(),
            effect: Effect::Operational,
            constraints: vec![],
        }]),
    );

    assert!(
        result.is_err(),
        "same-name capability projection must not widen effect authority"
    );
}

#[test]
fn derive_child_env_allows_same_name_with_weaker_effect() {
    let parent = Context::new().with_role_context(RoleContext::new(Role {
        name: "runner".to_string(),
        authority: vec![Capability {
            name: "sensor".to_string(),
            effect: Effect::Operational,
            constraints: vec![],
        }],
        obligations: vec![],
    }));

    let child = derive_child_env(
        &parent,
        ChildEnvProjection::new(ProcessId::new(), 0).with_role_authority(vec![Capability {
            name: "sensor".to_string(),
            effect: Effect::Epistemic,
            constraints: vec![],
        }]),
    )
    .expect("same-name weaker-effect projection succeeds");

    let role_context = child.role_context().expect("projected role exists");
    assert!(role_context.can_access(&Capability {
        name: "sensor".to_string(),
        effect: Effect::Epistemic,
        constraints: vec![],
    }));
    assert!(!role_context.can_access(&Capability {
        name: "sensor".to_string(),
        effect: Effect::Operational,
        constraints: vec![],
    }));
}

#[test]
fn derive_child_env_records_child_identity_metadata() {
    let parent = Context::new();
    let parent_process_id = ProcessId::new();
    let child_process_id = ProcessId::new();
    let child = derive_child_env(
        &parent,
        ChildEnvProjection::new(child_process_id, 7).with_parent_process_id(parent_process_id),
    )
    .expect("projection succeeds");

    let identity = child
        .process_identity()
        .expect("child process identity is projected");
    assert_eq!(identity.process_id, child_process_id);
    assert_eq!(identity.parent_process_id, Some(parent_process_id));
    assert_eq!(identity.child_index, 7);
}

#[test]
fn derive_child_env_exact_authority_blocks_stronger_runtime_access() {
    let parent = Context::new().with_role_context(RoleContext::new(Role {
        name: "runner".to_string(),
        authority: vec![Capability {
            name: "sensor".to_string(),
            effect: Effect::Epistemic,
            constraints: vec![],
        }],
        obligations: vec![],
    }));

    let child = derive_child_env(&parent, ChildEnvProjection::new(ProcessId::new(), 0))
        .expect("projection succeeds");
    let role_context = child.role_context().expect("projected role exists");

    assert!(!role_context.can_access(&Capability {
        name: "sensor".to_string(),
        effect: Effect::Operational,
        constraints: vec![],
    }));
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        .. ProptestConfig::default()
    })]

    #[test]
    fn child_authority_projection_never_widens_parent_authority(
        parent_names in prop::collection::btree_set("[a-z][a-z0-9_]{0,8}", 0..8),
        requested_names in prop::collection::btree_set("[a-z][a-z0-9_]{0,8}", 0..8),
    ) {
        let parent_names_vec: Vec<_> = parent_names.iter().cloned().collect();
        let requested_names_vec: Vec<_> = requested_names.iter().cloned().collect();
        let parent = Context::new().with_role_context(RoleContext::new(role_with_owned_authority(&parent_names_vec)));
        let requested_authority: Vec<_> = requested_names_vec.iter().cloned().map(capability).collect();

        let result = derive_child_env(
            &parent,
            ChildEnvProjection::new(ProcessId::new(), 0).with_role_authority(requested_authority),
        );
        let requested_is_subset = requested_names.is_subset(&parent_names);

        prop_assert_eq!(result.is_ok(), requested_is_subset);
        if let Ok(child) = result {
            let child_names: BTreeSet<_> = child
                .role_context()
                .expect("projected role exists")
                .active_role
                .authority
                .iter()
                .map(|capability| capability.name.clone())
                .collect();
            prop_assert!(child_names.is_subset(&parent_names));
            prop_assert_eq!(child_names, requested_names);
        }
    }
}
