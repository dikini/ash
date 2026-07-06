//! `RuntimeKernel` artifact-builder integration tests.

use ash_core::core_ash_contract::{MonitorEvaluationResult, RuntimeMonitorEvidence, TraceFactKind};
use ash_core::runtime::{RuntimeTraceEvent, RuntimeTraceFact};
use ash_core::runtime_kernel::{
    ApplicationAdmissionProfile, ApplicationAdmissionProfileDiagnostic,
    ApplicationBoundaryBindingDiagnostic, ApplicationBoundaryBindingManifest,
    ApplicationBoundaryBindings, ApplicationEntrypointDiagnostic, ApplicationEntrypointKind,
    ApplicationEntrypointMetadata, ApplicationRuntimeReport, ApplicationTerminalOutcome,
    ApplicationTraceBundle,
};
use ash_engine::runtime_artifact::{RuntimeArtifactBuildRequest, build_runtime_kernel_artifact};

fn request(source: &str) -> RuntimeArtifactBuildRequest {
    RuntimeArtifactBuildRequest::new(
        "workspace:/task-935",
        "workflows/demo.ash",
        "main",
        "default",
        "default",
        source,
        "engine-check:ok;warnings=0",
    )
}

#[test]
fn engine_builder_is_host_agnostic_for_one_shot_and_daemon_callers() {
    let source = "workflow main() { return 7 }";

    let one_shot = build_runtime_kernel_artifact(&request(source)).expect("one-shot artifact");
    let daemon = build_runtime_kernel_artifact(&request(source)).expect("daemon artifact");

    assert_eq!(one_shot, daemon);
    assert_eq!(
        one_shot.definition.relative_module_path,
        "workflows/demo.ash"
    );
    assert_eq!(one_shot.definition.workflow_name, "main");
    assert_eq!(one_shot.definition.source_identity, one_shot.source_hash);
    assert_eq!(one_shot.artifact.cache_key, one_shot.cache_key);
    assert_eq!(
        serde_json::to_value(one_shot.tcir.carrier_scope).expect("carrier scope json"),
        "alpha_checked_workflow_boundary"
    );
    assert_eq!(one_shot.bytecode.instruction_count, 1);
    assert!(
        !one_shot.bytecode.requires_source_reparse,
        "shared engine builder must not ask bytecode verification to reparse source"
    );
}

#[test]
fn engine_builder_changes_only_source_and_check_hashes_for_source_or_check_changes() {
    let baseline = build_runtime_kernel_artifact(&request("workflow main() { return 7 }"))
        .expect("baseline artifact");
    let changed_source = build_runtime_kernel_artifact(&request("workflow main() { return 8 }"))
        .expect("changed-source artifact");
    let changed_check = build_runtime_kernel_artifact(&RuntimeArtifactBuildRequest::new(
        "workspace:/task-935",
        "workflows/demo.ash",
        "main",
        "default",
        "default",
        "workflow main() { return 7 }",
        "engine-check:ok;warnings=1",
    ))
    .expect("changed-check artifact");

    assert_ne!(baseline.source_hash, changed_source.source_hash);
    assert_ne!(
        baseline.check_summary_hash,
        changed_check.check_summary_hash
    );
    assert_eq!(baseline.tcir, changed_source.tcir);
    assert_eq!(baseline.amir, changed_source.amir);
    assert_eq!(baseline.bytecode, changed_source.bytecode);
}

