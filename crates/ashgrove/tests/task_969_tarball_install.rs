use assert_cmd::Command;

mod support;

#[test]
fn task_969_tarball_install_rejects_unsafe_traversal_entry() {
    let archive = support::unsafe_tarball_fixture();
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("unsafe archive entry"));
}

#[test]
fn task_969_tarball_install_rejects_unsafe_absolute_entry() {
    let archive = support::unsafe_path_tarball_fixture("/tmp/ashgrove-escape");
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("unsafe archive entry"));
}

#[test]
fn task_969_tarball_install_rejects_unsafe_parent_traversal_entry() {
    let archive =
        support::unsafe_path_tarball_fixture("ash-0.1.0+test.tarball.traversal001/../../escape");
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("unsafe archive entry"));
}

#[test]
fn task_969_tarball_install_rejects_unsafe_hardlink_entry() {
    let archive = support::unsafe_hardlink_tarball_fixture();
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("unsafe archive entry type"));
}

#[test]
fn task_969_tarball_install_rejects_setuid_or_setgid_bits() {
    let archive = support::unsafe_mode_tarball_fixture(0o4755);
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("unsafe archive entry mode"));
}

#[cfg(unix)]
#[test]
fn task_969_tarball_install_rejects_non_executable_required_binary() {
    let archive =
        support::non_executable_toolchain_tarball_fixture("ash-0.1.0+test.tarball.noexec000001");
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("not executable"));

    assert!(
        !roots
            .toolchain("ash-0.1.0+test.tarball.noexec000001")
            .exists()
    );
}

#[test]
fn task_969_tarball_install_validates_and_records_digest() {
    let archive = support::toolchain_tarball_fixture("ash-0.1.0+test.tarball.cccccccccccc");
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.path().to_str().expect("utf8"),
        "--switch",
    ])
    .envs(roots.env())
    .assert()
    .success();

    let installed = roots.toolchain("ash-0.1.0+test.tarball.cccccccccccc");
    let record = std::fs::read_to_string(installed.join("install-record.toml")).expect("record");
    assert!(record.contains("sha256:"));
    assert!(record.contains("tarball_path"));
    assert!(record.contains(archive.path().to_str().expect("utf8 archive path")));
    assert!(record.contains("installed_at"));
}

#[test]
fn task_969_tarball_install_rejects_manifest_schema_without_stdlib_metadata() {
    let archive = support::toolchain_tarball_fixture_with_mutation(
        "ash-0.1.0+test.tarball.nostdmeta001",
        |root| {
            std::fs::write(
                root.join("manifest.toml"),
                "toolchain_id = \"ash-0.1.0+test.tarball.nostdmeta001\"\nversion = \"0.1.0\"\n",
            )
            .expect("manifest");
        },
    );
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains(
        "toolchain manifest missing stdlib metadata",
    ));

    assert!(
        !roots
            .toolchain("ash-0.1.0+test.tarball.nostdmeta001")
            .exists()
    );
}

#[test]
fn task_969_tarball_install_rejects_install_record_schema_mismatch() {
    let archive = support::toolchain_tarball_fixture_with_mutation(
        "ash-0.1.0+test.tarball.badrecord001",
        |root| {
            std::fs::write(
                root.join("install-record.toml"),
                "toolchain_id = \"ash-0.1.0+test.tarball.otherrecord01\"\nsource_kind = \"tarball\"\n",
            )
            .expect("record");
        },
    );
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("install record toolchain id"));
}

#[test]
fn task_969_tarball_install_rejects_non_tarball_install_record() {
    let archive = support::toolchain_tarball_fixture_with_mutation(
        "ash-0.1.0+test.tarball.sourcekind001",
        |root| {
            std::fs::write(
                root.join("install-record.toml"),
                "toolchain_id = \"ash-0.1.0+test.tarball.sourcekind001\"\nsource_kind = \"source\"\n",
            )
            .expect("record");
        },
    );
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("install record source_kind"));
}

#[test]
fn task_969_tarball_install_rejects_root_name_manifest_mismatch() {
    let archive = support::toolchain_tarball_fixture_with_mutation(
        "ash-0.1.0+test.tarball.rootmismatch1",
        |root| {
            std::fs::write(
                root.join("manifest.toml"),
                "toolchain_id = \"ash-0.1.0+test.tarball.otherroot001\"\nversion = \"0.1.0\"\n[stdlib]\nversion = \"0.1.0\"\npath = \"lib/ash/std\"\n[[standard_tools]]\nname = \"ash\"\npath = \"bin/ash\"\nrequired = true\n[[standard_tools]]\nname = \"ashgrove\"\npath = \"bin/ashgrove\"\nrequired = true\n",
            )
            .expect("manifest");
        },
    );
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("manifest toolchain id"));
}

#[test]
fn task_969_tarball_install_rejects_version_manifest_mismatch() {
    let archive = support::toolchain_tarball_fixture_with_mutation(
        "ash-0.1.0+test.tarball.versionmis01",
        |root| {
            std::fs::write(
                root.join("manifest.toml"),
                "toolchain_id = \"ash-0.1.0+test.tarball.versionmis01\"\nversion = \"9.9.9\"\n[stdlib]\nversion = \"0.1.0\"\npath = \"lib/ash/std\"\n[[standard_tools]]\nname = \"ash\"\npath = \"bin/ash\"\nrequired = true\n[[standard_tools]]\nname = \"ashgrove\"\npath = \"bin/ashgrove\"\nrequired = true\n",
            )
            .expect("manifest");
        },
    );
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("manifest version"));
}

#[test]
fn task_969_tarball_install_rejects_missing_stdlib_manifest() {
    let archive = support::toolchain_tarball_fixture_with_mutation(
        "ash-0.1.0+test.tarball.missingstd01",
        |root| {
            std::fs::remove_file(root.join("lib/ash/std/ash.toml")).expect("remove std manifest");
        },
    );
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("stdlib manifest"));
}

#[test]
fn task_969_tarball_install_reinstall_ignores_local_tarball_path_collision() {
    let archive = support::toolchain_tarball_fixture("ash-0.1.0+test.tarball.reinstall001");
    let archive_copy = tempfile::NamedTempFile::new().expect("archive copy");
    std::fs::copy(archive.path(), archive_copy.path()).expect("copy archive");
    let roots = support::xdg_fixture();

    for path in [archive.path(), archive_copy.path()] {
        let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
        cmd.args([
            "install",
            "--from",
            "tarball",
            "--path",
            path.to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .success();
    }
}
