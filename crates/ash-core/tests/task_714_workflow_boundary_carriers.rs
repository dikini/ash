//! RED tests for TASK-714 workflow boundary carrier substrate.

use ash_core::runtime::{
    FailureEntity, OperationalFailure, ProcessFailure, ProcessId, RunId, TowerLevel,
    WorkflowAdmissionContext, WorkflowBoundaryOutcome, WorkflowContractCheckEvidence,
    WorkflowEvidenceStatus, WorkflowFailure, WorkflowFailureKind, WorkflowReport,
    WorkflowReportStatus,
};
use ash_core::{Value, WorkflowId};

fn lower_process_failure(process_id: ProcessId, message: &str) -> ProcessFailure {
    ProcessFailure::new(
        process_id,
        OperationalFailure::new(
            TowerLevel::Proc,
            FailureEntity::Process(process_id),
            Value::String(message.to_string()),
            "String",
        ),
    )
}

#[test]
fn workflow_boundary_failure_outcome_preserves_process_causes_and_admission_metadata() {
    let workflow_id = WorkflowId::new();
    let run_id = RunId::new();
    let child_process = ProcessId::new();
    let observed_failure = lower_process_failure(child_process, "child failed");
    let admission = WorkflowAdmissionContext {
        active_role: Some("approver".to_string()),
        admitted_capabilities: vec!["payments.charge".to_string()],
        admitted_capability_bindings: Vec::new(),
        requires_evidence: vec!["request signature verified".to_string()],
    };
    let failure = WorkflowFailure::new(
        workflow_id,
        run_id,
        WorkflowFailureKind::BodyFailureEscaped,
        Some(observed_failure.failure.clone()),
    );

    let report = WorkflowReport::failed(workflow_id, run_id, failure.clone())
        .with_admission_context(admission.clone());
    let outcome = WorkflowBoundaryOutcome::failed(failure.clone(), report.clone());

    match outcome {
        WorkflowBoundaryOutcome::WorkflowFailed {
            failure: boundary_failure,
            report: boundary_report,
        } => {
            assert_eq!(boundary_failure.workflow_id, workflow_id);
            assert_eq!(boundary_failure.run_id, run_id);
            assert_eq!(
                boundary_failure.kind,
                WorkflowFailureKind::BodyFailureEscaped
            );
            assert_eq!(
                boundary_failure.cause.as_deref().map(|cause| cause.entity),
                Some(FailureEntity::Process(child_process))
            );
            assert_eq!(boundary_report.status, WorkflowReportStatus::Failed);
            assert_eq!(
                boundary_report.admission.active_role.as_deref(),
                Some("approver")
            );
            assert_eq!(
                boundary_report.admission.admitted_capabilities,
                vec!["payments.charge".to_string()]
            );
            assert_eq!(
                boundary_report.lower_process_failures,
                vec![observed_failure]
            );
        }
        other => panic!("expected workflow failure boundary outcome, got {other:?}"),
    }
}

#[test]
fn workflow_boundary_success_outcome_can_be_reported_without_external_sink() {
    let workflow_id = WorkflowId::new();
    let run_id = RunId::new();
    let admission = WorkflowAdmissionContext {
        active_role: None,
        admitted_capabilities: vec![],
        admitted_capability_bindings: Vec::new(),
        requires_evidence: vec!["host admission".to_string()],
    };

    let report = WorkflowReport::succeeded(workflow_id, run_id)
        .with_admission_context(admission)
        .with_result(Value::String("ok".to_string()));
    let outcome =
        WorkflowBoundaryOutcome::succeeded(Value::String("ok".to_string()), report.clone());

    match outcome {
        WorkflowBoundaryOutcome::WorkflowSucceeded { value, report } => {
            assert_eq!(value, Value::String("ok".to_string()));
            assert_eq!(report.workflow_id, workflow_id);
            assert_eq!(report.run_id, run_id);
            assert_eq!(report.status, WorkflowReportStatus::Succeeded);
            assert!(report.failure.is_none());
            assert!(report.external_report_sink.is_none());
            assert_eq!(
                report.requires_evidence,
                vec![WorkflowContractCheckEvidence {
                    clause: "host admission".to_string(),
                    status: WorkflowEvidenceStatus::Passed,
                    notes: vec!["host admission".to_string()],
                }]
            );
        }
        other => panic!("expected workflow success boundary outcome, got {other:?}"),
    }
}
