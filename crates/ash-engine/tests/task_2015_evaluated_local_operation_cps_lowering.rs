//! TASK-2015 inspection coverage for evaluated local symbolic-operation arguments.
//!
//! The checked Core/CPS bridge remains inspection-only under TASK-2004. These
//! assertions freeze its exact `Raise` evidence without promoting that bridge
//! to production execution.

use ash_core::cps::{Atom, Term};
use ash_engine::Engine;

const DECLARED_CLOCK_SOURCE: &str = r"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
fn main() -> Null {
    let delay = 0;
    TestClock::sleep(delay)
}
";

fn checked_entry(source: &str) -> (Engine, ash_engine::Entry) {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine.parse(source).expect("fixture parses");
    engine
        .check(&mut entry)
        .expect("checked local declared-operation fixture checks");
    (engine, entry)
}

#[test]
fn task_2015_checked_local_int_lowers_to_exact_declared_raise() {
    let (engine, entry) = checked_entry(DECLARED_CLOCK_SOURCE);

    let Term::Raise { op, args, row, .. } = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("checked local argument has a Raise inspection artifact")
    else {
        panic!("checked local declared operation must lower to CPS Raise");
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
fn task_2015_non_literal_local_initializer_fails_closed_at_cps_inspection_boundary() {
    let (engine, entry) = checked_entry(
        r"
interface Clock<T> { sleep(Int) -> Null }
type TestClock = SystemClock(Int);
impl Clock<TestClock> { sleep(milliseconds) = null }
fn main() -> Null {
    let delay = if true then 0 else 1;
    TestClock::sleep(delay)
}
",
    );

    let error = engine
        .lower_entry_to_checked_cps(&entry)
        .expect_err("unimplemented lexical initializer must fail closed during CPS inspection");
    assert!(
        error.to_string().contains(
            "declared-operation execution accepts only literal or previously bound local values"
        ),
        "unexpected checked CPS inspection diagnostic: {error}"
    );
}
