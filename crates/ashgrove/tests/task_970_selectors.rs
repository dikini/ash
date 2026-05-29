use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use sha2::{Digest, Sha256};

mod support;

#[test]
fn task_970_default_list_current_and_update_switch_are_selector_only() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.oldoldoldold");
    let new_id = "ash-0.1.0+tarball.switch970";
    let output = tempfile::tempdir().expect("output");
    let archive = support::produced_toolchain_tarball_in(new_id, output.path());

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["default", "ash-0.1.0+test.source.oldoldoldold"])
        .envs(roots.env())
        .assert()
        .success();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .arg("current")
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains("oldoldoldold"));

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .arg("list")
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains("oldoldoldold"))
        .stdout(predicates::str::contains(new_id).not());

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            new_id,
            "--from",
            "tarball",
            "--path",
            archive.to_str().expect("utf8 archive"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let current = std::fs::read_to_string(roots.config.path().join("ash/toolchains.toml")).unwrap();
    assert!(current.contains(new_id));
    assert!(
        roots
            .toolchain("ash-0.1.0+test.source.oldoldoldold")
            .join("manifest.toml")
            .is_file()
    );
    assert!(roots.toolchain(new_id).join("manifest.toml").is_file());
}

#[test]
fn task_970_current_fails_closed_when_default_selector_points_to_missing_toolchain() {
    let roots = support::xdg_fixture();
    let selector = roots.config.path().join("ash/toolchains.toml");
    std::fs::create_dir_all(selector.parent().expect("selector parent")).expect("selector dir");
    std::fs::write(
        selector,
        "default = \"ash-0.1.0+test.source.missing000001\"\n",
    )
    .expect("selector");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .arg("current")
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("is not installed"));
}

#[test]
fn task_970_list_uses_manifest_metadata_and_ignores_incomplete_directories() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.listvalid001");
    std::fs::create_dir_all(
        roots
            .data
            .path()
            .join("ash/toolchains/ash-0.1.0+test.source.listbroken01"),
    )
    .expect("broken dir");
    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["default", "ash-0.1.0+test.source.listvalid001"])
        .envs(roots.env())
        .assert()
        .success();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .arg("list")
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "ash-0.1.0+test.source.listvalid001",
        ))
        .stdout(predicates::str::contains("(default)"))
        .stdout(predicates::str::contains("listbroken01").not());
}

#[test]
fn task_970_update_without_switch_preserves_default_and_does_not_mutate_old_toolchain() {
    let roots = support::xdg_fixture();
    let old = support::source_fixture("ash-0.1.0+test.source.updateold001");
    let new = support::source_fixture("ash-0.1.0+test.source.updatenew001");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "install",
            "--from",
            "source",
            "--path",
            old.path().to_str().expect("utf8"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let old_manifest_before = std::fs::read_to_string(
        roots
            .toolchain("ash-0.1.0+test.source.updateold001")
            .join("manifest.toml"),
    )
    .expect("old manifest before");
    let old_record_before = std::fs::read_to_string(
        roots
            .toolchain("ash-0.1.0+test.source.updateold001")
            .join("install-record.toml"),
    )
    .expect("old record before");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            "ash-0.1.0+test.source.updatenew001",
            "--from",
            "source",
            "--path",
            new.path().to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .success();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .arg("current")
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains("updateold001"));

    assert!(
        roots
            .toolchain("ash-0.1.0+test.source.updatenew001")
            .join("manifest.toml")
            .is_file()
    );
    assert_eq!(
        old_manifest_before,
        std::fs::read_to_string(
            roots
                .toolchain("ash-0.1.0+test.source.updateold001")
                .join("manifest.toml")
        )
        .expect("old manifest after")
    );
    assert_eq!(
        old_record_before,
        std::fs::read_to_string(
            roots
                .toolchain("ash-0.1.0+test.source.updateold001")
                .join("install-record.toml")
        )
        .expect("old record after")
    );
}