#[test]
fn engine_builder_carries_application_entrypoint_metadata_over_checked_callable() {
    let request = RuntimeArtifactBuildRequest::new_application_entrypoint(
        "workspace:/task-1916",
        "src/app.ash",
        "main",
        "callable:src/app.ash::main",
        "runtime-target:application-entry:main",
        "default",
        "default",
        "fn main() -> Int { 7 }",
        "engine-check:ok;warnings=0;callable=main",
    )
    .expect("valid checked-callable application entrypoint metadata");

    let artifact = build_runtime_kernel_artifact(&request).expect("application entry artifact");

    assert_eq!(
        artifact.entrypoint.kind,
        ApplicationEntrypointKind::CheckedCallable
    );
    assert_eq!(artifact.entrypoint.name, "main");
    assert_eq!(
        artifact.entrypoint.callable_identity.as_deref(),
        Some("callable:src/app.ash::main")
    );
    assert_eq!(
        artifact.entrypoint.runtime_target_identity,
        "runtime-target:application-entry:main"
    );
    assert_eq!(artifact.entrypoint.relative_module_path, "src/app.ash");
    assert_eq!(artifact.invocation_packet.entrypoint, artifact.entrypoint);
    assert_eq!(
        artifact.invocation_packet.source_identity,
        artifact.source_hash
    );
    assert_eq!(
        artifact.invocation_packet.check_identity,
        artifact.check_summary_hash
    );
    assert_eq!(
        artifact.invocation_packet.runtime_target_identity,
        artifact.artifact.id.as_str()
    );
    assert_eq!(
        artifact.definition.workflow_name, "main",
        "legacy definition identity is only a compatibility mirror"
    );
}

#[test]
fn application_entrypoint_metadata_rejects_missing_callable_identity() {
    let err = ApplicationEntrypointMetadata::checked_callable(
        "main",
        "",
        "src/app.ash",
        "runtime-target:application-entry:main",
    )
    .expect_err("checked callable entrypoint requires callable identity");

    assert_eq!(
        err,
        ApplicationEntrypointDiagnostic::MissingCallableIdentity {
            entrypoint_name: "main".to_string(),
        }
    );
}

#[test]
fn application_entrypoint_diagnostics_are_structured() {
    let ambiguous = ApplicationEntrypointDiagnostic::ambiguous(
        "main",
        ["callable:app::main", "callable:other::main"],
    );
    assert!(matches!(
        ambiguous,
        ApplicationEntrypointDiagnostic::AmbiguousEntrypoint { .. }
    ));

    let stale = ApplicationEntrypointDiagnostic::stale("main", "source:old", "source:new");
    assert!(matches!(
        stale,
        ApplicationEntrypointDiagnostic::StaleEntrypoint { .. }
    ));

    let incompatible = ApplicationEntrypointDiagnostic::incompatible(
        "main",
        "expected zero-argument checked callable",
        "found legacy workflow compatibility header",
    );
    assert!(matches!(
        incompatible,
        ApplicationEntrypointDiagnostic::IncompatibleEntrypoint { .. }
    ));
}

#[test]
fn engine_builder_carries_admission_profile_boundary_in_invocation_packet() {
    let request = RuntimeArtifactBuildRequest::new_application_entrypoint(
        "workspace:/task-1917",
        "src/app.ash",
        "main",
        "callable:src/app.ash::main",
        "runtime-target:application-entry:main",
        "default",
        "default",
        "fn main() -> Int { 7 }",
        "engine-check:ok;warnings=0;callable=main",
    )
    .expect("valid checked-callable application entrypoint metadata")
    .with_admission_profile(
        ApplicationAdmissionProfile::runtime_boundary("allow", "cli:--admission-profile", false)
            .expect("valid non-authority admission profile boundary"),
    );

    let artifact = build_runtime_kernel_artifact(&request).expect("application entry artifact");
    let admission_profile = &artifact.invocation_packet.admission_profile;

    assert_eq!(admission_profile.name, "allow");
    assert_eq!(
        admission_profile.profile_identity,
        "admission-profile:allow"
    );
    assert_eq!(admission_profile.boundary_source, "cli:--admission-profile");
    assert!(
        !admission_profile.grants_authority,
        "profile selection metadata must not become authority"
    );
}

