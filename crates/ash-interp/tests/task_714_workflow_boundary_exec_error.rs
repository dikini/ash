//! TASK-714 tests for workflow boundary carrier ExecError projection.

use ash_core::runtime::{
    FailureEntity, RunId, TowerLevel, WorkflowBoundaryOutcome, WorkflowFailureKind,
};
use ash_core::{Value, WorkflowId};
use ash_interp::{ExecError, workflow_boundary_outcome_from_exec_result};
use proptest::prelude::*;

#[test]
fn workflow_boundary_adapter_preserves_exec_error_as_lower_cause() {
    let workflow_id = WorkflowId::new();
    let run_id = RunId::new();
    let lower = ExecError::ExecutionFailed("provider denied".to_string());

    let outcome =
        workflow_boundary_outcome_from_exec_result(workflow_id, run_id, Err(lower.clone()));

    match outcome {
        WorkflowBoundaryOutcome::WorkflowFailed { failure, report } => {
            assert_eq!(failure.kind, WorkflowFailureKind::BodyFailureEscaped);
            let cause = failure
                .cause
                .as_deref()
                .expect("lower exec error should be preserved as a workflow cause");
            assert_eq!(cause.tower, TowerLevel::Workflow);
            assert_eq!(cause.entity, FailureEntity::Run(run_id));
            assert_eq!(cause.payload, Value::String(lower.to_string()));
            assert_eq!(cause.payload_type, "ExecError");
            assert_eq!(report.lower_causes, vec![cause.clone()]);
        }
        other => panic!("expected workflow failure boundary outcome, got {other:?}"),
    }
}

proptest! {
    #[test]
    fn workflow_boundary_adapter_preserves_workflow_identity_and_failure_report(
        message in any::<String>(),
    ) {
        let workflow_id = WorkflowId::new();
        let run_id = RunId::new();
        let lower = ExecError::ExecutionFailed(message.clone());

        let outcome =
            workflow_boundary_outcome_from_exec_result(workflow_id, run_id, Err(lower.clone()));

        prop_assert_eq!(
            matches!(outcome, WorkflowBoundaryOutcome::WorkflowFailed { .. }),
            true
        );
        let WorkflowBoundaryOutcome::WorkflowFailed { failure, report } = outcome else {
            unreachable!("workflow boundary adapter must surface failures as workflow failures");
        };

        let cause = failure
            .cause
            .as_deref()
            .expect("workflow boundary failures preserve lower cause");

        prop_assert_eq!(failure.workflow_id, workflow_id);
        prop_assert_eq!(failure.run_id, run_id);
        prop_assert_eq!(failure.kind, WorkflowFailureKind::BodyFailureEscaped);
        prop_assert_eq!(cause.tower, TowerLevel::Workflow);
        prop_assert_eq!(cause.entity, FailureEntity::Run(run_id));
        prop_assert_eq!(&cause.payload, &Value::String(lower.to_string()));
        prop_assert_eq!(report.workflow_id, workflow_id);
        prop_assert_eq!(report.run_id, run_id);
        prop_assert_eq!(report.failure, Some(failure.clone()));
        prop_assert_eq!(report.lower_causes, vec![cause.clone()]);
    }
}
