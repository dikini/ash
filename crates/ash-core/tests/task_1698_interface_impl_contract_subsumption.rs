use ash_core::core_ash_contract::{
    ContractClauseSummary, ContractSubsumptionError, ContractSummary, CoreBlameParty,
    CoreBlamePolarity, PredicateEntailment, check_contract_subsumption,
};

fn summary(name: &str, requires: &str, ensures: &str) -> ContractSummary {
    ContractSummary::new(
        name,
        ContractClauseSummary::requires(requires),
        ContractClauseSummary::ensures(ensures),
    )
}

#[test]
fn accepts_impl_that_weakens_precondition_and_strengthens_postcondition() {
    let interface = summary("Stack::pop", "not_empty", "valid_result");
    let impl_contract = summary("ArrayStack::pop", "true", "valid_result_and_cached");
    let entailments = vec![
        PredicateEntailment::new("not_empty", "true"),
        PredicateEntailment::new("valid_result_and_cached", "valid_result"),
    ];

    let proof = check_contract_subsumption(&interface, &impl_contract, &entailments)
        .expect("weaker precondition and stronger postcondition should subsume");

    assert_eq!(proof.precondition_obligation().antecedent(), "not_empty");
    assert_eq!(proof.precondition_obligation().consequent(), "true");
    assert_eq!(
        proof.postcondition_obligation().antecedent(),
        "valid_result_and_cached"
    );
    assert_eq!(
        proof.postcondition_obligation().consequent(),
        "valid_result"
    );
}

#[test]
fn rejects_impl_that_strengthens_precondition() {
    let interface = summary("Stack::pop", "not_empty", "valid_result");
    let impl_contract = summary("ArrayStack::pop", "not_empty_and_valid", "valid_result");

    let error = check_contract_subsumption(&interface, &impl_contract, &[])
        .expect_err("strengthened impl precondition should be rejected");

    assert!(matches!(
        error,
        ContractSubsumptionError::PreconditionNotWeakened { .. }
    ));
}

#[test]
fn rejects_impl_that_weakens_postcondition() {
    let interface = summary("Stack::pop", "not_empty", "sorted_result");
    let impl_contract = summary("ArrayStack::pop", "not_empty", "some_result");
    let entailments = vec![PredicateEntailment::new("not_empty", "not_empty")];

    let error = check_contract_subsumption(&interface, &impl_contract, &entailments)
        .expect_err("weakened impl postcondition should be rejected");

    assert!(matches!(
        error,
        ContractSubsumptionError::PostconditionNotStrengthened { .. }
    ));
}

#[test]
fn clause_summaries_carry_blame_polarity() {
    let requires = ContractClauseSummary::requires("not_empty");
    let ensures = ContractClauseSummary::ensures("valid_result");

    assert_eq!(requires.blame().party, CoreBlameParty::Caller);
    assert_eq!(requires.blame().polarity, CoreBlamePolarity::Negative);
    assert_eq!(ensures.blame().party, CoreBlameParty::Callee);
    assert_eq!(ensures.blame().polarity, CoreBlamePolarity::Positive);
}
