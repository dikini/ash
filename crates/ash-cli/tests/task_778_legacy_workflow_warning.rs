//! TASK-778 CLI regression tests for legacy workflow deprecation warnings.

use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

fn write_legacy_workflow(source: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("legacy.ash");
    std::fs::write(&path, source).expect("write legacy workflow");
    (dir, path)
}

fn legacy_workflow_with_headers() -> (tempfile::TempDir, std::path::PathBuf) {
    write_legacy_workflow(
        r"workflow main
    plays role(Admin)
    requires: role(Auditor)
{
    done
}
",
    )
}

fn headerless_legacy_workflow() -> (tempfile::TempDir, std::path::PathBuf) {
    write_legacy_workflow(
        r"workflow main {
    done
}
",
    )
}

fn assert_legacy_workflow_warning_human_output(stdout: &str) {
    assert!(stdout.contains("Warning:"), "stdout={stdout}");
    assert!(
        stdout.contains("DeprecatedLegacyWorkflowDeclaration"),
        "stdout={stdout}"
    );
    assert!(
        !stdout.contains("[NEW] DeprecatedLegacyWorkflowDeclaration"),
        "warning should use the stable diagnostic code; stdout={stdout}"
    );
    assert!(
        stdout.contains("first-class Workflow") || stdout.contains("do:Workflow"),
        "warning should include a rewrite hint toward first-class workflow syntax; stdout={stdout}"
    );
}

#[test]
fn ash_check_legacy_workflow_warning_is_non_fatal_human() {
    let (_dir, path) = legacy_workflow_with_headers();

    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("check")
        .arg(&path)
        .output()
        .expect("run ash check");

    assert!(
        output.status.success(),
        "legacy workflow warning must be non-fatal\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_legacy_workflow_warning_human_output(&stdout);
}

#[test]
fn ash_check_headerless_legacy_workflow_warning_is_non_fatal_human() {
    let (_dir, path) = headerless_legacy_workflow();

    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("check")
        .arg(&path)
        .output()
        .expect("run ash check");

    assert!(
        output.status.success(),
        "headerless legacy workflow warning must be non-fatal\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_legacy_workflow_warning_human_output(&stdout);
}

#[test]
fn ash_check_legacy_workflow_warning_is_non_fatal_json_diagnostic() {
    let (_dir, path) = legacy_workflow_with_headers();

    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .args(["check", "--format", "json"])
        .arg(&path)
        .output()
        .expect("run ash check json");

    assert!(
        output.status.success(),
        "legacy workflow warning must not fail json check\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be JSON");

    assert_eq!(json["success"].as_bool(), Some(true), "json={json}");
    assert_eq!(json["exit_code"].as_i64(), Some(0), "json={json}");
    assert_eq!(
        json["summary"]["warning_count"].as_u64(),
        Some(1),
        "json={json}"
    );

    let diagnostics = json["diagnostics"]
        .as_array()
        .expect("diagnostics should be an array");
    assert_eq!(diagnostics.len(), 1, "json={json}");
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic["severity"].as_str(),
        Some("warning"),
        "json={json}"
    );
    assert_eq!(
        diagnostic["code"].as_str(),
        Some("DeprecatedLegacyWorkflowDeclaration"),
        "json={json}"
    );
    assert_eq!(
        diagnostic["location"]["line"].as_u64(),
        Some(1),
        "json={json}"
    );
    assert_eq!(
        diagnostic["location"]["column"].as_u64(),
        Some(1),
        "json={json}"
    );
    let message = diagnostic["message"].as_str().unwrap_or_default();
    assert_ne!(
        diagnostic["code"].as_str(),
        Some("[NEW] DeprecatedLegacyWorkflowDeclaration"),
        "json={json}"
    );
    assert!(
        message.contains("first-class Workflow") || message.contains("do:Workflow"),
        "warning should include a rewrite hint toward first-class workflow syntax; json={json}"
    );
}
