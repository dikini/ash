use ash_core::core_ash::{CoreSourceSpan, CoreType};
use ash_core::core_ash_contract::{
    ComposedContract, ContractDischargeRecord, ContractDischargeStatus, ContractEvidenceRef,
    ContractRecoverability, CoreBlameLabel, CoreBlameParty, CoreBlamePolarity, DiagnosticShape,
    DynamicPredicatePlan, LoweredPredicateBuilder, PredicateEnvironment, PredicateNode,
    RuntimeCheckPlan,
};

fn bool_ty() -> CoreType {
    CoreType::Base("Bool".into())
}

fn span() -> CoreSourceSpan {
    CoreSourceSpan {
        file: Some("task_1697.ash".into()),
        start: 1,
        end: 2,
    }
}

fn blame(boundary: &str) -> CoreBlameLabel {
    CoreBlameLabel::new(
        CoreBlameParty::Callee,
        CoreBlamePolarity::Positive,
        boundary,
    )
}

fn runtime_plan(boundary: &str) -> RuntimeCheckPlan {
    let predicate = LoweredPredicateBuilder::new(
        boundary,
        PredicateEnvironment::new(boundary, Vec::new(), Vec::new(), Vec::new()),
        PredicateNode::BoolLit(true),
        bool_ty(),
    )
    .dynamic_plan(DynamicPredicatePlan::Interpreter)
    .build();
    RuntimeCheckPlan::new(
        predicate.predicate_ref().clone(),
        PredicateEnvironment::new(boundary, Vec::new(), Vec::new(), Vec::new()).ref_(),
        DynamicPredicatePlan::Interpreter,
        blame(boundary),
        Vec::new(),
        DiagnosticShape::predicate_false("dynamic-check"),
        ContractRecoverability::TrapDefault,
    )
}

#[test]
fn discharge_records_static_evidence_dynamic_and_deferred_states() {
    let boundary = "fn:push:ensures";
    let evidence = ContractEvidenceRef::new("proof:push:ensures");
    let plan = runtime_plan(boundary);

    let static_record = ContractDischargeRecord::static_proven(
        "sorted-post",
        boundary,
        evidence.clone(),
        span(),
        Some(blame(boundary)),
    );
    let survived_testing = ContractDischargeRecord::evidence_survived_testing(
        "sorted-post",
        boundary,
        evidence.clone(),
        span(),
    );
    let dynamic = ContractDischargeRecord::dynamic(
        "sorted-post",
        boundary,
        plan.clone(),
        span(),
        Some(blame(boundary)),
    );
    let deferred =
        ContractDischargeRecord::deferred("sorted-post", boundary, "solver unavailable", span());

    assert!(matches!(
        static_record.status(),
        ContractDischargeStatus::StaticProven { .. }
    ));
    assert!(matches!(
        survived_testing.status(),
        ContractDischargeStatus::EvidenceSurvivedTesting { .. }
    ));
    assert!(
        matches!(dynamic.status(), ContractDischargeStatus::Dynamic { plan: p } if p.as_ref() == &plan)
    );
    assert!(
        matches!(deferred.status(), ContractDischargeStatus::Deferred { reason } if reason == "solver unavailable")
    );
}

#[test]
fn composed_contract_preserves_bind_obligation_metadata() {
    let boundary = "bind:producer:continuation";
    let evidence = ContractEvidenceRef::new("proof:bind:Q-implies-R");
    let producer =
        ContractDischargeRecord::static_proven("Q(a)", boundary, evidence.clone(), span(), None);
    let continuation = ContractDischargeRecord::deferred(
        "R(a)",
        boundary,
        "requires producer postcondition",
        span(),
    );
    let proof = LoweredPredicateBuilder::new(
        boundary,
        PredicateEnvironment::new(boundary, Vec::new(), Vec::new(), Vec::new()),
        PredicateNode::BoolLit(true),
        bool_ty(),
    )
    .build();

    let composed = ComposedContract::new(
        producer.discharge_ref(),
        continuation.discharge_ref(),
        "a",
        proof.predicate_ref().clone(),
        None,
        Some(evidence.clone()),
        span(),
    );

    assert_eq!(composed.producer_postcondition(), &producer.discharge_ref());
    assert_eq!(
        composed.continuation_precondition(),
        &continuation.discharge_ref()
    );
    assert_eq!(composed.intermediate_binder(), "a");
    assert_eq!(composed.evidence(), Some(&evidence));
}
