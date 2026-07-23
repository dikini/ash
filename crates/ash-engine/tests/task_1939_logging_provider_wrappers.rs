//! TASK-1939 logging stdlib wrapper/profile tests.

use ash_core::Value;
use ash_core::runtime::HostBoundaryOutcome;
use ash_engine::standard_profiles::StandardProviderProfile;
use ash_engine::{Engine, EngineError};

fn engine() -> Result<Engine, EngineError> {
    Engine::new().build()
}

async fn parse_check_execute(
    engine: &Engine,
    fixture: &str,
    source: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(fixture);
    let mut application = engine.parse_file_source(path, source)?;
    engine.check(&mut application)?;
    Ok(engine.execute(&application).await?)
}

#[tokio::test]
async fn stdlib_logging_wrappers_emit_structured_records_and_redacted_evidence() {
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
                debug_event <- debug("debug secret=alpha");
                info_event <- info("info secret=bravo");
                warn_event <- warn("warn secret=charlie");
                error_event <- error("error secret=delta");
                return debug_event.field_count + info_event.field_count + warn_event.field_count + error_event.field_count
            }
        }
    "#;

    let result = parse_check_execute(
        &engine,
        "task_1939_logging_provider_wrappers_success.ash",
        source,
    )
    .await
    .expect("logging stdlib wrappers should execute");
    assert_eq!(result, Value::Int(4));

    let evidence = engine.host_boundary_evidence().await;
    for operation in ["debug", "info", "warn", "error"] {
        assert!(
            evidence
                .iter()
                .any(|record| record.provider_name == "logging"
                    && record.operation_name == operation
                    && record.outcome == HostBoundaryOutcome::Succeeded
                    && record.authority_neutral),
            "{operation} should record authority-neutral log evidence: {evidence:?}"
        );
    }
    for secret in ["alpha", "bravo", "charlie", "delta"] {
        assert!(
            evidence
                .iter()
                .all(|record| !record.redacted_subject.contains(secret)),
            "logging evidence must redact secret message values: {evidence:?}"
        );
    }
}

#[tokio::test]
async fn logging_only_profile_denies_final_surface_log_writes_with_redacted_evidence() {
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
    .expect_err("logging-only profile should deny host log writes");
    assert!(error.to_string().contains("denied logging.info"), "{error}");

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence
            .iter()
            .any(|record| record.provider_name == "logging"
                && record.operation_name == "info"
                && record.outcome == HostBoundaryOutcome::Denied
                && record.authority_neutral),
        "denied logging attempt should record authority-neutral evidence: {evidence:?}"
    );
    assert!(
        evidence
            .iter()
            .all(|record| !record.redacted_subject.contains("secret=blocked")),
        "logging denial evidence must redact raw message values: {evidence:?}"
    );
}

#[tokio::test]
async fn logging_profile_selection_does_not_grant_unrelated_authority() {
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
    .expect_err("logging-only profile must not admit time authority");
    assert!(
        error
            .to_string()
            .contains("provider-backed builtin missing admitted time binding"),
        "{error}"
    );
}
