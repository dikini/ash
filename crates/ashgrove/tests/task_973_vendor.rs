use assert_cmd::Command;

mod support;

#[test]
fn task_973_vendor_materializes_locked_dependencies_and_check_is_read_only() {
    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.dep]\ngit = \"{}\"\ntag = \"v1\"\n",
            dep.url()
        ),
    )
    .expect("manifest");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["fetch", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .success();

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

#[test]
fn task_973_vendor_materializes_package_content_from_locked_cache_commit() {
    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    let v1 = dep.commit("v1");
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.dep]\ngit = \"{}\"\ntag = \"v1\"\n",
            dep.url()
        ),
    )
    .expect("manifest");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["fetch", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .success();

    dep.force_tag("v1", "v2");

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

    let vendored_lib =
        std::fs::read_to_string(project.path().join("vendor/ash/dep/lib.ash")).expect("lib");
    assert!(vendored_lib.contains("pub type Dep = Dep;"));
    assert!(!vendored_lib.contains("Dep2"));

    let provenance = std::fs::read_to_string(project.path().join("vendor/ash/dep/provenance.toml"))
        .expect("provenance");
    assert!(provenance.contains(&format!("commit = \"{v1}\"")));

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

    std::fs::write(project.path().join("vendor/ash/dep/lib.ash"), "tampered\n")
        .expect("tamper vendor content");
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
        .failure()
        .stderr(predicates::str::contains("vendor content does not match"));

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
    std::fs::create_dir_all(project.path().join("vendor/ash/evil")).expect("evil vendor dir");
    std::fs::write(
        project.path().join("vendor/ash/evil/lib.ash"),
        "pub type Evil = Evil;\n",
    )
    .expect("evil vendor content");
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
        .failure()
        .stderr(predicates::str::contains("unexpected package"));
}

#[test]
fn task_973_vendor_check_rejects_stale_provenance() {
    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.dep]\ngit = \"{}\"\ntag = \"v1\"\n",
            dep.url()
        ),
    )
    .expect("manifest");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["fetch", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .success();
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

    std::fs::write(
        project.path().join("vendor/ash/dep/provenance.toml"),
        "name = \"dep\"\ngit = \"file:///tmp/other\"\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\n",
    )
    .expect("tamper provenance");

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
        .failure()
        .stderr(predicates::str::contains("provenance does not match"));
}
