use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn ash_run_executes_entry_through_one_shot_runtime_kernel() {
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

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run")
        .arg("--dry-run")
        .arg(&entry_path)
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
    let entry_path = temp.path().join("identity.ash");
    fs::write(
        &entry_path,
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
        "#,
    )
    .expect("write entry");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    let output = cmd
        .arg("run")
        .arg(&entry_path)
        .arg("--format")
        .arg("json")
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.trim().is_empty(),
        "entry output should be empty for Ok {{ value: {{}} }}: {stdout}"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    let report: Value = serde_json::from_str(&stderr).expect("kernel report json on stderr");
    assert_eq!(report["host_mode"], "OneShot");
    assert_eq!(report["application"], "main");
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
        "checked_function_artifact"
    );
    assert_eq!(
        report["provider_registry"]["grants_admission_authority"],
        false
    );
}

#[test]
fn ash_run_reports_checked_callable_entrypoint_metadata_for_fn_main_source() {
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

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    let output = cmd
        .arg("run")
        .arg("--dry-run")
        .arg(&app_path)
        .arg("--format")
        .arg("json")
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    let report: Value = serde_json::from_str(&stderr).expect("kernel report json on stderr");
    let entrypoint = &report["artifact_summary"]["entrypoint"];
    assert_eq!(entrypoint["kind"], "checked_callable");
    assert_eq!(entrypoint["name"], "main");
    assert_eq!(entrypoint["relative_module_path"], "app.ash");
    assert_eq!(entrypoint["callable_identity"], "callable:app.ash::main");
    assert_eq!(
        entrypoint["runtime_target_identity"],
        "runtime-target:application-entry:main"
    );
    assert_eq!(
        report["artifact_summary"]["invocation_packet"]["entrypoint"],
        *entrypoint
    );
    assert!(
        report["artifact_summary"]["invocation_packet"]["source_identity"]
            .as_str()
            .is_some_and(|identity| !identity.is_empty())
    );
    assert!(
        report["artifact_summary"]["invocation_packet"]["check_identity"]
            .as_str()
            .is_some_and(|identity| !identity.is_empty())
    );
    assert!(
        report["artifact_summary"]["invocation_packet"]["runtime_target_identity"]
            .as_str()
            .is_some_and(|identity| !identity.is_empty())
    );
    assert_eq!(
        report["application_report"]["terminal_outcome"]["status"],
        "succeeded"
    );
    assert_eq!(
        report["application_report"]["source_identity"],
        report["artifact_summary"]["invocation_packet"]["source_identity"]
    );
    assert_eq!(
        report["application_report"]["check_identity"],
        report["artifact_summary"]["invocation_packet"]["check_identity"]
    );
    assert_eq!(
        report["application_report"]["entrypoint_identity"],
        entrypoint["runtime_target_identity"]
    );
    assert_eq!(
        report["application_report"]["trace_bundle"]["admission_facts"][0],
        "admission_profile:admission-profile:empty"
    );
    assert!(
        report["application_report"]["trace_bundle"]["boundary_facts"]
            .as_array()
            .expect("boundary facts")
            .iter()
            .any(|fact| fact == "boundary_source:cli:runtime-boundary")
    );
    assert_eq!(
        report["application_report"]["trace_bundle"]["grants_authority"],
        false
    );
    assert_eq!(
        report["application_report"]["trace_bundle"]["mutates_authority"],
        false
    );
    assert_eq!(report["application_report"]["grants_authority"], false);
    assert_eq!(report["application_report"]["mutates_authority"], false);
}

#[test]
fn ash_run_artifact_uses_checked_non_pure_main_function_provenance() {
    let temp = tempdir().expect("tempdir");
    let app_path = temp.path().join("non_pure.ash");
    fs::write(
        &app_path,
        r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> where row { process spawn } {
            Ok { value: {} }
        }
        "#,
    )
    .expect("write non-pure fn main source");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    let output = cmd
        .arg("run")
        .arg("--dry-run")
        .arg(&app_path)
        .arg("--format")
        .arg("json")
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    let report: Value = serde_json::from_str(&stderr).expect("kernel report json on stderr");
    let artifact = &report["artifact_summary"];

    assert_eq!(
        artifact["tcir"]["carrier_scope"], "checked_function_artifact",
        "the artifact must be produced from the checked fn main carrier"
    );
    assert_eq!(
        artifact["tcir"]["function_artifact"]["function_identity"],
        "callable:non_pure.ash::main"
    );
    assert_eq!(
        artifact["tcir"]["function_artifact"]["effect_row"], "process spawn",
        "the declared non-pure row must come from checking the source, not request text"
    );
    assert_eq!(
        artifact["tcir"]["target_display"],
        "function:callable:non_pure.ash::main"
    );
    assert_eq!(
        artifact["tcir"]["evidence_key"],
        "RuntimeKernelFunction<callable:non_pure.ash::main;process spawn>"
    );
    assert_eq!(
        artifact["amir"]["provenance"]["function_artifact"]["function_identity"],
        "callable:non_pure.ash::main"
    );
    assert_eq!(
        artifact["amir"]["provenance"]["function_artifact"]["effect_row"],
        "process spawn"
    );
    assert_eq!(artifact["amir"]["instruction_count"], 1);
}

#[test]
fn ash_run_reports_provider_boundary_bindings_without_authority_grants() {
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

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    let output = cmd
        .arg("run")
        .arg("--dry-run")
        .arg(&app_path)
        .arg("--")
        .arg("first")
        .arg("second")
        .env("ASH_RUNTIME_KERNEL_REPORT", "json")
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    let report: Value = serde_json::from_str(&stderr).expect("kernel report json on stderr");
    let boundary_bindings = &report["artifact_summary"]["invocation_packet"]["boundary_bindings"];
    assert_eq!(boundary_bindings["boundary_source"], "cli:runtime-boundary");
    assert_eq!(boundary_bindings["providers"][0], "Args:0");
    assert_eq!(boundary_bindings["providers"][1], "Args:1");
    assert_eq!(boundary_bindings["grants_authority"], false);
    assert_eq!(report["admission"]["capability_grants"], 0);
    assert_eq!(report["admission"]["resource_grants"], 0);
    assert_eq!(report["admission"]["action_grants"], 0);
    assert_eq!(
        report["provider_registry"]["grants_admission_authority"],
        false
    );
}

#[test]
fn ash_run_does_not_emit_verified_artifact_report_before_parse_check_success() {
    let temp = tempdir().expect("tempdir");
    let entry_path = temp.path().join("bad.ash");
    fs::write(&entry_path, "fn main() {").expect("write malformed entry");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    let output = cmd
        .arg("run")
        .arg(&entry_path)
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
