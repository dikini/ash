use assert_cmd::Command;

mod support;

#[test]
fn task_981_manifest_lock_preserve_registry_ready_package_metadata() {
    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nkind = \"app\"\n\n[dependencies.dep]\nversion = \"0.2.0\"\nregistry = \"ash.test\"\ngit = \"{}\"\ntag = \"v1\"\n",
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
    let package = first_locked_package(&lock);
    assert_eq!(
        package.get("name").and_then(toml::Value::as_str),
        Some("dep")
    );
    assert_eq!(
        package.get("version").and_then(toml::Value::as_str),
        Some("0.2.0")
    );
    assert_eq!(
        package.get("registry").and_then(toml::Value::as_str),
        Some("ash.test")
    );
    let expected_source = format!("git+{}", dep.url());
    assert_eq!(
        package.get("source").and_then(toml::Value::as_str),
        Some(expected_source.as_str())
    );
    assert_eq!(
        package
            .get("requested")
            .and_then(toml::Value::as_table)
            .and_then(|requested| requested.get("tag"))
            .and_then(toml::Value::as_str),
        Some("v1")
    );
    assert_eq!(
        package
            .get("resolved")
            .and_then(toml::Value::as_table)
            .and_then(|resolved| resolved.get("rev"))
            .and_then(toml::Value::as_str),
        Some(dep.commit("v1").as_str())
    );

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
        .success();
}

#[test]
fn task_981_vendor_provenance_records_registry_metadata_and_detects_drift() {
    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    let commit = dep.commit("v1");
    std::fs::write(
        project.path().join("ash.toml"),
        "[package]\nname = \"app\"\n",
    )
    .expect("manifest");
    std::fs::write(
        project.path().join("ash.lock"),
        format!(
            "[[package]]\nname = \"dep\"\nversion = \"0.2.0\"\nregistry = \"ash.test\"\nsource = \"git+{}\"\ngit = \"{}\"\ntag = \"v1\"\ncommit = \"{}\"\nrequested = {{ tag = \"v1\" }}\nresolved = {{ rev = \"{}\" }}\n",
            dep.url(),
            dep.url(),
            commit,
            commit
        ),
    )
    .expect("lock");
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

    let provenance_text =
        std::fs::read_to_string(project.path().join("vendor/ash/dep/provenance.toml"))
            .expect("provenance");
    let provenance: toml::Value =
        toml::from_str(&provenance_text).expect("serialized provenance TOML");
    assert_eq!(
        provenance.get("version").and_then(toml::Value::as_str),
        Some("0.2.0")
    );
    assert_eq!(
        provenance.get("registry").and_then(toml::Value::as_str),
        Some("ash.test")
    );
    let expected_source = format!("git+{}", dep.url());
    assert_eq!(
        provenance.get("source").and_then(toml::Value::as_str),
        Some(expected_source.as_str())
    );
    assert_eq!(
        provenance
            .get("resolved")
            .and_then(toml::Value::as_table)
            .and_then(|resolved| resolved.get("rev"))
            .and_then(toml::Value::as_str),
        Some(commit.as_str())
    );

    let mut drifted = provenance.as_table().expect("provenance table").clone();
    drifted.insert(
        "registry".to_string(),
        toml::Value::String("other.registry".to_string()),
    );
    std::fs::write(
        project.path().join("vendor/ash/dep/provenance.toml"),
        toml::to_string(&toml::Value::Table(drifted)).expect("drifted provenance TOML"),
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

#[test]
fn task_981_fetch_rejects_non_git_source_even_with_legacy_git() {
    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        "[package]\nname = \"app\"\n",
    )
    .expect("manifest");
    std::fs::write(
        project.path().join("ash.lock"),
        format!(
            "[[package]]\nname = \"dep\"\nversion = \"0.2.0\"\nregistry = \"ash.test\"\nsource = \"registry+https://registry.example.invalid/dep\"\ngit = \"{}\"\ncommit = \"{}\"\n",
            dep.url(),
            dep.commit("v1")
        ),
    )
    .expect("lock");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["fetch", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "ash.lock package source must be git+ URL",
        ));
}

#[test]
fn task_981_fetch_rejects_source_git_mismatch() {
    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        "[package]\nname = \"app\"\n",
    )
    .expect("manifest");
    std::fs::write(
        project.path().join("ash.lock"),
        format!(
            "[[package]]\nname = \"dep\"\nversion = \"0.2.0\"\nsource = \"git+file:///tmp/ash-task-981-other-dep\"\ngit = \"{}\"\ncommit = \"{}\"\n",
            dep.url(),
            dep.commit("v1")
        ),
    )
    .expect("lock");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["fetch", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "ash.lock package source does not match legacy git URL",
        ));
}

#[test]
fn task_981_registry_dependency_resolution_fails_closed_without_hosted_registry_support() {
    let project = support::project_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        "[package]\nname = \"app\"\n\n[dependencies.dep]\nversion = \"^0.2\"\nregistry = \"ash.test\"\n",
    )
    .expect("manifest");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["lock", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "hosted registry dependencies are not supported",
        ))
        .stderr(predicates::str::contains("dep"));
}

fn first_locked_package(lock: &toml::Value) -> &toml::value::Table {
    lock.get("package")
        .and_then(toml::Value::as_array)
        .and_then(|packages| packages.first())
        .and_then(toml::Value::as_table)
        .expect("first package")
}
