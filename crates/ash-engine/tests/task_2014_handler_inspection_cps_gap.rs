//! Diagnostic control for TASK-2014 handler-inspection execution RED.
//!
//! The sealed inspection artifact deliberately retains an unterminalized CPS
//! `Handle`. This proves the remaining gap is answer-continuation sealing in
//! the Engine execution API, not missing handler dispatch or a provider.

use ash_engine::Engine;

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

#[test]
fn unterminalized_handler_inspection_cps_requires_an_engine_answer_continuation() {
    let engine = Engine::new()
        .build()
        .expect("engine builds without a provider");
    let mut entry = engine.parse(ECHO_SLEEP_SOURCE).expect("fixture parses");
    engine.check(&mut entry).expect("fixture checks");
    let admission = engine
        .admit_checked_handler_inspection(&entry, "echo_sleep")
        .expect("fixture admits the inspection artifact");

    let error = ash_interp::cps::eval_checked_terminal(
        admission.checked_core().lowered(),
        &ash_core::cps::Env::new(),
        &ash_core::cps::HandlerChain::new(),
    )
    .expect_err("the raw inspection CPS term has no terminal answer continuation");
    assert!(
        error.to_string().contains("__handler_inspection_answer"),
        "the diagnostic must identify the missing terminal continuation rather than a provider: {error}"
    );
}