#[test]
fn task_970_source_update_builds_real_source_root_and_records_source_metadata() {
    let roots = support::xdg_fixture();
    let old_id = "ash-0.1.0+test.source.oldsource970";
    let new = support::source_workspace_fixture();
    let new_id = new.toolchain_id();
    support::install_fake_toolchain(&roots, old_id);

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["default", old_id])
        .envs(roots.env())
        .assert()
        .success();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            &new_id,
            "--from",
            "source",
            "--path",
            new.path().to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .success();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .arg("current")
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains(old_id));

    let installed = roots.toolchain(&new_id);
    assert!(installed.join("bin/ash").is_file());
    assert!(installed.join("bin/ashgrove").is_file());
    assert!(installed.join("lib/ash/std/ash.toml").is_file());

    let record = std::fs::read_to_string(installed.join("install-record.toml")).expect("record");
    assert!(record.contains("source_kind = \"source\""));
    assert!(record.contains(&format!("source_path = \"{}\"", new.path().display())));
    assert!(record.contains(&format!("source_rev = \"{}\"", new.revision())));
    assert!(record.contains("source_url = \"https://example.invalid/ash.git\""));
    assert!(record.contains("installed_at"));
}

#[test]
fn task_970_source_update_requires_to_to_match_real_source_identity() {
    let roots = support::xdg_fixture();
    let source = support::source_workspace_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            "ash-0.1.0+source.notthesource",
            "--from",
            "source",
            "--path",
            source.path().to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("does not match source toolchain"));

    assert!(!roots.toolchain("ash-0.1.0+source.notthesource").exists());
}

#[test]
fn task_970_tarball_update_accepts_producer_output_requires_identity_and_records_metadata() {
    let roots = support::xdg_fixture();
    let old = support::source_workspace_fixture();
    let old_id = old.toolchain_id();
    let tarball_id = "ash-0.1.0+tarball.producer970";
    let output = tempfile::tempdir().expect("output");
    let archive = support::produced_toolchain_tarball_in(tarball_id, output.path());

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "install",
            "--from",
            "source",
            "--path",
            old.path().to_str().expect("utf8"),
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
            "ash-0.1.0+tarball.wrong970",
            "--from",
            "tarball",
            "--path",
            archive.to_str().expect("utf8 archive"),
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "does not match tarball toolchain",
        ));

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            tarball_id,
            "--from",
            "tarball",
            "--path",
            archive.to_str().expect("utf8 archive"),
            "--digest",
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("tarball digest mismatch"));
    assert!(!roots.toolchain(tarball_id).exists());

    let digest = Sha256::digest(std::fs::read(&archive).expect("archive bytes"));
    let expected_digest = format!("sha256:{digest:x}");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            tarball_id,
            "--from",
            "tarball",
            "--path",
            archive.to_str().expect("utf8 archive"),
            "--digest",
            &expected_digest,
        ])
        .envs(roots.env())
        .assert()
        .success();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .arg("current")
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains(&old_id));

    let record = std::fs::read_to_string(roots.toolchain(tarball_id).join("install-record.toml"))
        .expect("record");
    assert!(record.contains("source_kind = \"tarball\""));
    assert!(record.contains("archive_schema_version = 1"));
    assert!(record.contains("tarball_path"));
    assert!(record.contains(archive.to_str().expect("utf8 archive")));
    assert!(record.contains("tarball_digest = \"sha256:"));
    assert!(record.contains("installed_at"));
}

#[test]
fn task_970_first_update_install_initializes_default_when_none_exists() {
    let roots = support::xdg_fixture();
    let archive = support::toolchain_tarball_fixture("ash-0.1.0+test.tarball.firstdefault1");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            "ash-0.1.0+test.tarball.firstdefault1",
            "--from",
            "tarball",
            "--path",
            archive.path().to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .success();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .arg("current")
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains("firstdefault1"));
}

#[test]
fn task_970_default_requires_exact_toolchain_id_when_versions_overlap() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.sameversion1");
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.tarball.sameversion2");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["default", "ash-0.1.0"])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("exact toolchain id"));
}

#[test]
fn task_970_bare_and_network_update_remain_rejected_until_release_index_policy_exists() {
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["update", "0.1.0"])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("release index policy"));

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            "ash-0.1.0+tarball.remote970",
            "--from",
            "tarball",
            "--url",
            "https://example.invalid/ash.tar.gz",
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("authenticated download policy"));
}
