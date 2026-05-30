use ashgrove::{SelectorMetadata, ToolchainId};
use assert_cmd::Command;
use predicates::prelude::*;

mod support;

#[test]
fn task_982_cleanup_dry_run_reports_lock_reachable_cache_and_unreachable_entries() {
    let roots = support::xdg_fixture();
    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    let locked_commit = dep.commit("v1");
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.dep]\ngit = \"{}\"\ntag = \"v1\"\n",
            dep.url()
        ),
    )
    .expect("manifest");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["fetch", "--project", project.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .success();

    let reachable_checkout = roots
        .cache
        .path()
        .join("ash/git/checkouts")
        .join(format!("dep-{}", support::git_url_digest(&dep.url())))
        .join(&locked_commit);
    let unreachable_checkout = roots
        .cache
        .path()
        .join("ash/git/checkouts/orphan-deadbeef/0123456789abcdef0123456789abcdef01234567");
    std::fs::create_dir_all(&unreachable_checkout).expect("unreachable checkout");
    std::fs::write(
        unreachable_checkout.join("lib.ash"),
        "pub type Orphan = Orphan;\n",
    )
    .expect("unreachable checkout content");

    let assert = Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["cleanup", "--cache", "--dry-run"])
        .current_dir(project.path())
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "reachable cache {}",
            reachable_checkout.display()
        )))
        .stdout(predicate::str::contains(format!(
            "would remove cache {}",
            unreachable_checkout.display()
        )));

    assert.stdout(predicate::str::contains("would remove cache").count(1));
    assert!(reachable_checkout.is_dir());
    assert!(unreachable_checkout.is_dir());
    assert!(project.path().join("ash.toml").is_file());
    assert!(project.path().join("ash.lock").is_file());
}

#[test]
fn task_982_cleanup_preserves_known_project_lock_referenced_checkouts_and_toolchains() {
    let roots = support::xdg_fixture();
    let known = support::project_fixture();
    let dep = support::git_dep_fixture();
    let locked_commit = dep.commit("v1");
    let keep_id = "ash-0.1.0+test.source.keep982001";
    let drop_id = "ash-0.1.0+test.source.drop982001";
    support::install_fake_toolchain(&roots, keep_id);
    support::install_fake_toolchain(&roots, drop_id);
    std::fs::write(
        known.path().join("ash.toml"),
        format!(
            "[package]\nname = \"known\"\n\n[toolchain]\nash = \"{keep_id}\"\n\n[dependencies.dep]\ngit = \"{}\"\ntag = \"v1\"\n",
            dep.url()
        ),
    )
    .expect("known manifest");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["fetch", "--project", known.path().to_str().expect("utf8")])
        .envs(roots.env())
        .assert()
        .success();

    let reachable_checkout = roots
        .cache
        .path()
        .join("ash/git/checkouts")
        .join(format!("dep-{}", support::git_url_digest(&dep.url())))
        .join(&locked_commit);
    let unreachable_checkout = roots
        .cache
        .path()
        .join("ash/git/checkouts/orphan-deadbeef/0123456789abcdef0123456789abcdef01234567");
    std::fs::create_dir_all(&unreachable_checkout).expect("unreachable checkout");
    let mut selectors = SelectorMetadata::empty();
    selectors.record_project_toolchain(
        known.path(),
        ToolchainId::parse(keep_id).expect("keep toolchain id"),
    );
    selectors
        .write_to_path(&roots.config.path().join("ash/toolchains.toml"))
        .expect("selector metadata");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["cleanup", "--cache", "--old-toolchains"])
        .envs(roots.env())
        .write_stdin("yes\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("protected project"))
        .stdout(predicate::str::contains("reachable cache"))
        .stdout(predicate::str::contains("removed"));

    assert!(reachable_checkout.is_dir());
    assert!(!unreachable_checkout.exists());
    assert!(roots.toolchain(keep_id).is_dir());
    assert!(!roots.toolchain(drop_id).exists());
    assert!(known.path().join("ash.toml").is_file());
    assert!(known.path().join("ash.lock").is_file());
}

#[test]
fn task_982_cleanup_never_deletes_project_local_ash_toml_or_ash_lock() {
    let roots = support::xdg_fixture();
    let project = support::locked_project_fixture();
    let unregistered = support::locked_project_fixture();
    let manifest = project.path().join("ash.toml");
    let lock = project.path().join("ash.lock");
    let unregistered_manifest = unregistered.path().join("ash.toml");
    let unregistered_lock = unregistered.path().join("ash.lock");
    let original_lock = std::fs::read_to_string(&lock).expect("original lock");
    let original_unregistered_lock =
        std::fs::read_to_string(&unregistered_lock).expect("unregistered lock");
    std::fs::write(
        &manifest,
        "[package]\nname = \"locked\"\n\n[toolchain]\nash = \"ash-0.1.0+test.source.keep982002\"\n",
    )
    .expect("manifest");
    std::fs::write(
        &unregistered_manifest,
        "[package]\nname = \"unregistered\"\n\n[toolchain]\nash = \"ash-0.1.0+test.source.unreg98202\"\n",
    )
    .expect("unregistered manifest");
    let original_manifest = std::fs::read_to_string(&manifest).expect("original manifest");
    let original_unregistered_manifest =
        std::fs::read_to_string(&unregistered_manifest).expect("unregistered manifest");
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.keep982002");
    let orphan = roots.toolchain("ash-0.1.0+test.source.invalid98202");
    std::fs::create_dir_all(&orphan).expect("orphan");
    std::fs::create_dir_all(roots.cache.path().join("ash/downloads")).expect("downloads");
    std::fs::write(
        roots.cache.path().join("ash/downloads/archive.tar.gz"),
        "archive",
    )
    .expect("archive");
    std::fs::create_dir_all(project.path().join(".ash/cache/git/checkouts/local"))
        .expect("project local cache");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "cleanup",
            "--project",
            project.path().to_str().expect("utf8"),
            "--cache",
            "--orphans",
            "--old-toolchains",
        ])
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicate::str::contains("protected project"));

    assert_eq!(
        std::fs::read_to_string(&manifest).expect("manifest after cleanup"),
        original_manifest
    );
    assert_eq!(
        std::fs::read_to_string(&lock).expect("lock after cleanup"),
        original_lock
    );
    assert_eq!(
        std::fs::read_to_string(&unregistered_manifest).expect("unregistered manifest after"),
        original_unregistered_manifest
    );
    assert_eq!(
        std::fs::read_to_string(&unregistered_lock).expect("unregistered lock after"),
        original_unregistered_lock
    );
    assert!(
        project
            .path()
            .join(".ash/cache/git/checkouts/local")
            .is_dir()
    );
    assert!(
        !roots
            .cache
            .path()
            .join("ash/downloads/archive.tar.gz")
            .exists()
    );
    assert!(!orphan.exists());
}
