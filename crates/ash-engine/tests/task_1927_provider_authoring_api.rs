//! TASK-1927 provider authoring API tests.

use ash_core::capability::{CapabilityProvider, validate_provider_authoring_metadata};
use ash_engine::providers::{FsProvider, ProcessProvider, StdioProvider, TimeProvider};

#[test]
fn standard_providers_expose_valid_authoring_metadata() {
    let providers: Vec<Box<dyn CapabilityProvider>> = vec![
        Box::new(StdioProvider::new()),
        Box::new(FsProvider::new()),
        Box::new(TimeProvider::new()),
        Box::new(ProcessProvider::new()),
    ];

    for provider in providers {
        let metadata = provider.provider_metadata();
        validate_provider_authoring_metadata(&metadata)
            .unwrap_or_else(|error| panic!("{} metadata invalid: {error}", provider.name()));
        assert!(
            !metadata.compatibility_shim,
            "{} should use explicit metadata, not a legacy shim",
            provider.name()
        );
        assert!(
            !metadata.operations.is_empty(),
            "{} should declare operation surface",
            provider.name()
        );
    }
}

#[test]
fn process_provider_declares_run_sandbox_and_provenance_policy() {
    let metadata = ProcessProvider::new().provider_metadata();
    let run = metadata.operation("run").expect("run operation metadata");

    assert_eq!(run.required_rows, vec!["process.run"]);
    assert_eq!(run.sandbox_policy.as_deref(), Some("host.process.run"));
    assert_eq!(
        run.provenance_policy.as_deref(),
        Some("host.process.run.redacted")
    );
}

#[test]
fn fs_provider_declares_path_constraints_for_file_operations() {
    let metadata = FsProvider::new().provider_metadata();

    for operation_name in ["read_to_string", "write", "remove_file"] {
        let operation = metadata
            .operation(operation_name)
            .unwrap_or_else(|| panic!("{operation_name} metadata should exist"));
        assert!(
            operation
                .constraints
                .iter()
                .any(|constraint| constraint == "paths"),
            "{operation_name} should declare path constraints"
        );
        assert!(
            operation
                .resources
                .iter()
                .any(|resource| resource == "filesystem"),
            "{operation_name} should declare filesystem resource usage"
        );
    }
}
