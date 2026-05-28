use assert_cmd::Command;

mod support;

#[test]
fn task_968_source_install_rejects_dirty_source_without_override() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.aaaaaaaaaaaa");
    std::fs::write(fixture.path().join(".dirty"), "dirty").expect("dirty marker");
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "source",
        "--path",
        fixture.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("dirty source"));
}

#[test]
fn task_968_source_install_publishes_toolchain_shape_with_override() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.bbbbbbbbbbbb");
    std::fs::write(fixture.path().join(".dirty"), "dirty").expect("dirty marker");
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "source",
        "--path",
        fixture.path().to_str().expect("utf8"),
        "--allow-dirty-source",
        "--switch",
    ])
    .envs(roots.env())
    .assert()
    .success();

    let installed = roots.toolchain("ash-0.1.0+test.source.bbbbbbbbbbbb");
    assert!(installed.join("bin/ash").is_file());
    assert!(installed.join("bin/ashgrove").is_file());
    assert!(installed.join("lib/ash/std/ash.toml").is_file());
    assert!(roots.config.path().join("ash/toolchains.toml").is_file());
}

#[test]
fn task_968_source_install_rejects_unidentified_archive_without_override() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.cccccccccccc");
    std::fs::remove_file(fixture.path().join(".source-rev")).expect("remove source rev");
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "source",
        "--path",
        fixture.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("unidentified source"));
}

#[test]
fn task_968_source_install_rejects_empty_source_rev_without_override() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.cddddddddddd");
    std::fs::write(fixture.path().join(".source-rev"), " \n").expect("empty source rev");
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "source",
        "--path",
        fixture.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("unidentified source"));
}

#[test]
fn task_968_source_install_records_source_metadata_and_overrides() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.dddddddddddd");
    std::fs::write(fixture.path().join(".dirty"), "dirty").expect("dirty marker");
    std::fs::write(
        fixture.path().join(".source-url"),
        "https://example.invalid/ash.git\n",
    )
    .expect("source url");
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "source",
        "--path",
        fixture.path().to_str().expect("utf8"),
        "--allow-dirty-source",
    ])
    .envs(roots.env())
    .assert()
    .success();

    let record = std::fs::read_to_string(
        roots
            .toolchain("ash-0.1.0+test.source.dddddddddddd")
            .join("install-record.toml"),
    )
    .expect("install record");
    assert!(record.contains("source_kind = \"source\""));
    assert!(record.contains("source_url = \"https://example.invalid/ash.git\""));
    assert!(record.contains("source_rev = \"abcdef1234567890\""));
    assert!(record.contains("build_profile = \"debug\""));
    assert!(record.contains("target_triple = "));
    assert!(record.contains("allow_dirty_source = true"));
    assert!(record.contains("allow_unidentified_source = false"));
    assert!(record.contains("reproducible = false"));
}

#[test]
fn task_968_source_install_records_unidentified_override_as_non_reproducible() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.eeeeeeeeeeee");
    std::fs::remove_file(fixture.path().join(".source-rev")).expect("remove source rev");
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "source",
        "--path",
        fixture.path().to_str().expect("utf8"),
        "--allow-unidentified-source",
    ])
    .envs(roots.env())
    .assert()
    .success();

    let record = std::fs::read_to_string(
        roots
            .toolchain("ash-0.1.0+test.source.eeeeeeeeeeee")
            .join("install-record.toml"),
    )
    .expect("install record");
    assert!(record.contains("allow_unidentified_source = true"));
    assert!(record.contains("reproducible = false"));
    assert!(!record.contains("source_rev = "));
}

#[test]
fn task_968_source_install_rejects_same_id_with_different_source_metadata() {
    let first = support::source_fixture("ash-0.1.0+test.source.ffffffffffff");
    let second = support::source_fixture("ash-0.1.0+test.source.ffffffffffff");
    std::fs::write(
        second.path().join("manifest.toml"),
        "toolchain_id = \"ash-0.1.0+test.source.ffffffffffff\"\nversion = \"0.1.1\"\n",
    )
    .expect("different manifest");
    let roots = support::xdg_fixture();

    let mut first_cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    first_cmd
        .args([
            "install",
            "--from",
            "source",
            "--path",
            first.path().to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .success();

    let mut second_cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    second_cmd
        .args([
            "install",
            "--from",
            "source",
            "--path",
            second.path().to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("metadata collision"));
}

#[test]
fn task_968_source_install_rejects_same_id_with_different_source_rev() {
    let first = support::source_fixture("ash-0.1.0+test.source.aaaaaaaa9999");
    let second = support::source_fixture("ash-0.1.0+test.source.aaaaaaaa9999");
    std::fs::write(second.path().join(".source-rev"), "fedcba0987654321\n")
        .expect("different source rev");
    let roots = support::xdg_fixture();

    let mut first_cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    first_cmd
        .args([
            "install",
            "--from",
            "source",
            "--path",
            first.path().to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .success();

    let mut second_cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    second_cmd
        .args([
            "install",
            "--from",
            "source",
            "--path",
            second.path().to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("metadata collision"));
}

#[test]
fn task_968_source_install_identical_reinstall_is_deterministic_noop() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.999999999999");
    let roots = support::xdg_fixture();

    for _ in 0..2 {
        let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
        cmd.args([
            "install",
            "--from",
            "source",
            "--path",
            fixture.path().to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .success();
    }

    let installed = roots.toolchain("ash-0.1.0+test.source.999999999999");
    assert!(installed.join("manifest.toml").is_file());
    assert!(
        !roots
            .data
            .path()
            .join("ash/toolchains/.staging/ash-0.1.0+test.source.999999999999")
            .exists()
    );
}
