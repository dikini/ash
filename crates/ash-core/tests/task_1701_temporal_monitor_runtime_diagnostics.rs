use ash_core::core_ash::CoreTrapReason;
use ash_core::core_ash_contract::{
    MonitorAuthorityEnv, MonitorEvaluationResult, MonitorFault, MonitorPlan, MonitorScope,
    TemporalContractDiagnostic, TemporalFormula, TemporalMonitorFaultDiagnostic, TraceAlphabet,
    TraceContract, TraceContractDischarge, TraceFactKind, TraceInterpretation,
    evaluate_temporal_monitor,
};

#[test]
fn monitor_result_states_are_explicit() {
    assert!(matches!(
        MonitorEvaluationResult::Satisfied,
        MonitorEvaluationResult::Satisfied
    ));
    assert!(matches!(
        MonitorEvaluationResult::Violated(TemporalContractDiagnostic::new(
            "trace:1",
            TemporalFormula::Eventually(TraceFactKind::Process),
            TraceInterpretation::Operational
        )),
        MonitorEvaluationResult::Violated(_)
    ));
    assert!(matches!(
        MonitorEvaluationResult::Pending,
        MonitorEvaluationResult::Pending
    ));
    assert!(matches!(
        MonitorEvaluationResult::Inconclusive("bounded window ended".into()),
        MonitorEvaluationResult::Inconclusive(_)
    ));
    assert!(matches!(
        MonitorEvaluationResult::Faulted(TemporalMonitorFaultDiagnostic::new(
            "trace:1",
            MonitorFault::OutOfScopeFact
        )),
        MonitorEvaluationResult::Faulted(_)
    ));
}

#[test]
fn temporal_violation_and_monitor_fault_have_distinct_trap_payloads() {
    let violation = TemporalContractDiagnostic::new(
        "trace:commit-after-approve",
        TemporalFormula::EventuallyAfter {
            after: TraceFactKind::Application,
            event: TraceFactKind::Process,
        },
        TraceInterpretation::Mixed,
    );
    let fault = TemporalMonitorFaultDiagnostic::new(
        "trace:commit-after-approve",
        MonitorFault::OutOfScopeFact,
    );

    let violation_trap = CoreTrapReason::TemporalContractViolation(violation.clone());
    let fault_trap = CoreTrapReason::TemporalMonitorFault(fault.clone());

    assert_ne!(violation_trap, fault_trap);
    assert!(
        matches!(violation_trap, CoreTrapReason::TemporalContractViolation(diag) if diag.contract_ref() == violation.contract_ref())
    );
    assert!(
        matches!(fault_trap, CoreTrapReason::TemporalMonitorFault(diag) if diag.fault() == fault.fault())
    );
}

#[test]
fn monitor_authority_env_consumes_recorded_facts_only() {
    let env = MonitorAuthorityEnv::recorded_facts_only(vec![TraceFactKind::Application]);

    assert!(env.can_consume(&TraceFactKind::Application));
    assert!(!env.can_consume(&TraceFactKind::Process));
    assert!(!env.has_provider_authority());
}

#[test]
fn runtime_monitor_evaluates_recorded_facts_to_satisfied_violation_or_fault() {
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

    assert_eq!(
        evaluate_temporal_monitor(
            &contract,
            &plan,
            &[TraceFactKind::Application, TraceFactKind::Process]
        ),
        MonitorEvaluationResult::Satisfied
    );
    assert!(matches!(
        evaluate_temporal_monitor(&contract, &plan, &[TraceFactKind::Application]),
        MonitorEvaluationResult::Violated(_)
    ));
    assert!(matches!(
        evaluate_temporal_monitor(&contract, &plan, &[TraceFactKind::Time]),
        MonitorEvaluationResult::Faulted(_)
    ));
}
