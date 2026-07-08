//! RED tests for TASK-715 workflow contract evidence schema/plumbing.

use ash_core::WorkflowId;
use ash_core::runtime::{
    ApplicationContractCheckEvidence, ApplicationEvidenceStatus, ApplicationReport,
    ApplicationReportStatus, RunId,
};
use proptest::prelude::*;

#[test]
fn application_report_carries_structured_requires_and_pending_ensures_evidence() {
    let application_id = WorkflowId::new();
    let run_id = RunId::new();
    let requires = vec![ApplicationContractCheckEvidence::passed(
        "request.signature",
        vec!["signature:verified".to_string()],
    )];
    let ensures = vec![ApplicationContractCheckEvidence::pending(
        "result.audit_recorded",
        vec!["deferred-to-task-716".to_string()],
    )];

    let report = ApplicationReport::succeeded(application_id, run_id)
        .with_requires_evidence(requires.clone())
        .with_ensures_evidence(ensures.clone());

    assert_eq!(report.status, ApplicationReportStatus::Succeeded);
    assert_eq!(report.requires_evidence, requires);
    assert_eq!(report.ensures_evidence, ensures);
    assert!(
        report
            .ensures_evidence
            .iter()
            .all(|entry| entry.status == ApplicationEvidenceStatus::Pending)
    );
}

proptest! {
    #[test]
    fn prop_application_report_preserves_identity_and_contract_evidence(
        requires_note in any::<String>(),
        ensures_note in any::<String>(),
    ) {
        let application_id = WorkflowId::new();
        let run_id = RunId::new();
        let requires = vec![ApplicationContractCheckEvidence::passed(
            "requires.host-admission",
            vec![requires_note.clone()],
        )];
        let ensures = vec![ApplicationContractCheckEvidence::pending(
            "ensures.report-commit",
            vec![ensures_note.clone()],
        )];

        let report = ApplicationReport::succeeded(application_id, run_id)
            .with_requires_evidence(requires.clone())
            .with_ensures_evidence(ensures.clone());

        prop_assert_eq!(report.application_id, application_id);
        prop_assert_eq!(report.run_id, run_id);
        prop_assert_eq!(report.requires_evidence, requires);
        prop_assert_eq!(report.ensures_evidence, ensures);
    }
}
