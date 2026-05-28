use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn task_966_help_lists_required_alpha_commands() {
    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("install"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("default"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("current"))
        .stdout(predicate::str::contains("remove"))
        .stdout(predicate::str::contains("cleanup"))
        .stdout(predicate::str::contains("fetch"))
        .stdout(predicate::str::contains("lock"))
        .stdout(predicate::str::contains("vendor"));
}

#[test]
fn task_966_bare_version_install_fails_closed() {
    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args(["install", "0.1.0-alpha.1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("release index"));
}
