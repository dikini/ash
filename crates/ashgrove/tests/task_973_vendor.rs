use assert_cmd::Command;

mod support;

#[test]
fn task_973_vendor_materializes_locked_dependencies_and_check_is_read_only() {
    let project = support::locked_project_fixture();
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "vendor",
            "--project",
            project.path().to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .success();

    let vendored = project.path().join("vendor/ash/dep/provenance.toml");
    assert!(vendored.is_file());
    let before = std::fs::metadata(&vendored)
        .expect("metadata")
        .modified()
        .expect("mtime");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "vendor",
            "--project",
            project.path().to_str().expect("utf8"),
            "--check",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let after = std::fs::metadata(&vendored)
        .expect("metadata")
        .modified()
        .expect("mtime");
    assert_eq!(before, after);
}

#[test]
fn task_973_vendor_rejects_lockfile_package_name_path_traversal() {
    let project = support::project_fixture();
    let roots = support::xdg_fixture();
    let parent = tempfile::tempdir().expect("vendor parent");
    let output = parent.path().join("vendor");
    let escape = parent.path().join("escape/provenance.toml");
    std::fs::write(
        project.path().join("ash.lock"),
        "[[package]]\nname = \"../escape\"\ngit = \"file:///tmp/dep\"\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\n",
    )
    .expect("lock");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "vendor",
            "--project",
            project.path().to_str().expect("utf8"),
            "--output",
            output.to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("invalid package name"));

    assert!(!escape.exists());
}
