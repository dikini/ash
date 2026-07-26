//! TASK-2010 RED coverage for the first statically resolvable source operation.

use ash_core::{
    core_ash::{CoreRowItem, CoreType},
    cps::Term,
};
use ash_engine::{
    ApplicationAdmissionOutcome, ApplicationAdmissionRequest, Engine,
    standard_profiles::StandardProviderProfile,
};

const TIME_SLEEP_SOURCE: &str = "fn main() -> Null { time::sleep(0) }";
const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

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

fn checked_time_sleep_entry(engine: &Engine) -> ash_engine::Entry {
    let mut entry = engine
        .parse(TIME_SLEEP_SOURCE)
        .expect("time::sleep source call parses");
    engine
        .check(&mut entry)
        .expect("registered time::sleep source operation checks");
    entry
}

#[test]
fn task_2010_time_sleep_source_call_has_a_canonical_non_granting_requirement_row() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = checked_time_sleep_entry(&engine);

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
                if path == &["time".to_string()] && operation == "sleep"
        )),
        "time::sleep must contribute its canonical requirement row: {row:?}"
    );
    assert!(
        entry
            .lowering_sidecars
            .entry_body_origin
            .label
            .contains("main"),
        "the source entry must retain an origin sidecar"
    );
}

#[tokio::test]
async fn task_2010_time_sleep_row_rejects_without_an_admitted_time_provider() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = checked_time_sleep_entry(&engine);

    let outcome = engine
        .admit_application_with_explicit_rows(request(&entry), &entry)
        .await;
    assert!(
        matches!(outcome, ApplicationAdmissionOutcome::Rejected { .. }),
        "a requirement row must not install time authority: {outcome:?}"
    );
}

#[tokio::test]
async fn task_2010_time_sleep_stays_closed_even_with_an_admitted_time_provider() {
    let engine = Engine::new().build().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::application_default(
            "task-2010-time",
            std::iter::empty::<&std::path::Path>(),
            std::iter::empty::<&str>(),
        ))
        .await
        .expect("application time provider installs");
    let entry = checked_time_sleep_entry(&engine);

    let error = engine
        .execute(&entry)
        .await
        .expect_err("generic source execution must not dispatch time::sleep before typed lowering");
    assert!(
        matches!(
            error,
            ash_interp::ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR
        ),
        "time::sleep must expose the exact checked Core/CPS closed-admission error"
    );
}

#[test]
fn task_2010_private_checked_lowering_represents_time_sleep_as_raise() {
    let engine = Engine::new().build().expect("engine builds");
    let entry = checked_time_sleep_entry(&engine);

    let Term::Raise { op, args, .. } = engine
        .lower_entry_to_checked_cps(&entry)
        .expect("supported source operation has a checked lowering artifact")
    else {
        panic!("time::sleep source operation must lower to CPS Raise");
    };
    assert_eq!(op.item.namespace, "time");
    assert_eq!(op.item.name, "sleep");
    assert_eq!(args, vec![ash_core::cps::Atom::Int(0)]);
}

#[test]
fn task_2010_direct_source_invoke_remains_rejected() {
    let engine = Engine::new().build().expect("engine builds");
    let mut entry = engine
        .parse("fn main() -> Unit { invoke(\"time\", \"sleep\", [0]) }")
        .expect("direct invoke fixture parses before type checking");

    let error = engine
        .check(&mut entry)
        .expect_err("direct source invoke must remain rejected");
    assert!(
        error
            .to_string()
            .contains("direct source invoke is not admitted"),
        "unexpected invoke rejection: {error}"
    );
}
