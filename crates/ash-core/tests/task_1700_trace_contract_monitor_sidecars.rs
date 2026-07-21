use ash_core::core_ash_contract::{
    ApplicationLedgerFact, MonitorPlan, MonitorScope, TemporalFormula, TraceAlphabet,
    TraceContract, TraceContractDischarge, TraceFactKind, TraceInterpretation,
};

#[test]
fn classifies_operational_normative_and_mixed_trace_alphabets() {
    assert_eq!(
        TraceAlphabet::new(vec![TraceFactKind::Process]).interpretation(),
        TraceInterpretation::Operational
    );
    assert_eq!(
        TraceAlphabet::new(vec![TraceFactKind::Application]).interpretation(),
        TraceInterpretation::Normative
    );
    assert_eq!(
        TraceAlphabet::new(vec![TraceFactKind::Process, TraceFactKind::Application])
            .interpretation(),
        TraceInterpretation::Mixed
    );
}

#[test]
fn trace_contract_is_separate_from_value_predicate_artifacts() {
    let alphabet = TraceAlphabet::new(vec![TraceFactKind::Process, TraceFactKind::Application]);
    let monitor = MonitorPlan::new(
        "monitor:commit-after-approve",
        MonitorScope::new(alphabet.clone()),
    );
    let contract = TraceContract::new(
        "trace:commit-after-approve",
        alphabet,
        TemporalFormula::EventuallyAfter {
            after: TraceFactKind::Application,
            event: TraceFactKind::Process,
        },
        TraceContractDischarge::RuntimeMonitor {
            plan: monitor.monitor_ref().to_owned(),
        },
    );

    assert_eq!(contract.interpretation(), TraceInterpretation::Mixed);
    assert!(
        contract.predicate_ref().is_none(),
        "trace contracts must not become value-level LoweredPredicate refs"
    );
}

#[test]
fn monitor_scope_rejects_facts_outside_alphabet() {
    let scope = MonitorScope::new(TraceAlphabet::new(vec![TraceFactKind::Application]));

    assert!(scope.accepts(&TraceFactKind::Application));
    assert!(!scope.accepts(&TraceFactKind::Process));
}

#[test]
fn application_ledger_fact_preserves_source_trace_link() {
    let fact = ApplicationLedgerFact::new("wf:approved", "trace:42");

    assert_eq!(fact.source_trace_ref(), "trace:42");
}
