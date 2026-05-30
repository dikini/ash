use assert_cmd::Command;

mod support;

#[test]
fn task_980_packaged_update_refreshes_stable_dispatcher_atomically() {
    let roots = support::xdg_fixture();
    let old_id = "ash-0.1.0+tarball.dispatcherold980";
    let new_id = "ash-0.1.0+tarball.dispatchernew980";
    let output = tempfile::tempdir().expect("output");
    let old_archive = packaged_manager_tarball(old_id, output.path(), "exit 41\n");
    let new_archive = packaged_manager_tarball(
        new_id,
        output.path(),
        "printf 'new-dispatch:%s\\n' \"$ASHGROVE_RUNNING_TOOLCHAIN\"\nexit 37\n",
    );

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "install",
            "--from",
            "tarball",
            "--path",
            old_archive.to_str().expect("utf8 archive"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            new_id,
            "--from",
            "tarball",
            "--path",
            new_archive.to_str().expect("utf8 archive"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let paths = support::ashgrove_paths(&roots);
    let launcher_bin = paths.launcher_bin();
    let dispatcher = launcher_bin.join(".ashgrove-dispatcher");
    assert!(dispatcher.is_file(), "stable dispatcher exists");
    assert!(
        launcher_bin
            .read_dir()
            .expect("launcher dir")
            .flatten()
            .all(|entry| !entry
                .file_name()
                .to_string_lossy()
                .starts_with(".ashgrove-dispatcher.tmp-")),
        "atomic refresh must not leave dispatcher temp files"
    );

    let lifecycle = std::fs::read_to_string(launcher_bin.join(".ashgrove-dispatcher.toml"))
        .expect("dispatcher lifecycle metadata");
    assert!(lifecycle.contains(&format!("manager_toolchain_id = \"{new_id}\"")));
    assert!(!lifecycle.contains(old_id));

    Command::new(launcher_bin.join("ash"))
        .envs(roots.env())
        .assert()
        .code(37)
        .stdout(predicates::str::contains(format!("new-dispatch:{new_id}")));
}

#[test]
fn task_980_remove_running_manager_toolchain_fails_for_task_980_aware_packaged_manager() {
    let roots = support::xdg_fixture();
    let old_id = "ash-0.1.0+tarball.managerold980";
    let new_id = "ash-0.1.0+tarball.managernew980";
    let output = tempfile::tempdir().expect("output");
    let old_archive = packaged_manager_tarball(old_id, output.path(), "echo old ash\n");
    let new_archive = packaged_manager_tarball(new_id, output.path(), "echo new ash\n");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "install",
            "--from",
            "tarball",
            "--path",
            old_archive.to_str().expect("utf8 archive"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            new_id,
            "--from",
            "tarball",
            "--path",
            new_archive.to_str().expect("utf8 archive"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let launcher = support::ashgrove_paths(&roots)
        .launcher_bin()
        .join("ashgrove");
    Command::new(launcher)
        .args(["remove", new_id, "--force"])
        .envs(roots.env())
        .env("ASH_TOOLCHAIN", old_id)
        .write_stdin(format!("{new_id}\n"))
        .assert()
        .failure()
        .stderr(predicates::str::contains("running manager"));

    assert!(
        roots.toolchain(new_id).is_dir(),
        "running manager toolchain must remain installed"
    );
}

#[test]
fn task_980_cleanup_dry_run_protects_packaged_dispatcher_owner_after_update() {
    let roots = support::xdg_fixture();
    let old_id = "ash-0.1.0+tarball.cleanupold980";
    let new_id = "ash-0.1.0+tarball.cleanupnew980";
    let output = tempfile::tempdir().expect("output");
    let old_archive = packaged_manager_tarball(old_id, output.path(), "echo old ash\n");
    let new_archive = packaged_manager_tarball(new_id, output.path(), "echo new ash\n");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "install",
            "--from",
            "tarball",
            "--path",
            old_archive.to_str().expect("utf8 archive"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            new_id,
            "--from",
            "tarball",
            "--path",
            new_archive.to_str().expect("utf8 archive"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let launcher = support::ashgrove_paths(&roots)
        .launcher_bin()
        .join("ashgrove");
    let cleanup = Command::new(launcher)
        .args(["cleanup", "--old-toolchains", "--dry-run"])
        .envs(roots.env())
        .env("ASH_TOOLCHAIN", old_id)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(cleanup).expect("cleanup stdout utf8");

    assert!(
        stdout.contains(&format!("protected running manager {old_id}")),
        "cleanup output must run through the selected old manager:\n{stdout}"
    );
    assert!(
        stdout.contains(&format!("protected running manager {new_id}")),
        "cleanup output must protect packaged dispatcher owner:\n{stdout}"
    );
    assert!(
        !stdout.contains(&format!(
            "would remove {}",
            roots.toolchain(new_id).display()
        )),
        "cleanup dry-run must not list packaged dispatcher owner as removable:\n{stdout}"
    );
    assert!(
        roots.toolchain(new_id).is_dir(),
        "dry-run cleanup must leave packaged dispatcher owner installed"
    );
}

#[test]
fn task_980_default_switch_does_not_rewrite_project_manifest() {
    let roots = support::xdg_fixture();
    let old_id = "ash-0.1.0+test.source.projectold980";
    let new_id = "ash-0.1.0+test.source.projectnew980";
    support::install_fake_toolchain(&roots, old_id);
    support::install_fake_toolchain(&roots, new_id);
    let project = support::project_fixture();
    let manifest = project.path().join("ash.toml");
    let lockfile = project.path().join("ash.lock");
    let manifest_before =
        format!("[package]\nname = \"task980\"\n\n[toolchain]\nash = \"{old_id}\"\n");
    let lock_before = "# task 980 lock sentinel\n";
    std::fs::write(&manifest, &manifest_before).expect("project manifest");
    std::fs::write(&lockfile, lock_before).expect("project lock");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["default", new_id])
        .current_dir(project.path())
        .envs(roots.env())
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&manifest).expect("manifest after default"),
        manifest_before
    );
    assert_eq!(
        std::fs::read_to_string(&lockfile).expect("lock after default"),
        lock_before
    );
    let selector =
        std::fs::read_to_string(roots.config.path().join("ash/toolchains.toml")).expect("selector");
    assert!(selector.contains(&format!("default = \"{new_id}\"")));
    assert!(!selector.contains(project.path().to_str().expect("utf8 project")));
}

fn packaged_manager_tarball(
    id: &str,
    output: &std::path::Path,
    ash_body: &str,
) -> std::path::PathBuf {
    let ash = tempfile::NamedTempFile::new().expect("ash");
    support::write_tool_script(ash.path(), ash_body);

    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");
    let package = std::process::Command::new(repo_root.join("scripts/package-ash-toolchain.sh"))
        .args([
            "--toolchain-id",
            id,
            "--output-dir",
            output.to_str().expect("utf8 output"),
        ])
        .env("ASH_PACKAGE_ASH_BIN", ash.path())
        .env(
            "ASH_PACKAGE_ASHGROVE_BIN",
            assert_cmd::cargo::cargo_bin("ashgrove"),
        )
        .output()
        .expect("run package producer");
    assert!(
        package.status.success(),
        "producer failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&package.stdout),
        String::from_utf8_lossy(&package.stderr)
    );
    let stdout = String::from_utf8(package.stdout).expect("producer stdout utf8");
    let archive = stdout
        .lines()
        .find_map(|line| line.strip_prefix("archive="))
        .map(std::path::PathBuf::from)
        .expect("archive output");
    assert!(archive.is_file(), "archive exists at {}", archive.display());
    archive
}
