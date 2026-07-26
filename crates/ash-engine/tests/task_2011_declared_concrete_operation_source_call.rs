//! TASK-2011 RED coverage for declaration-backed concrete operation calls.
//!
//! This fixture deliberately has no provider-execution assertion.  Mapping a
//! resolved declaration identity to runtime authority remains a separate
//! declared-operation-to-provider metadata seam.

use ash_core::{
    core_ash::{CoreRowItem, CoreType},
    cps::{Atom, Term},
};
use ash_engine::{ApplicationAdmissionOutcome, ApplicationAdmissionRequest, Engine};

const DECLARED_CLOCK_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
fn main() -> Null { TestClock::sleep(0) }
";

fn checked_declared_clock_entry(engine: &Engine) -> ash_engine::Entry {
    let mut entry = engine
        .parse(DECLARED_CLOCK_SOURCE)
        .expect("declared clock fixture parses");
    engine
        .check(&mut entry)
        .expect("declared concrete operation checks from registered metadata");
    entry
}

fn request(entry: &ash_engine::Entry) -> ApplicationAdmissionRequest {
    ApplicationAdmissionRequest {
        entry_name: "main".to_string(),
        body: entry.core.clone(),
        application_id: None,
        run_id: None,
        active_role: None,
        admitted_role: None,
        required_capabilities: Vec::new(),
        requires: Vec::new(),
        ensures: Vec::new(),
    }
}

#[test]
fn task_2011_declared_concrete_call_has_exact_non_granting_row_and_source_anchor() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = checked_declared_clock_entry(&engine);

    let CoreType::Function { row, .. } = entry
        .core_callable_types
        .get("main")
        .expect("checked entry has a Core callable type")
    else {
        panic!("main must retain a Core function type");
    };
    assert!(
        row.items.iter().any(|item| matches!(
            item,
            CoreRowItem::Operation { path, operation }
                if path == &["TestClock".to_string()] && operation == "sleep"
        )),
        "the registered impl identity must attach exactly TestClock::sleep: {row:?}"
    );
    assert!(
        entry
            .lowering_sidecars
            .entry_body_origin
            .label
            .contains("main"),
        "the source call boundary must retain an entry source anchor"
    );
}

#[tokio::test]
async fn task_2011_declared_concrete_row_does_not_grant_authority() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = checked_declared_clock_entry(&engine);

    let outcome = engine
        .admit_application_with_explicit_rows(request(&entry), &entry)
        .await;
    assert!(
        matches!(outcome, ApplicationAdmissionOutcome::Rejected { .. }),
        "a declared operation row must not install authority: {outcome:?}"
    );
}

#[test]
fn task_2011_declared_concrete_call_lowers_to_raise_with_declared_signature() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = checked_declared_clock_entry(&engine);

    let Term::Raise { op, args, row, .. } = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("resolved declared operation has a checked Raise inspection artifact")
    else {
        panic!("declared concrete operation must lower to CPS Raise");
    };
    assert_eq!(op.item.namespace, "TestClock");
    assert_eq!(op.item.name, "sleep");
    assert_eq!(op.arg_types, ["Int"]);
    assert_eq!(op.result_type, "Null");
    assert_eq!(args, vec![Atom::Int(0)]);
    assert_eq!(row.items.len(), 1);
    assert_eq!(row.items[0].namespace, "TestClock");
    assert_eq!(row.items[0].name, "sleep");
}

#[test]
fn task_2011_unknown_concrete_impl_fails_before_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Null { MissingClock::sleep(0) }")
        .expect("unknown concrete operation fixture parses");

    let error = engine
        .check(&mut entry)
        .expect_err("unknown concrete impl must fail during declaration resolution");
    assert!(
        error
            .to_string()
            .contains("unknown concrete impl 'MissingClock'"),
        "unexpected unknown-impl diagnostic: {error}"
    );
}

#[test]
fn task_2011_unknown_declared_operation_fails_before_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(
            r"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
fn main() -> Null { TestClock::wake(0) }
",
        )
        .expect("unknown operation fixture parses");

    let error = engine
        .check(&mut entry)
        .expect_err("unknown declared operation must fail during declaration resolution");
    assert!(
        error
            .to_string()
            .contains("concrete impl 'TestClock' has no operation 'wake'"),
        "unexpected unknown-operation diagnostic: {error}"
    );
}

#[test]
fn task_2011_declared_operation_argument_mismatch_fails_before_admission() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse(
            r#"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
fn main() -> Null { TestClock::sleep("zero") }
"#,
        )
        .expect("argument mismatch fixture parses");

    let error = engine
        .check(&mut entry)
        .expect_err("declared operation argument mismatch must fail before admission");
    assert!(
        error
            .to_string()
            .contains("TestClock::sleep: argument type mismatch"),
        "unexpected argument-mismatch diagnostic: {error}"
    );
}
