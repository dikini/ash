//! RED tests for TASK-715 application admission above interpreter execution.

use ash_core::capability::{
    CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
};
use ash_core::runtime::{
    ApplicationBoundaryOutcome, ApplicationFailureKind, ApplicationReportStatus, RunId,
};
use ash_core::{
    Capability, Constraint, Effect, Expr, Guard, Pattern, Provenance, Role, Span, Value, Workflow,
    WorkflowId,
};
use ash_engine::{
    ApplicationAdmissionOutcome, ApplicationAdmissionRequest, ApplicationContractRequirement,
    Engine,
};
use async_trait::async_trait;
use proptest::prelude::*;
use std::sync::Arc;

const fn honest_body() -> Workflow {
    Workflow::Ret {
        expr: Expr::Literal(Value::Int(7)),
    }
}

#[derive(Debug)]
struct StaticObserveProvider {
    name: &'static str,
    value: Value,
}

#[async_trait]
impl CapabilityProvider for StaticObserveProvider {
    fn name(&self) -> &str {
        self.name
    }

    fn effect(&self) -> Effect {
        Effect::Epistemic
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        Ok(self.value.clone())
    }

    async fn execute(&self, _action_name: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        Err(CapabilityError::ExecutionFailed(
            "observe-only provider".to_string(),
        ))
    }
}

#[derive(Debug)]
struct StaticActionProvider {
    name: &'static str,
    result: Value,
}

#[async_trait]
impl CapabilityProvider for StaticActionProvider {
    fn name(&self) -> &str {
        self.name
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new(self.name()).with_operation(
            ProviderOperationMetadata::new("charge", Effect::Operational)
                .with_required_row("payments.charge")
                .with_resource("payments")
                .with_sandbox_policy("test-sandbox")
                .with_provenance_policy("test-provenance"),
        )
    }

    async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        Err(CapabilityError::ExecutionFailed(
            "action-only provider".to_string(),
        ))
    }

    async fn execute(&self, action_name: &str, _args: &[Value]) -> Result<Value, CapabilityError> {
        if action_name == "charge" {
            Ok(self.result.clone())
        } else {
            Err(CapabilityError::ExecutionFailed(format!(
                "unexpected action: {action_name}"
            )))
        }
    }
}

fn observe_body(capability_name: &str) -> Workflow {
    Workflow::Observe {
        capability: Capability {
            name: capability_name.to_string(),
            effect: Effect::Epistemic,
            constraints: vec![],
        },
        pattern: Pattern::Variable {
            name: "reading".to_string(),
            span: Span::default(),
        },
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Variable {
                name: "reading".to_string(),
                span: Span::default(),
            },
        }),
    }
}

fn act_body(provider_name: &str, action_name: &str) -> Workflow {
    Workflow::Act {
        provider_name: provider_name.to_string(),
        action_name: action_name.to_string(),
        arguments: vec![],
        guard: Guard::Always,
        provenance: Provenance::default(),
        result_name: Some("result".to_string()),
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Variable {
                name: "result".to_string(),
                span: Span::default(),
            },
        }),
    }
}

fn admitted_role(name: &str) -> Role {
    Role {
        name: name.to_string(),
        authority: vec![],
        obligations: vec![],
    }
}

fn forced_invoke_workflow(provider_name: &str, action_name: &str) -> Workflow {
    Workflow::Ret {
        expr: Expr::FnApply {
            func: Box::new(Expr::Call {
                func: "invoke".to_string(),
                module: None,
                arguments: vec![
                    Expr::Literal(Value::String(provider_name.to_string())),
                    Expr::Literal(Value::String(action_name.to_string())),
                    Expr::Literal(Value::list_nil()),
                ],
            }),
            args: vec![Expr::Literal(Value::ActEnvToken)],
        },
    }
}

