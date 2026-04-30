use ash_core::workflow_carrier::{
    ContractPlan, OpenPostcondition, ProjectionEventKind, SourceOrigin, WorkflowBinder,
    WorkflowForm, WorkflowNodeId, WorkflowObligation, WorkflowProcProjection, lower_workflow_form,
};
use ash_core::workflow_contract::{ArithConstraint, PostPredicate, Requirement};

fn origin() -> SourceOrigin {
    SourceOrigin::Synthetic {
        parent_span: None,
        reason: "task-778 neutral projection preservation".to_string(),
    }
}

#[test]
fn requires_neutral_proc_node_is_preserved_inside_bind_until_evidence_optimization() {
    let requirement = Requirement::HasRole("Reviewer".to_string());
    let form = WorkflowForm::Bind {
        node: WorkflowNodeId(12),
        source: Box::new(WorkflowForm::Requires {
            node: WorkflowNodeId(10),
            requirement: requirement.clone(),
        }),
        binder: WorkflowBinder::Ignored,
        next: Box::new(WorkflowForm::Unit {
            node: WorkflowNodeId(11),
            value: "accepted",
        }),
    };

    let lowered = lower_workflow_form(&form, origin());

    let WorkflowProcProjection::Bind {
        node,
        source,
        binder,
        next,
    } = &lowered.proc_projection
    else {
        panic!(
            "neutral requires source must not be erased from sequential Proc projection: {:?}",
            lowered.proc_projection
        );
    };
    assert_eq!(*node, WorkflowNodeId(12));
    assert_eq!(*binder, WorkflowBinder::Ignored);
    assert!(
        matches!(
            **source,
            WorkflowProcProjection::Neutral {
                node: WorkflowNodeId(10)
            }
        ),
        "requires source must remain a neutral Proc node until evidence-preserving optimization: {source:?}"
    );
    assert!(
        matches!(
            **next,
            WorkflowProcProjection::Unit {
                node: WorkflowNodeId(11),
                ..
            }
        ),
        "bind continuation unit must remain present after neutral source: {next:?}"
    );

    assert!(lowered.projection_events.iter().any(|event| {
        event.node == WorkflowNodeId(10)
            && matches!(event.kind, ProjectionEventKind::Requires { requirement: ref r } if r == &requirement)
    }));
    assert!(lowered.projection_events.iter().any(|event| {
        event.node == WorkflowNodeId(12) && matches!(event.kind, ProjectionEventKind::Then)
    }));
    assert!(
        lowered
            .coverage
            .obligations
            .contains(&WorkflowObligation::RequirementMustHold {
                node: WorkflowNodeId(10),
                requirement: requirement.clone(),
            },)
    );
    assert!(matches!(
        lowered.contract.plan,
        ContractPlan::BindContract { ref first, ref second, .. }
            if matches!(**first, ContractPlan::RequirementContract { node: WorkflowNodeId(10), requirement: ref r } if r == &requirement)
                && matches!(**second, ContractPlan::EmptyContract { result_marker: Some("accepted") })
    ));
}

#[test]
fn ensures_neutral_proc_node_is_preserved_inside_bind_until_evidence_optimization() {
    let postcondition = OpenPostcondition {
        predicate: PostPredicate::ResultSatisfies(ArithConstraint::Gte(1)),
    };
    let form = WorkflowForm::Bind {
        node: WorkflowNodeId(22),
        source: Box::new(WorkflowForm::Ensures {
            node: WorkflowNodeId(20),
            postcondition: postcondition.clone(),
        }),
        binder: WorkflowBinder::Ignored,
        next: Box::new(WorkflowForm::Unit {
            node: WorkflowNodeId(21),
            value: 1,
        }),
    };

    let lowered = lower_workflow_form(&form, origin());

    let WorkflowProcProjection::Bind { source, next, .. } = &lowered.proc_projection else {
        panic!(
            "neutral ensures source must not be erased from sequential Proc projection: {:?}",
            lowered.proc_projection
        );
    };
    assert!(
        matches!(
            **source,
            WorkflowProcProjection::Neutral {
                node: WorkflowNodeId(20)
            }
        ),
        "ensures source must remain a neutral Proc node until evidence-preserving optimization: {source:?}"
    );
    assert!(
        matches!(
            **next,
            WorkflowProcProjection::Unit {
                node: WorkflowNodeId(21),
                ..
            }
        ),
        "bind continuation unit must remain present after neutral source: {next:?}"
    );

    assert!(lowered.projection_events.iter().any(|event| {
        event.node == WorkflowNodeId(20)
            && matches!(event.kind, ProjectionEventKind::Ensures { postcondition: ref p } if p == &postcondition)
    }));
    assert!(
        lowered
            .coverage
            .obligations
            .contains(&WorkflowObligation::OpenPostconditionTarget {
                node: WorkflowNodeId(20),
                postcondition: postcondition.clone(),
                target_type: "WorkflowResult".to_string(),
            },)
    );
    assert!(matches!(
        lowered.contract.plan,
        ContractPlan::BindContract { ref first, ref second, .. }
            if matches!(**first, ContractPlan::EnsuresContract { node: WorkflowNodeId(20), postcondition: ref p, .. } if p == &postcondition)
                && matches!(**second, ContractPlan::EmptyContract { result_marker: Some(1) })
    ));
}
