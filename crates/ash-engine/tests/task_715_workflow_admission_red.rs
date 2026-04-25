//! RED tests for TASK-715 workflow admission above interpreter execution.

use ash_core::runtime::{RunId, WorkflowFailureKind, WorkflowReportStatus};
use ash_core::{Expr, Value, Workflow, WorkflowId};
use ash_engine::{
    Engine, WorkflowAdmissionOutcome, WorkflowAdmissionRequest, WorkflowContractRequirement,
};
use proptest::prelude::*;

const fn honest_body() -> Workflow {
    Workflow::Ret {
        expr: Expr::Literal(Value::Int(7)),
    }
}

#[tokio::test]
async fn admission_creates_or_accepts_workflow_and_run_ids_before_body_execution() {
    let engine = Engine::new().build().expect("engine builds");
    let workflow_id = WorkflowId::new();
    let run_id = RunId::new();
    let request = WorkflowAdmissionRequest {
        workflow_name: "task_715_success".to_string(),
        workflow: honest_body(),
        workflow_id: Some(workflow_id),
        run_id: Some(run_id),
        active_role: Some("reviewer".to_string()),
        required_capabilities: vec!["payments.charge".to_string()],
        requires: vec![],
        ensures: vec!["result.audit_recorded".to_string()],
    };

    let outcome = engine.admit_workflow(request).await;

    match outcome {
        WorkflowAdmissionOutcome::Admitted { boundary } => {
            assert_eq!(boundary.workflow_id(), workflow_id);
            assert_eq!(boundary.run_id(), run_id);
            assert_eq!(boundary.report().status, WorkflowReportStatus::Succeeded);
            assert_eq!(
                boundary.report().admission.active_role.as_deref(),
                Some("reviewer")
            );
            assert_eq!(
                boundary.report().admission.admitted_capabilities,
                vec!["payments.charge".to_string()]
            );
            assert_eq!(boundary.report().ensures_evidence.len(), 1);
        }
        other @ WorkflowAdmissionOutcome::Rejected { .. } => {
            panic!("expected admitted workflow boundary carrier, got {other:?}")
        }
    }
}

#[tokio::test]
async fn role_or_capability_admission_failure_maps_to_structured_workflow_failure() {
    let engine = Engine::new().build().expect("engine builds");
    let request = WorkflowAdmissionRequest {
        workflow_name: "task_715_missing_authority".to_string(),
        workflow: honest_body(),
        workflow_id: None,
        run_id: None,
        active_role: Some("approver".to_string()),
        required_capabilities: vec!["payments.charge".to_string()],
        requires: vec![
            WorkflowContractRequirement::Role("auditor".to_string()),
            WorkflowContractRequirement::Capability("payments.refund".to_string()),
        ],
        ensures: vec![],
    };

    let outcome = engine.admit_workflow(request).await;

    match outcome {
        WorkflowAdmissionOutcome::Rejected { failure, report } => {
            assert!(matches!(
                failure.kind,
                WorkflowFailureKind::RoleAdmissionFailure
                    | WorkflowFailureKind::CapabilityAdmissionFailure
                    | WorkflowFailureKind::AdmissionFailure
            ));
            assert_eq!(report.status, WorkflowReportStatus::Failed);
            assert!(report.result.is_none());
            assert!(report.requires_evidence.is_empty());
        }
        other @ WorkflowAdmissionOutcome::Admitted { .. } => {
            panic!("expected structured admission rejection, got {other:?}")
        }
    }
}

#[tokio::test]
async fn requires_failure_prevents_body_execution_and_records_evidence_in_report() {
    let engine = Engine::new().build().expect("engine builds");
    let request = WorkflowAdmissionRequest {
        workflow_name: "task_715_requires_failure".to_string(),
        workflow: honest_body(),
        workflow_id: None,
        run_id: None,
        active_role: None,
        required_capabilities: vec![],
        requires: vec![WorkflowContractRequirement::Evidence {
            clause: "request.signature".to_string(),
            passed: false,
            notes: vec!["signature missing".to_string()],
        }],
        ensures: vec!["result.audit_recorded".to_string()],
    };

    let outcome = engine.admit_workflow(request).await;

    match outcome {
        WorkflowAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(failure.kind, WorkflowFailureKind::RequiresViolation);
            assert_eq!(report.status, WorkflowReportStatus::Failed);
            assert!(
                report.result.is_none(),
                "body must not execute on requires failure"
            );
            assert_eq!(report.requires_evidence.len(), 1);
            assert_eq!(
                report.ensures_evidence.len(),
                1,
                "ensures schema should already be present"
            );
        }
        other @ WorkflowAdmissionOutcome::Admitted { .. } => {
            panic!("expected requires rejection before body execution, got {other:?}")
        }
    }
}

proptest! {
    #[test]
    fn prop_explicit_ids_and_ensures_schema_are_preserved_across_admission(
        requires_note in any::<String>(),
        ensures_note in any::<String>(),
    ) {
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async move {
            let engine = Engine::new().build().expect("engine builds");
            let workflow_id = WorkflowId::new();
            let run_id = RunId::new();
            let request = WorkflowAdmissionRequest {
                workflow_name: "task_715_prop".to_string(),
                workflow: honest_body(),
                workflow_id: Some(workflow_id),
                run_id: Some(run_id),
                active_role: None,
                required_capabilities: vec![],
                requires: vec![WorkflowContractRequirement::Evidence {
                    clause: "host.check".to_string(),
                    passed: true,
                    notes: vec![requires_note.clone()],
                }],
                ensures: vec![format!("ensures::{ensures_note}")],
            };

            let outcome = engine.admit_workflow(request).await;
            let boundary = match outcome {
                WorkflowAdmissionOutcome::Admitted { boundary } => boundary,
                other @ WorkflowAdmissionOutcome::Rejected { .. } => {
            panic!("expected admitted workflow boundary carrier, got {other:?}")
        } ,
            };

            prop_assert_eq!(boundary.workflow_id(), workflow_id);
            prop_assert_eq!(boundary.run_id(), run_id);
            prop_assert_eq!(boundary.report().requires_evidence.len(), 1);
            prop_assert_eq!(boundary.report().ensures_evidence.len(), 1);
            Ok(())
        })?;
    }
}
