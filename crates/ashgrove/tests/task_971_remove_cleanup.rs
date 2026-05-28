use assert_cmd::Command;

mod support;

#[test]
fn task_971_remove_protects_default_and_running_manager() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.protected001");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["default", "ash-0.1.0+test.source.protected001"])
        .envs(roots.env())
        .assert()
        .success();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["remove", "ash-0.1.0+test.source.protected001"])
        .envs(roots.env())
        .env(
            "ASHGROVE_RUNNING_TOOLCHAIN",
            "ash-0.1.0+test.source.protected001",
        )
        .assert()
        .failure()
        .stderr(predicates::str::contains("running manager"));
}

#[test]
fn task_971_cleanup_dry_run_is_non_destructive() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.orphan000001");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["cleanup", "--dry-run", "--old-toolchains"])
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains("would remove"));

    assert!(
        roots
            .toolchain("ash-0.1.0+test.source.orphan000001")
            .is_dir()
    );
}
