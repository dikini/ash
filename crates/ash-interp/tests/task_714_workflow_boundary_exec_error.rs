//! TASK-714 tests for application boundary carrier ExecError projection.

use ash_core::runtime::{
    ApplicationBoundaryOutcome, ApplicationFailureKind, FailureBoundary, FailureEntity, RunId,
};
use ash_core::{ApplicationId, Value};
use ash_interp::{ExecError, application_boundary_outcome_from_exec_result};
use proptest::prelude::*;

#[test]
fn application_boundary_adapter_preserves_exec_error_as_lower_cause() {
    let application_id = ApplicationId::new();
    let run_id = RunId::new();
    let lower = ExecError::ExecutionFailed("provider denied".to_string());

    let outcome =
        application_boundary_outcome_from_exec_result(application_id, run_id, Err(lower.clone()));

    match outcome {
        ApplicationBoundaryOutcome::ApplicationFailed { failure, report } => {
            assert_eq!(failure.kind, ApplicationFailureKind::BodyFailureEscaped);
            let cause = failure
                .cause
                .as_deref()
                .expect("lower exec error should be preserved as a application cause");
            assert_eq!(cause.boundary, FailureBoundary::Application);
            assert_eq!(cause.entity, FailureEntity::Run(run_id));
            assert_eq!(cause.payload, Value::String(lower.to_string()));
            assert_eq!(cause.payload_type, "ExecError");
            assert_eq!(report.lower_causes, vec![cause.clone()]);
        }
        other => panic!("expected application failure boundary outcome, got {other:?}"),
    }
}

proptest! {
    #[test]
    fn application_boundary_adapter_preserves_application_identity_and_failure_report(
        message in any::<String>(),
    ) {
        let application_id = ApplicationId::new();
        let run_id = RunId::new();
        let lower = ExecError::ExecutionFailed(message.clone());

        let outcome =
            application_boundary_outcome_from_exec_result(application_id, run_id, Err(lower.clone()));

        prop_assert_eq!(
            matches!(outcome, ApplicationBoundaryOutcome::ApplicationFailed { .. }),
            true
        );
        let ApplicationBoundaryOutcome::ApplicationFailed { failure, report } = outcome else {
            unreachable!("application boundary adapter must surface failures as application failures");
        };

        let cause = failure
            .cause
            .as_deref()
            .expect("application boundary failures preserve lower cause");

        prop_assert_eq!(failure.application_id, application_id);
        prop_assert_eq!(failure.run_id, run_id);
        prop_assert_eq!(failure.kind, ApplicationFailureKind::BodyFailureEscaped);
        prop_assert_eq!(cause.boundary, FailureBoundary::Application);
        prop_assert_eq!(cause.entity, FailureEntity::Run(run_id));
        prop_assert_eq!(&cause.payload, &Value::String(lower.to_string()));
        prop_assert_eq!(report.application_id, application_id);
        prop_assert_eq!(report.run_id, run_id);
        prop_assert_eq!(report.failure, Some(failure.clone()));
        prop_assert_eq!(report.lower_causes, vec![cause.clone()]);
    }
}
