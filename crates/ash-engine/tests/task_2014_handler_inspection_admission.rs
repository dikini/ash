//! TASK-2014 RED contract for provenance-linked handler inspection admission.
//!
//! This is an inspection-only artifact: it validates typed Core/CPS evidence
//! and explicit frame-installation instructions, but must not execute a
//! handler, construct a frame, select a provider, or start an async operation.

use ash_core::Value;
use ash_engine::{
    CheckedHandlerInspectionAdmission, Engine,
    checked_cps_admission::{CheckedCpsAdmissionV1, FrameInstallationInstructionV1},
};

const ECHO_SLEEP_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler echo_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => ms,
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(0) with echo_sleep }
";

fn requires_opaque_issued_admission<F, Output>(_executor: F)
where
    F: Fn(&Engine, &CheckedHandlerInspectionAdmission) -> Output,
{
}

#[test]
fn checked_handler_entry_admits_a_root_handle_with_explicit_source_handler_authority() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(ECHO_SLEEP_SOURCE)
        .expect("narrow handler fixture parses");
    let expected_anchor = entry.lowering_sidecars.entry_body_origin.clone();
    engine
        .check(&mut entry)
        .expect("narrow handler fixture checks before inspection admission");

    let admission = engine
        .admit_checked_handler_inspection(&entry, "echo_sleep")
        .expect("same checked entry admits a validated handler inspection artifact");

    assert_eq!(admission.source_anchors(), &[expected_anchor]);
    let [
        FrameInstallationInstructionV1::SourceHandler {
            operation,
            handler_name,
            core_handle,
        },
    ] = admission.frame_installations()
    else {
        panic!("inspection admission must retain exactly one explicit SourceHandler instruction");
    };
    assert_eq!(handler_name, "echo_sleep");
    assert_eq!(operation, &admission.operation_identities()[0]);
    assert!(
        core_handle.path().is_empty(),
        "the selected narrow Core Handle must be the root admission locator"
    );
}

#[test]
fn unchecked_entry_cannot_admit_handler_inspection() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = engine
        .parse(ECHO_SLEEP_SOURCE)
        .expect("narrow handler fixture parses");

    let error = engine
        .admit_checked_handler_inspection(&entry, "echo_sleep")
        .expect_err("handler inspection admission requires the existing checked-entry provenance");
    assert!(
        error
            .to_string()
            .contains("source facts require Engine::check"),
        "unchecked admission must identify the retained check boundary: {error}"
    );
}

#[test]
fn foreign_entry_cannot_admit_handler_inspection_from_another_engine() {
    let engine_a = Engine::new().build().expect("first engine builds");
    let engine_b = Engine::new().build().expect("second engine builds");
    let mut entry_b = engine_b
        .parse(ECHO_SLEEP_SOURCE)
        .expect("second-engine fixture parses");
    engine_b
        .check(&mut entry_b)
        .expect("second-engine fixture checks");

    let error = engine_a
        .admit_checked_handler_inspection(&entry_b, "echo_sleep")
        .expect_err("the admitting Engine must own the checked Entry provenance");
    assert!(
        error.to_string().to_lowercase().contains("provenance"),
        "foreign-entry admission must identify the provenance boundary: {error}"
    );
}

#[test]
fn mutated_entry_anchor_cannot_admit_handler_inspection() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(ECHO_SLEEP_SOURCE)
        .expect("narrow handler fixture parses");
    engine
        .check(&mut entry)
        .expect("fixture checks before its anchor is forged");
    entry.lowering_sidecars.entry_body_origin.label = "forged inspection anchor".to_string();

    let error = engine
        .admit_checked_handler_inspection(&entry, "echo_sleep")
        .expect_err("inspection admission must retain the exact checked entry anchor");
    assert!(
        error.to_string().to_lowercase().contains("anchor"),
        "mutated-anchor admission must identify the checked-anchor boundary: {error}"
    );
}

#[tokio::test]
async fn sealed_handler_inspection_executes_the_closed_empty_identity_handler_without_provider() {
    let engine = Engine::new()
        .build()
        .expect("engine builds without a provider");
    let mut entry = engine
        .parse(ECHO_SLEEP_SOURCE)
        .expect("closed-empty echo handler fixture parses");
    engine
        .check(&mut entry)
        .expect("closed-empty echo handler fixture checks");
    let admission = engine
        .admit_checked_handler_inspection(&entry, "echo_sleep")
        .expect("the fixture admits a sealed handler inspection artifact");

    let value = engine
        .execute_checked_handler_inspection(&admission)
        .await
        .expect("the explicit inspected handler must return its Int payload without a provider");
    assert_eq!(value, Value::Int(0));
}

#[test]
fn handler_inspection_execution_accepts_only_the_opaque_engine_issued_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(ECHO_SLEEP_SOURCE)
        .expect("closed-empty echo handler fixture parses");
    engine.check(&mut entry).expect("fixture checks");
    let issued = engine
        .admit_checked_handler_inspection(&entry, "echo_sleep")
        .expect("Engine issues the narrow inspection artifact");

    let _reconstructed = CheckedCpsAdmissionV1::validate(
        issued.checked_core().clone(),
        engine
            .checked_source_facts_for_handler(&entry, "echo_sleep")
            .expect("same checked entry projects the public source facts"),
        issued.frame_installations().to_vec(),
    )
    .expect("public V1 validation can reconstruct equivalent evidence");

    // This type-level contract makes a reconstructed public V1 artifact
    // uncallable: the executor has no generic V1 parameter to accept it.
    requires_opaque_issued_admission(Engine::execute_checked_handler_inspection);
}

#[tokio::test]
async fn handler_inspection_execution_rejects_an_admission_issued_by_another_engine() {
    let issuing_engine = Engine::new().build().expect("issuing engine builds");
    let executing_engine = Engine::new().build().expect("executing engine builds");
    let mut entry = issuing_engine
        .parse(ECHO_SLEEP_SOURCE)
        .expect("closed-empty echo handler fixture parses");
    issuing_engine.check(&mut entry).expect("fixture checks");
    let admission = issuing_engine
        .admit_checked_handler_inspection(&entry, "echo_sleep")
        .expect("issuing engine admits the inspection artifact");

    let error = executing_engine
        .execute_checked_handler_inspection(&admission)
        .await
        .expect_err("a distinct Engine must reject foreign execution authority");
    assert!(
        error.to_string().to_lowercase().contains("provenance"),
        "foreign execution rejection must identify Engine-issued provenance: {error}"
    );
}
