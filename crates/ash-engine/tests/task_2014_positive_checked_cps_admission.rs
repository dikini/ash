//! TASK-2014 RED contract for the first positive Path-B admission.
//!
//! A handler-free literal entry needs its own provenance-linked production
//! admission artifact: the current V1 artifact deliberately selects checked
//! handler/application facts and therefore cannot represent this case.

use ash_core::Value;
use ash_engine::{Engine, ProductionExecutionBoundary};

#[tokio::test]
async fn literal_entry_admits_a_provenance_linked_checked_cps_artifact_and_runs_through_it() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Int { 42 }")
        .expect("literal entry parses");

    let admission = engine
        .admit_entry_to_checked_cps(&mut entry)
        .expect("a supported literal entry receives a sealed checked Core/CPS admission artifact");

    assert_eq!(
        admission.source_anchor(),
        &entry.lowering_sidecars.entry_body_origin,
        "the production admission artifact must retain the checked entry's source provenance"
    );
    assert_eq!(
        engine.production_execution_boundary(),
        ProductionExecutionBoundary::CheckedCoreCpsClosedAdmission,
        "the positive path must use the selected checked Core/CPS owner rather than reopening the legacy evaluator"
    );

    let value = engine
        .execute_checked_cps_admission(&admission)
        .await
        .expect("the sealed artifact executes through the checked CPS owner");
    assert_eq!(value, Value::Int(42));
}
