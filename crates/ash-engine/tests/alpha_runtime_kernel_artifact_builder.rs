//! `RuntimeKernel` artifact-builder integration tests.

use ash_core::Span;
use ash_core::core_ash::{CoreRow, CoreType};
use ash_core::core_ash_contract::{MonitorEvaluationResult, RuntimeMonitorEvidence, TraceFactKind};
use ash_core::runtime::{RuntimeTraceEvent, RuntimeTraceFact};
use ash_core::runtime_kernel::{
    ApplicationAdmissionProfile, ApplicationAdmissionProfileDiagnostic,
    ApplicationBoundaryBindingDiagnostic, ApplicationBoundaryBindingManifest,
    ApplicationBoundaryBindings, ApplicationEntrypointDiagnostic, ApplicationEntrypointKind,
    ApplicationEntrypointMetadata, ApplicationRuntimeReport, ApplicationTerminalOutcome,
    ApplicationTraceBundle, CheckedFunctionArtifact,
};
use ash_core::semantic_summary::{SourceAnchor, SourceOrigin};
use ash_engine::runtime_artifact::{RuntimeArtifactBuildRequest, build_runtime_kernel_artifact};
use std::collections::HashMap;

fn request(source: &str) -> RuntimeArtifactBuildRequest {
    RuntimeArtifactBuildRequest::new_application_entrypoint(
        "workspace:/task-935",
        "applications/demo.ash",
        "main",
        "callable:applications/demo.ash::main",
        "runtime-target:application-entry:main",
        "default",
        "default",
        checked_function(source, "callable:applications/demo.ash::main"),
        source,
        "engine-check:ok;warnings=0",
    )
    .expect("valid checked function request")
}

fn checked_function(source: &str, function_identity: &str) -> CheckedFunctionArtifact {
    CheckedFunctionArtifact {
        function_identity: function_identity.to_string(),
        effect_row: CoreRow::default(),
        body: ash_core::Expr::Literal(ash_core::Value::Int(7)),
        source_anchor: SourceAnchor::new(
            SourceOrigin::File("applications/demo.ash".to_string()),
            Some(Span {
                start: 0,
                end: source.len(),
            }),
            "checked-function:main",
        ),
        result_type: CoreType::Base("Int".to_string()),
    }
}

#[test]
fn engine_builder_is_host_agnostic_for_one_shot_and_daemon_callers() {
    let source = "fn main() { return 7 }";

    let one_shot = build_runtime_kernel_artifact(&request(source)).expect("one-shot artifact");
    let daemon = build_runtime_kernel_artifact(&request(source)).expect("daemon artifact");

    assert_eq!(one_shot, daemon);
    assert_eq!(
        one_shot.definition.relative_module_path,
        "applications/demo.ash"
    );
    assert_eq!(one_shot.definition.entry_name, "main");
    assert_eq!(one_shot.definition.source_identity, one_shot.source_hash);
    assert_eq!(one_shot.artifact.cache_key, one_shot.cache_key);
    assert_eq!(
        serde_json::to_value(one_shot.tcir.carrier_scope).expect("carrier scope json"),
        "checked_function_artifact"
    );
    assert_eq!(
        one_shot.tcir.target_display,
        "function:callable:applications/demo.ash::main"
    );
    assert_eq!(
        one_shot.tcir.evidence_key,
        "RuntimeKernelFunction<callable:applications/demo.ash::main;pure>"
    );
    assert_eq!(one_shot.bytecode.instruction_count, 1);
    assert!(
        !one_shot.bytecode.requires_source_reparse,
        "shared engine builder must not ask bytecode verification to reparse source"
    );
}

#[test]
fn engine_builder_changes_only_source_and_check_hashes_for_source_or_check_changes() {
    let baseline = build_runtime_kernel_artifact(&request("fn main() { return 7 }"))
        .expect("baseline artifact");
    let changed_source = build_runtime_kernel_artifact(&request("fn main() { return 8 }"))
        .expect("changed-source artifact");
    let mut changed_check_request = request("fn main() { return 7 }");
    changed_check_request.check_summary = "engine-check:ok;warnings=1".to_string();
    let changed_check =
        build_runtime_kernel_artifact(&changed_check_request).expect("changed-check artifact");

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
        checked_function("fn main() -> Int { 7 }", "callable:src/app.ash::main"),
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
    assert_eq!(artifact.definition.entry_name, "main");
}

