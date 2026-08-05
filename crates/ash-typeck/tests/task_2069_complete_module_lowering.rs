//! TASK-2069 activation contract.
//!
//! This target is intentionally limited to TASK-2069's activation boundary.

use std::fs;
use std::path::Path;

#[test]
fn task_2069_activation_contract_declares_non_authorizing_handoff() {
    let task_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../docs/plan/tasks/TASK-2069-complete-module-lowering-and-engine-transport-fencing.md",
    );
    let task = fs::read_to_string(&task_path).expect("TASK-2069 task file exists");

    assert!(task.contains("**Status:** In progress"));
    assert!(task.contains(
        "**Semantic coverage map:** [TASK-2069 record](../SEMANTIC-RULE-COVERAGE.md#task-2069-complete-module-lowering-and-engine-transport-fencing)"
    ));

    for evidence_id in [
        "TEST-MOD-REAL-005-FULL-DEFINITION-BODY-LOWERING",
        "TEST-MOD-REAL-005-ENGINE-CHECKED-TRANSPORT",
        "TEST-MOD-REAL-005-BODY-LOWERING-REJECTION",
        "TEST-MOD-REAL-005-PROVENANCE-REWRITE",
        "TEST-MOD-REAL-005-SCANNER-AUTHORITY-REJECTION",
        "TEST-MOD-REAL-005-CANONICAL-CACHE-KEY",
        "TEST-MOD-REAL-005-FILE-INLINE-LOWERING-PARITY",
    ] {
        assert!(
            task.contains(evidence_id),
            "TASK-2069 must reserve evidence identifier {evidence_id}"
        );
    }
}
