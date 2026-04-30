use ash_core::workflow_carrier::{
    ActLowerSummary, ContractPlan, OpenPostcondition, ProcContractSummary, ProcFailureSummary,
    ProcLowerSummary, ProcProvenanceSummary, ProcResourceAuthoritySummary, ProjectionEventKind,
    SourceOrigin, WorkflowBinder, WorkflowForm, WorkflowNodeId, WorkflowObligation,
    WorkflowProcProjection, lower_workflow_form,
};
use ash_core::workflow_contract::{ArithConstraint, Effect, PostPredicate, Requirement};

fn origin() -> SourceOrigin {
    SourceOrigin::Synthetic {
        parent_span: None,
        reason: "task-774 test".to_string(),
    }
}

fn role_req(role: &str) -> Requirement {
    Requirement::HasRole(role.to_string())
}

fn post_eq(lhs: &str, rhs: &str) -> OpenPostcondition {
    OpenPostcondition {
        predicate: PostPredicate::Eq(lhs.to_string(), rhs.to_string()),
    }
}

#[test]
fn proc_summary_defaults_are_explicitly_conservative() {
    assert!(ProcFailureSummary::default().conservative);
    assert!(ProcResourceAuthoritySummary::default().conservative);
    assert!(ProcProvenanceSummary::default().conservative);
}

#[test]
fn unit_bind_then_shape_lowers_to_public_proc_projection_and_ordered_events() {
    let form = WorkflowForm::Bind {
        node: WorkflowNodeId(3),
        source: Box::new(WorkflowForm::Bind {
            node: WorkflowNodeId(2),
            source: Box::new(WorkflowForm::Unit {
                node: WorkflowNodeId(1),
                value: "first",
            }),
            binder: WorkflowBinder::Named("x".to_string()),
            next: Box::new(WorkflowForm::Unit {
                node: WorkflowNodeId(4),
                value: "second",
            }),
        }),
        binder: WorkflowBinder::Ignored,
        next: Box::new(WorkflowForm::Unit {
            node: WorkflowNodeId(5),
            value: "third",
        }),
    };

    let lowered = lower_workflow_form(&form, origin());

    assert!(matches!(
        lowered.proc_projection,
        WorkflowProcProjection::Bind {
            node: WorkflowNodeId(3),
            binder: WorkflowBinder::Ignored,
            ..
        }
    ));
    assert!(matches!(
        lowered.projection_events.as_slice(),
        [
            event1,
            event2,
            event4,
            event3,
            event5,
        ] if event1.node == WorkflowNodeId(1)
            && matches!(event1.kind, ProjectionEventKind::Unit { value_erased: false })
            && event2.node == WorkflowNodeId(2)
            && matches!(event2.kind, ProjectionEventKind::Bind { binder: WorkflowBinder::Named(ref name) } if name == "x")
            && event4.node == WorkflowNodeId(4)
            && matches!(event4.kind, ProjectionEventKind::Unit { value_erased: false })
            && event3.node == WorkflowNodeId(3)
            && matches!(event3.kind, ProjectionEventKind::Then)
            && event5.node == WorkflowNodeId(5)
            && matches!(event5.kind, ProjectionEventKind::Unit { value_erased: false })
    ));
}

