//! TASK-1940 standard provider/admission profile tests.

use ash_core::ast::Predicate;
use ash_core::capability::CapabilityProvider;
use ash_core::runtime::HostBoundaryOutcome;
use ash_core::{Capability, CapabilityBindingKind, Constraint, Effect, Value};
use ash_engine::providers::{FsProvider, HttpProvider, TimeProvider};
use ash_engine::standard_profiles::{StandardProfileKind, StandardProviderProfile};
use ash_runtime::RuntimeState;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn readonly_filesystem_profile_admits_read_rows_and_denies_writes() {
    let root = tempdir().expect("tempdir");
    let allowed_file = root.path().join("input.txt");
    std::fs::write(&allowed_file, "phase-198").expect("write fixture");

    let provider = Arc::new(FsProvider::new());
    let profile = StandardProviderProfile::read_only_filesystem("phase198-readonly", [root.path()]);
    let runtime = RuntimeState::new().with_provider("fs", provider);
    let installed = profile
        .install(&runtime)
        .await
        .expect("readonly profile installs");

    assert!(!installed.grants_authority);
    assert_eq!(installed.binding_ids.len(), 1);
    assert!(
        installed
            .sandbox_policies
            .iter()
            .any(|policy| policy == "host.fs.read_to_string")
    );

    let binding = runtime
        .capability_binding(installed.binding_ids[0])
        .await
        .expect("binding admitted");
    let CapabilityBindingKind::HostProvider {
        provider_name,
        admitted_capabilities,
    } = binding.kind
    else {
        panic!("expected host provider binding");
    };
    assert_eq!(provider_name, "fs");
    assert!(admitted_capabilities.contains(&"fs.read".to_string()));
    assert!(!admitted_capabilities.contains(&"fs.write".to_string()));

    let ctx = runtime
        .create_capability_context_for_bindings(&installed.binding_ids)
        .await
        .expect("projected capability context");
    let write = ctx
        .execute(
            "fs",
            "write",
            &[
                Value::String(allowed_file.display().to_string()),
                Value::String("blocked".to_string()),
            ],
        )
        .await;
    assert!(write.is_err(), "read-only profile must not expose fs.write");
}

#[tokio::test]
async fn deterministic_test_profile_installs_fixed_clock_provider() {
    let fixed_epoch_millis = 1_700_000_000_000_u64;
    let fixed_epoch_value = i64::try_from(fixed_epoch_millis).expect("fixture fits i64");
    let profile = StandardProviderProfile::deterministic_test("phase198-test", fixed_epoch_millis);
    let runtime = RuntimeState::new();
    let installed = profile
        .install(&runtime)
        .await
        .expect("deterministic profile installs");

    assert_eq!(installed.kind, StandardProfileKind::DeterministicTest);
    assert!(!installed.grants_authority);
    assert_eq!(installed.binding_ids.len(), 1);

    let time = runtime
        .get_provider("time")
        .expect("time provider installed");
    let value = time
        .execute("epoch_millis", &[])
        .await
        .expect("epoch_millis executes");
    assert_eq!(value, Value::Int(fixed_epoch_value));

    let binding = runtime
        .capability_binding(installed.binding_ids[0])
        .await
        .expect("time binding admitted");
    let CapabilityBindingKind::HostProvider {
        provider_name,
        admitted_capabilities,
    } = binding.kind
    else {
        panic!("expected time host provider binding");
    };
    assert_eq!(provider_name, "time");
    assert_eq!(
        admitted_capabilities,
        vec![
            "time.now".to_string(),
            "time.now_iso".to_string(),
            "time.epoch_millis".to_string(),
            "time.sleep".to_string()
        ]
    );

    let ctx = runtime
        .create_capability_context_for_bindings(&installed.binding_ids)
        .await
        .expect("projected deterministic context");
    let value = ctx
        .observe(&Capability {
            name: "time".to_string(),
            effect: Effect::Epistemic,
            constraints: vec![Constraint {
                predicate: Predicate {
                    name: "epoch_millis".to_string(),
                    arguments: vec![],
                },
            }],
        })
        .await
        .expect("projected epoch_millis observe executes");
    assert_eq!(value, Value::Int(fixed_epoch_value));

    let evidence = runtime.host_boundary_evidence().await;
    assert!(
        evidence.iter().any(|record| record.provider_name == "time"
            && record.operation_name == "epoch_millis"
            && record.outcome == HostBoundaryOutcome::Succeeded
            && record.authority_neutral),
        "clock observation should record authority-neutral evidence: {evidence:?}"
    );

    let denied_sleep = ctx.execute("time", "sleep", &[Value::Int(1)]).await;
    assert!(
        denied_sleep.is_err(),
        "deterministic clock profile must deny wall-clock sleep"
    );
    let evidence = runtime.host_boundary_evidence().await;
    assert!(
        evidence.iter().any(|record| record.provider_name == "time"
            && record.operation_name == "sleep"
            && record.outcome == HostBoundaryOutcome::Denied),
        "sleep denial should record host-boundary evidence: {evidence:?}"
    );
}