#[tokio::test]
async fn admission_creates_or_accepts_application_and_run_ids_before_body_execution() {
    let engine = Engine::new().build().expect("engine builds");
    let application_id = WorkflowId::new();
    let run_id = RunId::new();
    let request = ApplicationAdmissionRequest {
        entry_name: "task_715_success".to_string(),
        workflow: honest_body(),
        application_id: Some(application_id),
        run_id: Some(run_id),
        active_role: Some("reviewer".to_string()),
        admitted_role: Some(admitted_role("reviewer")),
        required_capabilities: vec!["payments.charge".to_string()],
        requires: vec![],
        ensures: vec!["result.audit_recorded".to_string()],
    };

    let outcome = engine.admit_application(request).await;

    match outcome {
        ApplicationAdmissionOutcome::Admitted { boundary } => {
            assert_eq!(boundary.application_id(), application_id);
            assert_eq!(boundary.run_id(), run_id);
            assert!(matches!(
                boundary.report().status,
                ApplicationReportStatus::Succeeded | ApplicationReportStatus::Failed
            ));
            assert_eq!(
                boundary.report().admission.active_role.as_deref(),
                Some("reviewer")
            );
            assert_eq!(
                boundary.report().admission.admitted_capabilities,
                vec!["payments.charge".to_string()]
            );
            assert_eq!(boundary.report().ensures_evidence.len(), 1);
            assert!(
                boundary
                    .report()
                    .ensures_evidence
                    .iter()
                    .all(|entry| !matches!(
                        entry.status,
                        ash_core::runtime::ApplicationEvidenceStatus::Pending
                    )),
                "TASK-716 completion should resolve carried ensures evidence before reporting the admitted boundary outcome"
            );
        }
        other @ ApplicationAdmissionOutcome::Rejected { .. } => {
            panic!("expected admitted application boundary carrier, got {other:?}")
        }
    }
}

#[tokio::test]
async fn role_or_capability_admission_failure_maps_to_structured_application_failure() {
    let engine = Engine::new().build().expect("engine builds");
    let request = ApplicationAdmissionRequest {
        entry_name: "task_715_missing_authority".to_string(),
        workflow: honest_body(),
        application_id: None,
        run_id: None,
        active_role: Some("approver".to_string()),
        admitted_role: Some(admitted_role("approver")),
        required_capabilities: vec!["payments.charge".to_string()],
        requires: vec![
            ApplicationContractRequirement::Role("auditor".to_string()),
            ApplicationContractRequirement::Capability("payments.refund".to_string()),
        ],
        ensures: vec![],
    };

    let outcome = engine.admit_application(request).await;

    match outcome {
        ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert!(matches!(
                failure.kind,
                ApplicationFailureKind::RoleAdmissionFailure
                    | ApplicationFailureKind::CapabilityAdmissionFailure
                    | ApplicationFailureKind::AdmissionFailure
            ));
            assert_eq!(report.status, ApplicationReportStatus::Failed);
            assert!(report.result.is_none());
            assert!(report.requires_evidence.is_empty());
        }
        other @ ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("expected structured admission rejection, got {other:?}")
        }
    }
}

#[tokio::test]
async fn requires_failure_prevents_body_execution_and_records_evidence_in_report() {
    let engine = Engine::new().build().expect("engine builds");
    let request = ApplicationAdmissionRequest {
        entry_name: "task_715_requires_failure".to_string(),
        workflow: honest_body(),
        application_id: None,
        run_id: None,
        active_role: None,
        admitted_role: None,
        required_capabilities: vec![],
        requires: vec![ApplicationContractRequirement::Evidence {
            clause: "request.signature".to_string(),
            passed: false,
            notes: vec!["signature missing".to_string()],
        }],
        ensures: vec!["result.audit_recorded".to_string()],
    };

    let outcome = engine.admit_application(request).await;

    match outcome {
        ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(failure.kind, ApplicationFailureKind::RequiresViolation);
            assert_eq!(report.status, ApplicationReportStatus::Failed);
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
        other @ ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("expected requires rejection before body execution, got {other:?}")
        }
    }
}

