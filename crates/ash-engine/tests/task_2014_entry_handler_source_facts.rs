//! TASK-2014 RED contracts for provenance-linked checked handler source facts.
//!
//! This is an Engine-entry evidence boundary only. It must retain source facts
//! from the same checked entry and must not construct Core/CPS, frames, or
//! runtime authority.

use ash_engine::Engine;

const ABSORB_SLEEP_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
handler absorb_sleep(comp: () -> { TestClock::sleep } Null) -> Null {
    on comp {
        TestClock::sleep(milliseconds, resume) => null,
        done(value) => value,
    }
}
fn main() -> Null { handle TestClock::sleep(0) with absorb_sleep }
";

#[test]
fn checked_entry_projects_handler_facts_with_its_exact_entry_body_anchor() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(ABSORB_SLEEP_SOURCE)
        .expect("absorb_sleep source parses");
    let expected_anchor = entry.lowering_sidecars.entry_body_origin.clone();

    engine
        .check(&mut entry)
        .expect("absorb_sleep source checks before source-fact projection");
    let facts = engine
        .checked_source_facts_for_handler(&entry, "absorb_sleep")
        .expect("a checked entry projects its checked handler facts");

    assert_eq!(facts.handler_name(), "absorb_sleep");
    assert_eq!(facts.source_anchors(), &[expected_anchor]);
    assert_eq!(facts.handler_clauses().len(), 1);
}

#[test]
fn unchecked_entry_cannot_project_handler_source_facts() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = engine
        .parse(ABSORB_SLEEP_SOURCE)
        .expect("absorb_sleep source parses");

    let error = engine
        .checked_source_facts_for_handler(&entry, "absorb_sleep")
        .expect_err("source facts require the same entry to have passed Engine::check");

    assert!(
        error
            .to_string()
            .contains("source facts require Engine::check"),
        "unchecked-entry rejection must identify the check boundary: {error}"
    );
}

#[test]
fn checked_facts_cannot_be_projected_for_a_same_id_entry_from_another_engine() {
    let engine_a = Engine::new().build().expect("first engine builds");
    let mut entry_a = engine_a
        .parse(ABSORB_SLEEP_SOURCE)
        .expect("first absorb_sleep source parses");
    engine_a
        .check(&mut entry_a)
        .expect("first absorb_sleep source checks");

    let engine_b = Engine::new().build().expect("second engine builds");
    let entry_b = engine_b
        .parse(ABSORB_SLEEP_SOURCE)
        .expect("second absorb_sleep source parses with its own entry identity");

    let error = engine_a
        .checked_source_facts_for_handler(&entry_b, "absorb_sleep")
        .expect_err("Engine A must not project checked facts onto Engine B's same-ID entry");
    let message = error.to_string().to_lowercase();

    assert!(
        message.contains("source facts")
            && (message.contains("engine::check")
                || message.contains("provenance")
                || message.contains("entry")),
        "cross-engine rejection must identify the source-fact provenance boundary: {error}"
    );
}

#[test]
fn checked_facts_reject_a_mutated_entry_body_anchor() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(ABSORB_SLEEP_SOURCE)
        .expect("absorb_sleep source parses");
    engine
        .check(&mut entry)
        .expect("absorb_sleep source checks");
    entry.lowering_sidecars.entry_body_origin.label = "forged handler entry anchor".to_string();

    let error = engine
        .checked_source_facts_for_handler(&entry, "absorb_sleep")
        .expect_err("source-fact projection must reject a mutable anchor that differs from check");
    let message = error.to_string().to_lowercase();

    assert!(
        message.contains("source facts")
            && (message.contains("anchor") || message.contains("provenance")),
        "mutated-anchor rejection must identify the source-fact provenance boundary: {error}"
    );
}

#[tokio::test]
async fn handler_source_facts_do_not_admit_placeholder_runtime_execution() {
    let engine = Engine::new().build().expect("engine builds");

    let error = engine.run(ABSORB_SLEEP_SOURCE).await.expect_err(
        "handler source without typed Core/CPS admission must not execute a placeholder",
    );
    let message = error.to_string();

    assert!(
        message.contains("checked Core/CPS")
            && (message.contains("admission") || message.contains("typed lowering")),
        "handler execution must fail at the checked Core/CPS boundary, got: {message}"
    );
}