#[test]
fn sandboxed_http_profile_declares_explicit_host_and_method_expectations() {
    let profile = StandardProviderProfile::sandboxed_http("phase198-http", ["api.example.com"]);

    assert_eq!(profile.kind(), StandardProfileKind::SandboxedHttp);
    assert_eq!(
        profile.provider_rows("http"),
        Some(vec![
            "http.get".to_string(),
            "http.head".to_string(),
            "http.post".to_string(),
            "http.put".to_string(),
            "http.delete".to_string(),
        ])
    );
    assert!(
        profile
            .sandbox_policies()
            .iter()
            .all(|policy| policy.starts_with("host.http."))
    );
    assert!(
        !profile.grants_authority(),
        "profile metadata must not grant authority"
    );
}

#[tokio::test]
async fn malformed_and_authority_widening_profiles_fail_closed() {
    let runtime = RuntimeState::new();
    let malformed = StandardProviderProfile::deterministic_test("", 0);
    let error = malformed
        .install(&runtime)
        .await
        .expect_err("empty profile name must fail closed");
    assert!(
        error
            .to_string()
            .contains("standard profile is missing name"),
        "{error}"
    );

    let widening = StandardProviderProfile::deterministic_test("widening", 0)
        .with_authority_grant_for_test(true);
    let error = widening
        .install(&runtime)
        .await
        .expect_err("authority-widening profile metadata must fail closed");
    assert!(
        error.to_string().contains("must not grant authority"),
        "{error}"
    );
}

#[tokio::test]
async fn stale_profile_rows_fail_closed_as_incompatible_metadata() {
    let runtime = RuntimeState::new();
    let stale = StandardProviderProfile::deterministic_test("phase198-stale", 0)
        .with_provider_rows_for_test("time", ["time.now", "time.stale"]);

    let error = stale
        .install(&runtime)
        .await
        .expect_err("stale profile row metadata must fail closed");
    assert!(
        error.to_string().contains("time.stale"),
        "diagnostic should identify stale row: {error}"
    );
}

#[tokio::test]
async fn application_default_profile_installs_explicit_standard_provider_rows() {
    let root = tempdir().expect("tempdir");
    let runtime = RuntimeState::new();
    let profile = StandardProviderProfile::application_default(
        "phase198-app",
        [root.path()],
        ["api.example.com"],
    );
    let installed = profile
        .install(&runtime)
        .await
        .expect("application default profile installs");

    assert_eq!(installed.kind, StandardProfileKind::ApplicationDefault);
    assert_eq!(installed.binding_ids.len(), 4);
    assert!(!installed.grants_authority);
    assert!(runtime.get_provider("fs").is_some());
    assert!(runtime.get_provider("http").is_some());
    assert!(runtime.get_provider("time").is_some());
    assert!(runtime.get_provider("logging").is_some());

    let mut admitted = Vec::new();
    for binding_id in installed.binding_ids {
        let binding = runtime
            .capability_binding(binding_id)
            .await
            .expect("application binding admitted");
        let CapabilityBindingKind::HostProvider {
            provider_name,
            admitted_capabilities,
        } = binding.kind
        else {
            panic!("expected host provider binding");
        };
        admitted.push((provider_name, admitted_capabilities));
    }
    admitted.sort_by(|left, right| left.0.cmp(&right.0));

    assert_eq!(admitted[0].0, "fs");
    assert_eq!(
        admitted[0].1,
        vec![
            "fs.exists".to_string(),
            "fs.read".to_string(),
            "fs.metadata".to_string(),
            "fs.read_dir".to_string()
        ]
    );
    assert_eq!(admitted[1].0, "http");
    assert_eq!(admitted[2].0, "logging");
    assert_eq!(admitted[3].0, "time");
}

#[tokio::test]
async fn logging_only_profile_records_denied_log_attempt_evidence() {
    let profile = StandardProviderProfile::logging_only("phase198-log");
    let runtime = RuntimeState::new();
    let installed = profile
        .install(&runtime)
        .await
        .expect("logging profile installs");
    let ctx = runtime
        .create_capability_context_for_bindings(&installed.binding_ids)
        .await
        .expect("projected logging context");
    let denied = ctx
        .execute(
            "logging",
            "info",
            &[Value::String("secret=123".to_string())],
        )
        .await;
    assert!(
        denied.is_err(),
        "default logging-only profile denies host log writes"
    );

    let evidence = runtime.host_boundary_evidence().await;
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].provider_name, "logging");
    assert_eq!(evidence[0].operation_name, "info");
    assert_eq!(evidence[0].outcome, HostBoundaryOutcome::Denied);
    assert!(evidence[0].authority_neutral);
    assert!(!evidence[0].redacted_subject.contains("secret=123"));
}