#[tokio::test]
async fn active_role_without_admitted_role_is_rejected_before_execution() {
    let engine = Engine::new().build().expect("engine builds");
    let request = ApplicationAdmissionRequest {
        entry_name: "task_715_requires_truthful_role_projection".to_string(),
        workflow: honest_body(),
        application_id: None,
        run_id: None,
        active_role: Some("reviewer".to_string()),
        admitted_role: None,
        required_capabilities: vec![],
        requires: vec![],
        ensures: vec![],
    };

    let outcome = engine.admit_application(request).await;

    match outcome {
        ApplicationAdmissionOutcome::Rejected { failure, report } => {
            assert_eq!(failure.kind, ApplicationFailureKind::RoleAdmissionFailure);
            assert_eq!(report.status, ApplicationReportStatus::Failed);
        }
        other @ ApplicationAdmissionOutcome::Admitted { .. } => {
            panic!("expected role-admission rejection for missing admitted role, got {other:?}")
        }
    }
}

#[tokio::test]
async fn empty_admitted_capability_surface_denies_all_runtime_access() {
    let engine = Engine::new()
        .with_custom_provider(
            "secret",
            Arc::new(StaticObserveProvider {
                name: "secret",
                value: Value::Int(9),
            }),
        )
        .build()
        .expect("engine builds");

    let request = ApplicationAdmissionRequest {
        entry_name: "task_715_empty_projection".to_string(),
        workflow: observe_body("secret"),
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
                assert_eq!(failure.kind, ApplicationFailureKind::BodyFailureEscaped);
                assert_eq!(report.status, ApplicationReportStatus::Failed);
                let lower = failure
                    .cause
                    .as_deref()
                    .expect("empty admission surface should block runtime capability access");
                assert!(
                    format!("{}", lower.payload).contains("capability not available: secret"),
                    "expected unadmitted provider failure, got {lower:?}"
                );
            }
            other @ ApplicationBoundaryOutcome::ApplicationSucceeded { .. } => {
                panic!("expected boundary failure for empty capability surface, got {other:?}")
            }
        },
        other @ ApplicationAdmissionOutcome::Rejected { .. } => {
            panic!("expected admitted application boundary outcome, got {other:?}")
        }
    }
}

#[tokio::test]
async fn admitted_capability_surface_denies_runtime_access_to_omitted_provider() {
    let engine = Engine::new()
        .with_custom_provider(
            "allowed",
            Arc::new(StaticObserveProvider {
                name: "allowed",
                value: Value::Int(1),
            }),
        )
        .with_custom_provider(
            "secret",
            Arc::new(StaticObserveProvider {
                name: "secret",
                value: Value::Int(9),
            }),
        )
        .build()
        .expect("engine builds");

    let request = ApplicationAdmissionRequest {
        entry_name: "task_715_capability_projection".to_string(),
        workflow: observe_body("secret"),
        application_id: None,
        run_id: None,
        active_role: None,
        admitted_role: None,
        required_capabilities: vec!["allowed".to_string()],
        requires: vec![],
        ensures: vec![],
    };

    let outcome = engine.admit_application(request).await;

    match outcome {
        ApplicationAdmissionOutcome::Admitted { boundary } => match boundary.outcome() {
            ApplicationBoundaryOutcome::ApplicationFailed { failure, report } => {
                assert_eq!(failure.kind, ApplicationFailureKind::BodyFailureEscaped);
                assert_eq!(report.status, ApplicationReportStatus::Failed);
                let lower = failure
                    .cause
                    .as_deref()
                    .expect("lower cause should be preserved for omitted provider access");
                assert!(
                    format!("{}", lower.payload).contains("capability not available: secret"),
                    "expected omitted provider failure, got {lower:?}"
                );
            }
            other @ ApplicationBoundaryOutcome::ApplicationSucceeded { .. } => {
                panic!("expected boundary failure for omitted capability access, got {other:?}")
            }
        },
        other @ ApplicationAdmissionOutcome::Rejected { .. } => {
            panic!("expected admitted application boundary outcome, got {other:?}")
        }
    }
}

