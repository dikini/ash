//! TASK-2005/TASK-2008 RED parity contract for one handler-body trap.
//!
//! The case is one private differential witness for the exact `trap_sleep`
//! source. It must compare a case-locked direct semantic derivation with an
//! opaque checked-handler inspection target; it never calls `Engine::run` or
//! creates a production or legacy fallback route.

use ash_engine::differential::{
    CaseComparisonStatus, DifferentialHarness, ObservableDimension, ParityDisposition,
    RelationStatus, RustExecutionTarget,
};
use serde_json::json;
use std::path::PathBuf;

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/differential/corpus")
}

#[test]
fn abortive_trap_sleep_pair_projects_the_canonical_v1_trap_envelope_without_a_fallback() {
    let harness = DifferentialHarness::load(corpus_root())
        .expect("the exact trap_sleep terminal parity fixture loads");

    let report = harness.run_case(
        "phase202-source-trap-sleep-handler-terminal",
        RustExecutionTarget::DirectRuntime,
    );

    assert_eq!(report.direct_runtime_status(), CaseComparisonStatus::Passed);
    assert_eq!(
        report.actual_result(),
        Some(&json!({
            "schema_version": 1,
            "kind": "trap",
            "reason": "division by zero",
        }))
    );
    let envelope = report
        .actual_result()
        .expect("the paired handler trap exposes one terminal envelope");
    for legacy_field in ["outcome_class", "payload", "_variant"] {
        assert!(
            envelope.get(legacy_field).is_none(),
            "canonical V1 terminal evidence must not leak `{legacy_field}`: {envelope}"
        );
    }
    assert_eq!(report.checked_core_cps_relation(), RelationStatus::Passed);
    assert!(matches!(
        report
            .parity_report()
            .disposition_for(ObservableDimension::StructuredTraps),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-CPS-TRAP-001" && owner == "TASK-2005"
    ));
}
