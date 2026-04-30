#![allow(missing_docs)]

use ash_core::workflow_carrier::{
    ContractPlan, OpenPostcondition, ProcContractSummary, ProcLowerSummary, ProjectionEventKind,
    SourceOrigin, WorkflowBinder, WorkflowForm, WorkflowNodeId, lower_workflow_form,
};
use ash_core::workflow_contract::{ArithConstraint, PostPredicate, Requirement, RolePolicy};
use ash_engine::legacy_workflow_adapter::{
    LegacyWorkflowAdapterError, UnsupportedLegacyBodyConstruct,
    legacy_workflow_def_to_workflow_form, legacy_workflow_source_origin,
};
use ash_parser::{
    new_input,
    parse_module::{parse_resume, parse_yield},
    workflow_def,
};
use winnow::Parser;

fn parse_workflow(source: &str) -> ash_parser::surface::WorkflowDef {
    workflow_def
        .parse(new_input(source))
        .expect("workflow should parse")
}

fn has_workflow_result_ensures(plan: &ContractPlan<()>) -> bool {
    match plan {
        ContractPlan::EnsuresContract {
            postcondition,
            target,
            ..
        } => {
            postcondition.predicate == PostPredicate::ResultSatisfies(ArithConstraint::Gte(1))
                && *target == ash_core::workflow_carrier::PostconditionTarget::WorkflowResult
        }
        ContractPlan::BindContract { first, second, .. } => {
            has_workflow_result_ensures(first) || has_workflow_result_ensures(second)
        }
        ContractPlan::ScopeContract { plan, .. } => has_workflow_result_ensures(plan),
        _ => false,
    }
}

fn contains_body_placeholder(form: &WorkflowForm<()>) -> bool {
    match form {
        WorkflowForm::FromProc { summary, .. } => summary
            .contract_summary
            .as_ref()
            .and_then(|summary| summary.public_anchor.as_deref())
            .is_some_and(|anchor| anchor.starts_with("legacy_body_as_proc_summary")),
        WorkflowForm::Bind { source, next, .. } => {
            contains_body_placeholder(source) || contains_body_placeholder(next)
        }
        WorkflowForm::Scope { body, .. } => contains_body_placeholder(body),
        _ => false,
    }
}

fn public_contract_event_kinds(form: &WorkflowForm<()>) -> Vec<ProjectionEventKind> {
    lower_workflow_form(
        form,
        ash_core::workflow_carrier::SourceOrigin::Synthetic {
            parent_span: None,
            reason: "test equivalence".to_string(),
        },
    )
    .projection_events
    .into_iter()
    .filter_map(|event| match event.kind {
        ProjectionEventKind::Requires { .. } | ProjectionEventKind::Ensures { .. } => {
            Some(event.kind)
        }
        _ => None,
    })
    .collect()
}

#[test]
fn legacy_header_events_lower_in_source_order_with_any_role_or_semantics() {
    let workflow = parse_workflow(
        "workflow guarded requires: any_role([Reviewer, Approver]) ensures: result > 0 requires: role(Auditor) { done }",
    );

    let form = legacy_workflow_def_to_workflow_form(&workflow).expect("legacy adapter succeeds");
    let lowered = lower_workflow_form(&form, legacy_workflow_source_origin(&workflow));

    let contract_events: Vec<_> = lowered
        .projection_events
        .iter()
        .filter_map(|event| match &event.kind {
            ProjectionEventKind::Requires { requirement } => {
                Some(format!("requires:{requirement:?}"))
            }
            ProjectionEventKind::Ensures { postcondition } => {
                Some(format!("ensures:{postcondition:?}"))
            }
            _ => None,
        })
        .collect();

    assert_eq!(
        contract_events,
        vec![
            "requires:AnyRole(RolePolicy { roles: [\"Reviewer\", \"Approver\"] })",
            "ensures:OpenPostcondition { predicate: ResultSatisfies(Gt(0)) }",
            "requires:HasRole(\"Auditor\")",
        ],
        "header requires/ensures events must enter the shared projection path in source order"
    );

    assert!(matches!(
        &lowered.contract.admission.requirements[0],
        Requirement::AnyRole(policy) if policy.roles == ["Reviewer", "Approver"]
    ));
    assert_eq!(
        lowered.contract.admission.requirements[1],
        Requirement::HasRole("Auditor".to_string())
    );
}

#[test]
fn legacy_ensures_targets_successful_workflow_result_and_body_is_conservative_from_proc() {
    let workflow = parse_workflow("workflow positive ensures: result >= 1 { done }");

    let form = legacy_workflow_def_to_workflow_form(&workflow).expect("legacy adapter succeeds");
    let lowered = lower_workflow_form(&form, legacy_workflow_source_origin(&workflow));

    assert!(
        has_workflow_result_ensures(&lowered.contract.plan),
        "legacy ensures must target the successful workflow result, not a delayed/ambient value"
    );

    assert!(
        contains_body_placeholder(&form),
        "this slice should expose the conservative legacy body FromProc placeholder honestly"
    );
}

