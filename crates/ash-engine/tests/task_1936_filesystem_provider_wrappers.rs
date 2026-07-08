//! TASK-1936 filesystem stdlib wrapper/profile tests.

use ash_core::Value;
use ash_core::runtime::HostBoundaryOutcome;
use ash_engine::standard_profiles::StandardProviderProfile;
use ash_engine::{Engine, EngineError};
use tempfile::tempdir;

fn engine() -> Result<Engine, EngineError> {
    Engine::new().build()
}

#[tokio::test]
async fn stdlib_fs_wrappers_execute_through_read_write_profile_and_record_evidence() {
    let root = tempdir().expect("tempdir");
    let file = root.path().join("message.txt");
    std::fs::write(&file, "before").expect("write fixture");
    let path = file.display().to_string();
    let root_path = root.path().display().to_string();

    let engine = engine().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::read_write_filesystem(
            "task-1936-rw",
            [root.path()],
        ))
        .await
        .expect("profile installs");

    let source = format!(
        r#"
        type PathBuf = PathBuf {{ inner: String }};

        fn main() -> String {{
            do {{
                fs::write_string(PathBuf {{ inner: "{path}" }}, "after");
                fs::append(PathBuf {{ inner: "{path}" }}, "!");
                exists <- fs::exists(PathBuf {{ inner: "{path}" }});
                metadata <- meta::metadata(PathBuf {{ inner: "{path}" }});
                entries <- dir::read_dir(PathBuf {{ inner: "{root_path}" }});
                contents <- fs::read_to_string(PathBuf {{ inner: "{path}" }});
                return contents
            }}
        }}
    "#
    );

    let result = engine
        .run(&source)
        .await
        .expect("filesystem stdlib wrappers should execute");
    assert_eq!(result, Value::String("after!".to_string()));

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.iter().any(|record| record.provider_name == "fs"
            && record.operation_name == "write_string"
            && record.outcome == HostBoundaryOutcome::Succeeded
            && record.authority_neutral),
        "write_string should record authority-neutral success evidence: {evidence:?}"
    );
    for operation in ["append", "exists", "metadata", "read_dir"] {
        assert!(
            evidence.iter().any(|record| record.provider_name == "fs"
                && record.operation_name == operation
                && record.outcome == HostBoundaryOutcome::Succeeded
                && record.authority_neutral),
            "{operation} should record authority-neutral success evidence: {evidence:?}"
        );
    }
    assert!(
        evidence.iter().any(|record| record.provider_name == "fs"
            && record.operation_name == "read_to_string"
            && record.outcome == HostBoundaryOutcome::Succeeded
            && record.authority_neutral),
        "read_to_string should record authority-neutral success evidence: {evidence:?}"
    );
    assert!(
        evidence
            .iter()
            .all(|record| !record.redacted_subject.contains(&path)),
        "filesystem evidence must redact raw path arguments: {evidence:?}"
    );
}

#[tokio::test]
async fn stdlib_fs_wrappers_deny_outside_profile_before_host_effects() {
    let root = tempdir().expect("allowed tempdir");
    let outside = tempdir().expect("outside tempdir");
    let blocked = outside.path().join("blocked.txt");
    let blocked_path = blocked.display().to_string();

    let engine = engine().expect("engine builds");
    engine
        .install_standard_profile(StandardProviderProfile::read_write_filesystem(
            "task-1936-rw",
            [root.path()],
        ))
        .await
        .expect("profile installs");

    let source = format!(
        r#"
        type PathBuf = PathBuf {{ inner: String }};

        fn main() -> Int {{
            do {{
                fs::write_string(PathBuf {{ inner: "{blocked_path}" }}, "blocked");
                return 0
            }}
        }}
    "#
    );

    let error = engine
        .run(&source)
        .await
        .expect_err("outside profile path should be denied");
    assert!(
        error.to_string().contains("denied fs.write_string"),
        "{error}"
    );
    assert!(
        !blocked.exists(),
        "sandbox denial must happen before writing outside the allowed path"
    );

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.iter().any(|record| record.provider_name == "fs"
            && record.operation_name == "write_string"
            && record.outcome == HostBoundaryOutcome::Denied),
        "denial should record redacted host-boundary evidence: {evidence:?}"
    );
    assert!(
        evidence
            .iter()
            .all(|record| !record.redacted_subject.contains(&blocked_path)),
        "denial evidence must redact raw path arguments: {evidence:?}"
    );
}
