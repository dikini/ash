use ash_core::core_ash::CoreSourceSpan;
use ash_core::core_ash_contract::{
    ContractDischargeRecord, ContractEvidenceRef, MonitorEvaluationResult, RuntimeMonitorEvidence,
    TemporalContractDiagnostic, TemporalFormula, TraceAlphabet, TraceContract, TraceFactKind,
    TraceInterpretation,
};

fn span() -> CoreSourceSpan {
    CoreSourceSpan {
        file: Some("task_1900.ash".into()),
        start: 1,
        end: 2,
    }
}

fn monitor_evidence() -> RuntimeMonitorEvidence {
    RuntimeMonitorEvidence::new(
        "monitor:no-race:channel",
        "no-race",
        "fn:process:ensures",
        MonitorEvaluationResult::Satisfied,
    )
}

#[test]
fn runtime_monitor_evidence_construction_and_accessors() {
    let evidence = monitor_evidence();
    assert_eq!(evidence.monitor_ref().as_str(), "monitor:no-race:channel");
    assert_eq!(evidence.contract_ref().as_str(), "no-race");
    assert_eq!(evidence.boundary().as_str(), "fn:process:ensures");
    assert!(matches!(
        evidence.outcome(),
        MonitorEvaluationResult::Satisfied
    ));
    assert!(evidence.redacted());

    let unredacted = evidence.with_redacted(false);
    assert!(!unredacted.redacted());
}

#[test]
fn contract_discharge_record_carries_monitor_evidence() {
    let evidence = monitor_evidence();
    let record = ContractDischargeRecord::static_proven(
        "no-race",
        "fn:process:ensures",
        ContractEvidenceRef::new("proof:no-race"),
        span(),
        None,
    )
    .with_monitor_evidence(vec![evidence.clone()]);

    assert_eq!(record.monitor_evidence().len(), 1);
    assert_eq!(record.monitor_evidence()[0], evidence);
}

#[test]
fn runtime_monitor_evidence_serializes_and_deserializes() {
    let evidence = monitor_evidence();
    let json = serde_json::to_string(&evidence).expect("serialize");
    let roundtrip: RuntimeMonitorEvidence = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(evidence, roundtrip);
}

#[test]
fn monitor_evidence_does_not_grant_predicate_authority() {
    let evidence = monitor_evidence();
    // Authority is a capability of ObservationEvidence; RuntimeMonitorEvidence
    // is a plain record and is never a row admission discharge.
    assert!(evidence.redacted());
    assert!(!matches!(
        evidence.outcome(),
        MonitorEvaluationResult::Violated(_)
    ));
    // A monitor evidence row is not an admission discharge: it carries no
    // grants_predicate_authority semantics, so it cannot discharge operation,
    // resource, role, or policy rows.
}

#[test]
fn monitor_evidence_redacted_appears_in_diagnostic() {
    let contract = TraceContract::new(
        "no-race",
        TraceAlphabet::new(vec![TraceFactKind::Channel, TraceFactKind::Resource]),
        TemporalFormula::Always(TraceFactKind::Channel),
        ash_core::core_ash_contract::TraceContractDischarge::RuntimeMonitor {
            plan: "monitor:no-race:channel".to_string(),
        },
    );
    let outcome = MonitorEvaluationResult::Violated(TemporalContractDiagnostic::new(
        "no-race",
        contract.formula().clone(),
        TraceInterpretation::Normative,
    ));
    let evidence = RuntimeMonitorEvidence::new(
        "monitor:no-race:channel",
        "no-race",
        "fn:process:ensures",
        outcome,
    );
    assert!(evidence.redacted());
    if let MonitorEvaluationResult::Violated(diag) = evidence.outcome() {
        assert_eq!(diag.contract_ref(), "no-race");
    } else {
        panic!("expected violated outcome");
    }
}
