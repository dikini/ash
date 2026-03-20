//! Integration tests for `ash run` observable output.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn run_success_stdout_is_only_final_value() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("main.ash");
    fs::write(&workflow_path, "workflow main { ret 0; }\n").expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .success()
        .stdout(predicate::eq("0\n"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn run_output_file_writes_value_without_status_banner() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("main.ash");
    let output_path = temp.path().join("result.txt");
    fs::write(&workflow_path, "workflow main { ret 7; }\n").expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run")
        .arg(&workflow_path)
        .arg("--output")
        .arg(&output_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert_eq!(fs::read_to_string(output_path).expect("read output"), "7");
}

#[test]
fn run_parse_error_is_observably_distinct() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("broken.ash");
    fs::write(&workflow_path, "workflow main { ret ; }\n").expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("parse error"));
}

#[test]
fn run_trace_parse_error_is_observably_distinct() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("broken-trace.ash");
    fs::write(&workflow_path, "workflow main { ret ; }\n").expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg("--trace").arg(&workflow_path);

    cmd.assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("parse error"));
}
