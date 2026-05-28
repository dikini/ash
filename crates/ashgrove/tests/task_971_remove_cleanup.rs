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
fn task_971_remove_force_protects_live_daemon_state() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.daemon000001");
    let daemon_dir = roots.state.path().join("ash/daemon");
    std::fs::create_dir_all(&daemon_dir).expect("daemon dir");
    std::fs::write(
        daemon_dir.join("daemon-1.toml"),
        "toolchain_id = \"ash-0.1.0+test.source.daemon000001\"\n",
    )
    .expect("daemon state");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["remove", "ash-0.1.0+test.source.daemon000001", "--force"])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("live daemon"));

    assert!(
        roots
            .toolchain("ash-0.1.0+test.source.daemon000001")
            .is_dir()
    );
}

#[test]
fn task_971_remove_protects_current_project_pin_without_force() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.project000001");
    let project = support::project_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        "[toolchain]\nash = \"ash-0.1.0+test.source.project000001\"\n",
    )
    .expect("project manifest");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["remove", "ash-0.1.0+test.source.project000001"])
        .current_dir(project.path())
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("current project"));

    assert!(
        roots
            .toolchain("ash-0.1.0+test.source.project000001")
            .is_dir()
    );
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
