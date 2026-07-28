//! TASK-1928 trusted runtime adapter registry tests.

use ash_core::capability::{ProviderAuthoringMetadata, ProviderOperationMetadata};
use ash_core::core_ash_contract::TraceFactKind;
use ash_core::{Effect, RuntimeTraceEvent, TrustedRuntimeAdapter, TrustedRuntimeAdapterDiagnostic};
use ash_runtime::RuntimeState;

fn process_provider_metadata() -> ProviderAuthoringMetadata {
    ProviderAuthoringMetadata::new("process").with_operation(
        ProviderOperationMetadata::new("run", Effect::Operational)
            .with_required_row("process.run")
            .with_resource("process")
            .with_sandbox_policy("host.process.run")
            .with_provenance_policy("host.process.run.redacted"),
    )
}

fn process_run_adapter(version: &str) -> TrustedRuntimeAdapter {
    TrustedRuntimeAdapter::new_provider_operation(
        "host.process.run.adapter",
        version,
        "ash-runtime-secret-trust-root",
        "admitted-provider:process",
        "host.process.run",
        "host.process.run.redacted",
        "report.host.process.run",
        "process",
        "run",
        "process.run",
        false,
    )
    .expect("test adapter metadata should be valid")
}

#[tokio::test]
async fn trusted_runtime_adapter_registration_retains_metadata_and_reports_identity() {
    let runtime_state = RuntimeState::new();
    let adapter = runtime_state
        .register_trusted_runtime_adapter(process_run_adapter("1.0.0"))
        .await
        .expect("valid adapter should register");

    let retained = runtime_state
        .trusted_runtime_adapter("host.process.run.adapter")
        .await
        .expect("adapter should be retained by name");

    assert_eq!(retained.id, adapter.id);
    assert_eq!(retained.version, "1.0.0");
    assert_eq!(retained.report_identity, "report.host.process.run");

    let facts = runtime_state.runtime_trace_facts().await;
    assert!(facts.iter().any(|fact| {
        fact.kind == TraceFactKind::Operation
            && fact.event == RuntimeTraceEvent::Register
            && fact
                .subject
                .contains("adapter:host.process.run.adapter:1.0.0")
            && fact.subject.contains("report.host.process.run")
            && !fact.subject.contains("secret")
    }));
}

#[tokio::test]
async fn trusted_runtime_adapter_lookup_fails_closed_for_unknown_or_stale_version() {
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_trusted_runtime_adapter(process_run_adapter("1.0.0"))
        .await
        .expect("valid adapter should register");

    let missing = runtime_state
        .require_trusted_runtime_adapter("missing.adapter", "1.0.0")
        .await
        .expect_err("unknown adapters must fail closed");
    assert_eq!(
        missing,
        TrustedRuntimeAdapterDiagnostic::UnknownAdapter {
            adapter_name: "missing.adapter".to_string(),
        }
    );

    let stale = runtime_state
        .require_trusted_runtime_adapter("host.process.run.adapter", "2.0.0")
        .await
        .expect_err("stale adapter versions must fail closed");
    assert_eq!(
        stale,
        TrustedRuntimeAdapterDiagnostic::StaleAdapter {
            adapter_name: "host.process.run.adapter".to_string(),
            requested_version: "2.0.0".to_string(),
            registered_version: "1.0.0".to_string(),
        }
    );
}

#[tokio::test]
async fn trusted_runtime_adapter_must_match_provider_metadata_before_execution() {
    let runtime_state = RuntimeState::new();
    runtime_state
        .register_trusted_runtime_adapter(process_run_adapter("1.0.0"))
        .await
        .expect("valid adapter should register");

    let adapter = runtime_state
        .validate_trusted_runtime_adapter_for_provider_operation(
            "host.process.run.adapter",
            "1.0.0",
            &process_provider_metadata(),
            "run",
        )
        .await
        .expect("adapter should match provider metadata");
    assert_eq!(adapter.report_identity, "report.host.process.run");

    let wrong_metadata = ProviderAuthoringMetadata::new("process").with_operation(
        ProviderOperationMetadata::new("which", Effect::Operational)
            .with_required_row("process.which")
            .with_resource("process")
            .with_sandbox_policy("host.process.which")
            .with_provenance_policy("host.process.which.redacted"),
    );
    let err = runtime_state
        .validate_trusted_runtime_adapter_for_provider_operation(
            "host.process.run.adapter",
            "1.0.0",
            &wrong_metadata,
            "which",
        )
        .await
        .expect_err("mismatched provider metadata must fail closed");

    assert_eq!(
        err,
        TrustedRuntimeAdapterDiagnostic::IncompatibleAdapter {
            adapter_name: "host.process.run.adapter".to_string(),
            reason: "adapter target process.run/process.run does not match provider metadata process.which"
                .to_string(),
        }
    );
}