#[test]
fn runtime_artifact_request_rejects_checked_function_identity_mismatched_with_entrypoint() {
    let error = RuntimeArtifactBuildRequest::new_application_entrypoint(
        "workspace:/task-1972",
        "src/app.ash",
        "main",
        "callable:src/app.ash::main",
        "runtime-target:application-entry:main",
        "default",
        "default",
        checked_function("fn main() -> Int { 7 }", "callable:src/app.ash::other"),
        "fn main() -> Int { 7 }",
        "engine-check:ok",
    )
    .expect_err("entrypoint identity must match the checked function identity");

    assert_eq!(
        error,
        ApplicationEntrypointDiagnostic::IncompatibleEntrypoint {
            entrypoint_name: "main".to_string(),
            expected: "callable:src/app.ash::main".to_string(),
            actual: "callable:src/app.ash::other".to_string(),
        }
    );
}

#[test]
fn runtime_artifact_amir_provenance_changes_with_checked_function_body() {
    fn checked_main(source: &str) -> CheckedFunctionArtifact {
        let engine = ash_engine::Engine::new().build().expect("engine builds");
        let mut entry = engine
            .parse_entry_source(source)
            .expect("fn main source parses");
        engine
            .check_entry_artifact(
                &mut entry,
                "callable:src/app.ash::main",
                SourceAnchor::new(
                    SourceOrigin::File("src/app.ash".to_string()),
                    Some(Span {
                        start: 0,
                        end: source.len(),
                    }),
                    "checked-function:main",
                ),
            )
            .expect("fn main source checks and lowers")
    }

    let source_returning_seven = "fn main() -> Int { 7 }";
    let source_returning_eight = "fn main() -> Int { 8 }";
    let build = |source: &str| {
        RuntimeArtifactBuildRequest::new_application_entrypoint(
            "workspace:/task-1972",
            "src/app.ash",
            "main",
            "callable:src/app.ash::main",
            "runtime-target:application-entry:main",
            "default",
            "default",
            checked_main(source),
            source,
            "engine-check:ok",
        )
        .expect("matching checked function request")
    };

    let seven = build_runtime_kernel_artifact(&build(source_returning_seven))
        .expect("artifact for checked body returning seven");
    let eight = build_runtime_kernel_artifact(&build(source_returning_eight))
        .expect("artifact for checked body returning eight");

    assert_ne!(
        seven.amir.provenance, eight.amir.provenance,
        "AMIR provenance must preserve the checked function body rather than lowering every entry to a Null placeholder"
    );
}

#[test]
fn runtime_artifact_cache_identity_changes_with_checked_function_body() {
    fn checked_main(source: &str) -> CheckedFunctionArtifact {
        let engine = ash_engine::Engine::new().build().expect("engine builds");
        let mut entry = engine
            .parse_entry_source(source)
            .expect("fn main source parses");
        engine
            .check_entry_artifact(
                &mut entry,
                "callable:src/app.ash::main",
                SourceAnchor::new(
                    SourceOrigin::File("src/app.ash".to_string()),
                    Some(Span {
                        start: 0,
                        end: source.len(),
                    }),
                    "checked-function:main",
                ),
            )
            .expect("fn main source checks and lowers")
    }

    let source_returning_seven = "fn main() -> Int { 7 }";
    let source_returning_eight = "fn main() -> Int { 8 }";
    let shared_source = "fn main() -> Int { 0 }";
    let build = |checked_function: CheckedFunctionArtifact| {
        RuntimeArtifactBuildRequest::new_application_entrypoint(
            "workspace:/task-1972",
            "src/app.ash",
            "main",
            "callable:src/app.ash::main",
            "runtime-target:application-entry:main",
            "default",
            "default",
            checked_function,
            shared_source,
            "engine-check:ok",
        )
        .expect("matching checked function request")
    };

    let seven = build_runtime_kernel_artifact(&build(checked_main(source_returning_seven)))
        .expect("artifact for checked body returning seven");
    let eight = build_runtime_kernel_artifact(&build(checked_main(source_returning_eight)))
        .expect("artifact for checked body returning eight");

    assert_ne!(
        seven.amir.provenance, eight.amir.provenance,
        "AMIR provenance must distinguish the checked bodies"
    );
    assert_ne!(
        seven.cache_key, eight.cache_key,
        "cache keys must include the checked/lowered body, not only source and check-summary text"
    );
    assert_ne!(
        seven.artifact.id, eight.artifact.id,
        "artifact identities must change whenever checked-function provenance changes"
    );
}

