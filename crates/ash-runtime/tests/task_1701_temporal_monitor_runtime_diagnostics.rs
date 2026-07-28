use ash_core::core_ash_contract::{
    MonitorEvaluationResult, MonitorPlan, MonitorScope, TemporalFormula, TraceAlphabet,
    TraceContract, TraceContractDischarge, TraceFactKind, evaluate_temporal_monitor,
};

#[test]
fn temporal_monitor_runtime_diagnostics_are_available_to_interp_boundary() {
    let alphabet = TraceAlphabet::new(vec![TraceFactKind::Application, TraceFactKind::Process]);
    let plan = MonitorPlan::new(
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
            plan: plan.monitor_ref().to_owned(),
        },
    );

    let violation = evaluate_temporal_monitor(&contract, &plan, &[TraceFactKind::Application]);
    assert!(matches!(violation, MonitorEvaluationResult::Violated(_)));

    let fault = evaluate_temporal_monitor(&contract, &plan, &[TraceFactKind::Time]);
    assert!(matches!(fault, MonitorEvaluationResult::Faulted(_)));
}
