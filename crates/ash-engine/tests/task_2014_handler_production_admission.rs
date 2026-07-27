//! TASK-2013/TASK-2014 RED contract for sealed source-handler production admission.
//!
//! The sole admitted handler slice is a closed-empty local `absorb_sleep`
//! computation. Production success must come through checked Core/CPS, while
//! `Engine::execute` remains a closed generic entrypoint.

use ash_core::{Value, ast::Expr};
use ash_engine::{
    CheckedHandlerProductionAdmission, Engine, Entry, ProductionTerminalClassification,
};
use std::{collections::HashMap, fs};

const ABSORB_SLEEP_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler absorb_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => resume(ms),
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(0) with absorb_sleep }
";

// This is intentionally distinct from the returning `absorb_sleep` control:
// its clause receives the affine continuation but aborts without invoking it.
// The fixed division is the first real source-level handler-body trap witness;
// it must not be represented by a forged admission token or direct evaluation.
const TRAP_SLEEP_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler trap_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => 1 / 0,
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(0) with trap_sleep }
";

const DIFFERENT_HANDLER_NAME_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler another_handler(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => resume(ms),
        done(value) => value,
    }
}
fn main() -> Int { handle TestClock::sleep(0) with another_handler }
";

const NONIDENTITY_DONE_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Int }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
handler absorb_sleep(comp: () -> { TestClock::sleep } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => resume(ms),
        done(value) => value + 1,
    }
}
fn main() -> Int { handle TestClock::sleep(0) with absorb_sleep }
";

const NONEMPTY_RESIDUAL_ABSORB_SLEEP_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Int }
interface Audit<T> { record(Int) -> Int }
type TestClock = SystemClock(Int);
type TestAudit = SystemAudit(Int);
impl Clock<TestClock> { sleep(milliseconds) = milliseconds }
impl Audit<TestAudit> { record(value) = value }
handler absorb_sleep(comp: () -> { TestClock::sleep, TestAudit::record } Int) -> Int {
    on comp {
        TestClock::sleep(milliseconds, resume) => resume(milliseconds),
        done(value) => value,
    }
}
fn main() -> Int {
    handle { TestClock::sleep(0); TestAudit::record(0) } with absorb_sleep
}
";

fn checked_entry(engine: &Engine, source: &str) -> Entry {
    let mut entry = engine.parse(source).expect("handler fixture parses");
    engine.check(&mut entry).expect("handler fixture checks");
    entry
}

fn requires_opaque_handler_production_admission<F, Output>(_executor: F)
where
    F: Fn(&Engine, &CheckedHandlerProductionAdmission) -> Output,
{
}

#[tokio::test]
async fn closed_empty_absorb_sleep_runs_only_through_checked_cps_production_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(ABSORB_SLEEP_SOURCE)
        .expect("closed-empty local handler fixture parses");
    engine
        .check(&mut entry)
        .expect("closed-empty local handler fixture checks");

    let direct_error = engine
        .execute(&entry)
        .await
        .expect_err("the generic execute entrypoint remains closed");
    assert!(
        direct_error
            .to_string()
            .contains("no validated production typed lowering is available"),
        "generic direct execution must retain the closed-admission boundary: {direct_error}"
    );

    assert_eq!(
        engine
            .run(ABSORB_SLEEP_SOURCE)
            .await
            .expect("the Engine run route must admit the one sealed handler via checked Core/CPS"),
        Value::Int(0),
    );
}

#[tokio::test]
async fn closed_empty_absorb_sleep_runs_from_a_file_only_through_checked_cps_production_admission()
{
    let engine = Engine::new().build().expect("engine builds");
    let temporary_directory = tempfile::tempdir().expect("temporary source directory");
    let source_path = temporary_directory.path().join("absorb-sleep.ash");
    fs::write(&source_path, ABSORB_SLEEP_SOURCE).expect("handler fixture source writes");

    assert_eq!(
        engine
            .run_file(&source_path)
            .await
            .expect("the Engine file route must admit the sealed handler via checked Core/CPS"),
        Value::Int(0),
    );
}

#[tokio::test]
async fn admitted_abortive_trap_sleep_reports_its_language_trap_after_checked_cps_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(TRAP_SLEEP_SOURCE)
        .expect("the exact abortive trap_sleep fixture parses");
    engine
        .check(&mut entry)
        .expect("the unused affine resume and fixed division type-check");

    let admission = engine
        .admit_production_checked_handler(&mut entry)
        .expect("the exact trap_sleep source must receive a sealed checked-CPS admission");
    let error = engine
        .execute_production_checked_handler(&admission)
        .await
        .expect_err("the admitted abortive clause must trap after admission");
    assert!(
        error
            .to_string()
            .to_lowercase()
            .contains("division by zero"),
        "the post-admission handler-body failure must retain a language division reason: {error}"
    );
}

#[tokio::test]
async fn generic_execute_with_input_stays_closed_for_a_checked_sealed_handler_entry() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = checked_entry(&engine, ABSORB_SLEEP_SOURCE);

    let error = engine
        .execute_with_input(&entry, HashMap::new())
        .await
        .expect_err("the generic input route must not consume a checked handler admission");
    assert_eq!(
        error.to_string(),
        "application execution failed: checked Core/CPS admission rejected: no validated production typed lowering is available",
        "generic input execution must remain outside the sealed handler token route"
    );
}

#[test]
fn unchecked_handler_entry_is_not_production_admission_authority() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(ABSORB_SLEEP_SOURCE)
        .expect("closed-empty local handler fixture parses");

    let error = engine
        .admit_production_checked_handler(&mut entry)
        .expect_err("production admission must require a prior Engine::check");
    assert!(
        error.to_string().contains("Engine::check"),
        "unchecked source entries must reject at the checked-facts boundary: {error}"
    );
}

