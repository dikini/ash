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
    surface::{
        ActionRef, CheckTarget, Expr, Literal, Name, ObligationRef, OperationalTarget, Pattern,
        Workflow,
    },
    token::Span,
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
        "workflow guarded plays role(Admin) requires: any_role([Reviewer, Approver]) ensures: result > 0 requires: role(Auditor) { done }",
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
            "requires:HasRole(\"Admin\")",
            "requires:AnyRole(RolePolicy { roles: [\"Reviewer\", \"Approver\"] })",
            "ensures:OpenPostcondition { predicate: ResultSatisfies(Gt(0)) }",
            "requires:HasRole(\"Auditor\")",
        ],
        "header requires/ensures events must enter the shared projection path in source order"
    );

    assert!(matches!(
        &lowered.contract.admission.requirements[0],
        Requirement::HasRole(role) if role == "Admin"
    ));
    assert!(matches!(
        &lowered.contract.admission.requirements[1],
        Requirement::AnyRole(policy) if policy.roles == ["Reviewer", "Approver"]
    ));
    assert_eq!(
        lowered.contract.admission.requirements[2],
        Requirement::HasRole("Auditor".to_string())
    );
}

#[test]
fn legacy_capability_resource_and_uses_headers_lower_in_source_order() {
    let workflow = parse_workflow(
        r#"workflow authority_flow plays role(Admin) capabilities: [filesystem @ { paths: ["/tmp/*"], read: true }, network] owns cache: CacheResource uses store: Store = StoreImpl(cache) requires: role(Auditor) { done }"#,
    );

    let form = legacy_workflow_def_to_workflow_form(&workflow).expect("legacy adapter succeeds");
    let lowered = lower_workflow_form(&form, legacy_workflow_source_origin(&workflow));

    let authority_and_contract_events: Vec<_> = lowered
        .projection_events
        .iter()
        .filter_map(|event| match &event.kind {
            ProjectionEventKind::Requires { requirement } => {
                Some(format!("requires:{requirement:?}"))
            }
            ProjectionEventKind::Authority { authority } => {
                Some(format!("authority:{authority:?}"))
            }
            _ => None,
        })
        .collect();

    assert_eq!(
        authority_and_contract_events,
        vec![
            "requires:HasRole(\"Admin\")",
            "authority:RequiredCapability(WorkflowRequiredCapability { capability: \"filesystem\", constraints: [(\"paths\", Array([String(\"/tmp/*\")])), (\"read\", Bool(true))] })",
            "authority:RequiredCapability(WorkflowRequiredCapability { capability: \"network\", constraints: [] })",
            "authority:OwnedResource(WorkflowOwnedResourceSummary { name: \"cache\", ty: \"CacheResource\" })",
            "authority:UsedBinding(WorkflowUsedBindingSummary { name: \"store\", interface: \"Store\", implementation: \"StoreImpl(cache)\" })",
            "requires:HasRole(\"Auditor\")",
        ],
        "legacy authority headers must enter the shared projection path in source order instead of being skipped"
    );

    assert!(lowered.coverage.authority.iter().any(|key| key.node.0 > 0));
    assert!(lowered.coverage.resources.iter().any(|key| key.node.0 > 0));
    assert!(lowered.coverage.obligations.iter().any(|obligation| {
        matches!(
            obligation,
            ash_core::workflow_carrier::WorkflowObligation::RequiredCapabilityCovered {
                capability,
                mode,
                ..
            } if capability == "filesystem" && mode == "required capability"
        )
    }));
    assert!(lowered.coverage.obligations.iter().any(|obligation| {
        matches!(
            obligation,
            ash_core::workflow_carrier::WorkflowObligation::ResourceAvailable {
                resource,
                access_mode,
                ..
            } if resource == "cache" && access_mode == "owned resource"
        )
    }));
    assert!(lowered.coverage.obligations.iter().any(|obligation| {
        matches!(
            obligation,
            ash_core::workflow_carrier::WorkflowObligation::CapabilityBindingAvailable {
                binding,
                interface,
                ..
            } if binding == "store" && interface == "Store"
        )
    }));
}

#[test]
fn legacy_empty_capabilities_header_does_not_fabricate_authority() {
    let workflow =
        parse_workflow("workflow empty_caps capabilities: [] requires: role(Auditor) { done }");

    let form = legacy_workflow_def_to_workflow_form(&workflow).expect("legacy adapter succeeds");
    let lowered = lower_workflow_form(&form, legacy_workflow_source_origin(&workflow));

    let events: Vec<_> = lowered
        .projection_events
        .iter()
        .filter_map(|event| match &event.kind {
            ProjectionEventKind::Authority { authority } => {
                Some(format!("authority:{authority:?}"))
            }
            ProjectionEventKind::Requires { requirement } => {
                Some(format!("requires:{requirement:?}"))
            }
            _ => None,
        })
        .collect();

    assert_eq!(events, vec!["requires:HasRole(\"Auditor\")"]);
}

