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

#[test]
fn task_971_cleanup_project_dry_run_plans_without_touching_project_files() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.projectdryrun");
    let project = support::project_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        "[toolchain]\nash = \"ash-0.1.0+test.source.projectdryrun\"\n",
    )
    .expect("project manifest");
    std::fs::write(project.path().join("ash.lock"), "# locked\n").expect("project lock");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "cleanup",
            "--project",
            project.path().to_str().expect("utf8"),
            "--dry-run",
            "--old-toolchains",
        ])
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains("protected project"));

    assert!(project.path().join("ash.toml").is_file());
    assert!(project.path().join("ash.lock").is_file());
    assert!(
        roots
            .toolchain("ash-0.1.0+test.source.projectdryrun")
            .is_dir()
    );
}

#[test]
fn task_971_cleanup_project_bare_dry_run_prints_non_destructive_plan() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.bareplan01");
    let project = support::project_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        "[toolchain]\nash = \"ash-0.1.0+test.source.bareplan01\"\n",
    )
    .expect("project manifest");
    std::fs::write(project.path().join("ash.lock"), "# locked\n").expect("project lock");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "cleanup",
            "--project",
            project.path().to_str().expect("utf8"),
            "--dry-run",
        ])
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains("protected project"))
        .stdout(predicates::str::contains(
            "no destructive cleanup will occur",
        ));

    assert_eq!(
        std::fs::read_to_string(project.path().join("ash.toml")).expect("manifest"),
        "[toolchain]\nash = \"ash-0.1.0+test.source.bareplan01\"\n"
    );
    assert_eq!(
        std::fs::read_to_string(project.path().join("ash.lock")).expect("lock"),
        "# locked\n"
    );
    assert!(roots.toolchain("ash-0.1.0+test.source.bareplan01").is_dir());
}

#[test]
fn task_971_cleanup_old_toolchains_preserves_protected_and_removes_unprotected() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.keepdefault");
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.removeme0001");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["default", "ash-0.1.0+test.source.keepdefault"])
        .envs(roots.env())
        .assert()
        .success();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["cleanup", "--old-toolchains"])
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains("removed"))
        .stdout(predicates::str::contains("protected default"));

    assert!(
        roots
            .toolchain("ash-0.1.0+test.source.keepdefault")
            .is_dir()
    );
    assert!(
        !roots
            .toolchain("ash-0.1.0+test.source.removeme0001")
            .exists()
    );
}

#[test]
fn task_971_cleanup_cache_flag_removes_only_ash_cache_children() {
    let roots = support::xdg_fixture();
    let ash_cache = roots.cache.path().join("ash");
    std::fs::create_dir_all(ash_cache.join("downloads")).expect("downloads");
    std::fs::write(ash_cache.join("downloads/archive.tar.gz"), "cache").expect("cache file");
    std::fs::create_dir_all(roots.cache.path().join("outside-ash")).expect("outside");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["cleanup", "--cache"])
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains("removed cache"));

    assert!(!ash_cache.join("downloads").exists());
    assert!(roots.cache.path().join("outside-ash").is_dir());
}

#[test]
fn task_971_cleanup_orphans_removes_invalid_toolchain_dirs_only() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.valid000001");
    let orphan = roots.toolchain("ash-0.1.0+test.source.invalid001");
    std::fs::create_dir_all(&orphan).expect("orphan dir");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["cleanup", "--orphans"])
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains("removed orphan"));

    assert!(
        roots
            .toolchain("ash-0.1.0+test.source.valid000001")
            .is_dir()
    );
    assert!(!orphan.exists());
}

#[test]
fn task_971_cleanup_old_toolchains_preserves_live_running_and_selector_pins() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.livecleanup");
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.runningclean");
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.selectorpin");

    let daemon_dir = roots.state.path().join("ash/daemon");
    std::fs::create_dir_all(&daemon_dir).expect("daemon dir");
    std::fs::write(
        daemon_dir.join("daemon-1.toml"),
        "toolchain_id = \"ash-0.1.0+test.source.livecleanup\"\n",
    )
    .expect("daemon state");

    let selector_dir = roots.config.path().join("ash");
    std::fs::create_dir_all(&selector_dir).expect("selector dir");
    std::fs::write(
        selector_dir.join("toolchains.toml"),
        "[projects]\n\"/tmp/known-project\" = \"ash-0.1.0+test.source.selectorpin\"\n",
    )
    .expect("selector metadata");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["cleanup", "--old-toolchains"])
        .envs(roots.env())
        .env(
            "ASHGROVE_RUNNING_TOOLCHAIN",
            "ash-0.1.0+test.source.runningclean",
        )
        .assert()
        .success()
        .stdout(predicates::str::contains("protected live daemon"))
        .stdout(predicates::str::contains("protected running manager"))
        .stdout(predicates::str::contains("protected project"));

    assert!(
        roots
            .toolchain("ash-0.1.0+test.source.livecleanup")
            .is_dir()
    );
    assert!(
        roots
            .toolchain("ash-0.1.0+test.source.runningclean")
            .is_dir()
    );
    assert!(
        roots
            .toolchain("ash-0.1.0+test.source.selectorpin")
            .is_dir()
    );
}
