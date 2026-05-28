use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;

mod support;

#[test]
fn task_970_default_list_current_and_update_switch_are_selector_only() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.oldoldoldold");
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.newnewnewnew");

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
        .stdout(predicates::str::contains("newnewnewnew"));

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            "ash-0.1.0+test.source.newnewnewnew",
            "--from",
            "existing",
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let current = std::fs::read_to_string(roots.config.path().join("ash/toolchains.toml")).unwrap();
    assert!(current.contains("newnewnewnew"));
    assert!(
        roots
            .toolchain("ash-0.1.0+test.source.oldoldoldold")
            .join("manifest.toml")
            .is_file()
    );
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
