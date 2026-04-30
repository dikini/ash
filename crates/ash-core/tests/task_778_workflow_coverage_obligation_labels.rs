use ash_core::workflow_carrier::{
    ActLowerSummary, AlignmentKey, CoverageError, OpenPostcondition, ProcContractSummary,
    ProcLowerSummary, ProjectionKind, SourceOrigin, WorkflowBinder, WorkflowForm, WorkflowNodeId,
    WorkflowObligation, lower_workflow_form,
};
use ash_core::workflow_contract::{ArithConstraint, PostPredicate, Requirement};

fn origin() -> SourceOrigin {
    SourceOrigin::Synthetic {
        parent_span: None,
        reason: "task-778 coverage labels".to_string(),
    }
}

#[test]
fn lower_proc_and_act_obligations_report_obligations_component_and_specific_label() {
    let proc_summary = ProcLowerSummary {
        coverage_obligation_nodes: vec![WorkflowNodeId(20)],
        contract_summary: Some(ProcContractSummary {
            obligations: vec![WorkflowNodeId(21)],
            public_anchor: Some("proc.admin_body".to_string()),
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
            summary: proc_summary,
        }),
        binder: WorkflowBinder::Ignored,
        next: Box::new(WorkflowForm::FromAct {
            node: WorkflowNodeId(30),
            summary: act_summary,
        }),
    };

    let lowered = lower_workflow_form(&form, origin());

    let proc_obligation = lowered
        .coverage
        .obligations
        .iter()
        .find(|obligation| {
            matches!(
                obligation,
                WorkflowObligation::LowerProcCovered {
                    node: WorkflowNodeId(20),
                    ..
                }
            )
        })
        .expect("from_proc must emit a lower Proc coverage obligation");

    assert_eq!(proc_obligation.evidence_component(), "obligations");
    assert_eq!(
        proc_obligation.diagnostic_label(),
        "lower Proc contract coverage"
    );
    assert!(
        proc_obligation
            .diagnostic_message()
            .contains("proc.admin_body"),
        "lower Proc diagnostic should include public anchor when available: {}",
        proc_obligation.diagnostic_message()
    );

    let act_obligation = lowered
        .coverage
        .obligations
        .iter()
        .find(|obligation| {
            matches!(
                obligation,
                WorkflowObligation::LowerActCovered {
                    node: WorkflowNodeId(30),
                    ..
                }
            )
        })
        .expect("from_act must emit a lower Act coverage obligation");

    assert_eq!(act_obligation.evidence_component(), "obligations");
    assert_eq!(
        act_obligation.diagnostic_label(),
        "lower Act contract coverage"
    );
    assert!(
        act_obligation.diagnostic_message().contains("node 30"),
        "lower Act diagnostic should identify the failed node: {}",
        act_obligation.diagnostic_message()
    );
}

#[test]
fn coverage_errors_name_failed_evidence_component_and_projection() {
    let missing_projection = CoverageError::MissingProjectionEvent {
        key: AlignmentKey {
            node: WorkflowNodeId(77),
            projection: ProjectionKind::Failure,
        },
    };

    assert_eq!(missing_projection.evidence_component(), "failure");
    assert!(
        missing_projection.to_string().contains("failure"),
        "CoverageError Display should mention failed evidence component: {missing_projection}"
    );
    assert!(
        missing_projection.to_string().contains("node 77"),
        "CoverageError Display should mention node id: {missing_projection}"
    );
    assert!(
        missing_projection.to_string().contains("projection event"),
        "CoverageError Display should explain missing projection event: {missing_projection}"
    );

    let opaque = CoverageError::OpaqueSummaryRejected {
        node: WorkflowNodeId(88),
        imported_name: "external::opaque_workflow".to_string(),
    };

    assert_eq!(opaque.evidence_component(), "obligations");
    assert!(
        opaque.to_string().contains("obligations"),
        "opaque summary rejection should be reported against obligations evidence: {opaque}"
    );
    assert!(
        opaque.to_string().contains("external::opaque_workflow"),
        "opaque summary diagnostic should name imported summary: {opaque}"
    );
}

#[test]
fn requirement_and_postcondition_obligations_distinguish_proof_boundaries() {
    let requirement = Requirement::HasRole("Reviewer".to_string());
    let must_hold = WorkflowObligation::RequirementMustHold {
        node: WorkflowNodeId(101),
        requirement: requirement.clone(),
    };

    assert_eq!(must_hold.evidence_component(), "obligations");
    assert_eq!(
        must_hold.diagnostic_label(),
        "workflow requirement coverage"
    );
    let must_hold_message = must_hold.diagnostic_message();
    assert!(
        must_hold_message.contains("must be proven"),
        "{must_hold_message}"
    );
    assert!(
        must_hold_message.contains("admission"),
        "{must_hold_message}"
    );
    assert!(
        must_hold_message.contains("obligations evidence"),
        "{must_hold_message}"
    );
    assert!(
        must_hold_message.contains("node 101"),
        "{must_hold_message}"
    );

    let refinement = WorkflowObligation::RequirementRefinementCovered {
        node: WorkflowNodeId(102),
        requirement,
    };
    assert_eq!(
        refinement.diagnostic_label(),
        "requirement refinement coverage"
    );
    let refinement_message = refinement.diagnostic_message();
    assert!(
        refinement_message.contains("assumed"),
        "{refinement_message}"
    );
    assert!(
        refinement_message.contains("refines checking context"),
        "{refinement_message}"
    );
    assert!(
        refinement_message.contains("not final proof"),
        "{refinement_message}"
    );
    assert!(
        refinement_message.contains("node 102"),
        "{refinement_message}"
    );

    let postcondition = WorkflowObligation::OpenPostconditionTarget {
        node: WorkflowNodeId(103),
        postcondition: OpenPostcondition {
            predicate: PostPredicate::ResultSatisfies(ArithConstraint::Gte(1)),
        },
        target_type: "WorkflowResult".to_string(),
    };
    assert_eq!(
        postcondition.diagnostic_label(),
        "open postcondition target coverage"
    );
    let postcondition_message = postcondition.diagnostic_message();
    assert!(
        postcondition_message.contains("successful result boundary"),
        "{postcondition_message}"
    );
    assert!(
        postcondition_message.contains("WorkflowResult"),
        "{postcondition_message}"
    );
    assert!(
        postcondition_message.contains("obligations evidence"),
        "{postcondition_message}"
    );
    assert!(
        postcondition_message.contains("node 103"),
        "{postcondition_message}"
    );
}
