//! TASK-1672: Tracing docs consistency.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("ash-core crate lives under crates/ash-core")
        .to_path_buf()
}

fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

#[test]
fn task_1672_tracing_requirements_and_spec_terms_remain_documented() {
    let task_spec =
        read(repo_root().join("docs/plan/tasks/TASK-1672-core-mode-tracing-observability.md"));

    for required in [
        "TraceEvent::ThunkConstructed",
        "TraceEvent::ThunkForceStarted",
        "TraceEvent::ThunkBodyEvaluationStarted",
        "TraceEvent::ThunkBodyEvaluationCompleted",
        "TraceEvent::ThunkForceCompleted",
        "TraceEvent::MemoCacheFilled",
        "TraceEvent::MemoCacheHit",
        "TraceEvent::MemoReplayFailure",
        "TraceEvent::MemoReentrantRejected",
        "\"success\"",
        "\"trap\"",
        "\"unhandled-effect\"",
        "\"runtime-error\"",
    ] {
        assert!(
            task_spec.contains(required),
            "tracing task should continue to document {required}"
        );
    }

    let spec = read(repo_root().join("docs/spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md"));
    assert!(
        spec.contains("memo re-entrant-force rejection") || spec.contains("re-entrant force"),
        "SPEC-101 should document re-entrant memo force behavior"
    );
    assert!(
        spec.contains("memo force starts")
            || spec.contains("memo force start")
            || spec.contains("memo force end"),
        "SPEC-101 should document memo force lifecycle for observability"
    );
}
