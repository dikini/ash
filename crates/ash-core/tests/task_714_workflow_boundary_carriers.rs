//! RED tests for TASK-714 application boundary carrier substrate.

use ash_core::runtime::{
    ApplicationAdmissionContext, ApplicationBoundaryOutcome, ApplicationContractCheckEvidence,
    ApplicationEvidenceStatus, ApplicationFailure, ApplicationFailureKind, ApplicationReport,
    ApplicationReportStatus, FailureBoundary, FailureEntity, OperationalFailure, ProcessFailure,
    ProcessId, RunId,
};
use ash_core::{ApplicationId, Value};

fn lower_process_failure(process_id: ProcessId, message: &str) -> ProcessFailure {
    ProcessFailure::new(
        process_id,
        OperationalFailure::new(
            FailureBoundary::Process,
            FailureEntity::Process(process_id),
            Value::String(message.to_string()),
            "String",
        ),
    )
}

#[test]
fn application_boundary_failure_outcome_preserves_process_causes_and_admission_metadata() {
    let application_id = ApplicationId::new();
    let run_id = RunId::new();
    let child_process = ProcessId::new();
    let observed_failure = lower_process_failure(child_process, "child failed");
    let admission = ApplicationAdmissionContext {
        admitted_capabilities: vec!["payments.charge".to_string()],
        admitted_capability_bindings: Vec::new(),
        requires_evidence: vec!["request signature verified".to_string()],
    };
    let failure = ApplicationFailure::new(
        application_id,
        run_id,
        ApplicationFailureKind::BodyFailureEscaped,
        Some(observed_failure.failure.clone()),
    );

    let report = ApplicationReport::failed(application_id, run_id, failure.clone())
        .with_admission_context(admission.clone());
    let outcome = ApplicationBoundaryOutcome::failed(failure.clone(), report.clone());

    match outcome {
        ApplicationBoundaryOutcome::ApplicationFailed {
            failure: boundary_failure,
            report: boundary_report,
        } => {
            assert_eq!(boundary_failure.application_id, application_id);
            assert_eq!(boundary_failure.run_id, run_id);
            assert_eq!(
                boundary_failure.kind,
                ApplicationFailureKind::BodyFailureEscaped
            );
            assert_eq!(
                boundary_failure.cause.as_deref().map(|cause| cause.entity),
                Some(FailureEntity::Process(child_process))
            );
            assert_eq!(boundary_report.status, ApplicationReportStatus::Failed);
            assert_eq!(
                boundary_report.admission.admitted_capabilities,
                vec!["payments.charge".to_string()]
            );
            assert_eq!(
                boundary_report.lower_process_failures,
                vec![observed_failure]
            );
        }
        other => panic!("expected application failure boundary outcome, got {other:?}"),
    }
}

#[test]
fn application_boundary_success_outcome_can_be_reported_without_external_sink() {
    let application_id = ApplicationId::new();
    let run_id = RunId::new();
    let admission = ApplicationAdmissionContext {
        admitted_capabilities: vec![],
        admitted_capability_bindings: Vec::new(),
        requires_evidence: vec!["host admission".to_string()],
    };

    let report = ApplicationReport::succeeded(application_id, run_id)
        .with_admission_context(admission)
        .with_result(Value::String("ok".to_string()));
    let outcome =
        ApplicationBoundaryOutcome::succeeded(Value::String("ok".to_string()), report.clone());

    match outcome {
        ApplicationBoundaryOutcome::ApplicationSucceeded { value, report } => {
            assert_eq!(value, Value::String("ok".to_string()));
            assert_eq!(report.application_id, application_id);
            assert_eq!(report.run_id, run_id);
            assert_eq!(report.status, ApplicationReportStatus::Succeeded);
            assert!(report.failure.is_none());
            assert!(report.external_report_sink.is_none());
            assert_eq!(
                report.requires_evidence,
                vec![ApplicationContractCheckEvidence {
                    clause: "host admission".to_string(),
                    status: ApplicationEvidenceStatus::Passed,
                    notes: vec!["host admission".to_string()],
                }]
            );
        }
        other => panic!("expected application success boundary outcome, got {other:?}"),
    }
}