#[tokio::test]
async fn read_write_filesystem_profile_records_success_and_failure_evidence() {
    let root = tempdir().expect("tempdir");
    let allowed_file = root.path().join("output.txt");
    let outside_dir = tempdir().expect("outside tempdir");
    let outside_file = outside_dir.path().join("blocked.txt");

    let runtime = RuntimeState::new();
    let profile =
        StandardProviderProfile::read_write_filesystem("phase198-readwrite", [root.path()]);
    let installed = profile
        .install(&runtime)
        .await
        .expect("read-write profile installs");
    let ctx = runtime
        .create_capability_context_for_bindings(&installed.binding_ids)
        .await
        .expect("projected fs context");

    ctx.execute(
        "fs",
        "write",
        &[
            Value::String(allowed_file.display().to_string()),
            Value::String("ok".to_string()),
        ],
    )
    .await
    .expect("allowed write succeeds");
    let failed = ctx
        .execute(
            "fs",
            "write",
            &[
                Value::String(outside_file.display().to_string()),
                Value::String("blocked".to_string()),
            ],
        )
        .await;
    assert!(failed.is_err(), "outside allowed path should fail closed");

    let evidence = runtime.host_boundary_evidence().await;
    assert!(
        evidence.iter().any(|record| record.provider_name == "fs"
            && record.operation_name == "write"
            && record.outcome == HostBoundaryOutcome::Succeeded),
        "allowed write should record success evidence: {evidence:?}"
    );
    assert!(
        evidence.iter().any(|record| record.provider_name == "fs"
            && record.operation_name == "write"
            && record.outcome == HostBoundaryOutcome::Denied),
        "outside allowed path should record denied sandbox evidence: {evidence:?}"
    );
    assert!(
        evidence.iter().all(|record| !record
            .redacted_subject
            .contains(outside_file.to_string_lossy().as_ref())),
        "evidence must not include raw filesystem paths: {evidence:?}"
    );
}

#[tokio::test]
async fn sandboxed_http_profile_denies_blocked_host_before_provider_execution() {
    let runtime = RuntimeState::new();
    let profile = StandardProviderProfile::sandboxed_http("phase198-http", ["api.example.com"]);
    let installed = profile
        .install(&runtime)
        .await
        .expect("http profile installs");
    let ctx = runtime
        .create_capability_context_for_bindings(&installed.binding_ids)
        .await
        .expect("projected http context");

    let denied = ctx
        .execute(
            "http",
            "get",
            &[Value::String(
                "https://blocked.example.com/data".to_string(),
            )],
        )
        .await;
    assert!(denied.is_err(), "blocked HTTP host must fail closed");

    let evidence = runtime.host_boundary_evidence().await;
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].provider_name, "http");
    assert_eq!(evidence[0].operation_name, "get");
    assert_eq!(evidence[0].outcome, HostBoundaryOutcome::Denied);
    assert!(
        !evidence[0].redacted_subject.contains("blocked.example.com"),
        "HTTP evidence must redact host argument"
    );
}

#[test]
fn standard_provider_metadata_validates_for_phase_198_families() {
    let providers: Vec<Arc<dyn CapabilityProvider>> = vec![
        Arc::new(FsProvider::new()),
        Arc::new(HttpProvider::new()),
        Arc::new(TimeProvider::mock(0)),
        Arc::new(ash_engine::providers::LoggingProvider::new()),
    ];

    for provider in providers {
        let metadata = provider.provider_metadata();
        ash_core::capability::validate_provider_authoring_metadata(&metadata)
            .unwrap_or_else(|error| panic!("{} metadata invalid: {error}", provider.name()));
        for operation in metadata.operations {
            assert!(
                operation.sandbox_policy.is_some(),
                "{}.{} missing sandbox policy",
                metadata.provider_name,
                operation.operation_name
            );
            assert!(
                operation.provenance_policy.is_some(),
                "{}.{} missing provenance policy",
                metadata.provider_name,
                operation.operation_name
            );
            assert!(!operation.grants_authority);
        }
    }
}

#[test]
fn stdlib_provider_and_evidence_modules_are_current_surface() {
    let std_src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../std/src");
    let engine = ash_engine::Engine::new().build().expect("engine builds");

    for relative in [
        "io/fs.ash",
        "io/dir.ash",
        "io/path.ash",
        "http.ash",
        "time.ash",
        "logging.ash",
        "evidence.ash",
    ] {
        let path = std_src.join(relative);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {relative}: {error}"));
        assert!(
            !source.contains("remain deferred"),
            "{relative} should not describe Phase 198 provider wrappers as deferred"
        );
        engine
            .check_module_file(&path)
            .unwrap_or_else(|error| panic!("check_module_file failed for {relative}: {error}"));
    }

    let lib = std::fs::read_to_string(std_src.join("lib.ash")).expect("read std lib");
    assert!(
        lib.contains("pub use logging::{debug, info, warn, error};"),
        "std lib should re-export logging provider helpers"
    );
    assert!(
        lib.contains("pub use evidence::{"),
        "std lib should re-export evidence helpers"
    );
}
