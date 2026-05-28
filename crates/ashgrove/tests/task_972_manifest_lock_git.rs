use assert_cmd::Command;

mod support;

#[test]
fn task_972_lock_rejects_unpinned_git_dependency() {
    let project = support::project_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        "[package]\nname = \"app\"\n\n[dependencies.dep]\ngit = \"file:///tmp/dep\"\n",
    )
    .expect("manifest");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["lock", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("unpinned"));
}

#[test]
fn task_972_lock_check_detects_manifest_drift() {
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
        .args(["lock", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .success();

    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.dep]\ngit = \"{}\"\ntag = \"v2\"\n",
            dep.url()
        ),
    )
    .expect("manifest drift");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "lock",
            "--project",
            project.path().to_str().expect("utf8"),
            "--check",
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("lockfile drift"));
}

#[test]
fn task_972_lock_serializes_dependency_values_without_toml_injection() {
    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.dep]\ngit = \"{}\"\ntag = \"v1\\n[[package]]\\nname = \\\"evil\\\"\"\nrev = \"v1\"\n",
            dep.url()
        ),
    )
    .expect("manifest");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["lock", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .success();

    let lock_text = std::fs::read_to_string(project.path().join("ash.lock")).expect("lock");
    let lock: toml::Value = toml::from_str(&lock_text).expect("serialized lock TOML");
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .expect("packages");
    assert_eq!(packages.len(), 1);
    assert_eq!(
        packages[0].get("name").and_then(toml::Value::as_str),
        Some("dep")
    );
    assert_eq!(
        packages[0].get("tag").and_then(toml::Value::as_str),
        Some("v1\n[[package]]\nname = \"evil\"")
    );
}