#[test]
fn legacy_ensures_targets_successful_workflow_result_and_body_enters_from_proc() {
    let workflow = parse_workflow("workflow positive ensures: result >= 1 { done }");

    let form = legacy_workflow_def_to_workflow_form(&workflow).expect("legacy adapter succeeds");
    let lowered = lower_workflow_form(&form, legacy_workflow_source_origin(&workflow));

    assert!(
        has_workflow_result_ensures(&lowered.contract.plan),
        "legacy ensures must target the successful workflow result, not a delayed/ambient value"
    );

    assert!(
        contains_body_placeholder(&form),
        "this slice should expose the legacy body FromProc summary honestly"
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
#[allow(clippy::too_many_lines)]
fn legacy_body_summary_carries_explicit_non_conservative_full_summary_fields_for_supported_subset()
{
    let mut workflow = parse_workflow(
        "workflow full_summary capabilities: [clock] owns cache: CacheResource requires: role(Auditor) { done }",
    );
    let span = Span::default();
    workflow.body = Workflow::Seq {
        first: Box::new(Workflow::Let {
            pattern: Pattern::Variable {
                name: Name::from("x"),
                span,
            },
            expr: Expr::Literal(Literal::Int(1)),
            continuation: None,
            span,
        }),
        second: Box::new(Workflow::Seq {
            first: Box::new(Workflow::Check {
                target: CheckTarget::Obligation(ObligationRef {
                    role: Name::from("audit"),
                    condition: Expr::Literal(Literal::Bool(true)),
                }),
                continuation: None,
                span,
            }),
            second: Box::new(Workflow::Oblige {
                obligation: Name::from("audit"),
                span,
            }),
            span,
        }),
        span,
    };

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
        Some(false),
        "supported legacy FromProc summaries must carry a complete failure summary for the supported subset"
    );
    assert_eq!(
        summary
            .resource_authority_summary
            .as_ref()
            .map(|summary| summary.conservative),
        Some(false),
        "supported legacy FromProc summaries must carry a complete resource-authority summary for the supported subset"
    );
    assert_eq!(
        summary
            .provenance_summary
            .as_ref()
            .map(|summary| summary.conservative),
        Some(false),
        "supported legacy FromProc summaries must carry a complete provenance summary for the supported subset"
    );
    let resource_summary = summary
        .resource_authority_summary
        .as_ref()
        .expect("resource summary exists");
    assert!(
        resource_summary
            .resources
            .iter()
            .any(|name| name == "cache")
    );
    assert!(
        resource_summary
            .resources
            .iter()
            .any(|name| name == "capability:clock")
    );
    let failure_summary = summary
        .failure_summary
        .as_ref()
        .expect("failure summary exists");
    for route in ["check.failure", "obligation.failure"] {
        assert!(
            failure_summary.routes.iter().any(|actual| actual == route),
            "supported legacy body summaries must include {route}"
        );
    }
    let provenance_summary = summary
        .provenance_summary
        .as_ref()
        .expect("provenance summary exists");
    for kind in ["let", "check", "oblige"] {
        assert!(
            provenance_summary
                .event_kinds
                .iter()
                .any(|actual| actual == kind),
            "supported legacy body summaries must include {kind} provenance"
        );
    }
    assert!(
        matches!(summary.source_origin, Some(SourceOrigin::Synthetic { .. })),
        "supported legacy FromProc summaries must carry explicit source-origin metadata"
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn legacy_supported_body_summary_accounts_for_construct_specific_failure_and_provenance() {
    let span = Span::default();
    let mut workflow = parse_workflow("workflow construct_summary { done }");
    workflow.body = Workflow::Seq {
        first: Box::new(Workflow::Check {
            target: CheckTarget::Obligation(ObligationRef {
                role: Name::from("audit"),
                condition: Expr::Literal(Literal::Bool(true)),
            }),
            continuation: None,
            span,
        }),
        second: Box::new(Workflow::Seq {
            first: Box::new(Workflow::Oblige {
                obligation: Name::from("audit"),
                span,
            }),
            second: Box::new(Workflow::Seq {
                first: Box::new(Workflow::Must {
                    body: Box::new(Workflow::Done { span }),
                    span,
                }),
                second: Box::new(Workflow::Seq {
                    first: Box::new(Workflow::For {
                        pattern: Pattern::Variable {
                            name: Name::from("item"),
                            span,
                        },
                        collection: Expr::Variable {
                            name: Name::from("items"),
                            span,
                        },
                        body: Box::new(Workflow::Done { span }),
                        span,
                    }),
                    second: Box::new(Workflow::With {
                        capability: Name::from("clock"),
                        body: Box::new(Workflow::Act {
                            action: ActionRef {
                                target: OperationalTarget::Symbolic {
                                    capability_name: Name::from("tick"),
                                },
                                args: Vec::new(),
                            },
                            guard: None,
                            result_name: None,
                            continuation: None,
                            span,
                        }),
                        span,
                    }),
                    span,
                }),
                span,
            }),
            span,
        }),
        span,
    };

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

    let resource_summary = summary
        .resource_authority_summary
        .as_ref()
        .expect("resource summary exists");
    for resource in ["body:with:clock", "body:act:tick"] {
        assert!(
            resource_summary
                .resources
                .iter()
                .any(|actual| actual == resource),
            "supported legacy body summaries must include {resource} authority"
        );
    }
    let failure_summary = summary
        .failure_summary
        .as_ref()
        .expect("failure summary exists");
    for route in ["check.failure", "obligation.failure", "must.enforced"] {
        assert!(
            failure_summary.routes.iter().any(|actual| actual == route),
            "supported legacy body summaries must include {route}"
        );
    }
    let provenance_summary = summary
        .provenance_summary
        .as_ref()
        .expect("provenance summary exists");
    for kind in ["check", "oblige", "must", "for", "with", "act"] {
        assert!(
            provenance_summary
                .event_kinds
                .iter()
                .any(|actual| actual == kind),
            "supported legacy body summaries must include {kind} provenance"
        );
    }
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
