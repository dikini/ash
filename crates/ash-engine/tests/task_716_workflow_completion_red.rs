//! RED tests for TASK-716 workflow completion/report construction.

use ash_core::runtime::{
    ApplicationBoundaryOutcome, ApplicationEvidenceStatus, ApplicationFailureKind,
    ApplicationReportStatus,
};
use ash_core::workflow_contract::Span as ContractSpan;
use ash_core::{Expr, Role, RoleObligationRef, Span, Value, Workflow};
use ash_engine::{ApplicationAdmissionOutcome, ApplicationAdmissionRequest, Engine};

const fn honest_body() -> Workflow {
    Workflow::Ret {
        expr: Expr::Literal(Value::Int(7)),
    }
}

fn body_with_undischarged_local_obligation() -> Workflow {
    Workflow::Seq {
        first: Box::new(Workflow::Oblige {
            name: "audit".to_string(),
            span: ContractSpan { start: 0, end: 0 },
        }),
        second: Box::new(honest_body()),
    }
}

fn body_with_escaping_lower_failure() -> Workflow {
    Workflow::Ret {
        expr: Expr::Variable {
            name: "missing_value".to_string(),
            span: Span { start: 0, end: 0 },
        },
    }
}

fn body_with_untaken_obligation_branch() -> Workflow {
    Workflow::If {
        condition: Expr::Literal(Value::Bool(false)),
        then_branch: Box::new(Workflow::Seq {
            first: Box::new(Workflow::Oblige {
                name: "audit".to_string(),
                span: ContractSpan { start: 0, end: 0 },
            }),
            second: Box::new(Workflow::Ret {
                expr: Expr::Literal(Value::Int(1)),
            }),
        }),
        else_branch: Box::new(Workflow::Ret {
            expr: Expr::Literal(Value::Int(2)),
        }),
    }
}

fn admitted_role_with_obligation(name: &str, obligation: &str) -> Role {
    Role {
        name: name.to_string(),
        authority: vec![],
        obligations: vec![RoleObligationRef {
            name: obligation.to_string(),
        }],
    }
}

#[tokio::test]
async fn completion_must_not_report_success_with_pending_ensures_placeholders() {
    let engine = Engine::new().build().expect("engine builds");
    let request = ApplicationAdmissionRequest {
        entry_name: "task_716_ensures_pending".to_string(),
        workflow: honest_body(),
        application_id: None,
        run_id: None,
        active_role: None,
        admitted_role: None,
        required_capabilities: vec![],
        requires: vec![],
        ensures: vec!["result.audit_recorded".to_string()],
    };

    let outcome = engine.admit_application(request).await;

    match outcome {
        ApplicationAdmissionOutcome::Admitted { boundary } => match boundary.outcome() {
            ApplicationBoundaryOutcome::ApplicationSucceeded { report, .. } => {
                assert!(
                    report
                        .ensures_evidence
                        .iter()
                        .all(|entry| entry.status != ApplicationEvidenceStatus::Pending),
                    "workflow completion must resolve ensures before reporting success"
                );
            }
            ApplicationBoundaryOutcome::ApplicationFailed { failure, report } => {
                assert_eq!(failure.kind, ApplicationFailureKind::EnsuresViolation);
                assert_eq!(report.status, ApplicationReportStatus::Failed);
                assert!(
                    report
                        .ensures_evidence
                        .iter()
                        .any(|entry| entry.status == ApplicationEvidenceStatus::Failed),
                    "ensures failure must be recorded as failed evidence"
                );
                assert!(report.external_report_sink.is_none());
            }
        },
        other @ ApplicationAdmissionOutcome::Rejected { .. } => {
            panic!("completion-time ensures should not reject admission, got {other:?}")
        }
    }
}

#[tokio::test]
async fn undischarged_local_obligations_become_application_boundary_failure_with_local_report() {
    let engine = Engine::new().build().expect("engine builds");
    let request = ApplicationAdmissionRequest {
        entry_name: "task_716_local_obligation".to_string(),
        workflow: body_with_undischarged_local_obligation(),
        application_id: None,
        run_id: None,
        active_role: None,
        admitted_role: None,
        required_capabilities: vec![],
        requires: vec![],
        ensures: vec![],
    };

    let outcome = engine.admit_application(request).await;

    match outcome {
        ApplicationAdmissionOutcome::Admitted { boundary } => match boundary.outcome() {
            ApplicationBoundaryOutcome::ApplicationFailed { failure, report } => {
                assert_eq!(
                    failure.kind,
                    ApplicationFailureKind::LocalObligationsUndischarged
                );
                assert_eq!(report.status, ApplicationReportStatus::Failed);
                assert_eq!(
                    report.failure.as_ref().map(|failure| failure.kind),
                    Some(ApplicationFailureKind::LocalObligationsUndischarged)
                );
                assert!(report.external_report_sink.is_none());
                assert!(
                    !report.obligation_evidence.is_empty(),
                    "boundary failure should retain obligation completion evidence locally"
                );
            }
            other @ ApplicationBoundaryOutcome::ApplicationSucceeded { .. } => {
                panic!("expected application-boundary completion failure, got {other:?}")
            }
        },
        other @ ApplicationAdmissionOutcome::Rejected { .. } => {
            panic!("completion-time obligations should not reject admission, got {other:?}")
        }
    }
}

