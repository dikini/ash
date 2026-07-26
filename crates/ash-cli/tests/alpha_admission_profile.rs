use assert_cmd::Command;
use serde_json::json;
use std::fs;
use tempfile::tempdir;

#[test]
fn reject_admission_profile_reports_admission_failure_before_body_output() {
    let temp = tempdir().expect("tempdir");
    let entry_path = temp.path().join("entry.ash");
    let output_path = temp.path().join("sentinel.json");
    fs::write(
        &entry_path,
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        "#,
    )
    .expect("write entry");

    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("run")
        .arg("--admission-profile")
        .arg("reject")
        .arg("--output")
        .arg(&output_path)
        .arg("--format")
        .arg("json")
        .arg(&entry_path)
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .failure()
        .get_output()
        .clone();

    let terminal: serde_json::Value = serde_json::from_slice(
        &fs::read(&output_path).expect("rejected admission writes its terminal envelope"),
    )
    .expect("rejected admission terminal envelope is JSON");
    assert_eq!(
        terminal,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "admission",
            "outcome": "rejected"
        }),
        "rejected admission must not create a body-derived result"
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
    let json_start = stderr
        .find('{')
        .expect("kernel rejection report starts with json object");
    let json_end = stderr
        .rfind("\n}")
        .expect("kernel rejection report ends with json object")
        + 2;
    let report_text = &stderr[json_start..json_end];
    let report: serde_json::Value =
        serde_json::from_str(report_text).expect("kernel rejection report json on stderr");
    assert_eq!(
        report["application_report"]["terminal_outcome"]["status"],
        "rejected"
    );
    assert_eq!(
        report["application_report"]["admission_profile"]["profile_identity"],
        "admission-profile:reject"
    );
    assert_eq!(
        report["application_report"]["trace_bundle"]["grants_authority"],
        false
    );
    assert_eq!(
        report["application_report"]["trace_bundle"]["mutates_authority"],
        false
    );
    assert!(
        !stderr.contains("parse error") && !stderr.contains("runtime error"),
        "admission rejection must be distinct from parse/body failure: {stderr}"
    );
}

#[test]
fn default_empty_admission_profile_remains_admitted() {
    let temp = tempdir().expect("tempdir");
    let entry_path = temp.path().join("entry.ash");
    fs::write(
        &entry_path,
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        "#,
    )
    .expect("write entry");

    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("run")
        .arg(&entry_path)
        .arg("--format")
        .arg("json")
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .success()
        .get_output()
        .clone();

    let terminal: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("admitted entry emits a canonical terminal envelope");
    assert_eq!(
        terminal,
        json!({
            "schema_version": 1,
            "kind": "return",
            "value": {}
        }),
        "Ok unit entry must remain a unit return, not a body-value result"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    let report: serde_json::Value =
        serde_json::from_str(&stderr).expect("kernel report json on stderr");
    assert_eq!(report["admission"]["status"], "admitted");
}

#[test]
fn allow_admission_profile_is_reported_as_non_authority_boundary_metadata() {
    let temp = tempdir().expect("tempdir");
    let app_path = temp.path().join("app.ash");
    fs::write(
        &app_path,
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        "#,
    )
    .expect("write fn main source");

    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("run")
        .arg("--admission-profile")
        .arg("allow")
        .arg("--dry-run")
        .arg(&app_path)
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    let report: serde_json::Value =
        serde_json::from_str(&stderr).expect("kernel report json on stderr");
    assert_eq!(report["admission"]["status"], "admitted");
    assert_eq!(report["admission"]["profile"], "allow");
    assert_eq!(report["admission"]["profile_boundary"]["name"], "allow");
    assert_eq!(
        report["admission"]["profile_boundary"]["profile_identity"],
        "admission-profile:allow"
    );
    assert_eq!(
        report["admission"]["profile_boundary"]["boundary_source"],
        "cli:--admission-profile"
    );
    assert_eq!(
        report["admission"]["profile_boundary"]["grants_authority"],
        false
    );
    assert_eq!(
        report["artifact_summary"]["invocation_packet"]["admission_profile"],
        report["admission"]["profile_boundary"]
    );
    assert_eq!(report["admission"]["capability_grants"], 0);
    assert_eq!(report["admission"]["resource_grants"], 0);
    assert_eq!(report["admission"]["action_grants"], 0);
    assert_eq!(
        report["provider_registry"]["grants_admission_authority"],
        false
    );
}

#[test]
fn execution_failure_still_emits_runtime_kernel_report() {
    let temp = tempdir().expect("tempdir");
    let entry_path = temp.path().join("ordinary-error.ash");
    fs::write(
        &entry_path,
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> { Err { error: RuntimeError(1, "missing_name") } }
        "#,
    )
    .expect("write entry");

    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("run")
        .arg(&entry_path)
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .failure()
        .get_output()
        .clone();

    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stdout.is_empty(),
        "runtime failure must not emit a body value"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    let json_start = stderr
        .find('{')
        .unwrap_or_else(|| panic!("kernel report starts with json object; stderr={stderr}"));
    let json_end = stderr
        .rfind("\n}")
        .expect("kernel report ends with json object")
        + 2;
    let report_text = &stderr[json_start..json_end];
    let report: serde_json::Value =
        serde_json::from_str(report_text).expect("kernel report json on execution failure");
    assert_eq!(report["host_mode"], "OneShot");
    assert_eq!(report["application"], "main");
    assert_eq!(report["admission"]["status"], "admitted");
    assert_eq!(
        report["application_report"]["terminal_outcome"]["status"],
        "failed"
    );
    assert!(
        report["application_report"]["terminal_outcome"]["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("runtime error"))
    );
    assert_eq!(
        report["artifact_summary"]["tcir"]["carrier_scope"],
        "checked_function_artifact"
    );
}
