//! TASK-978 runtime-support payload identity tests.

use ash_core::core_ash::{CoreRow, CoreType};
use ash_core::runtime_kernel::CheckedFunctionArtifact;
use ash_core::semantic_summary::{SourceAnchor, SourceOrigin};
use ash_core::{Expr, Span, Value};
use ash_engine::runtime_artifact::{RuntimeArtifactBuildRequest, build_runtime_kernel_artifact};

#[test]
fn task_978_runtime_artifact_records_selected_toolchain_runtime_support_identity() {
    let source = "fn main() { return 7 }";
    let baseline = RuntimeArtifactBuildRequest::new_application_entrypoint(
        "workspace:/task-978",
        "applications/runtime-support.ash",
        "main",
        "callable:applications/runtime-support.ash::main",
        "runtime-target:application-entry:main",
        "default",
        "default",
        CheckedFunctionArtifact {
            function_identity: "callable:applications/runtime-support.ash::main".to_string(),
            effect_row: CoreRow::default(),
            body: Expr::Literal(Value::Int(7)),
            source_anchor: SourceAnchor::new(
                SourceOrigin::File("applications/runtime-support.ash".to_string()),
                Some(Span {
                    start: 0,
                    end: source.len(),
                }),
                "checked-function:main",
            ),
            result_type: CoreType::Base("Int".to_string()),
        },
        source,
        "engine-check:ok;warnings=0",
    )
    .expect("checked function request");
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
