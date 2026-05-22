use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn ash_run_executes_entry_through_one_shot_runtime_kernel() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("entry.ash");
    fs::write(
        &workflow_path,
        r#"
        use result::Result
        use runtime::RuntimeError

        workflow main() -> Result<(), RuntimeError> { done; }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run")
        .arg("--dry-run")
        .arg(&workflow_path)
        .env("ASH_RUNTIME_KERNEL_REPORT", "1");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Dry run successful"))
        .stderr(predicate::str::contains("runtime_kernel.host_mode=OneShot"))
        .stderr(predicate::str::contains(
            "runtime_kernel.admission=admitted",
        ));
}

#[test]
fn ash_run_reports_kernel_instance_and_artifact_identity() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("identity.ash");
    fs::write(
        &workflow_path,
        r#"
        workflow main {
            ret 42;
        }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    let output = cmd
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
    assert!(stdout.contains("42"), "workflow output missing: {stdout}");

    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    let report: Value = serde_json::from_str(&stderr).expect("kernel report json on stderr");
    assert_eq!(report["host_mode"], "OneShot");
    assert_eq!(report["workflow"], "main");
    assert!(
        report["kernel_id"]
            .as_str()
            .is_some_and(|id| !id.is_empty())
    );
    assert!(
        report["instance_id"]
            .as_str()
            .is_some_and(|id| !id.is_empty())
    );
    assert!(
        report["definition_id"]
            .as_str()
            .is_some_and(|id| !id.is_empty())
    );
    assert!(
        report["artifact_id"]
            .as_str()
            .is_some_and(|id| !id.is_empty())
    );
    assert_eq!(report["admission"]["status"], "admitted");
    assert_eq!(report["admission"]["capability_grants"], 0);
    assert_eq!(report["admission"]["resource_grants"], 0);
    assert_eq!(report["admission"]["action_grants"], 0);
    assert_eq!(
        report["admission"]["capability_grant_ids"]
            .as_array()
            .expect("capability grant detail list")
            .len(),
        0
    );
    assert_eq!(
        report["admission"]["resource_grant_ids"]
            .as_array()
            .expect("resource grant detail list")
            .len(),
        0
    );
    assert_eq!(
        report["admission"]["action_grant_details"]
            .as_array()
            .expect("action grant detail list")
            .len(),
        0
    );
    assert_eq!(
        report["artifact_summary"]["tcir"]["carrier_scope"],
        "alpha_checked_workflow_boundary"
    );
    assert_eq!(
        report["provider_registry"]["grants_admission_authority"],
        false
    );
}

#[test]
fn ash_run_does_not_emit_verified_artifact_report_before_parse_check_success() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("bad.ash");
    fs::write(&workflow_path, "workflow main {").expect("write malformed workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    let output = cmd
        .arg("run")
        .arg(&workflow_path)
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .failure()
        .get_output()
        .clone();

    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(
        stderr.contains("parse"),
        "parse diagnostics should remain visible: {stderr}"
    );
    assert!(
        !stderr.contains("\"artifact_summary\""),
        "parse-invalid source must not emit a verified artifact summary: {stderr}"
    );
    assert!(
        !stderr.contains("\"verifier\": \"verified\""),
        "parse-invalid source must not be reported as verified: {stderr}"
    );
}
