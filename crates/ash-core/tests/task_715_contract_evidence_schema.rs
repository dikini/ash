//! RED tests for TASK-715 workflow contract evidence schema/plumbing.

use ash_core::WorkflowId;
use ash_core::runtime::{
    RunId, WorkflowContractCheckEvidence, WorkflowEvidenceStatus, WorkflowReport,
    WorkflowReportStatus,
};
use proptest::prelude::*;

#[test]
fn workflow_report_carries_structured_requires_and_pending_ensures_evidence() {
    let workflow_id = WorkflowId::new();
    let run_id = RunId::new();
    let requires = vec![WorkflowContractCheckEvidence::passed(
        "request.signature",
        vec!["signature:verified".to_string()],
    )];
    let ensures = vec![WorkflowContractCheckEvidence::pending(
        "result.audit_recorded",
        vec!["deferred-to-task-716".to_string()],
    )];

    let report = WorkflowReport::succeeded(workflow_id, run_id)
        .with_requires_evidence(requires.clone())
        .with_ensures_evidence(ensures.clone());

    assert_eq!(report.status, WorkflowReportStatus::Succeeded);
    assert_eq!(report.requires_evidence, requires);
    assert_eq!(report.ensures_evidence, ensures);
    assert!(
        report
            .ensures_evidence
            .iter()
            .all(|entry| entry.status == WorkflowEvidenceStatus::Pending)
    );
}

proptest! {
    #[test]
    fn prop_workflow_report_preserves_identity_and_contract_evidence(
        requires_note in any::<String>(),
        ensures_note in any::<String>(),
    ) {
        let workflow_id = WorkflowId::new();
        let run_id = RunId::new();
        let requires = vec![WorkflowContractCheckEvidence::passed(
            "requires.host-admission",
            vec![requires_note.clone()],
        )];
        let ensures = vec![WorkflowContractCheckEvidence::pending(
            "ensures.report-commit",
            vec![ensures_note.clone()],
        )];

        let report = WorkflowReport::succeeded(workflow_id, run_id)
            .with_requires_evidence(requires.clone())
            .with_ensures_evidence(ensures.clone());

        prop_assert_eq!(report.workflow_id, workflow_id);
        prop_assert_eq!(report.run_id, run_id);
        prop_assert_eq!(report.requires_evidence, requires);
        prop_assert_eq!(report.ensures_evidence, ensures);
    }
}
