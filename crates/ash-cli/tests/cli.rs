use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Ash"))
        .stdout(predicate::str::contains("check"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("trace"))
        .stdout(predicate::str::contains("repl"))
        .stdout(predicate::str::contains("dot"));
}

#[test]
fn test_check_help() {
    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["check", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Type check"));
}

#[test]
fn test_run_help() {
    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["run", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Execute"));
}

#[test]
fn test_dot_help() {
    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["dot", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Graphviz"));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_check_nonexistent_file() {
    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["check", "nonexistent.ash"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_dot_nonexistent_file() {
    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["dot", "nonexistent.ash"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_check_rejects_undefined_pure_function_calls() {
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), "workflow main() -> Int { ret missing(1) }\n").unwrap();

    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["check", file.path().to_str().unwrap()]);
    cmd.assert().failure().stdout(
        predicate::str::contains("unknown function")
            .or(predicate::str::contains("call to unknown function")),
    );
}

#[test]
fn test_check_rejects_capability_as_pure_function_syntax() {
    let file = NamedTempFile::new().unwrap();
    fs::write(
        file.path(),
        "workflow main() -> String { ret Stdio::read_line() }\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["check", file.path().to_str().unwrap()]);
    cmd.assert().failure().stdout(
        predicate::str::contains("capability").and(predicate::str::contains("not a function")),
    );
}

#[test]
fn check_rejects_broken_dispatch_named_workflow_file_as_module() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("dispatch.ash");
    fs::write(&path, "workflow broken( {\n  this is not valid ash\n}\n").unwrap();

    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["check", path.to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("FAILED"));
}

#[test]
fn check_rejects_broken_user_std_llm_dispatch_workflow_file_as_module() {
    let dir = TempDir::new().unwrap();
    let std_llm = dir.path().join("std").join("llm");
    fs::create_dir_all(&std_llm).unwrap();
    let path = std_llm.join("dispatch.ash");
    fs::write(&path, "workflow broken( {\n  this is not valid ash\n}\n").unwrap();

    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["check", path.to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("FAILED"));
}

#[test]
fn check_rejects_broken_mod_named_workflow_file_as_module() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("mod.ash");
    fs::write(&path, "workflow broken( {\n  this is not valid ash\n}\n").unwrap();

    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["check", path.to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("FAILED"));
}
