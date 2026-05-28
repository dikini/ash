use assert_cmd::Command;
use predicates::prelude::*;

mod support;

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
fn task_966_all_required_subcommands_have_help_smoke() {
    for subcommand in [
        "install", "update", "default", "list", "current", "remove", "cleanup", "fetch", "lock",
        "vendor",
    ] {
        Command::cargo_bin("ashgrove")
            .expect("ashgrove binary")
            .args([subcommand, "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains(subcommand));
    }
}

#[test]
fn task_966_bare_version_install_fails_closed() {
    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args(["install", "0.1.0-alpha.1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("release index"));
}

#[test]
fn task_966_bare_version_update_fails_closed_before_release_index_policy() {
    let roots = support::xdg_fixture();
    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args(["update", "0.1.0-alpha.1"])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicate::str::contains("release index"));
}

#[test]
fn task_966_isolated_cli_smoke_tests_are_non_zero_and_fail_closed() {
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove binary")
        .arg("current")
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicate::str::contains("no default Ash toolchain"));

    Command::cargo_bin("ashgrove")
        .expect("ashgrove binary")
        .arg("fetch")
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicate::str::contains("ash.toml"));

    Command::cargo_bin("ashgrove")
        .expect("ashgrove binary")
        .arg("cleanup")
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not implemented"));
}

#[test]
fn task_966_incomplete_commands_fail_closed_in_isolated_xdg_roots() {
    for args in [
        vec!["install"],
        vec!["update"],
        vec!["default"],
        vec!["remove"],
        vec!["cleanup"],
        vec!["current"],
        vec!["fetch", "--project", "missing-project"],
        vec!["lock", "--project", "missing-project"],
        vec!["vendor", "--project", "missing-project"],
    ] {
        let roots = support::xdg_fixture();

        Command::cargo_bin("ashgrove")
            .expect("ashgrove binary")
            .args(&args)
            .envs(roots.env())
            .assert()
            .failure();

        assert!(
            roots.home.path().exists(),
            "fixture home should stay isolated for {args:?}"
        );
    }
}
