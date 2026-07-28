//! TASK-439: file-backed Rust-first differential harness contract tests.
//!
//! The harness must consume Phase-202 rule IDs and expected/allowed result
//! envelopes. A missing checked Core/CPS relation is an owned unsupported
//! result, never a successful comparison.

use super::{CaseComparisonStatus, DifferentialHarness, RelationStatus, RustExecutionTarget};
use std::path::PathBuf;

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/differential/corpus")
}

#[test]
fn file_backed_exact_case_runs_direct_runtime_against_phase202_rule_ids() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case("phase202-return-unit", RustExecutionTarget::DirectRuntime);

    assert_eq!(report.case_id(), "phase202-return-unit");
    assert!(report.canonical_rule_ids().contains("SEM-CPS-RETURN-001"));
    assert_eq!(report.direct_runtime_status(), CaseComparisonStatus::Passed);
}

#[test]
fn file_backed_allowed_set_case_accepts_any_declared_external_outcome() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case(
        "phase202-bounded-external",
        RustExecutionTarget::DirectRuntime,
    );

    assert_eq!(report.expectation_kind(), "allowed_set");
    assert_eq!(report.direct_runtime_status(), CaseComparisonStatus::Passed);
}

#[test]
fn fixture_without_checked_core_cps_relation_is_owned_unsupported_not_a_pass() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case(
        "phase202-bounded-external",
        RustExecutionTarget::DirectRuntime,
    );

    assert_eq!(
        report.checked_core_cps_relation(),
        RelationStatus::Unsupported {
            owner: "TASK-2004".to_string(),
            relation: "direct-runtime-to-checked-core-cps".to_string(),
        }
    );
    assert_ne!(
        report.checked_core_cps_relation(),
        RelationStatus::Passed,
        "an unavailable Core/CPS relation must not be reported as a passing comparison"
    );
}

#[test]
fn active_cps_kernel_return_fixture_runs_through_the_distinct_checked_cps_target() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case(
        "cps-kernel-return-int-7",
        RustExecutionTarget::CheckedCoreCpsPrototype,
    );

    assert_eq!(report.case_id(), "cps-kernel-return-int-7");
    assert!(report.canonical_rule_ids().contains("SEM-CPS-RETURN-001"));
    assert_eq!(report.direct_runtime_status(), CaseComparisonStatus::Passed);
    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "int", "value": 7}},
        }))
    );
}

#[test]
fn unbound_active_cps_kernel_return_is_rejected_before_terminal_comparison() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case(
        "cps-kernel-return-unbound",
        RustExecutionTarget::CheckedCoreCpsPrototype,
    );

    assert!(matches!(
        report.direct_runtime_status(),
        CaseComparisonStatus::Failed { reason }
            if reason.contains("validation error")
                && reason.contains("unbound_terminal_value")
    ));
    assert!(
        report.actual_result().is_none(),
        "a rejected canonical CPS input must not manufacture a terminal result"
    );
}

#[test]
fn active_cps_kernel_custom_trap_fixture_normalizes_the_exact_canonical_envelope() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case(
        "cps-kernel-trap-custom-domain",
        RustExecutionTarget::CheckedCoreCpsPrototype,
    );

    assert_eq!(report.case_id(), "cps-kernel-trap-custom-domain");
    assert!(report.canonical_rule_ids().contains("SEM-CPS-TRAP-001"));
    assert_eq!(report.direct_runtime_status(), CaseComparisonStatus::Passed);
    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "trap",
            "payload": {"kind": "trap", "reason": "kernel-custom-domain"},
        }))
    );
}

#[test]
fn non_v1_active_cps_kernel_input_is_rejected_before_terminal_comparison() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case(
        "cps-kernel-trap-invalid-schema",
        RustExecutionTarget::CheckedCoreCpsPrototype,
    );

    assert!(matches!(
        report.direct_runtime_status(),
        CaseComparisonStatus::Failed { reason }
            if reason.contains("requires `ash-cps-kernel-input/v1`")
    ));
    assert!(report.actual_result().is_none());
}

#[test]
fn active_cps_kernel_jump_fixture_projects_its_affine_continuation_return() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case(
        "cps-kernel-jump-return-int-7",
        RustExecutionTarget::CheckedCoreCpsPrototype,
    );

    assert_eq!(report.case_id(), "cps-kernel-jump-return-int-7");
    assert!(report.canonical_rule_ids().contains("SEM-CPS-JUMP-001"));
    assert_eq!(report.direct_runtime_status(), CaseComparisonStatus::Passed);
    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "int", "value": 7}},
        }))
    );
}

