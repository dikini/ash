use ash_core::core_ash_contract::{TraceAlphabet, TraceFactKind, TraceInterpretation};
use ash_core::runtime::{
    ActorCallPolicy, ActorProtocol, ExternalActorAdapter, ExternalActorDiagnostic,
    SupervisorDiagnostic, SupervisorPolicy, SupervisorRuntimeProfile,
};
use ash_core::runtime_kernel::{
    AdmissionIdentity, ApplicationArtifactIdentity, ApplicationDefinitionIdentity,
    ApplicationInstanceIdentity, ArtifactVersion, ProviderRegistryIdentity,
    RuntimeArtifactCacheKey, RuntimeConfigId, RuntimeEngineRelationship, RuntimeHostMode,
    RuntimeKernelIdentity, RuntimeProfileId, RuntimeProfileIdentity, RuntimeRootSet,
    RuntimeRootSetId,
};
use ash_core::{CapabilityBindingId, ProcessId, ResourceId};

fn roots() -> RuntimeRootSet {
    RuntimeRootSet::new(
        RuntimeRootSetId::new("workspace:/repo"),
        vec!["src".into()],
        vec!["stdlib".into()],
        vec!["config".into()],
        ".ash/state",
        ".ash/cache",
        ".ash/log",
    )
}

fn profile() -> RuntimeProfileIdentity {
    RuntimeProfileIdentity::new(
        RuntimeProfileId::new("alpha-local"),
        RuntimeConfigId::new("local-config"),
        vec!["profile=alpha-local".into(), "config=local-config".into()],
    )
}

fn cache_key(roots: &RuntimeRootSet, profile: &RuntimeProfileIdentity) -> RuntimeArtifactCacheKey {
    RuntimeArtifactCacheKey::new(
        roots.id.clone(),
        profile.profile_id.clone(),
        profile.config_id.clone(),
        "sha256:source",
        "sha256:summary",
        ArtifactVersion::new("bytecode-v1"),
    )
}

#[test]
fn runtime_kernel_ids_cover_root_definition_artifact_instance_and_host_mode() {
    let roots = roots();
    let profile = profile();
    let cache_key = cache_key(&roots, &profile);
    let definition = ApplicationDefinitionIdentity::new(
        roots.id.clone(),
        "applications/build.ash",
        "main",
        profile.profile_id.clone(),
        profile.config_id.clone(),
        "sha256:source",
    );
    let artifact = ApplicationArtifactIdentity::new(
        definition.id.clone(),
        cache_key.clone(),
        ArtifactVersion::new("bytecode-v1"),
    );
    let provider_registry =
        ProviderRegistryIdentity::new(vec!["fs".into(), "clock".into(), "fs".into()]);
    let instance = ApplicationInstanceIdentity::admit(
        RuntimeHostMode::OneShot,
        definition.id.clone(),
        artifact.id.clone(),
        profile.clone(),
        provider_registry.clone(),
        AdmissionIdentity::empty(),
    );
    let root_process_id = ProcessId::new();
    let process_tree = instance.process_tree(root_process_id);
    let kernel = RuntimeKernelIdentity::new(
        RuntimeHostMode::OneShot,
        roots.clone(),
        cache_key.clone(),
        RuntimeEngineRelationship::ExistingAshEngineEmbedded,
    );

    assert_eq!(kernel.host_mode, RuntimeHostMode::OneShot);
    assert_eq!(kernel.roots, roots);
    assert_eq!(kernel.cache_key, cache_key);
    assert_eq!(
        kernel.engine_relationship,
        RuntimeEngineRelationship::ExistingAshEngineEmbedded
    );
    assert_eq!(definition.root_id, roots.id);
    assert_eq!(
        definition.relative_module_path.as_str(),
        "applications/build.ash"
    );
    assert_eq!(definition.entry_name.as_str(), "main");
    assert_eq!(definition.profile_id, profile.profile_id);
    assert_eq!(definition.config_id, profile.config_id);
    assert_eq!(definition.source_identity, "sha256:source");
    assert_eq!(artifact.definition_id, definition.id);
    assert_eq!(artifact.cache_key, cache_key);
    assert_eq!(instance.host_mode, RuntimeHostMode::OneShot);
    assert_eq!(instance.definition_id, definition.id);
    assert_eq!(instance.artifact_id, artifact.id);
    assert_eq!(instance.profile, profile);
    assert_eq!(process_tree.application_instance_id, instance.id);
    assert_eq!(process_tree.root_process_id(), root_process_id);
    assert_eq!(process_tree.rooted_in(), instance.id);
    assert_eq!(instance.provider_registry, provider_registry);
    assert_eq!(provider_registry.provider_names, vec!["clock", "fs"]);
    assert!(
        !provider_registry.grants_admission_authority(),
        "provider registry identity must not become admission authority"
    );
    assert!(
        !instance.admission.has_authority_grants(),
        "admission authority must be explicit and separate from provider registration"
    );
}

