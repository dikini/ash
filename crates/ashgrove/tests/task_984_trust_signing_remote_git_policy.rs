use assert_cmd::Command;
use predicates::prelude::*;
use sha2::Digest;

mod support;

const FULL_REV: &str = "0123456789abcdef0123456789abcdef01234567";
const BAD_SHA256: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

#[test]
fn task_984_tarball_signature_failure_fails_before_publish() {
    let roots = support::xdg_fixture();
    let toolchain_id = "ash-0.1.0+tarball.signature984";
    let archive = support::toolchain_tarball_fixture_with_mutation(toolchain_id, |root| {
        std::fs::write(
            root.join("manifest.toml"),
            format!(
                r#"toolchain_id = "{toolchain_id}"
version = "0.1.0"
archive_schema_version = 1
source_kind = "tarball"

[stdlib]
version = "0.1.0"
path = "lib/ash/std"

[runtime_support]
identity = "ash-runtime-support:0.1.0"
path = "lib/ash/std/src/runtime"
required = true

[trust.release]
signature_required = true

[[standard_tools]]
name = "ash"
path = "bin/ash"
required = true

[[standard_tools]]
name = "ashgrove"
path = "bin/ashgrove"
required = true
"#
            ),
        )
        .expect("manifest");
    });
    std::fs::write(
        release_signature_sidecar_path(archive.path()),
        format!(
            "schema_version = 1\ntoolchain_id = \"{toolchain_id}\"\ntarball_digest = \"{BAD_SHA256}\"\nsignature = \"test-signature\"\n"
        ),
    )
    .expect("signature sidecar");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "install",
            "--from",
            "tarball",
            "--path",
            archive.path().to_str().expect("archive path"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicate::str::contains("signature"))
        .stderr(predicate::str::contains("mismatch"));

    assert!(!roots.toolchain(toolchain_id).exists());
}

#[test]
fn task_984_source_archive_attestation_failure_fails_before_publish() {
    let roots = support::xdg_fixture();
    let toolchain_id = "ash-0.1.0+test.source.attestation984";
    let source = support::source_fixture(toolchain_id);
    std::fs::write(
        source.path().join("release-source.toml"),
        r#"schema_version = 1
origin_commit = "abcdef1234567890"

[attestation]
required = true
origin_commit = "deadbeefdeadbeef"
"#,
    )
    .expect("release source metadata");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "install",
            "--from",
            "source",
            "--path",
            source.path().to_str().expect("source path"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicate::str::contains("attestation"))
        .stderr(predicate::str::contains("origin_commit"));

    assert!(!roots.toolchain(toolchain_id).exists());
}

#[test]
fn task_984_source_archive_missing_attestation_fails_before_publish() {
    let roots = support::xdg_fixture();
    let toolchain_id = "ash-0.1.0+test.source.attestationmissing984";
    let source = support::source_fixture(toolchain_id);
    std::fs::write(
        source.path().join("release-source.toml"),
        r#"schema_version = 1
origin_commit = "abcdef1234567890"
"#,
    )
    .expect("release source metadata");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "install",
            "--from",
            "source",
            "--path",
            source.path().to_str().expect("source path"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicate::str::contains("attestation"))
        .stderr(predicate::str::contains("required"));

    assert!(!roots.toolchain(toolchain_id).exists());
}

