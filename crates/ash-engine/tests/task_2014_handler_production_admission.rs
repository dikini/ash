//! TASK-2013/TASK-2014 RED contract for sealed source-handler production admission.
//!
//! The sole admitted handler slice is a closed-empty local `absorb_sleep`
//! computation. Production success must come through checked Core/CPS, while
//! `Engine::execute` remains a closed generic entrypoint.

use ash_core::{Value, ast::Expr};
use ash_engine::{Engine, Entry};
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
fn checked_handler_entry_from_a_foreign_engine_is_rejected() {
    let issuing_engine = Engine::new().build().expect("issuing engine builds");
    let mut entry = checked_entry(&issuing_engine, ABSORB_SLEEP_SOURCE);
    let foreign_engine = Engine::new().build().expect("foreign engine builds");

    let error = foreign_engine
        .admit_production_checked_handler(&mut entry)
        .expect_err("a foreign Engine must not issue handler production authority");
    assert!(
        error.to_string().contains("issued by this Engine"),
        "foreign entry rejection must stay at the Engine provenance boundary: {error}"
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
