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
fn task_972_lock_rejects_superseded_manifest_metadata_conflict() {
    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[toolchain]\nash = \"ash-0.1.0+test\"\n\n[dependencies.dep]\ngit = \"{}\"\ntag = \"v1\"\n",
            dep.url()
        ),
    )
    .expect("manifest");
    std::fs::write(
        project.path().join(".ash.toml"),
        "[toolchain]\nash = \"ash-superseded\"\n",
    )
    .expect("superseded manifest");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["lock", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("superseded .ash.toml conflicts"));
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

#[test]
fn task_972_fetch_materializes_exact_lock_commit_in_xdg_cache() {
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

    let checkout = roots
        .cache
        .path()
        .join("ash/git/checkouts")
        .join(format!("dep-{}", support::git_url_digest(&dep.url())))
        .join(&v1);
    let materialized = std::fs::read_to_string(checkout.join("lib.ash")).expect("checkout lib");
    assert!(materialized.contains("pub type Dep = Dep;"));
    assert!(!materialized.contains("Dep2"));
}

#[test]
fn task_972_fetch_uses_existing_lock_commit_when_manifest_tag_moves() {
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
        .args(["lock", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .success();

    dep.force_tag("v1", "v2");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["fetch", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .success();

    let lock_text = std::fs::read_to_string(project.path().join("ash.lock")).expect("lock");
    assert!(lock_text.contains(&format!("commit = \"{v1}\"")));
    let checkout = roots
        .cache
        .path()
        .join("ash/git/checkouts")
        .join(format!("dep-{}", support::git_url_digest(&dep.url())))
        .join(&v1);
    let materialized = std::fs::read_to_string(checkout.join("lib.ash")).expect("checkout lib");
    assert!(!materialized.contains("Dep2"));
}

#[test]
fn task_972_lock_expands_abbreviated_rev_to_full_hash() {
    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    let full = dep.commit("v1");
    let short = &full[..12];
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.dep]\ngit = \"{}\"\nrev = \"{}\"\n",
            dep.url(),
            short
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
    let package = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .and_then(|packages| packages.first())
        .expect("package");
    assert_eq!(
        package.get("commit").and_then(toml::Value::as_str),
        Some(full.as_str())
    );
    assert_eq!(
        package.get("rev").and_then(toml::Value::as_str),
        Some(full.as_str())
    );
}

#[test]
fn task_972_lock_preserves_reserved_trust_fields_on_rewrite() {
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
    std::fs::write(
        project.path().join("ash.lock"),
        "[trust]\nsigning = \"none\"\nfuture = \"kept\"\n\n[[package]]\nname = \"stale\"\ngit = \"file:///tmp/stale\"\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\n",
    )
    .expect("stale lock");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["lock", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .success();

    let lock_text = std::fs::read_to_string(project.path().join("ash.lock")).expect("lock");
    let lock: toml::Value = toml::from_str(&lock_text).expect("serialized lock TOML");
    let trust = lock
        .get("trust")
        .and_then(toml::Value::as_table)
        .expect("trust");
    assert_eq!(
        trust.get("signing").and_then(toml::Value::as_str),
        Some("none")
    );
    assert_eq!(
        trust.get("future").and_then(toml::Value::as_str),
        Some("kept")
    );
}
