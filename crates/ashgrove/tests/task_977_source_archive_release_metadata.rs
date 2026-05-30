use assert_cmd::Command;

mod support;

#[test]
fn task_977_packaged_source_archive_records_reproducible_release_metadata() {
    let source = support::source_workspace_fixture();
    let package_dir = tempfile::tempdir().expect("package dir");
    let extract_dir = tempfile::tempdir().expect("extract dir");
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("scripts/package-ash-source-archive.sh");

    let output = std::process::Command::new("bash")
        .arg(script)
        .arg("--source-root")
        .arg(source.path())
        .arg("--origin-commit")
        .arg(source.revision())
        .arg("--output-dir")
        .arg(package_dir.path())
        .arg("--version")
        .arg("0.1.0")
        .output()
        .expect("package source archive");
    assert!(
        output.status.success(),
        "packaging failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("package stdout utf8");
    let archive = package_stdout_value(&stdout, "archive");

    let status = std::process::Command::new("tar")
        .arg("-xzf")
        .arg(&archive)
        .arg("-C")
        .arg(extract_dir.path())
        .status()
        .expect("extract source archive");
    assert!(status.success(), "extract source archive");
    let archive_root = extract_dir
        .path()
        .join(format!("ash-0.1.0+source.{}", &source.revision()[..12]));
    assert!(archive_root.join("Cargo.toml").is_file());
    assert!(archive_root.join("std/src").is_dir());
    assert!(archive_root.join("release-source.toml").is_file());
    assert!(archive_root.join(".source-rev").is_file());

    let roots = support::xdg_fixture();
    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "source",
        "--path",
        archive_root.to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .success();

    let id = format!("ash-0.1.0+source.{}", &source.revision()[..12]);
    let record = install_record(&roots, &id);
    assert_eq!(
        record
            .get("source_origin_commit")
            .and_then(toml::Value::as_str),
        Some(source.revision())
    );
    assert_eq!(
        record.get("source_rev").and_then(toml::Value::as_str),
        Some(source.revision())
    );
    assert_digest_field(&record, "source_archive_digest");
    assert!(record.get("dirty_source_digest").is_none());
    assert_eq!(
        record.get("reproducible").and_then(toml::Value::as_bool),
        Some(true)
    );
}

#[test]
fn task_977_source_archive_without_release_metadata_fails_closed() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.archive977a");
    let _ = std::fs::remove_file(fixture.path().join("release-source.toml"));
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
    .stderr(predicates::str::contains("release-source metadata"));

    assert!(
        !roots
            .toolchain("ash-0.1.0+test.source.archive977a")
            .exists()
    );
}

#[test]
fn task_977_source_archive_with_origin_commit_records_reproducible_metadata() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.archive977b");
    std::fs::write(
        fixture.path().join("release-source.toml"),
        "schema_version = 1\norigin_commit = \"abcdef1234567890\"\n\n[attestation]\norigin_commit = \"abcdef1234567890\"\n",
    )
    .expect("release-source metadata");
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
    .success();

    let record = install_record(&roots, "ash-0.1.0+test.source.archive977b");
    assert_eq!(
        record
            .get("source_origin_commit")
            .and_then(toml::Value::as_str),
        Some("abcdef1234567890")
    );
    assert_digest_field(&record, "source_archive_digest");
    assert_eq!(
        record.get("reproducible").and_then(toml::Value::as_bool),
        Some(true)
    );
}

#[test]
fn task_977_allow_unidentified_source_marks_non_reproducible_with_archive_digest() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.archive977c");
    let _ = std::fs::remove_file(fixture.path().join("release-source.toml"));
    std::fs::remove_file(fixture.path().join(".source-rev")).expect("remove legacy source rev");
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

    let record = install_record(&roots, "ash-0.1.0+test.source.archive977c");
    assert!(record.get("source_origin_commit").is_none());
    assert_digest_field(&record, "source_archive_digest");
    assert_eq!(
        record
            .get("allow_unidentified_source")
            .and_then(toml::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        record.get("reproducible").and_then(toml::Value::as_bool),
        Some(false)
    );
}

fn install_record(roots: &support::XdgFixture, id: &str) -> toml::map::Map<String, toml::Value> {
    let text =
        std::fs::read_to_string(roots.toolchain(id).join("install-record.toml")).expect("record");
    toml::from_str::<toml::Value>(&text)
        .expect("parse record")
        .as_table()
        .expect("record table")
        .clone()
}

fn assert_digest_field(record: &toml::map::Map<String, toml::Value>, key: &str) {
    let digest = record
        .get(key)
        .and_then(toml::Value::as_str)
        .expect("digest field");
    let hex = digest.strip_prefix("sha256:").expect("sha256 prefix");
    assert_eq!(hex.len(), 64);
    assert!(hex.chars().all(|ch| ch.is_ascii_hexdigit()));
}

fn package_stdout_value(stdout: &str, key: &str) -> String {
    stdout
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}=")))
        .unwrap_or_else(|| panic!("missing {key}= in package output:\n{stdout}"))
        .to_string()
}
