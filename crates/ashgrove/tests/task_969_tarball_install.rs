use assert_cmd::Command;

mod support;

#[test]
fn task_969_release_producer_output_installs_under_temp_xdg_roots() {
    let output = tempfile::tempdir().expect("output");
    let ash = tempfile::NamedTempFile::new().expect("ash");
    let ashgrove = tempfile::NamedTempFile::new().expect("ashgrove");
    support::write_tool_script(ash.path(), "echo produced ash\n");
    support::write_tool_script(ashgrove.path(), "echo produced ashgrove\n");

    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");
    let package = std::process::Command::new(repo_root.join("scripts/package-ash-toolchain.sh"))
        .args([
            "--toolchain-id",
            "ash-0.1.0+tarball.producer969",
            "--output-dir",
            output.path().to_str().expect("utf8 output"),
        ])
        .env("ASH_PACKAGE_ASH_BIN", ash.path())
        .env("ASH_PACKAGE_ASHGROVE_BIN", ashgrove.path())
        .output()
        .expect("run package producer");
    assert!(
        package.status.success(),
        "producer failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&package.stdout),
        String::from_utf8_lossy(&package.stderr)
    );
    let stdout = String::from_utf8(package.stdout).expect("producer stdout utf8");
    let archive = stdout
        .lines()
        .find_map(|line| line.strip_prefix("archive="))
        .map(std::path::PathBuf::from)
        .expect("archive output");
    assert!(archive.is_file(), "archive exists at {}", archive.display());

    let roots = support::xdg_fixture();
    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.to_str().expect("utf8 archive"),
        "--switch",
    ])
    .envs(roots.env())
    .assert()
    .success();

    let installed = roots.toolchain("ash-0.1.0+tarball.producer969");
    let manifest = std::fs::read_to_string(installed.join("manifest.toml")).expect("manifest");
    assert!(manifest.contains("archive_schema_version = 1"));
    assert!(manifest.contains("source_kind = \"tarball\""));
    assert!(installed.join("bin/ash").is_file());
    assert!(installed.join("bin/ashgrove").is_file());
    assert!(installed.join("lib/ash/std/ash.toml").is_file());

    let record = std::fs::read_to_string(installed.join("install-record.toml")).expect("record");
    assert!(record.contains("archive_schema_version = 1"));
    assert!(record.contains("tarball_path"));
    assert!(record.contains(archive.to_str().expect("utf8 archive")));
    assert!(record.contains("tarball_digest = \"sha256:"));
    assert!(record.contains("installed_at"));
}

#[test]
fn task_969_release_producer_rejects_unsafe_metadata_inputs() {
    let output = tempfile::tempdir().expect("output");
    let ash = tempfile::NamedTempFile::new().expect("ash");
    let ashgrove = tempfile::NamedTempFile::new().expect("ashgrove");
    support::write_tool_script(ash.path(), "echo produced ash\n");
    support::write_tool_script(ashgrove.path(), "echo produced ashgrove\n");

    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");
    let package = std::process::Command::new(repo_root.join("scripts/package-ash-toolchain.sh"))
        .args([
            "--toolchain-id",
            "ash-0.1.0+tarball.\"injected",
            "--output-dir",
            output.path().to_str().expect("utf8 output"),
        ])
        .env("ASH_PACKAGE_ASH_BIN", ash.path())
        .env("ASH_PACKAGE_ASHGROVE_BIN", ashgrove.path())
        .output()
        .expect("run package producer");

    assert!(
        !package.status.success(),
        "producer accepted unsafe metadata\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&package.stdout),
        String::from_utf8_lossy(&package.stderr)
    );
    assert!(
        String::from_utf8_lossy(&package.stderr).contains("invalid toolchain id"),
        "stderr should explain invalid toolchain id, got: {}",
        String::from_utf8_lossy(&package.stderr)
    );
}

#[test]
fn task_969_tarball_install_rejects_missing_archive_schema_version() {
    let archive = support::toolchain_tarball_fixture_with_mutation(
        "ash-0.1.0+test.tarball.missingschema1",
        |root| {
            support::remove_toml_line(root.join("manifest.toml"), "archive_schema_version");
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
    .stderr(predicates::str::contains("archive schema version"));
}

#[test]
fn task_969_tarball_install_rejects_unsupported_archive_schema_version() {
    let archive = support::toolchain_tarball_fixture_with_mutation(
        "ash-0.1.0+test.tarball.badschema001",
        |root| {
            support::replace_toml_line(
                root.join("manifest.toml"),
                "archive_schema_version",
                "archive_schema_version = 2",
            );
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
    .stderr(predicates::str::contains("archive schema version"));
}

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
                "toolchain_id = \"ash-0.1.0+test.tarball.nostdmeta001\"\nversion = \"0.1.0\"\narchive_schema_version = 1\n",
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
                "toolchain_id = \"ash-0.1.0+test.tarball.otherrecord01\"\nsource_kind = \"tarball\"\narchive_schema_version = 1\n",
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
                "toolchain_id = \"ash-0.1.0+test.tarball.sourcekind001\"\nsource_kind = \"source\"\narchive_schema_version = 1\n",
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
                "toolchain_id = \"ash-0.1.0+test.tarball.otherroot001\"\nversion = \"0.1.0\"\narchive_schema_version = 1\n[stdlib]\nversion = \"0.1.0\"\npath = \"lib/ash/std\"\n[runtime_support]\nidentity = \"ash-runtime-support:0.1.0\"\npath = \"lib/ash/std/src/runtime\"\nrequired = true\n[[standard_tools]]\nname = \"ash\"\npath = \"bin/ash\"\nrequired = true\n[[standard_tools]]\nname = \"ashgrove\"\npath = \"bin/ashgrove\"\nrequired = true\n",
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
                "toolchain_id = \"ash-0.1.0+test.tarball.versionmis01\"\nversion = \"9.9.9\"\narchive_schema_version = 1\n[stdlib]\nversion = \"0.1.0\"\npath = \"lib/ash/std\"\n[runtime_support]\nidentity = \"ash-runtime-support:0.1.0\"\npath = \"lib/ash/std/src/runtime\"\nrequired = true\n[[standard_tools]]\nname = \"ash\"\npath = \"bin/ash\"\nrequired = true\n[[standard_tools]]\nname = \"ashgrove\"\npath = \"bin/ashgrove\"\nrequired = true\n",
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
