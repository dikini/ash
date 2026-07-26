//! TASK-2014 RED coverage for the public execution handoff of an Engine-sealed
//! `time::sleep` admission.
//!
//! The test deliberately obtains the opaque token through real Engine
//! admission.  The execution API receives neither a row, raw CPS term, frame
//! instruction, nor provider object.

use ash_core::Value;
use ash_engine::{Engine, ProductionCheckedCpsOutcome, standard_profiles::StandardProviderProfile};

const SLEEP_ZERO: &str = "fn main() -> Null { time::sleep(0) }";

async fn install_application_time_profile(engine: &Engine) {
    engine
        .install_standard_profile(StandardProviderProfile::application_default(
            "task-2014-async-production-execution",
            std::iter::empty::<&std::path::Path>(),
            std::iter::empty::<&str>(),
        ))
        .await
        .expect("the standard application profile installs the exact time provider");
    engine
        .register_time_sleep_provider_binding()
        .expect("the Engine validates and seals the exact time.sleep binding");
}

#[tokio::test(start_paused = true)]
async fn admitted_time_sleep_completion_returns_through_checked_cps() {
    let engine = Engine::new().build().expect("engine builds");
    install_application_time_profile(&engine).await;
    let mut entry = engine.parse(SLEEP_ZERO).expect("fixture parses");
    let admission = engine
        .admit_production_checked_cps(&mut entry)
        .expect("only a real checked source entry mints the opaque production token");

    // The Engine, not a row/CPS caller, creates the execution-only envelope
    // from a successful opaque admission. There intentionally is no public
    // control-only constructor that can run before admission.
    let (control, _cancellation) = engine
        .new_production_run_control(&admission, None)
        .expect("only the issuing Engine may create control for its sealed token");
    let outcome = engine
        .execute_production_checked_cps(&admission, control)
        .await
        .expect("a sealed matching provider frame executes through checked CPS");

    assert!(matches!(
        outcome,
        ProductionCheckedCpsOutcome::Return(Value::Null)
    ));
}

#[tokio::test(start_paused = true)]
async fn a_foreign_engine_cannot_create_control_for_another_engines_admission() {
    let issuing_engine = Engine::new().build().expect("issuing engine builds");
    install_application_time_profile(&issuing_engine).await;
    let mut entry = issuing_engine.parse(SLEEP_ZERO).expect("fixture parses");
    let admission = issuing_engine
        .admit_production_checked_cps(&mut entry)
        .expect("the issuing Engine seals the real production token");
    let foreign_engine = Engine::new().build().expect("foreign engine builds");

    assert!(
        foreign_engine
            .new_production_run_control(&admission, None)
            .is_err(),
        "run control must be created only after admission and only by its issuing Engine"
    );
}

#[tokio::test(start_paused = true)]
async fn an_unrepresentable_post_admission_deadline_rejects_control_creation() {
    let engine = Engine::new().build().expect("engine builds");
    install_application_time_profile(&engine).await;
    let mut entry = engine.parse(SLEEP_ZERO).expect("fixture parses");
    let admission = engine
        .admit_production_checked_cps(&mut entry)
        .expect("the real checked source entry seals production admission");

    let control = engine.new_production_run_control(&admission, Some(std::time::Duration::MAX));

    assert!(
        control.is_err(),
        "deadline overflow must reject control creation, not manufacture an immediately-expired control"
    );
}
