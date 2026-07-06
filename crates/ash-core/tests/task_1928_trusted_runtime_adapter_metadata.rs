//! TASK-1928 trusted runtime adapter metadata tests.

use ash_core::{
    TrustedRuntimeAdapter, TrustedRuntimeAdapterDiagnostic, TrustedRuntimeAdapterTarget,
};

#[test]
fn trusted_runtime_adapter_carries_identity_version_and_report_metadata() {
    let adapter = TrustedRuntimeAdapter::new_provider_operation(
        "host.process.run.adapter",
        "1.0.0",
        "ash-runtime",
        "admitted-provider:process",
        "host.process.run",
        "host.process.run.redacted",
        "report.host.process.run",
        "process",
        "run",
        "process.run",
        false,
    )
    .expect("valid adapter metadata should construct");

    assert_eq!(adapter.name, "host.process.run.adapter");
    assert_eq!(adapter.version, "1.0.0");
    assert_eq!(adapter.sandbox_policy, "host.process.run");
    assert_eq!(adapter.provenance_policy, "host.process.run.redacted");
    assert_eq!(adapter.report_identity, "report.host.process.run");
    assert_eq!(
        adapter.target,
        TrustedRuntimeAdapterTarget::ProviderOperation {
            provider_name: "process".to_string(),
            operation_name: "run".to_string(),
            required_row: "process.run".to_string(),
        }
    );
}

#[test]
fn trusted_runtime_adapter_requires_metadata_target() {
    let err = TrustedRuntimeAdapter::new_provider_operation(
        "host.process.run.adapter",
        "1.0.0",
        "ash-runtime",
        "admitted-provider:process",
        "host.process.run",
        "host.process.run.redacted",
        "report.host.process.run",
        "process",
        "",
        "process.run",
        false,
    )
    .expect_err("provider operation adapters must reference operation metadata");

    assert_eq!(
        err,
        TrustedRuntimeAdapterDiagnostic::MissingProviderMetadataReference {
            adapter_name: "host.process.run.adapter".to_string(),
        }
    );
}

#[test]
fn trusted_runtime_adapter_rejects_authority_widening() {
    let err = TrustedRuntimeAdapter::new_builtin_host_hook(
        "builtin.process.run.adapter",
        "1.0.0",
        "ash-runtime",
        "builtin-hook:std.process.run",
        "host.process.run",
        "host.process.run.redacted",
        "report.builtin.process.run",
        "std.process.run",
        "process",
        "run",
        true,
    )
    .expect_err("trusted adapters must not grant authority");

    assert_eq!(
        err,
        TrustedRuntimeAdapterDiagnostic::AuthorityWideningAdapter {
            adapter_name: "builtin.process.run.adapter".to_string(),
        }
    );
}
