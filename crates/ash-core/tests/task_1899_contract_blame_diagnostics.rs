//! Phase 194 TASK-1899 tests for structured contract blame diagnostics.

use ash_core::core_ash::{CoreName, CoreSourceSpan};
use ash_core::core_ash_contract::{
    ContractDiagnostic, CoreBlameLabel, CoreBlameParty, CoreBlamePolarity, CoreBoundaryId,
    EvidenceRef, PredicateClassification, PredicateFault, PredicateFaultDiagnostic, PredicateHash,
    PredicateId, PredicateRef,
};

fn span() -> CoreSourceSpan {
    CoreSourceSpan {
        file: Some("task_1899.ash".into()),
        start: 10,
        end: 20,
    }
}

fn predicate_ref() -> PredicateRef {
    PredicateRef {
        id: PredicateId::new("pred:positive-requires"),
        stable_hash: PredicateHash::new(
            "0000000000000000000000000000000000000000000000000000000000000000",
        ),
        boundary: CoreBoundaryId::new("boundary:fn:inc:requires"),
        source_span: Some(span()),
    }
}

fn blame_label(party: CoreBlameParty, polarity: CoreBlamePolarity) -> CoreBlameLabel {
    CoreBlameLabel::new(party, polarity, CoreBoundaryId::from("fn:inc:requires"))
}

#[test]
fn contract_diagnostic_construction_and_accessors() {
    let diagnostic = ContractDiagnostic::new(
        predicate_ref(),
        "x > 0",
        blame_label(CoreBlameParty::Caller, CoreBlamePolarity::Negative),
        PredicateClassification::Dynamic,
        vec![],
    )
    .with_evidence_refs(vec![EvidenceRef::new("test", "sorted")])
    .with_redacted(true);

    assert_eq!(diagnostic.predicate().id.as_str(), "pred:positive-requires");
    assert_eq!(diagnostic.contract_text(), "x > 0");
    assert_eq!(diagnostic.blame().party, CoreBlameParty::Caller);
    assert_eq!(diagnostic.blame().polarity, CoreBlamePolarity::Negative);
    assert_eq!(
        diagnostic.predicate_classification(),
        PredicateClassification::Dynamic
    );
    assert_eq!(diagnostic.evidence_refs().len(), 1);
    assert_eq!(diagnostic.evidence_refs()[0].family(), "test");
    assert_eq!(diagnostic.evidence_refs()[0].identity(), "sorted");
    assert!(diagnostic.redacted());
}

#[test]
fn predicate_fault_diagnostic_construction_and_accessors() {
    let fault = PredicateFault::MissingBinder(CoreName::from("missing"));
    let diagnostic = PredicateFaultDiagnostic::new(
        predicate_ref(),
        "x > 0",
        blame_label(CoreBlameParty::Callee, CoreBlamePolarity::Positive),
        fault.clone(),
        vec![],
    )
    .with_evidence_refs(vec![EvidenceRef::new("monitor", "no-race")])
    .with_redacted(false);

    assert_eq!(diagnostic.predicate().id.as_str(), "pred:positive-requires");
    assert_eq!(diagnostic.contract_text(), "x > 0");
    assert_eq!(diagnostic.blame().party, CoreBlameParty::Callee);
    assert_eq!(diagnostic.blame().polarity, CoreBlamePolarity::Positive);
    assert_eq!(diagnostic.fault(), &fault);
    assert_eq!(diagnostic.evidence_refs().len(), 1);
    assert_eq!(diagnostic.evidence_refs()[0].family(), "monitor");
    assert!(diagnostic.evidence_refs()[0].identity().contains("no-race"));
    assert!(!diagnostic.redacted());
}

#[test]
fn contract_diagnostic_defaults_redacted_and_empty_evidence() {
    let diagnostic = ContractDiagnostic::new(
        predicate_ref(),
        "true",
        blame_label(CoreBlameParty::Impl, CoreBlamePolarity::Negative),
        PredicateClassification::Static,
        vec![],
    );

    assert!(diagnostic.evidence_refs().is_empty());
    assert!(diagnostic.redacted());
}

#[test]
fn contract_diagnostic_serializes_and_deserializes() {
    let diagnostic = ContractDiagnostic::new(
        predicate_ref(),
        "x > 0",
        blame_label(CoreBlameParty::Caller, CoreBlamePolarity::Negative),
        PredicateClassification::Dynamic,
        vec![],
    )
    .with_evidence_refs(vec![EvidenceRef::new("law", "monotonic.bounds")])
    .with_redacted(true);

    let json = serde_json::to_string(&diagnostic).expect("serialize");
    let roundtrip: ContractDiagnostic = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(diagnostic, roundtrip);
}

#[test]
fn predicate_fault_diagnostic_serializes_and_deserializes() {
    let diagnostic = PredicateFaultDiagnostic::new(
        predicate_ref(),
        "x > 0",
        blame_label(CoreBlameParty::Runtime, CoreBlamePolarity::Positive),
        PredicateFault::EvaluatorTrap("unexpected".into()),
        vec![],
    )
    .with_evidence_refs(vec![EvidenceRef::new("observation", "redacted")]);

    let json = serde_json::to_string(&diagnostic).expect("serialize");
    let roundtrip: PredicateFaultDiagnostic = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(diagnostic, roundtrip);
}

#[test]
fn redaction_flag_is_preserved_by_with_redacted() {
    let diagnostic = ContractDiagnostic::new(
        predicate_ref(),
        "x > 0",
        blame_label(CoreBlameParty::Caller, CoreBlamePolarity::Negative),
        PredicateClassification::Dynamic,
        vec![],
    );

    assert!(diagnostic.redacted());
    let unredacted = diagnostic.with_redacted(false);
    assert!(!unredacted.redacted());
    let re_redacted = unredacted.with_redacted(true);
    assert!(re_redacted.redacted());
}