#[test]
fn active_cps_kernel_jump_rejects_a_continuation_absent_from_the_explicit_store() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case(
        "cps-kernel-jump-absent-continuation",
        RustExecutionTarget::CheckedCoreCpsPrototype,
    );

    assert!(matches!(
        report.direct_runtime_status(),
        CaseComparisonStatus::Failed { reason }
            if reason.contains("validation error") && reason.contains("missing_continuation")
    ));
    assert!(report.actual_result().is_none());
}

#[test]
fn active_cps_kernel_v2_letval_fixture_binds_before_exact_terminal_projection() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case(
        "cps-kernel-v2-letval-return-int-7",
        RustExecutionTarget::CheckedCoreCpsPrototype,
    );

    assert_eq!(report.case_id(), "cps-kernel-v2-letval-return-int-7");
    assert!(report.canonical_rule_ids().contains("SEM-CPS-LETVAL-001"));
    assert!(report.canonical_rule_ids().contains("SEM-CPS-RETURN-001"));
    assert_eq!(report.direct_runtime_status(), CaseComparisonStatus::Passed);
    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "int", "value": 7}},
        }))
    );
}

#[test]
fn active_cps_kernel_v2_letval_rejects_a_body_that_returns_another_variable() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case(
        "cps-kernel-v2-letval-return-wrong-variable",
        RustExecutionTarget::CheckedCoreCpsPrototype,
    );

    assert!(matches!(
        report.direct_runtime_status(),
        CaseComparisonStatus::Failed { reason }
            if reason.contains("validation error")
                && reason.contains("LetVal body must return bound variable")
    ));
    assert!(report.actual_result().is_none());
}

#[test]
fn active_cps_kernel_v3_letprim_integer_addition_projects_its_bound_result() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case(
        "cps-kernel-v3-letprim-int-add-return-7",
        RustExecutionTarget::CheckedCoreCpsPrototype,
    );

    assert_eq!(report.case_id(), "cps-kernel-v3-letprim-int-add-return-7");
    assert!(report.canonical_rule_ids().contains("SEM-CPS-PRIM-001"));
    assert!(report.canonical_rule_ids().contains("SEM-CPS-RETURN-001"));
    assert_eq!(report.direct_runtime_status(), CaseComparisonStatus::Passed);
    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "int", "value": 7}},
        }))
    );
}

#[test]
fn active_cps_kernel_v3_letprim_rejects_an_unsupported_primitive_before_projection() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case(
        "cps-kernel-v3-letprim-unsupported-primitive",
        RustExecutionTarget::CheckedCoreCpsPrototype,
    );

    assert!(matches!(
        report.direct_runtime_status(),
        CaseComparisonStatus::Failed { reason }
            if reason.contains("validation error")
                && reason.contains("unsupported v3 primitive")
    ));
    assert!(report.actual_result().is_none());
}

#[test]
fn active_cps_kernel_v4_literal_if_selects_the_true_branch_before_terminal_projection() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case(
        "cps-kernel-v4-if-true-return-int-7",
        RustExecutionTarget::CheckedCoreCpsPrototype,
    );

    assert_eq!(report.case_id(), "cps-kernel-v4-if-true-return-int-7");
    assert!(report.canonical_rule_ids().contains("SEM-CPS-IF-001"));
    assert!(report.canonical_rule_ids().contains("SEM-CPS-RETURN-001"));
    assert_eq!(report.direct_runtime_status(), CaseComparisonStatus::Passed);
    assert_eq!(
        report.actual_result(),
        Some(&serde_json::json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "int", "value": 7}},
        }))
    );
}

#[test]
fn active_cps_kernel_v4_if_rejects_a_nonboolean_condition_before_projection() {
    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");

    let report = harness.run_case(
        "cps-kernel-v4-if-nonboolean-condition",
        RustExecutionTarget::CheckedCoreCpsPrototype,
    );

    assert!(matches!(
        report.direct_runtime_status(),
        CaseComparisonStatus::Failed { reason }
            if reason.contains("validation error")
                && reason.contains("v4 If condition must be a literal Bool")
    ));
    assert!(
        report.actual_result().is_none(),
        "an invalid v4 condition must not manufacture a terminal result"
    );
}
