use assert_cmd::Command;
use sha2::{Digest, Sha256};

mod support;

#[test]
fn task_979_url_install_without_digest_or_signed_index_fails_closed() {
    let roots = support::xdg_fixture();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "install",
            "--from",
            "tarball",
            "--url",
            "http://127.0.0.1:9/ash.tar.gz",
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("authenticated"));

    assert!(!roots.toolchain("ash-0.1.0+tarball.url979").exists());
}

#[test]
fn task_979_url_install_records_authenticated_url_and_digest() {
    let roots = support::xdg_fixture();
    let output = tempfile::tempdir().expect("output");
    let toolchain_id = "ash-0.1.0+tarball.urlinstall979";
    let archive = support::produced_toolchain_tarball_in(toolchain_id, output.path());
    let archive_bytes = std::fs::read(&archive).expect("archive bytes");
    let digest = format!("sha256:{:x}", Sha256::digest(&archive_bytes));
    let url = format!("file://{}", archive.display());

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "install", "--from", "tarball", "--url", &url, "--digest", &digest, "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let record = std::fs::read_to_string(roots.toolchain(toolchain_id).join("install-record.toml"))
        .expect("record");
    assert!(record.contains("source_kind = \"tarball\""));
    assert!(record.contains(&format!("tarball_url = \"{url}\"")));
    assert!(record.contains(&format!("tarball_digest = \"{digest}\"")));
    assert!(record.contains("tarball_authentication = \"explicit-digest\""));
}

#[test]
fn task_979_update_from_release_index_rejects_unsigned_or_digest_mismatch() {
    let roots = support::xdg_fixture();
    let old_id = "ash-0.1.0+test.source.oldurl979";
    support::install_fake_toolchain(&roots, old_id);

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["default", old_id])
        .envs(roots.env())
        .assert()
        .success();

    let output = tempfile::tempdir().expect("output");
    let toolchain_id = "ash-0.1.0+tarball.urlupdate979";
    let archive = support::produced_toolchain_tarball_in(toolchain_id, output.path());
    let url = format!("file://{}", archive.display());

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            toolchain_id,
            "--from",
            "tarball",
            "--url",
            &url,
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("authenticated download policy"));

    assert!(!roots.toolchain(toolchain_id).exists());

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            toolchain_id,
            "--from",
            "tarball",
            "--url",
            &url,
            "--digest",
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("tarball digest mismatch"));

    assert!(!roots.toolchain(toolchain_id).exists());
    let selector =
        std::fs::read_to_string(roots.config.path().join("ash/toolchains.toml")).expect("selector");
    assert!(selector.contains(old_id));
    assert!(!selector.contains(toolchain_id));
}
