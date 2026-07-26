//! TASK-1938 clock/time stdlib wrapper/profile tests under strict closed admission.
//!
//! The wrapper declarations, imports, request shapes, and profile registration remain checked at
//! the source boundary. Positive host behavior deliberately awaits authorized frame installation
//! and the async CPS host driver; generic source execution must not revive direct dispatch.

use ash_engine::standard_profiles::StandardProviderProfile;
use ash_engine::{Engine, EngineError};
use ash_interp::ExecError;

const CLOSED_ADMISSION_ERROR: &str =
    "checked Core/CPS admission rejected: no validated production typed lowering is available";

fn engine() -> Result<Engine, EngineError> {
    Engine::new().build()
}

async fn parse_check_execute(
    engine: &Engine,
    fixture: &str,
    source: &str,
) -> Result<ExecError, Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(fixture);
    let mut application = engine.parse_file_source(path, source)?;
    engine.check(&mut application)?;
    let error = engine
        .execute(&application)
        .await
        .expect_err("generic source execution must reject without checked Core/CPS admission");
    Ok(error)
}

fn assert_closed_admission(error: ExecError) {
    assert!(
        matches!(error, ExecError::ExecutionFailed(message) if message == CLOSED_ADMISSION_ERROR),
        "generic source execution must expose the exact canonical closed-admission error"
    );
}

#[tokio::test]
async fn stdlib_time_wrappers_parse_check_then_reject_before_provider_execution() {
    let fixed_epoch_millis = 1_700_000_000_123_u64;

    let engine = engine().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::deterministic_test(
            "task-1938-clock",
            fixed_epoch_millis,
        ))
        .await
        .expect("deterministic profile installs");

    let source = r"
        use time::{epoch_millis, now, now_iso}

        fn main() -> Int {
            do {
                let first = epoch_millis();
                let snapshot = now();
                let iso = now_iso();
                let second = epoch_millis();
                return first + second - snapshot.epoch_millis
            }
        }
    ";

    let error = parse_check_execute(
        &engine,
        "task_1938_clock_time_provider_wrappers_deterministic.ash",
        source,
    )
    .await
    .expect("time stdlib wrappers should parse, check, and reach closed admission");
    assert_closed_admission(error);

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.is_empty(),
        "closed admission must prevent time provider execution and host evidence: {evidence:?}"
    );
}

#[tokio::test]
async fn deterministic_profile_sleep_request_rejects_before_provider_execution() {
    let engine = engine().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::deterministic_test(
            "task-1938-clock",
            1_700_000_000_000,
        ))
        .await
        .expect("deterministic profile installs");

    let source = r"
        use time::{sleep}

        fn main() -> Int {
            do {
                sleep(1);
                return 0
            }
        }
    ";

    let error = parse_check_execute(
        &engine,
        "task_1938_clock_time_provider_wrappers_sleep_denied.ash",
        source,
    )
    .await
    .expect("sleep request shape should parse, check, and reach closed admission");
    assert_closed_admission(error);

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.is_empty(),
        "closed admission must prevent the deterministic sleep request from reaching a provider: {evidence:?}"
    );
}

#[tokio::test]
async fn application_profile_time_request_rejects_before_provider_execution() {
    let engine = engine().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::application_default(
            "task-1938-app",
            std::iter::empty::<&std::path::Path>(),
            std::iter::empty::<&str>(),
        ))
        .await
        .expect("application profile installs");

    let source = r"
        use time::{epoch_millis, sleep}

        fn main() -> Int {
            do {
                let before = epoch_millis();
                sleep(0);
                let after = epoch_millis();
                return after - before
            }
        }
    ";

    let error = parse_check_execute(
        &engine,
        "task_1938_clock_time_provider_wrappers_application.ash",
        source,
    )
    .await
    .expect("application time request shape should parse, check, and reach closed admission");
    assert_closed_admission(error);

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.is_empty(),
        "closed admission must prevent the application time request from reaching a provider: {evidence:?}"
    );
}
