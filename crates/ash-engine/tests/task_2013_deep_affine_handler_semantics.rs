//! TASK-2013 RED contract for the first deep, affine source-handler slice.
//!
//! This is one exact local program, not a generic token or hand-built CPS
//! term. Its source evidence fixes two clause identities and an empty
//! residual row; admission must still be the sole authority that installs the
//! handler frame.

use ash_core::Value;
use ash_engine::Engine;

const DEEP_AFFINE_CLOCK_SOURCE: &str = r"
interface Clock<T> {
    sleep(Int) -> Int
    wake(Int) -> Int
}

type TestClock = SystemClock(Int);
impl Clock<TestClock> {
    sleep(milliseconds) = milliseconds
    wake(milliseconds) = milliseconds
}

handler deep_affine_clock(comp: () -> { TestClock::sleep, TestClock::wake } Int) -> Int {
    on comp {
        TestClock::sleep(ms, resume) => resume(ms),
        TestClock::wake(ms, resume) => resume(ms),
        done(value) => value + 100,
    }
}

fn main() -> Int {
    handle {
        TestClock::sleep(0);
        TestClock::wake(1);
        TestClock::sleep(2);
        7
    } with deep_affine_clock
}
";

#[tokio::test]
async fn deep_affine_handler_reinstalls_before_each_resumed_tail_raise_and_runs_done_once() {
    let engine = Engine::new()
        .build()
        .expect("engine builds without providers");
    let mut entry = engine
        .parse(DEEP_AFFINE_CLOCK_SOURCE)
        .expect("the exact deep-affine source fixture parses");
    engine.check(&mut entry).expect(
        "each clause uses its own resume binder at most once and the source fixture checks",
    );

    let source_facts = engine
        .checked_source_facts_for_handler(&entry, "deep_affine_clock")
        .expect("the checked handler/application facts remain Engine-owned source evidence");
    assert_eq!(
        source_facts
            .handler_clauses()
            .iter()
            .map(|clause| clause.operation().operation())
            .collect::<Vec<_>>(),
        ["sleep", "wake"],
        "the admission input preserves source clause order instead of reconstructing clauses from a row"
    );
    assert!(
        source_facts.residual_rows()[0].is_closed_empty(),
        "the two handled concrete operations are structurally discharged; the empty row does not install a frame"
    );

    assert_eq!(
        engine
            .run(DEEP_AFFINE_CLOCK_SOURCE)
            .await
            .expect("the exact deep-affine source handler is admitted through checked Core/CPS"),
        Value::Int(107),
        "sleep resumes into wake and a second sleep under the reinstalled handler, then normal return alone invokes done(7)"
    );
}
