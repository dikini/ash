//! Integration tests for `ash trace` observable output.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn trace_stdout_is_only_trace_document_with_minimal_contract_fields() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("main.ash");
    fs::write(&workflow_path, "workflow main { ret 0; }\n").expect("write workflow");

    let output = Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("trace")
        .arg(&workflow_path)
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).expect("utf8 stdout");
    let json: Value = serde_json::from_str(&stdout).expect("trace output is json");

    assert!(json.get("trace_id").is_some(), "missing trace_id: {stdout}");
    assert!(json.get("workflow").is_some(), "missing workflow: {stdout}");
    assert!(
        json.get("started_at").is_some(),
        "missing started_at: {stdout}"
    );
    assert!(json.get("events").is_some(), "missing events: {stdout}");
    assert!(
        json.get("final_value").is_some(),
        "missing final_value: {stdout}"
    );
}

#[test]
fn trace_output_file_contains_only_document() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("main.ash");
    let output_path = temp.path().join("trace.json");
    fs::write(&workflow_path, "workflow main { ret 1; }\n").expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("trace")
        .arg(&workflow_path)
        .arg("--output")
        .arg(&output_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let file_output = fs::read_to_string(output_path).expect("read trace output");
    let json: Value = serde_json::from_str(&file_output).expect("trace file is json");
    assert!(json.get("trace_id").is_some());
    assert!(json.get("final_value").is_some());
}
