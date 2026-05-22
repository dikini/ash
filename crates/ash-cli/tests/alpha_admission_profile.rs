use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn reject_admission_profile_reports_admission_failure_before_body_output() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("entry.ash");
    let output_path = temp.path().join("sentinel.json");
    fs::write(
        &workflow_path,
        r#"
        workflow main {
            ret 42;
        }
        "#,
    )
    .expect("write workflow");

    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("run")
        .arg("--admission-profile")
        .arg("reject")
        .arg("--output")
        .arg(&output_path)
        .arg("--format")
        .arg("json")
        .arg(&workflow_path)
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        !output_path.exists(),
        "rejected admission must not create the body-derived output sentinel"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        !stdout.contains("42"),
        "rejected admission must not emit body result on stdout: {stdout}"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(
        stderr.contains("admission") && stderr.contains("rejected"),
        "diagnostic should classify this as admission rejection: {stderr}"
    );
    assert!(
        stderr.contains("\"status\": \"rejected\""),
        "kernel report should expose rejected admission status: {stderr}"
    );
    assert!(
        !stderr.contains("\"artifact_summary\"") && !stderr.contains("\"verifier\": \"verified\""),
        "rejected admission must not be reported as a verified body artifact: {stderr}"
    );
    assert!(
        !stderr.contains("parse error") && !stderr.contains("runtime error"),
        "admission rejection must be distinct from parse/body failure: {stderr}"
    );
}

#[test]
fn default_empty_admission_profile_remains_admitted() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("entry.ash");
    fs::write(
        &workflow_path,
        r#"
        workflow main {
            ret 7;
        }
        "#,
    )
    .expect("write workflow");

    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("run")
        .arg(&workflow_path)
        .arg("--format")
        .arg("json")
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains("7"),
        "body result should still execute: {stdout}"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    let report: serde_json::Value =
        serde_json::from_str(&stderr).expect("kernel report json on stderr");
    assert_eq!(report["admission"]["status"], "admitted");
}

#[test]
fn execution_failure_still_emits_runtime_kernel_report() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("ordinary-error.ash");
    fs::write(
        &workflow_path,
        r#"
        workflow main {
            observe missing;
        }
        "#,
    )
    .expect("write workflow");

    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("run")
        .arg(&workflow_path)
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .failure()
        .get_output()
        .clone();

    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    let json_start = stderr
        .find('{')
        .expect("kernel report starts with json object");
    let json_end = stderr
        .rfind("\n}")
        .expect("kernel report ends with json object")
        + 2;
    let report_text = &stderr[json_start..json_end];
    let report: serde_json::Value =
        serde_json::from_str(report_text).expect("kernel report json on execution failure");
    assert_eq!(report["host_mode"], "OneShot");
    assert_eq!(report["workflow"], "main");
    assert_eq!(report["admission"]["status"], "admitted");
    assert_eq!(
        report["artifact_summary"]["tcir"]["carrier_scope"],
        "alpha_checked_workflow_boundary"
    );
}
