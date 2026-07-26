//! TASK-1939 logging stdlib wrapper/profile tests under strict closed admission.
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
async fn stdlib_logging_wrappers_parse_check_then_reject_before_provider_execution() {
    let engine = engine().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::application_default(
            "task-1939-app",
            std::iter::empty::<&std::path::Path>(),
            std::iter::empty::<&str>(),
        ))
        .await
        .expect("application profile installs");

    let source = r#"
        use logging::{debug, info, warn, error}

        fn main() -> Int {
            do {
                let debug_event = debug("debug secret=alpha");
                let info_event = info("info secret=bravo");
                let warn_event = warn("warn secret=charlie");
                let error_event = error("error secret=delta");
                return debug_event.field_count + info_event.field_count + warn_event.field_count + error_event.field_count
            }
        }
    "#;

    let error = parse_check_execute(
        &engine,
        "task_1939_logging_provider_wrappers_success.ash",
        source,
    )
    .await
    .expect("logging stdlib wrappers should parse, check, and reach closed admission");
    assert_closed_admission(error);

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.is_empty(),
        "closed admission must prevent logging provider execution and host evidence: {evidence:?}"
    );
}

#[tokio::test]
async fn logging_only_profile_log_request_rejects_before_provider_execution() {
    let engine = engine().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::logging_only("task-1939-log"))
        .await
        .expect("logging-only profile installs");

    let source = r#"
        use logging::{info}

        fn main() -> Int {
            do {
                info("secret=blocked");
                return 0
            }
        }
    "#;

    let error = parse_check_execute(
        &engine,
        "task_1939_logging_provider_wrappers_denied.ash",
        source,
    )
    .await
    .expect("logging request shape should parse, check, and reach closed admission");
    assert_closed_admission(error);

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.is_empty(),
        "closed admission must prevent the logging request from reaching a provider: {evidence:?}"
    );
}

#[tokio::test]
async fn logging_profile_unrelated_time_request_rejects_before_provider_execution() {
    let engine = engine().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::logging_only("task-1939-log"))
        .await
        .expect("logging-only profile installs");

    let source = r"
        use time::{epoch_millis}

        fn main() -> Int {
            do {
                epoch_millis();
                return 0
            }
        }
    ";

    let error = parse_check_execute(
        &engine,
        "task_1939_logging_provider_wrappers_unadmitted_time.ash",
        source,
    )
    .await
    .expect("unrelated time request shape should parse, check, and reach closed admission");
    assert_closed_admission(error);

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.is_empty(),
        "closed admission must prevent the unrelated time request from reaching a provider: {evidence:?}"
    );
}