#[test]
fn runtime_kernel_host_modes_share_definition_and_artifact_identity() {
    let roots = roots();
    let profile = profile();
    let cache_key = cache_key(&roots, &profile);
    let definition = ApplicationDefinitionIdentity::new(
        roots.id.clone(),
        "applications/deploy.ash",
        "main",
        profile.profile_id.clone(),
        profile.config_id.clone(),
        "sha256:source",
    );
    let artifact = ApplicationArtifactIdentity::new(
        definition.id.clone(),
        cache_key,
        ArtifactVersion::new("bytecode-v1"),
    );
    let provider_registry = ProviderRegistryIdentity::new(vec!["clock".into()]);
    let admission = AdmissionIdentity::empty()
        .with_capability_grant(CapabilityBindingId::new())
        .with_resource_grant(ResourceId::new());

    let one_shot = ApplicationInstanceIdentity::admit(
        RuntimeHostMode::OneShot,
        definition.id.clone(),
        artifact.id.clone(),
        profile.clone(),
        provider_registry.clone(),
        admission.clone(),
    );
    let daemon = ApplicationInstanceIdentity::admit(
        RuntimeHostMode::Daemon,
        definition.id.clone(),
        artifact.id.clone(),
        profile,
        provider_registry,
        admission,
    );

    assert_eq!(one_shot.host_mode, RuntimeHostMode::OneShot);
    assert_eq!(daemon.host_mode, RuntimeHostMode::Daemon);
    assert_eq!(one_shot.definition_id, daemon.definition_id);
    assert_eq!(one_shot.artifact_id, daemon.artifact_id);
    assert_ne!(
        one_shot.id, daemon.id,
        "each host-level start admits a distinct application instance"
    );
    assert!(one_shot.admission.has_authority_grants());
    assert!(daemon.admission.has_authority_grants());
    assert_ne!(
        one_shot
            .process_tree(ProcessId::new())
            .application_instance_id,
        daemon
            .process_tree(ProcessId::new())
            .application_instance_id
    );
}

#[test]
fn supervisor_runtime_profiles_are_authority_neutral_and_fail_closed() {
    let supervisor_process_id = ProcessId::new();
    let profile =
        SupervisorRuntimeProfile::bounded_restart("supervisor:main", supervisor_process_id, 2)
            .expect("bounded restart supervisor profile is supported");

    assert_eq!(profile.profile_name, "supervisor:main");
    assert_eq!(profile.supervisor_process_id, supervisor_process_id);
    assert_eq!(
        profile.policy,
        SupervisorPolicy::BoundedRestart { max_restarts: 2 }
    );
    assert!(!profile.grants_authority);

    assert_eq!(
        SupervisorRuntimeProfile::runtime_boundary(
            "supervisor:bad",
            supervisor_process_id,
            SupervisorPolicy::Unsupported {
                reason: "unbounded restart".to_string(),
            },
            false,
        )
        .expect_err("unsupported supervisor policies fail closed"),
        SupervisorDiagnostic::UnsupportedPolicy {
            profile_name: "supervisor:bad".to_string(),
            reason: "unbounded restart".to_string(),
        }
    );
    assert!(matches!(
        SupervisorRuntimeProfile::runtime_boundary(
            "supervisor:authority",
            supervisor_process_id,
            SupervisorPolicy::Cancel,
            true,
        )
        .expect_err("supervisor profiles cannot grant authority"),
        SupervisorDiagnostic::AuthorityWideningProfile { .. }
    ));
}

#[test]
fn external_actor_adapters_are_authority_neutral_and_fail_closed() {
    let adapter = ExternalActorAdapter::new(
        "actor:payments",
        ActorProtocol::HttpJson,
        "PaymentRequest",
        "{id: String, amount: Int}",
        "String",
        "capability:payments.charge",
        ActorCallPolicy::bounded(2, 5_000),
        false,
    )
    .expect("typed external actor adapter is supported");

    assert_eq!(adapter.name, "actor:payments");
    assert_eq!(adapter.protocol, ActorProtocol::HttpJson);
    assert_eq!(adapter.actor_type, "PaymentRequest");
    assert_eq!(adapter.inbound_schema, "{id: String, amount: Int}");
    assert_eq!(adapter.outbound_schema, "String");
    assert_eq!(adapter.capability_boundary, "capability:payments.charge");
    assert_eq!(adapter.policy, ActorCallPolicy::bounded(2, 5_000));
    assert_eq!(adapter.ownership, "owned-sendable");
    assert!(!adapter.grants_authority);

    assert_eq!(
        ExternalActorAdapter::new(
            "actor:unsupported",
            ActorProtocol::Unsupported {
                reason: "raw socket actor protocol has no typed adapter".to_string(),
            },
            "LegacyRequest",
            "String",
            "String",
            "capability:unsupported.call",
            ActorCallPolicy::bounded(0, 100),
            false,
        )
        .expect_err("unsupported actor protocols fail closed"),
        ExternalActorDiagnostic::UnsupportedProtocol {
            adapter_name: "actor:unsupported".to_string(),
            reason: "raw socket actor protocol has no typed adapter".to_string(),
        }
    );
    assert!(matches!(
        ExternalActorAdapter::new(
            "actor:authority",
            ActorProtocol::HttpJson,
            "AuthorityRequest",
            "String",
            "String",
            "capability:authority.call",
            ActorCallPolicy::bounded(0, 100),
            true,
        )
        .expect_err("actor adapters cannot grant authority"),
        ExternalActorDiagnostic::AuthorityWideningAdapter { .. }
    ));

    let alphabet = TraceAlphabet::new(vec![TraceFactKind::ExternalActor]);
    assert_eq!(alphabet.interpretation(), TraceInterpretation::Operational);
}
