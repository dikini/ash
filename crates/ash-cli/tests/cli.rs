use assert_cmd::Command;
use predicates::prelude::*;

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