#[test]
fn task_984_tarball_signature_sidecar_evidence_allows_required_signature() {
    let roots = support::xdg_fixture();
    let toolchain_id = "ash-0.1.0+tarball.signaturevalid984";
    let archive = support::toolchain_tarball_fixture_with_mutation(toolchain_id, |root| {
        std::fs::write(
            root.join("manifest.toml"),
            format!(
                r#"toolchain_id = "{toolchain_id}"
version = "0.1.0"
archive_schema_version = 1
source_kind = "tarball"

[stdlib]
version = "0.1.0"
path = "lib/ash/std"

[runtime_support]
identity = "ash-runtime-support:0.1.0"
path = "lib/ash/std/src/runtime"
required = true

[trust.release]
signature_required = true

[[standard_tools]]
name = "ash"
path = "bin/ash"
required = true

[[standard_tools]]
name = "ashgrove"
path = "bin/ashgrove"
required = true
"#
            ),
        )
        .expect("manifest");
    });
    let archive_bytes = std::fs::read(archive.path()).expect("archive bytes");
    let digest = format!("sha256:{:x}", sha2::Sha256::digest(&archive_bytes));
    std::fs::write(
        release_signature_sidecar_path(archive.path()),
        format!(
            "schema_version = 1\ntoolchain_id = \"{toolchain_id}\"\ntarball_digest = \"{digest}\"\nsignature = \"test-signature\"\n"
        ),
    )
    .expect("signature sidecar");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "install",
            "--from",
            "tarball",
            "--path",
            archive.path().to_str().expect("archive path"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    assert!(roots.toolchain(toolchain_id).exists());
}

#[test]
fn task_984_update_rejects_unsigned_release_index_before_publish() {
    let roots = support::xdg_fixture();
    let old_id = "ash-0.1.0+test.source.old984";
    support::install_fake_toolchain(&roots, old_id);
    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["default", old_id])
        .envs(roots.env())
        .assert()
        .success();

    let output = tempfile::tempdir().expect("output");
    let toolchain_id = "ash-0.1.0+tarball.releaseindex984";
    let archive = support::produced_toolchain_tarball_in(toolchain_id, output.path());
    let release_index = output.path().join("release-index.toml");
    std::fs::write(
        &release_index,
        format!(
            "[[release]]\ntoolchain_id = \"{toolchain_id}\"\ntarball_url = \"file://{}\"\n",
            archive.display()
        ),
    )
    .expect("release index");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            toolchain_id,
            "--from",
            "tarball",
            "--release-index",
            release_index.to_str().expect("release index path"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsigned release index"));

    assert!(!roots.toolchain(toolchain_id).exists());
}

#[test]
fn task_984_lock_check_rejects_required_signature_mismatch() {
    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.dep]\nversion = \"0.2.0\"\nregistry = \"ash.test\"\ngit = \"{}\"\ntag = \"v1\"\n",
            dep.url()
        ),
    )
    .expect("manifest");
    let roots = support::xdg_fixture();
    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "lock",
            "--project",
            project.path().to_str().expect("project"),
        ])
        .envs(roots.env())
        .assert()
        .success();

    let mut lock_text = std::fs::read_to_string(project.path().join("ash.lock")).expect("lock");
    lock_text.push_str(&format!(
        "\n[signing.lock]\nrequired = true\npackage_manifest_digest = \"{BAD_SHA256}\"\n"
    ));
    std::fs::write(project.path().join("ash.lock"), lock_text).expect("tamper lock signing");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "lock",
            "--project",
            project.path().to_str().expect("project"),
            "--check",
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicate::str::contains("signature"))
        .stderr(predicate::str::contains("mismatch"));
}

#[test]
fn task_984_remote_git_fetch_rejects_untrusted_protocols() {
    let project = support::project_fixture();
    std::fs::write(
        project.path().join("ash.lock"),
        format!("[[package]]\nname = \"dep\"\ngit = \"git://example.invalid/dep.git\"\ncommit = \"{FULL_REV}\"\n"),
    )
    .expect("lock");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "fetch",
            "--project",
            project.path().to_str().expect("project"),
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicate::str::contains("untrusted git protocol"));
}

#[test]
fn task_984_remote_git_fetch_records_authenticated_origin_without_secrets() {
    let project = support::project_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.dep]\ngit = \"https://user:secret@example.invalid/org/dep.git\"\nrev = \"{FULL_REV}\"\n"
        ),
    )
    .expect("manifest");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "lock",
            "--project",
            project.path().to_str().expect("project"),
        ])
        .envs(roots.env())
        .assert()
        .success();

    let lock_text = std::fs::read_to_string(project.path().join("ash.lock")).expect("lock");
    assert!(lock_text.contains("authenticated_origin = \"credentials-redacted\""));
    assert!(lock_text.contains("https://example.invalid/org/dep.git"));
    assert!(!lock_text.contains("user"));
    assert!(!lock_text.contains("secret"));
}

#[test]
fn task_984_lockfile_never_serializes_credentials_from_authenticated_remote() {
    let project = support::project_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.dep]\ngit = \"https://token:supersecret@example.invalid/dep.git\"\nrev = \"{FULL_REV}\"\n"
        ),
    )
    .expect("manifest");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "lock",
            "--project",
            project.path().to_str().expect("project"),
        ])
        .envs(roots.env())
        .assert()
        .success();

    let lock_text = std::fs::read_to_string(project.path().join("ash.lock")).expect("lock");
    assert!(!lock_text.contains("token"));
    assert!(!lock_text.contains("supersecret"));
    assert!(!lock_text.contains("token:supersecret@"));
}

#[test]
fn task_984_lock_rejects_credential_bearing_ssh_url_before_serialization() {
    let project = support::project_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"app\"\n\n[dependencies.dep]\ngit = \"ssh://token:supersecret@example.invalid/org/dep.git\"\nrev = \"{FULL_REV}\"\n"
        ),
    )
    .expect("manifest");
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "lock",
            "--project",
            project.path().to_str().expect("project"),
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicate::str::contains("credentials"))
        .stderr(predicate::str::contains("token:supersecret").not())
        .stderr(predicate::str::contains("token").not())
        .stderr(predicate::str::contains("supersecret").not());

    let lock_path = project.path().join("ash.lock");
    if lock_path.exists() {
        let lock_text = std::fs::read_to_string(lock_path).expect("lock");
        assert!(!lock_text.contains("token"));
        assert!(!lock_text.contains("supersecret"));
    }
}

fn release_signature_sidecar_path(archive: &std::path::Path) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("{}.release-signature.toml", archive.display()))
}