#[tokio::test]
async fn escaped_lower_failure_retains_local_report_linkage_and_lower_execution_evidence() {
    let engine = Engine::new().build().expect("engine builds");
    let request = ApplicationAdmissionRequest {
        entry_name: "task_716_lower_failure".to_string(),
        workflow: body_with_escaping_lower_failure(),
        application_id: None,
        run_id: None,
        active_role: None,
        admitted_role: None,
        required_capabilities: vec![],
        requires: vec![],
        ensures: vec![],
    };

    let outcome = engine.admit_application(request).await;

    match outcome {
        ApplicationAdmissionOutcome::Admitted { boundary } => match boundary.outcome() {
            ApplicationBoundaryOutcome::ApplicationFailed { failure, report } => {
                let lower = failure
                    .cause
                    .as_deref()
                    .expect("escaped lower failures must preserve lower cause linkage")
                    .clone();

                assert_eq!(failure.kind, ApplicationFailureKind::BodyFailureEscaped);
                assert_eq!(report.status, ApplicationReportStatus::Failed);
                assert_eq!(report.failure, Some(failure.clone()));
                assert_eq!(report.lower_causes, vec![lower]);
                assert!(
                    !(report.evidence.is_empty()
                        && report.provenance.is_empty()
                        && report.lower_process_failures.is_empty()),
                    "TASK-716 should project execution-record/process-summary evidence into the local application report"
                );
                assert!(report.external_report_sink.is_none());
            }
            other @ ApplicationBoundaryOutcome::ApplicationSucceeded { .. } => {
                panic!("expected escaped lower failure at application boundary, got {other:?}")
            }
        },
        other @ ApplicationAdmissionOutcome::Rejected { .. } => {
            panic!("body failure escape should not look like admission rejection, got {other:?}")
        }
    }
}

#[tokio::test]
async fn untaken_obligation_branch_does_not_trigger_boundary_completion_failure() {
    let engine = Engine::new().build().expect("engine builds");
    let request = ApplicationAdmissionRequest {
        entry_name: "task_716_branch_obligation_precision".to_string(),
        workflow: body_with_untaken_obligation_branch(),
        application_id: None,
        run_id: None,
        active_role: None,
        admitted_role: None,
        required_capabilities: vec![],
        requires: vec![],
        ensures: vec![],
    };

    let outcome = engine.admit_application(request).await;

    match outcome {
        ApplicationAdmissionOutcome::Admitted { boundary } => match boundary.outcome() {
            ApplicationBoundaryOutcome::ApplicationSucceeded { report, value, .. } => {
                assert_eq!(report.status, ApplicationReportStatus::Succeeded);
                assert_eq!(value, &Value::Int(2));
                assert!(report.obligation_evidence.is_empty());
            }
            other @ ApplicationBoundaryOutcome::ApplicationFailed { .. } => {
                panic!(
                    "untaken obligation branches must not manufacture completion failures, got {other:?}"
                )
            }
        },
        other @ ApplicationAdmissionOutcome::Rejected { .. } => {
            panic!("untaken obligation branches should not reject admission, got {other:?}")
        }
    }
}

#[tokio::test]
async fn admitted_role_obligations_are_visible_at_runtime_completion_boundary() {
    let engine = Engine::new().build().expect("engine builds");
    let request = ApplicationAdmissionRequest {
        entry_name: "task_716_role_obligation_projection".to_string(),
        workflow: honest_body(),
        application_id: None,
        run_id: None,
        active_role: Some("reviewer".to_string()),
        admitted_role: Some(admitted_role_with_obligation("reviewer", "audit")),
        required_capabilities: vec![],
        requires: vec![],
        ensures: vec![],
    };

    let outcome = engine.admit_application(request).await;

    match outcome {
        ApplicationAdmissionOutcome::Admitted { boundary } => match boundary.outcome() {
            ApplicationBoundaryOutcome::ApplicationFailed { failure, report } => {
                assert_eq!(
                    failure.kind,
                    ApplicationFailureKind::RoleObligationsUndischarged
                );
                assert_eq!(report.status, ApplicationReportStatus::Failed);
                assert!(
                    report
                        .obligation_evidence
                        .iter()
                        .any(|entry| entry == "active_role:reviewer"),
                    "active role should be visible in retained completion evidence"
                );
                assert!(
                    report
                        .obligation_evidence
                        .iter()
                        .any(|entry| entry == "role_pending:audit"),
                    "admitted role obligations should remain visible at completion"
                );
            }
            other @ ApplicationBoundaryOutcome::ApplicationSucceeded { .. } => {
                panic!("expected role-obligation completion failure, got {other:?}")
            }
        },
        other @ ApplicationAdmissionOutcome::Rejected { .. } => {
            panic!("expected admitted application boundary outcome, got {other:?}")
        }
    }
}