#[tokio::test]
async fn forced_invoke_uses_projected_admission_surface_in_hidden_act_env() {
    let engine = Engine::new()
        .with_custom_provider(
            "payments",
            Arc::new(StaticActionProvider {
                name: "payments",
                result: Value::Int(42),
            }),
        )
        .build()
        .expect("engine builds");

    let request = ApplicationAdmissionRequest {
        entry_name: "task_715_invoke_projection".to_string(),
        workflow: forced_invoke_workflow("payments", "charge"),
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
            ApplicationBoundaryOutcome::ApplicationFailed { failure, .. } => {
                let lower = failure
                    .cause
                    .as_deref()
                    .expect("hidden ActEnv invoke should preserve lower cause");
                assert!(
                    format!("{}", lower.payload).contains(
                        "authority boundary failure: provider payments lacks RuntimeKernel admission"
                    ),
                    "expected projected hidden ActEnv admission failure, got {lower:?}"
                );
            }
            other @ ApplicationBoundaryOutcome::ApplicationSucceeded { .. } => {
                panic!("expected hidden ActEnv projection failure, got {other:?}")
            }
        },
        other @ ApplicationAdmissionOutcome::Rejected { .. } => {
            panic!("expected admitted application boundary outcome, got {other:?}")
        }
    }
}

#[tokio::test]
async fn action_qualified_admitted_capability_projects_provider_for_runtime_act() {
    let engine = Engine::new()
        .with_custom_provider(
            "payments",
            Arc::new(StaticActionProvider {
                name: "payments",
                result: Value::Int(42),
            }),
        )
        .build()
        .expect("engine builds");

    let request = ApplicationAdmissionRequest {
        entry_name: "task_715_action_projection".to_string(),
        workflow: act_body("payments", "charge"),
        application_id: None,
        run_id: None,
        active_role: None,
        admitted_role: None,
        required_capabilities: vec!["payments.charge".to_string()],
        requires: vec![],
        ensures: vec![],
    };

    let outcome = engine.admit_application(request).await;

    match outcome {
        ApplicationAdmissionOutcome::Admitted { boundary } => match boundary.outcome() {
            ApplicationBoundaryOutcome::ApplicationSucceeded { value, report, .. } => {
                assert_eq!(value, &Value::Int(42));
                assert_eq!(report.status, ApplicationReportStatus::Succeeded);
                assert_eq!(
                    report.admission.admitted_capabilities,
                    vec!["payments.charge".to_string()]
                );
                assert!(
                    !report.admission.admitted_capability_bindings.is_empty(),
                    "RuntimeKernel admission must carry explicit binding ids, not only string grants"
                );
            }
            other @ ApplicationBoundaryOutcome::ApplicationFailed { .. } => {
                panic!("expected projected action capability success, got {other:?}")
            }
        },
        other @ ApplicationAdmissionOutcome::Rejected { .. } => {
            panic!("expected admitted application boundary outcome, got {other:?}")
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
            let application_id = WorkflowId::new();
            let run_id = RunId::new();
            let request = ApplicationAdmissionRequest {
                entry_name: "task_715_prop".to_string(),
                workflow: honest_body(),
                application_id: Some(application_id),
                run_id: Some(run_id),
                active_role: None,
                admitted_role: None,
                required_capabilities: vec![],
                requires: vec![ApplicationContractRequirement::Evidence {
                    clause: "host.check".to_string(),
                    passed: true,
                    notes: vec![requires_note.clone()],
                }],
                ensures: vec![format!("ensures::{ensures_note}")],
            };

            let outcome = engine.admit_application(request).await;
            let boundary = match outcome {
                ApplicationAdmissionOutcome::Admitted { boundary } => boundary,
                other @ ApplicationAdmissionOutcome::Rejected { .. } => {
            panic!("expected admitted application boundary carrier, got {other:?}")
        } ,
            };

            prop_assert_eq!(boundary.application_id(), application_id);
            prop_assert_eq!(boundary.run_id(), run_id);
            prop_assert_eq!(boundary.report().requires_evidence.len(), 1);
            prop_assert_eq!(boundary.report().ensures_evidence.len(), 1);
            Ok(())
        })?;
    }
}