#[test]
fn requires_and_ensures_survive_lowering_as_contract_metadata_and_obligations() {
    let requirement = role_req("operator");
    let postcondition = post_eq("result", "ok");
    let form: WorkflowForm<()> = WorkflowForm::Bind {
        node: WorkflowNodeId(12),
        source: Box::new(WorkflowForm::Requires {
            node: WorkflowNodeId(10),
            requirement: requirement.clone(),
        }),
        binder: WorkflowBinder::Ignored,
        next: Box::new(WorkflowForm::Ensures {
            node: WorkflowNodeId(11),
            postcondition: postcondition.clone(),
        }),
    };

    let lowered = lower_workflow_form(&form, origin());

    assert!(matches!(
        lowered.proc_projection,
        WorkflowProcProjection::Bind { .. }
    ));
    assert_eq!(
        lowered.contract.admission.requirements,
        vec![requirement.clone()]
    );
    assert!(matches!(
        lowered.contract.plan,
        ContractPlan::BindContract { ref first, ref second, .. }
            if matches!(**first, ContractPlan::RequirementContract { node: WorkflowNodeId(10), requirement: ref r } if r == &requirement)
            && matches!(**second, ContractPlan::EnsuresContract { node: WorkflowNodeId(11), postcondition: ref p, .. } if p == &postcondition)
    ));
    assert_eq!(
        lowered.coverage.obligations,
        vec![
            WorkflowObligation::RequirementMustHold {
                node: WorkflowNodeId(10),
                requirement: requirement.clone(),
            },
            WorkflowObligation::OpenPostconditionTarget {
                node: WorkflowNodeId(11),
                postcondition: postcondition.clone(),
                target_type: "WorkflowResult".to_string(),
            },
        ]
    );
    assert!(lowered.projection_events.iter().any(|event| {
        event.node == WorkflowNodeId(10)
            && matches!(event.kind, ProjectionEventKind::Requires { requirement: ref r } if r == &requirement)
    }));
    assert!(lowered.projection_events.iter().any(|event| {
        event.node == WorkflowNodeId(11)
            && matches!(event.kind, ProjectionEventKind::Ensures { postcondition: ref p } if p == &postcondition)
    }));
}

#[test]
fn explicit_proc_and_act_lifts_preserve_summaries_and_delayed_coverage_obligations() {
    let proc_summary = ProcLowerSummary {
        coverage_obligation_nodes: vec![WorkflowNodeId(20)],
        contract_summary: Some(ProcContractSummary {
            obligations: vec![WorkflowNodeId(21)],
            public_anchor: Some("proc.anchor".to_string()),
        }),
        ..Default::default()
    };
    let act_summary = ActLowerSummary {
        coverage_obligation_nodes: vec![WorkflowNodeId(30)],
        contract_summary: None,
    };
    let form: WorkflowForm<()> = WorkflowForm::Bind {
        node: WorkflowNodeId(40),
        source: Box::new(WorkflowForm::FromProc {
            node: WorkflowNodeId(20),
            summary: proc_summary.clone(),
        }),
        binder: WorkflowBinder::Ignored,
        next: Box::new(WorkflowForm::FromAct {
            node: WorkflowNodeId(30),
            summary: act_summary.clone(),
        }),
    };

    let lowered = lower_workflow_form(&form, origin());

    assert!(lowered.projection_events.iter().any(|event| {
        event.node == WorkflowNodeId(20)
            && matches!(event.kind, ProjectionEventKind::FromProc { ref summary } if summary == &proc_summary)
    }));
    assert!(lowered.projection_events.iter().any(|event| {
        event.node == WorkflowNodeId(30)
            && matches!(event.kind, ProjectionEventKind::FromAct { ref summary } if summary == &act_summary)
    }));
    assert!(
        lowered
            .coverage
            .obligations
            .contains(&WorkflowObligation::LowerProcCovered {
                node: WorkflowNodeId(20),
                summary: proc_summary.contract_summary.clone().unwrap(),
            })
    );
    assert!(
        lowered
            .coverage
            .obligations
            .contains(&WorkflowObligation::LowerActCovered {
                node: WorkflowNodeId(30),
                summary: Default::default(),
            })
    );
}

#[test]
fn requirement_variants_are_not_erased_from_admission_metadata() {
    let requirement = Requirement::HasCapability {
        cap: "fs.read".to_string(),
        min_effect: Effect::Epistemic,
    };
    let form = WorkflowForm::<()>::Requires {
        node: WorkflowNodeId(50),
        requirement: requirement.clone(),
    };

    let lowered = lower_workflow_form(&form, origin());

    assert_eq!(lowered.contract.admission.requirements, vec![requirement]);
    assert!(matches!(
        lowered.proc_projection,
        WorkflowProcProjection::Neutral {
            node: WorkflowNodeId(50)
        }
    ));

    let arithmetic = Requirement::Arithmetic {
        var: "n".to_string(),
        constraint: ArithConstraint::Gte(0),
    };
    let lowered_arithmetic = lower_workflow_form(
        &WorkflowForm::<()>::Requires {
            node: WorkflowNodeId(51),
            requirement: arithmetic.clone(),
        },
        origin(),
    );
    assert_eq!(
        lowered_arithmetic.contract.admission.requirements,
        vec![arithmetic]
    );
}