#[test]
fn legacy_body_summary_carries_lower_coverage_obligations_for_supported_body_nodes() {
    let workflow = parse_workflow("workflow body_summary { let x = 1 ret x }");

    let form = legacy_workflow_def_to_workflow_form(&workflow).expect("legacy adapter succeeds");
    let lowered = lower_workflow_form(&form, legacy_workflow_source_origin(&workflow));

    let Some(summary) = lowered
        .projection_events
        .iter()
        .find_map(|event| match &event.kind {
            ProjectionEventKind::FromProc { summary } => Some(summary),
            _ => None,
        })
    else {
        panic!("legacy body must enter the shared lowering path as FromProc");
    };

    assert_eq!(
        summary.coverage_obligation_nodes,
        summary
            .contract_summary
            .as_ref()
            .expect("supported legacy bodies carry a Proc contract summary")
            .obligations,
        "body coverage obligations and Proc contract obligations must stay aligned"
    );
    assert!(
        summary.coverage_obligation_nodes.len() >= 2,
        "oblige/check/ret body should not be represented as an obligation-free opaque body"
    );
    assert_eq!(
        summary
            .contract_summary
            .as_ref()
            .and_then(|summary| summary.public_anchor.as_deref()),
        Some("legacy_body_as_proc_summary:body_summary")
    );
}

#[test]
fn legacy_body_summary_carries_explicit_conservative_full_summary_fields() {
    let workflow = parse_workflow("workflow full_summary { let x = 1 ret x }");

    let form = legacy_workflow_def_to_workflow_form(&workflow).expect("legacy adapter succeeds");
    let lowered = lower_workflow_form(&form, legacy_workflow_source_origin(&workflow));

    let Some(summary) = lowered
        .projection_events
        .iter()
        .find_map(|event| match &event.kind {
            ProjectionEventKind::FromProc { summary } => Some(summary),
            _ => None,
        })
    else {
        panic!("legacy body must lower as FromProc");
    };

    assert_eq!(
        summary
            .failure_summary
            .as_ref()
            .map(|summary| summary.conservative),
        Some(true),
        "supported legacy FromProc summaries must carry an explicit conservative failure summary"
    );
    assert_eq!(
        summary
            .resource_authority_summary
            .as_ref()
            .map(|summary| summary.conservative),
        Some(true),
        "supported legacy FromProc summaries must carry an explicit conservative resource-authority summary"
    );
    assert_eq!(
        summary
            .provenance_summary
            .as_ref()
            .map(|summary| summary.conservative),
        Some(true),
        "supported legacy FromProc summaries must carry an explicit conservative provenance summary"
    );
    assert!(
        matches!(summary.source_origin, Some(SourceOrigin::Synthetic { .. })),
        "supported legacy FromProc summaries must carry explicit source-origin metadata"
    );
}

#[test]
fn legacy_body_adapter_rejects_opaque_receive_construct_with_diagnostic() {
    let workflow = parse_workflow("workflow opaque { receive { _ => done } }");

    let err = legacy_workflow_def_to_workflow_form(&workflow)
        .expect_err("opaque receive bodies must reject conservatively");

    assert!(matches!(
        err,
        LegacyWorkflowAdapterError::UnsupportedBody {
            construct: UnsupportedLegacyBodyConstruct::Receive,
            ..
        }
    ));
}

#[test]
fn legacy_body_adapter_rejects_opaque_yield_and_resume_constructs_with_diagnostics() {
    let cases = [
        (
            parse_yield
                .parse(new_input("yield role(manager) request resume response : TransferResponse { Approved => { done } }"))
                .expect("yield body should parse"),
            UnsupportedLegacyBodyConstruct::Yield,
        ),
        (
            parse_resume
                .parse(new_input("resume approved : ApprovalResponse"))
                .expect("resume body should parse"),
            UnsupportedLegacyBodyConstruct::Resume,
        ),
    ];

    for (body, construct) in cases {
        let mut workflow = parse_workflow("workflow opaque { done }");
        workflow.body = body;
        let err = legacy_workflow_def_to_workflow_form(&workflow)
            .expect_err("opaque yield/resume bodies must reject conservatively");

        assert!(matches!(
            err,
            LegacyWorkflowAdapterError::UnsupportedBody {
                construct: actual,
                ..
            } if actual == construct
        ));
    }
}

#[test]
fn legacy_and_first_class_forms_produce_equivalent_public_contract_events() {
    let workflow = parse_workflow(
        "workflow equivalent requires: any_role([Reviewer, Approver]) ensures: result >= 1 { done }",
    );

    let legacy_form =
        legacy_workflow_def_to_workflow_form(&workflow).expect("legacy adapter succeeds");
    let first_class_form = WorkflowForm::Bind {
        node: WorkflowNodeId(10),
        source: Box::new(WorkflowForm::Requires {
            node: WorkflowNodeId(11),
            requirement: Requirement::AnyRole(RolePolicy {
                roles: vec!["Reviewer".to_string(), "Approver".to_string()],
            }),
        }),
        binder: WorkflowBinder::Ignored,
        next: Box::new(WorkflowForm::Bind {
            node: WorkflowNodeId(12),
            source: Box::new(WorkflowForm::Ensures {
                node: WorkflowNodeId(13),
                postcondition: OpenPostcondition {
                    predicate: PostPredicate::ResultSatisfies(ArithConstraint::Gte(1)),
                },
            }),
            binder: WorkflowBinder::Ignored,
            next: Box::new(WorkflowForm::FromProc {
                node: WorkflowNodeId(14),
                summary: ProcLowerSummary {
                    coverage_obligation_nodes: Vec::new(),
                    contract_summary: Some(ProcContractSummary {
                        obligations: Vec::new(),
                        public_anchor: Some("first_class_body".to_string()),
                    }),
                    ..Default::default()
                },
            }),
        }),
    };

    assert_eq!(
        public_contract_event_kinds(&legacy_form),
        public_contract_event_kinds(&first_class_form),
        "legacy header translation and first-class workflow forms must expose equivalent public contract event sequences modulo source/body metadata"
    );
}
