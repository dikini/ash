//! TASK-978 runtime-support payload identity tests.

use ash_engine::runtime_artifact::{RuntimeArtifactBuildRequest, build_runtime_kernel_artifact};

#[test]
fn task_978_runtime_artifact_records_selected_toolchain_runtime_support_identity() {
    let source = "fn main() { return 7 }";
    let baseline = RuntimeArtifactBuildRequest::new(
        "workspace:/task-978",
        "workflows/runtime-support.ash",
        "main",
        "default",
        "default",
        source,
        "engine-check:ok;warnings=0",
    );
    let selected_runtime_support = baseline
        .clone()
        .with_runtime_support_identity("ash-runtime-support:0.1.0");

    let without_identity = build_runtime_kernel_artifact(&baseline).expect("baseline artifact");
    let with_identity =
        build_runtime_kernel_artifact(&selected_runtime_support).expect("runtime support artifact");

    assert_ne!(
        without_identity.check_summary_hash,
        with_identity.check_summary_hash
    );
    assert_ne!(without_identity.cache_key, with_identity.cache_key);
    assert_ne!(without_identity.artifact.id, with_identity.artifact.id);
}