#[test]
fn runtime_artifact_cache_identity_changes_with_checked_function_result_type() {
    let source = "fn main() -> Int { 7 }";
    let checked_function = checked_function(source, "callable:applications/demo.ash::main");
    let mut changed_result_type = checked_function.clone();
    changed_result_type.result_type = CoreType::Base("String".to_string());

    let build = |checked_function: CheckedFunctionArtifact| {
        RuntimeArtifactBuildRequest::new_application_entrypoint(
            "workspace:/task-1972",
            "applications/demo.ash",
            "main",
            "callable:applications/demo.ash::main",
            "runtime-target:application-entry:main",
            "default",
            "default",
            checked_function,
            source,
            "engine-check:ok",
        )
        .expect("matching checked function request")
    };

    let int_artifact = build_runtime_kernel_artifact(&build(checked_function))
        .expect("artifact for checked Int result");
    let string_artifact = build_runtime_kernel_artifact(&build(changed_result_type))
        .expect("artifact for checked String result");

    assert_ne!(
        int_artifact.cache_key, string_artifact.cache_key,
        "cache keys must include the complete checked TCIR, including result type"
    );
    assert_ne!(
        int_artifact.artifact.id, string_artifact.artifact.id,
        "artifact identities must change whenever checked result-type provenance changes"
    );
}

#[test]
fn runtime_artifact_cache_identity_is_independent_of_checked_record_insertion_order() {
    let source = "fn main() -> Record { payload }";
    let record_in_order = (0..8).fold(HashMap::new(), |mut fields, index| {
        fields.insert(format!("field_{index}"), ash_core::Value::Int(index));
        fields
    });
    let record_in_reverse_order = (0..8).rev().fold(HashMap::new(), |mut fields, index| {
        fields.insert(format!("field_{index}"), ash_core::Value::Int(index));
        fields
    });
    let mut first_checked = checked_function(source, "callable:applications/demo.ash::main");
    first_checked.body =
        ash_core::Expr::Literal(ash_core::Value::Record(Box::new(record_in_order)));
    let mut second_checked = first_checked.clone();
    second_checked.body =
        ash_core::Expr::Literal(ash_core::Value::Record(Box::new(record_in_reverse_order)));

    let build = |checked_function: CheckedFunctionArtifact| {
        RuntimeArtifactBuildRequest::new_application_entrypoint(
            "workspace:/task-1972",
            "applications/demo.ash",
            "main",
            "callable:applications/demo.ash::main",
            "runtime-target:application-entry:main",
            "default",
            "default",
            checked_function,
            source,
            "engine-check:ok",
        )
        .expect("matching checked function request")
    };

    let first = build_runtime_kernel_artifact(&build(first_checked))
        .expect("artifact for record inserted in source order");
    let second = build_runtime_kernel_artifact(&build(second_checked))
        .expect("artifact for record inserted in reverse order");

    assert_eq!(
        first.cache_key, second.cache_key,
        "logically equal checked records must have a canonical cache identity"
    );
    assert_eq!(
        first.artifact.id, second.artifact.id,
        "logically equal checked records must have a canonical artifact identity"
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
        "found removed application declaration metadata",
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
        checked_function("fn main() -> Int { 7 }", "callable:src/app.ash::main"),
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
        checked_function("fn main() -> Int { 7 }", "callable:src/app.ash::main"),
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
        checked_function("fn main() -> Int { 7 }", "callable:src/app.ash::main"),
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
