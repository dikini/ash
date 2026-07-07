//! TASK-1938 clock/time stdlib wrapper/profile tests.

use ash_core::Value;
use ash_core::runtime::HostBoundaryOutcome;
use ash_engine::standard_profiles::StandardProviderProfile;
use ash_engine::{Engine, EngineError};

fn engine() -> Result<Engine, EngineError> {
    Engine::new().build()
}

#[tokio::test]
async fn stdlib_time_wrappers_execute_through_deterministic_profile_and_record_evidence() {
    let fixed_epoch_millis = 1_700_000_000_123_u64;
    let fixed_epoch_value = i64::try_from(fixed_epoch_millis).expect("fixture fits i64");

    let engine = engine().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::deterministic_test(
            "task-1938-clock",
            fixed_epoch_millis,
        ))
        .await
        .expect("deterministic profile installs");

    let source = r"
        fn main() -> Int {
            do {
                first <- time::epoch_millis();
                snapshot <- time::now();
                iso <- time::now_iso();
                second <- time::epoch_millis();
                return first + second - snapshot.epoch_millis
            }
        }
    ";

    let result = engine
        .run(source)
        .await
        .expect("time stdlib wrappers should execute");
    assert_eq!(result, Value::Int(fixed_epoch_value));

    let evidence = engine.host_boundary_evidence().await;
    for operation in ["epoch_millis", "now", "now_iso"] {
        assert!(
            evidence.iter().any(|record| record.provider_name == "time"
                && record.operation_name == operation
                && record.outcome == HostBoundaryOutcome::Succeeded
                && record.authority_neutral),
            "{operation} should record authority-neutral success evidence: {evidence:?}"
        );
    }
    assert!(
        evidence.iter().all(|record| !record
            .redacted_subject
            .contains(&fixed_epoch_millis.to_string())),
        "clock evidence must redact raw clock values: {evidence:?}"
    );
}

#[tokio::test]
async fn deterministic_profile_denies_sleep_before_wall_clock_delay() {
    let engine = engine().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::deterministic_test(
            "task-1938-clock",
            1_700_000_000_000,
        ))
        .await
        .expect("deterministic profile installs");

    let source = r"
        fn main() -> Int {
            do {
                time::sleep(1);
                return 0
            }
        }
    ";

    let error = engine
        .run(source)
        .await
        .expect_err("deterministic profile should deny sleep");
    assert!(error.to_string().contains("denied time.sleep"), "{error}");

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.iter().any(|record| record.provider_name == "time"
            && record.operation_name == "sleep"
            && record.outcome == HostBoundaryOutcome::Denied),
        "sleep denial should record redacted host-boundary evidence: {evidence:?}"
    );
}

#[tokio::test]
async fn application_default_profile_allows_real_clock_observation_and_sleep_attempt_evidence() {
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
        fn main() -> Int {
            do {
                before <- time::epoch_millis();
                time::sleep(0);
                after <- time::epoch_millis();
                return after - before
            }
        }
    ";

    let result = engine
        .run(source)
        .await
        .expect("application profile should allow time observation and zero sleep");
    assert!(
        matches!(result, Value::Int(delta) if delta >= 0),
        "real clock delta should be non-negative: {result:?}"
    );

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.iter().any(|record| record.provider_name == "time"
            && record.operation_name == "sleep"
            && record.outcome == HostBoundaryOutcome::Succeeded
            && record.authority_neutral),
        "sleep attempt should record authority-neutral success evidence: {evidence:?}"
    );
}
