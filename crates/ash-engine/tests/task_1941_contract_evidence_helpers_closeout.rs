//! TASK-1941 contract/evidence helper and Phase 198 closeout tests.

use ash_core::Value;
use ash_engine::{Engine, EngineError};

fn engine() -> Result<Engine, EngineError> {
    Engine::new().build()
}

#[tokio::test]
async fn evidence_helpers_execute_as_authority_free_final_surface_imports() {
    let engine = engine().expect("engine builds");
    let temp_dir = tempfile::tempdir().expect("temp dir created");

    for (helper, argument) in [
        ("has_evidence", "3"),
        ("is_redacted", "true"),
        ("is_authority_neutral", "true"),
        ("provider_outcome_is_success", "\"succeeded\""),
        ("provider_outcome_is_denied", "\"denied\""),
        ("provider_outcome_is_failure", "\"failed\""),
    ] {
        let source = format!(
            r"
            use evidence::{{{helper}}}

            fn main() -> Bool {{
                do {{
                    return {helper}({argument})
                }}
            }}
        "
        );
        let source_path = temp_dir.path().join(format!("{helper}.ash"));
        std::fs::write(&source_path, source).expect("write helper fixture");
        let result = engine
            .run_file(&source_path)
            .await
            .unwrap_or_else(|error| panic!("{helper} should execute without authority: {error}"));
        assert_eq!(result, Value::Bool(true), "{helper} returned {result:?}");
    }

    assert!(
        engine.host_boundary_evidence().await.is_empty(),
        "evidence helpers must inspect values without acquiring host/provider authority"
    );
}

#[test]
fn phase_198_final_surface_fixture_inventory_is_present() {
    let fixture_names = [
        "task_1936_filesystem_provider_wrappers.rs",
        "task_1937_http_provider_wrappers.rs",
        "task_1938_clock_time_provider_wrappers.rs",
        "task_1939_logging_provider_wrappers.rs",
    ];
    let tests_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

    for fixture in fixture_names {
        assert!(
            tests_dir.join(fixture).exists(),
            "Phase 198 closeout requires final-surface fixture {fixture}"
        );
    }
}
