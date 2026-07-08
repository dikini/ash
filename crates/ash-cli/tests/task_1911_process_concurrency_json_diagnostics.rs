use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn ash_check_json_reports_structured_process_channel_row_boundary_diagnostic() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("invalid-process-channel-row.ash");
    std::fs::write(
        &path,
        r#"
        fn invalid(job: Int) -> Int
        where
            row {
                process spawn,
                channel jobs,
                requires_fact
            }
        {
            job
        }

        fn main() { 0 }
        "#,
    )
    .expect("write invalid process/channel fixture");

    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .args(["check", "--format", "json"])
        .arg(&path)
        .output()
        .expect("run ash check json");
    assert!(
        !output.status.success(),
        "invalid row boundary should fail\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("ash check should emit JSON diagnostics");
    let diagnostics = json["diagnostics"]
        .as_array()
        .expect("diagnostics should be an array");
    assert!(!diagnostics.is_empty(), "json={json}");

    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic["severity"].as_str(), Some("error"));
    assert!(
        diagnostic["message"].as_str().is_some_and(|message| {
            message.contains("unsupported")
                && message.contains("row")
                && message.contains("requires")
        }),
        "json={json}"
    );
    assert!(diagnostic["location"]["file"].as_str().is_some());
    assert!(diagnostic["location"]["line"].as_u64().is_some());
    assert!(diagnostic["location"]["column"].as_u64().is_some());
}
