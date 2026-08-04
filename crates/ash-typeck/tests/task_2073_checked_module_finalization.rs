//! TASK-2073 activation contract and RED inventory.
//!
//! The executable contract records the active Type-layer owner. The ignored
//! tests describe the first implementation seam and will be enabled once the
//! finalization carrier API exists.

use std::fs;
use std::path::Path;

#[test]
fn task_2073_activation_contract_is_recorded_before_finalization_implementation() {
    let task_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/plan/tasks/TASK-2073-checked-module-finalization-and-export-closure.md");
    let task = fs::read_to_string(&task_path).expect("TASK-2073 task file exists");

    assert!(task.contains("**Status:** In progress"));
    assert!(task.contains("**Owned rule:** MOD-REAL-003"));
    assert!(task.contains("CanonicalCollectedModuleSnapshot"));
    assert!(task.contains("CanonicalParsedImportResult"));
    assert!(task.contains("export closure"));
    assert!(task.contains("**Implementation:** partial"));
    assert!(task.contains("**Evidence:** none"));
    assert!(task.contains("**Parity:** below_spec"));
    assert!(task.contains("## TDD Steps"));
    assert!(task.contains("atomic"));
}

#[test]
#[ignore = "RED: finalization carrier API is not implemented yet"]
fn red_checked_private_and_public_facts_preserve_provenance() {
    todo!("exercise checked private/public identity and source anchors");
}

#[test]
#[ignore = "RED: finalization carrier API is not implemented yet"]
fn red_final_pub_use_requires_export_closed_targets() {
    todo!("exercise final public-use export closure");
}

#[test]
#[ignore = "RED: finalization carrier API is not implemented yet"]
fn red_stale_forged_and_incomplete_inputs_reject_atomically() {
    todo!("exercise dependency and snapshot revalidation");
}

#[test]
#[ignore = "RED: finalization carrier API is not implemented yet"]
fn red_file_and_inline_final_interfaces_have_equal_projection() {
    todo!("exercise normalized Type-layer final-interface parity");
}