#[test]
fn application_admission_profile_diagnostics_are_structured_and_fail_closed() {
    assert_eq!(
        ApplicationAdmissionProfile::runtime_boundary("", "cli:--admission-profile", false)
            .expect_err("missing profile name must fail closed"),
        ApplicationAdmissionProfileDiagnostic::MissingProfileName
    );
    assert!(matches!(
        ApplicationAdmissionProfile::runtime_boundary(
            "allow capabilities",
            "cli:--admission-profile",
            false
        )
        .expect_err("malformed profile name must fail closed"),
        ApplicationAdmissionProfileDiagnostic::MalformedProfileName { .. }
    ));
    assert!(matches!(
        ApplicationAdmissionProfile::runtime_boundary("allow", "cli:--admission-profile", true)
            .expect_err("authority-widening profile metadata must fail closed"),
        ApplicationAdmissionProfileDiagnostic::AuthorityWideningProfile { .. }
    ));
    assert!(matches!(
        ApplicationAdmissionProfileDiagnostic::stale("allow", "profile:v1", "profile:v2"),
        ApplicationAdmissionProfileDiagnostic::StaleProfile { .. }
    ));
    assert!(matches!(
        ApplicationAdmissionProfileDiagnostic::incompatible(
            "allow",
            "resource profile default",
            "provider profile production"
        ),
        ApplicationAdmissionProfileDiagnostic::IncompatibleProfile { .. }
    ));
}

#[test]
fn engine_builder_carries_application_boundary_bindings_in_invocation_packet() {
    let bindings = ApplicationBoundaryBindings::from_manifest(
        "engine:test-boundary",
        ApplicationBoundaryBindingManifest {
            roles: vec!["role:operator".to_string()],
            policies: vec!["policy:nightly".to_string()],
            resources: vec!["resource:cache".to_string()],
            providers: vec!["provider:stdio".to_string()],
            contracts: vec!["contract:preflight".to_string()],
            grants_authority: false,
        },
    )
    .expect("valid non-authority boundary bindings");
    let request = RuntimeArtifactBuildRequest::new_application_entrypoint(
        "workspace:/task-1918",
        "src/app.ash",
        "main",
        "callable:src/app.ash::main",
        "runtime-target:application-entry:main",
        "default",
        "default",
        "fn main() -> Int { 7 }",
        "engine-check:ok;warnings=0;callable=main",
    )
    .expect("valid checked-callable application entrypoint metadata")
    .with_boundary_bindings(bindings.clone());

    let artifact = build_runtime_kernel_artifact(&request).expect("application entry artifact");

    assert_eq!(artifact.invocation_packet.boundary_bindings, bindings);
    assert_eq!(
        artifact.invocation_packet.boundary_bindings.boundary_source,
        "engine:test-boundary"
    );
    assert_eq!(
        artifact.invocation_packet.boundary_bindings.roles,
        ["role:operator"]
    );
    assert_eq!(
        artifact.invocation_packet.boundary_bindings.policies,
        ["policy:nightly"]
    );
    assert_eq!(
        artifact.invocation_packet.boundary_bindings.resources,
        ["resource:cache"]
    );
    assert_eq!(
        artifact.invocation_packet.boundary_bindings.providers,
        ["provider:stdio"]
    );
    assert_eq!(
        artifact.invocation_packet.boundary_bindings.contracts,
        ["contract:preflight"]
    );
    assert!(
        !artifact
            .invocation_packet
            .boundary_bindings
            .grants_authority,
        "boundary binding metadata must not discharge rows or grant authority"
    );
}

