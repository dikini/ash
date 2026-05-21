//! `RuntimeKernel` artifact-builder integration tests.

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
