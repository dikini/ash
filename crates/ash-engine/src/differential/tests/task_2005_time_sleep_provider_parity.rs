//! TASK-2005/TASK-439 regression coverage for the fixed time-sleep provider pair.
//!
//! The corpus metadata admits exactly one standard-profile direct route and one
//! private checked-CPS provider-frame discharge.  It is deliberately not a
//! generic provider, operation, handler, or production-CPS fixture protocol.

use super::{
    CaseComparisonStatus, DifferentialHarness, ObservableDimension, ParityDisposition,
    RelationStatus, RustExecutionTarget,
};
use crate::Engine;
use ash_core::cps::{Atom, EffectItemKind, Term};
use serde_json::json;
use std::path::PathBuf;

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/differential/corpus")
}

#[test]
fn standard_time_profile_and_private_provider_frame_discharge_only_time_sleep_to_null() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Null { time::sleep(0) }")
        .expect("time::sleep fixture parses");
    engine
        .check(&mut entry)
        .expect("time::sleep fixture checks");
    let Term::Raise { op, args, .. } = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("time::sleep has a private checked CPS lowering")
    else {
        panic!("time::sleep must lower to a private CPS Raise");
    };
    assert_eq!(op.item.namespace, "time");
    assert_eq!(op.item.name, "sleep");
    assert_eq!(op.item.kind, EffectItemKind::Capability);
    assert_eq!(op.arg_types, ["Int"]);
    assert_eq!(op.result_type, "Null");
    assert_eq!(args, vec![Atom::Int(0)]);

    let harness = DifferentialHarness::load(corpus_root()).expect("corpus should load");
    let report = harness.run_case(
        "phase202-time-sleep-provider-discharge",
        RustExecutionTarget::DirectRuntime,
    );

    assert!(
        report
            .canonical_rule_ids()
            .contains("SEM-EFFECT-LOOKUP-001")
    );
    assert!(report.canonical_rule_ids().contains("SEM-EFFECT-RAISE-001"));
    assert_eq!(
        report.direct_runtime_status(),
        CaseComparisonStatus::Passed,
        "the standard profile must admit and execute only time::sleep(0)"
    );
    assert_eq!(
        report.actual_result(),
        Some(&json!({
            "outcome_class": "return",
            "payload": {"kind": "value", "value": {"type": "null"}},
        })),
        "the direct standard-profile operation must return Null"
    );
    assert_eq!(
        report.checked_core_cps_relation(),
        RelationStatus::Passed,
        "the private checked CPS Raise(time::sleep, Int -> Null) must be discharged by its sole provider frame"
    );
    assert!(matches!(
        report
            .parity_report()
            .disposition_for(ObservableDimension::AllowedExternalOutcomes),
        Some(ParityDisposition::Compared {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-EFFECT-LOOKUP-001" && owner == "TASK-2005"
    ));
    assert!(matches!(
        report
            .parity_report()
            .disposition_for(ObservableDimension::FrameOrdering),
        Some(ParityDisposition::Unsupported {
            canonical_rule_id,
            owner,
        }) if canonical_rule_id == "SEM-EFFECT-HANDLE-001" && owner == "TASK-2005"
    ));
}
