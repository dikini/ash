//! RED tests for TASK-716 application-boundary completion/report construction.

use ash_core::runtime::{
    ApplicationContractCheckEvidence, ApplicationEvidenceStatus, ApplicationFailure,
    ApplicationFailureKind, ApplicationReport, ApplicationReportStatus, FailureBoundary,
    FailureEntity, OperationalFailure, ProcessId, RunId,
};
use ash_core::{ApplicationId, Value};

#[test]
fn ensures_violation_reports_failed_ensures_evidence_not_pending_placeholders() {
    let application_id = ApplicationId::new();
    let run_id = RunId::new();
    let failure = ApplicationFailure::new(
        application_id,
        run_id,
        ApplicationFailureKind::EnsuresViolation,
        None,
    );

    let report =
        ApplicationReport::failed(application_id, run_id, failure).with_ensures_evidence(vec![
            ApplicationContractCheckEvidence::pending(
                "result.audit_recorded",
                vec!["task-716 red".to_string()],
            ),
        ]);

    assert_eq!(report.status, ApplicationReportStatus::Failed);
    assert!(
        report
            .ensures_evidence
            .iter()
            .any(|entry| entry.status == ApplicationEvidenceStatus::Failed),
        "an ensures violation report must carry failed ensures evidence"
    );
    assert!(
        report
            .ensures_evidence
            .iter()
            .all(|entry| entry.status != ApplicationEvidenceStatus::Pending),
        "completion-boundary reports must not leave ensures checks pending"
    );
}

#[test]
fn local_obligations_undischarged_report_requires_obligation_evidence_even_without_sink() {
    let application_id = ApplicationId::new();
    let run_id = RunId::new();
    let failure = ApplicationFailure::new(
        application_id,
        run_id,
        ApplicationFailureKind::LocalObligationsUndischarged,
        None,
    );

    let report = ApplicationReport::failed(application_id, run_id, failure);

    assert_eq!(report.status, ApplicationReportStatus::Failed);
    assert!(
        report.external_report_sink.is_none(),
        "TASK-716 requires a minimal local report even without an external sink"
    );
    assert!(
        !report.obligation_evidence.is_empty(),
        "undischarged-obligation failures must record application-boundary obligation evidence"
    );
}

#[test]
fn escaped_process_failure_preserves_lower_cause_and_report_linkage() {
    let application_id = ApplicationId::new();
    let run_id = RunId::new();
    let process_id = ProcessId::new();
    let lower = OperationalFailure::new(
        FailureBoundary::Process,
        FailureEntity::Process(process_id),
        Value::String("provider denied".to_string()),
        "ExecError",
    );
    let failure = ApplicationFailure::new(
        application_id,
        run_id,
        ApplicationFailureKind::BodyFailureEscaped,
        Some(lower.clone()),
    );

    let report = ApplicationReport::failed(application_id, run_id, failure.clone());

    assert_eq!(report.status, ApplicationReportStatus::Failed);
    assert_eq!(report.failure, Some(failure.clone()));
    assert_eq!(report.lower_causes, vec![lower.clone()]);
    assert_eq!(report.lower_process_failures.len(), 1);
    assert_eq!(report.lower_process_failures[0].process_id, process_id);
    assert_eq!(report.lower_process_failures[0].failure, lower);
}