#[test]
fn unchecked_trap_sleep_preserves_the_normal_engine_check_prerequisite() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(TRAP_SLEEP_SOURCE)
        .expect("the exact trap_sleep source parses before checking");

    let error = engine
        .admit_production_checked_handler(&mut entry)
        .expect_err("an unchecked trap_sleep source must not receive production authority");
    assert!(
        error.to_string().contains("Engine::check"),
        "the ordinary checked-entry prerequisite remains the diagnostic boundary for unchecked trap_sleep: {error}"
    );
}

#[test]
fn production_handler_execution_accepts_only_the_opaque_issued_admission_type() {
    // `CheckedHandlerProductionAdmission` has no public constructor and all
    // fields are private, so external callers can neither alter the sealed
    // CPS term nor manufacture a non-trapping trap_sleep admission. This
    // compile-time contract prevents widening execution to public V1 evidence.
    requires_opaque_handler_production_admission(Engine::execute_production_checked_handler);
}

#[test]
fn checked_handler_entry_from_a_foreign_engine_is_rejected() {
    let issuing_engine = Engine::new().build().expect("issuing engine builds");
    let mut entry = checked_entry(&issuing_engine, TRAP_SLEEP_SOURCE);
    let foreign_engine = Engine::new().build().expect("foreign engine builds");

    let error = foreign_engine
        .admit_production_checked_handler(&mut entry)
        .expect_err("a foreign Engine must not issue trap_sleep production authority");
    assert_eq!(
        error.classification(),
        ProductionTerminalClassification::MissingAdmission,
        "foreign Engine provenance means no local admission token, not malformed checked Core/CPS"
    );
}

#[test]
fn forged_trap_sleep_source_anchor_is_invalid_checked_core_cps_not_missing_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = checked_entry(&engine, TRAP_SLEEP_SOURCE);
    entry.lowering_sidecars.entry_body_origin.label = "forged trap_sleep source anchor".to_string();

    let error = engine
        .admit_production_checked_handler(&mut entry)
        .expect_err("a forged checked trap_sleep source anchor cannot mint a token");
    assert_eq!(
        error.classification(),
        ProductionTerminalClassification::InvalidCheckedCoreCps,
        "tampered checked provenance is an invalid purported checked Core/CPS artifact"
    );
}

#[test]
fn forged_trap_sleep_public_core_is_invalid_checked_core_cps_not_missing_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = checked_entry(&engine, TRAP_SLEEP_SOURCE);
    entry.core = Expr::Literal(Value::Int(99));

    let error = engine
        .admit_production_checked_handler(&mut entry)
        .expect_err("a forged checked trap_sleep public Core cannot mint a token");
    assert_eq!(
        error.classification(),
        ProductionTerminalClassification::InvalidCheckedCoreCps,
        "tampered checked Core is an invalid purported checked Core/CPS artifact"
    );
}

#[test]
fn mutated_handler_body_anchor_is_rejected_before_production_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = checked_entry(&engine, ABSORB_SLEEP_SOURCE);
    entry.lowering_sidecars.entry_body_origin.label = "forged handler body origin".to_string();

    let error = engine
        .admit_production_checked_handler(&mut entry)
        .expect_err("a mutated public body anchor must not become handler authority");
    assert!(
        error
            .to_string()
            .contains("source anchor does not match the canonical parsed entry provenance"),
        "source-anchor mutation must reject before typed handler admission: {error}"
    );
}

#[test]
fn mutated_public_legacy_core_is_rejected_before_handler_production_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = checked_entry(&engine, ABSORB_SLEEP_SOURCE);
    entry.core = Expr::Literal(Value::Int(99));

    let error = engine
        .admit_production_checked_handler(&mut entry)
        .expect_err("a public legacy Core mutation must not become handler authority");
    assert!(
        error
            .to_string()
            .contains("Core does not match the canonical parsed entry provenance"),
        "legacy Core mutation must reject before typed handler admission: {error}"
    );
}

#[test]
fn checked_nonsealed_handler_name_remains_closed_to_production_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = checked_entry(&engine, DIFFERENT_HANDLER_NAME_SOURCE);

    let error = engine
        .admit_production_checked_handler(&mut entry)
        .expect_err("only absorb_sleep may receive the sealed production token");
    assert!(
        error.to_string().contains("sealed absorb_sleep handler"),
        "handler-name rejection must remain at production admission: {error}"
    );
}

#[test]
fn checked_nonidentity_done_clause_remains_closed_to_production_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = checked_entry(&engine, NONIDENTITY_DONE_SOURCE);

    let error = engine
        .admit_production_checked_handler(&mut entry)
        .expect_err("the sealed production slice must reject nonidentity done clauses");
    assert!(
        error
            .to_string()
            .contains("done clause must be identity for the current Core handler lowering"),
        "nonidentity done must reject at the typed Core handler boundary: {error}"
    );
}

#[test]
fn checked_absorb_sleep_with_a_nonempty_residual_row_rejects_before_production_execution() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = checked_entry(&engine, NONEMPTY_RESIDUAL_ABSORB_SLEEP_SOURCE);

    let error = engine
        .admit_production_checked_handler(&mut entry)
        .expect_err("the sealed handler token must not authorize a residual operation frame");
    assert_eq!(
        error.to_string(),
        "type error: resolver-produced residual operation facts do not match the checked residual row",
        "the nonempty residual row must reject while admission is still collecting explicit authority"
    );
}