#[test]
fn application_boundary_binding_diagnostics_are_structured_and_fail_closed() {
    assert_eq!(
        ApplicationBoundaryBindings::from_manifest(
            "engine:test-boundary",
            ApplicationBoundaryBindingManifest {
                roles: vec![String::new()],
                ..ApplicationBoundaryBindingManifest::default()
            },
        )
        .expect_err("missing role binding must fail closed"),
        ApplicationBoundaryBindingDiagnostic::MissingBindingIdentity {
            family: "role".to_string()
        }
    );
    assert!(matches!(
        ApplicationBoundaryBindings::from_manifest(
            "engine:test-boundary",
            ApplicationBoundaryBindingManifest {
                providers: vec!["provider with spaces".to_string()],
                ..ApplicationBoundaryBindingManifest::default()
            },
        )
        .expect_err("malformed provider binding must fail closed"),
        ApplicationBoundaryBindingDiagnostic::MalformedBindingIdentity { .. }
    ));
    assert!(matches!(
        ApplicationBoundaryBindings::from_manifest(
            "engine:test-boundary",
            ApplicationBoundaryBindingManifest {
                contracts: vec!["contract:preflight".to_string()],
                grants_authority: true,
                ..ApplicationBoundaryBindingManifest::default()
            },
        )
        .expect_err("authority-widening boundary metadata must fail closed"),
        ApplicationBoundaryBindingDiagnostic::AuthorityWideningBinding { .. }
    ));
    assert!(matches!(
        ApplicationBoundaryBindingDiagnostic::stale(
            "contract",
            "contract:preflight",
            "evidence:v1",
            "evidence:v2"
        ),
        ApplicationBoundaryBindingDiagnostic::StaleBinding { .. }
    ));
    assert!(matches!(
        ApplicationBoundaryBindingDiagnostic::incompatible(
            "resource",
            "resource:cache",
            "read-only row evidence",
            "write-required handler"
        ),
        ApplicationBoundaryBindingDiagnostic::IncompatibleBinding { .. }
    ));
}

#[test]
fn application_trace_bundle_and_report_project_invocation_identity_without_authority() {
    let bindings = ApplicationBoundaryBindings::from_manifest(
        "engine:test-boundary",
        ApplicationBoundaryBindingManifest {
            contracts: vec!["contract:preflight".to_string()],
            providers: vec!["provider:stdio".to_string()],
            grants_authority: false,
            ..ApplicationBoundaryBindingManifest::default()
        },
    )
    .expect("valid non-authority boundary bindings");
    let request = RuntimeArtifactBuildRequest::new_application_entrypoint(
        "workspace:/task-1919",
        "src/app.ash",
        "main",
        "callable:src/app.ash::main",
        "runtime-target:application-entry:main",
        "default",
        "default",
        "fn main() -> Int { 7 }",
        "engine-check:ok;warnings=0;callable=main",
    )
    .expect("valid checked-callable application entrypoint metadata")
    .with_boundary_bindings(bindings);
    let artifact = build_runtime_kernel_artifact(&request).expect("application entry artifact");
    let process_fact = RuntimeTraceFact::new(
        TraceFactKind::Process,
        RuntimeTraceEvent::Complete,
        "process:main",
    );
    let monitor_evidence = RuntimeMonitorEvidence::new(
        "monitor:application",
        "contract:preflight",
        "boundary:main",
        MonitorEvaluationResult::Pending,
    );

    let trace_bundle = ApplicationTraceBundle::from_invocation_packet(
        &artifact.invocation_packet,
        vec![process_fact.clone()],
        vec![monitor_evidence.clone()],
    );
    let report = ApplicationRuntimeReport::new(
        &artifact.invocation_packet,
        ApplicationTerminalOutcome::succeeded(),
        trace_bundle.clone(),
    );

    assert_eq!(trace_bundle.source_identity, artifact.source_hash);
    assert_eq!(trace_bundle.check_identity, artifact.check_summary_hash);
    assert_eq!(
        trace_bundle.entrypoint_identity,
        "runtime-target:application-entry:main"
    );
    assert_eq!(
        trace_bundle.admission_profile_identity,
        "admission-profile:empty"
    );
    assert_eq!(
        trace_bundle.boundary_evidence_identity,
        artifact
            .invocation_packet
            .boundary_bindings
            .redacted_evidence_identity
    );
    assert_eq!(trace_bundle.process_facts, [process_fact]);
    assert_eq!(trace_bundle.contract_evidence, ["contract:preflight"]);
    assert_eq!(trace_bundle.monitor_evidence, [monitor_evidence]);
    assert!(!trace_bundle.grants_authority);
    assert!(!trace_bundle.mutates_authority);
    assert_eq!(report.terminal_outcome.status.as_str(), "succeeded");
    assert_eq!(report.trace_bundle, trace_bundle);
    assert!(!report.grants_authority);
    assert!(!report.mutates_authority);
}
