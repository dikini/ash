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
        .stdout(predicate::str::contains("fmt"))
        .stdout(predicate::str::contains("daemon"))
        .stdout(predicate::str::contains("template"));
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
fn test_check_rejects_undefined_pure_function_calls() {
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), "fn main() -> Int { missing(1) }\n").unwrap();

    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["check", file.path().to_str().unwrap()]);
    cmd.assert().failure().stdout(predicate::str::contains(
        "call to unknown function 'missing'",
    ));
}

#[test]
fn test_check_rejects_capability_as_pure_function_syntax() {
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), "fn main() -> String { Stdio::read_line() }\n").unwrap();

    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["check", file.path().to_str().unwrap()]);
    cmd.assert().failure().stdout(
        predicate::str::contains("'Stdio::read_line' is a capability, not a function")
            .and(predicate::str::contains("use capability syntax instead of")),
    );
}

#[test]
fn check_rejects_broken_dispatch_named_entry_file_as_module() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("dispatch.ash");
    fs::write(&path, "fn broken( {\n  this is not valid ash\n}\n").unwrap();

    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["check", path.to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("FAILED"));
}

#[test]
fn check_rejects_broken_user_std_llm_dispatch_entry_file_as_module() {
    let dir = TempDir::new().unwrap();
    let std_llm = dir.path().join("std").join("llm");
    fs::create_dir_all(&std_llm).unwrap();
    let path = std_llm.join("dispatch.ash");
    fs::write(&path, "fn broken( {\n  this is not valid ash\n}\n").unwrap();

    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["check", path.to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("FAILED"));
}

#[test]
fn check_rejects_broken_mod_named_entry_file_as_module() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("mod.ash");
    fs::write(&path, "fn broken( {\n  this is not valid ash\n}\n").unwrap();

    let mut cmd = Command::cargo_bin("ash").unwrap();
    cmd.args(["check", path.to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("FAILED"));
}
