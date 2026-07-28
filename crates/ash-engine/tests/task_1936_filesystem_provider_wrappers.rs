//! TASK-1936 filesystem stdlib wrapper/profile tests under strict closed admission.
//!
//! The wrapper declarations, imports, request shapes, and profile registration remain checked at
//! the source boundary. Positive host behavior deliberately awaits authorized frame installation
//! and the async CPS host driver; generic source execution must not revive direct dispatch.

use ash_engine::standard_profiles::StandardProviderProfile;
use ash_engine::{Engine, EngineError};
use ash_runtime::ExecError;
use tempfile::tempdir;

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
async fn stdlib_fs_wrappers_parse_check_then_reject_before_provider_execution() {
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
        use io::fs::{{write_string, append, exists, read_to_string}}
        use io::meta::{{metadata}}
        use io::dir::{{read_dir}}
        use io::path::{{PathBuf}}

        type PathBuf = PathBuf {{ inner: String }};

        fn main() -> String {{
            do {{
                write_string(PathBuf {{ inner: "{path}" }}, "after");
                append(PathBuf {{ inner: "{path}" }}, "!");
                let exists_value = exists(PathBuf {{ inner: "{path}" }});
                let metadata_value = metadata(PathBuf {{ inner: "{path}" }});
                let entries = read_dir(PathBuf {{ inner: "{root_path}" }});
                let contents = read_to_string(PathBuf {{ inner: "{path}" }});
                return contents
            }}
        }}
    "#
    );

    let error = parse_check_execute(
        &engine,
        "task_1936_filesystem_provider_wrappers_read_write.ash",
        &source,
    )
    .await
    .expect("filesystem stdlib wrappers should parse, check, and reach closed admission");
    assert_closed_admission(error);

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.is_empty(),
        "closed admission must prevent filesystem provider execution and host evidence: {evidence:?}"
    );
    assert_eq!(
        std::fs::read_to_string(&file).expect("fixture remains readable"),
        "before"
    );
}

#[tokio::test]
async fn stdlib_fs_wrapper_denial_shape_rejects_before_provider_execution() {
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
        use io::fs::{{write_string}}
        use io::path::{{PathBuf}}

        type PathBuf = PathBuf {{ inner: String }};

        fn main() -> Int {{
            do {{
                write_string(PathBuf {{ inner: "{blocked_path}" }}, "blocked");
                return 0
            }}
        }}
    "#
    );

    let error = parse_check_execute(
        &engine,
        "task_1936_filesystem_provider_wrappers_denied.ash",
        &source,
    )
    .await
    .expect("denial request shape should parse, check, and reach closed admission");
    assert_closed_admission(error);
    assert!(
        !blocked.exists(),
        "sandbox denial must happen before writing outside the allowed path"
    );

    let evidence = engine.host_boundary_evidence().await;
    assert!(
        evidence.is_empty(),
        "closed admission must prevent the denied filesystem request from reaching a provider: {evidence:?}"
    );
}
