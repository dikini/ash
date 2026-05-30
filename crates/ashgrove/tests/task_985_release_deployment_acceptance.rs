use assert_cmd::Command;
use predicates::prelude::*;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

mod support;

#[test]
fn task_985_source_archive_to_runtime_support_to_cleanup_flow_passes() {
    let source = support::source_workspace_fixture();
    let package_dir = tempfile::tempdir().expect("package dir");
    let extract_dir = tempfile::tempdir().expect("extract dir");
    let script = workspace_root().join("scripts/package-ash-source-archive.sh");

    let package = std::process::Command::new("bash")
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
        package.status.success(),
        "source archive packaging failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&package.stdout),
        String::from_utf8_lossy(&package.stderr)
    );
    let stdout = String::from_utf8(package.stdout).expect("package stdout utf8");
    let archive = package_stdout_value(&stdout, "archive");

    let extract = std::process::Command::new("tar")
        .arg("-xzf")
        .arg(&archive)
        .arg("-C")
        .arg(extract_dir.path())
        .status()
        .expect("extract source archive");
    assert!(extract.success(), "extract source archive");
    let source_id = source.toolchain_id();
    let archive_root = extract_dir.path().join(&source_id);
    assert!(archive_root.join("release-source.toml").is_file());

    let roots = support::xdg_fixture();
    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "install",
            "--from",
            "source",
            "--path",
            archive_root.to_str().expect("utf8 archive root"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let manifest = read_toml(roots.toolchain(&source_id).join("manifest.toml"));
    assert_eq!(
        manifest
            .get("runtime_support")
            .and_then(|value| value.get("identity"))
            .and_then(toml::Value::as_str),
        Some("ash-runtime-support:0.1.0")
    );
    assert!(
        roots
            .toolchain(&source_id)
            .join("lib/ash/std/src/runtime")
            .is_dir()
    );

    let record = read_toml(roots.toolchain(&source_id).join("install-record.toml"));
    assert_eq!(
        record
            .get("source_origin_commit")
            .and_then(toml::Value::as_str),
        Some(source.revision())
    );
    assert_eq!(
        record.get("reproducible").and_then(toml::Value::as_bool),
        Some(true)
    );
    assert_digest_field(&record, "source_archive_digest");

    let project = support::project_fixture();
    let dep = support::git_dep_fixture();
    let locked_commit = dep.commit("v1");
    std::fs::write(
        project.path().join("ash.toml"),
        format!(
            "[package]\nname = \"task985-source\"\n\n[toolchain]\nash = \"{source_id}\"\n\n[dependencies.dep]\ngit = \"{}\"\ntag = \"v1\"\n",
            dep.url()
        ),
    )
    .expect("project manifest");

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
        .join(locked_commit);
    assert!(reachable_checkout.is_dir());

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "cleanup",
            "--project",
            project.path().to_str().expect("utf8"),
            "--cache",
            "--old-toolchains",
            "--dry-run",
        ])
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "protected default {source_id}"
        )))
        .stdout(predicate::str::contains(format!(
            "reachable cache {}",
            reachable_checkout.display()
        )));

    let selector =
        std::fs::read_to_string(roots.config.path().join("ash/toolchains.toml")).expect("selector");
    assert!(selector.contains(&format!("default = \"{source_id}\"")));
    assert!(roots.toolchain(&source_id).is_dir());
    assert!(project.path().join("ash.toml").is_file());
    assert!(project.path().join("ash.lock").is_file());
}

#[test]
fn task_985_tarball_url_release_index_trust_and_dispatcher_flow_passes() {
    let roots = support::xdg_fixture();
    let output = tempfile::tempdir().expect("output");
    let old_id = "ash-0.1.0+tarball.task985old";
    let new_id = "ash-0.1.0+tarball.task985new";
    let old_archive = packaged_manager_tarball(
        old_id,
        output.path(),
        "printf 'old-task985:%s:%s\\n' \"$ASHGROVE_RUNNING_TOOLCHAIN\" \"$ASH_RUNTIME_SUPPORT_IDENTITY\"\n",
        false,
    );
    let new_archive = packaged_manager_tarball(
        new_id,
        output.path(),
        "printf 'new-task985:%s:%s\\n' \"$ASHGROVE_RUNNING_TOOLCHAIN\" \"$ASH_RUNTIME_SUPPORT_IDENTITY\"\n",
        true,
    );
    write_valid_release_signature_sidecar(new_id, &new_archive);
    let new_digest = archive_digest(&new_archive);
    let new_url = format!("file://{}", new_archive.display());

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "install",
            "--from",
            "tarball",
            "--path",
            old_archive.to_str().expect("utf8 archive"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let unsigned_index = output.path().join("release-index.toml");
    std::fs::write(
        &unsigned_index,
        format!("[[release]]\ntoolchain_id = \"{new_id}\"\ntarball_url = \"{new_url}\"\n"),
    )
    .expect("unsigned release index");
    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            new_id,
            "--from",
            "tarball",
            "--release-index",
            unsigned_index.to_str().expect("utf8 release index"),
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsigned release index"));
    assert!(!roots.toolchain(new_id).exists());

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            new_id,
            "--from",
            "tarball",
            "--url",
            &new_url,
            "--digest",
            &new_digest,
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let record = std::fs::read_to_string(roots.toolchain(new_id).join("install-record.toml"))
        .expect("install record");
    assert!(record.contains(&format!("tarball_url = \"{new_url}\"")));
    assert!(record.contains(&format!("tarball_digest = \"{new_digest}\"")));
    assert!(record.contains("tarball_authentication = \"explicit-digest\""));

    let launcher_bin = support::ashgrove_paths(&roots).launcher_bin();
    let lifecycle = std::fs::read_to_string(launcher_bin.join(".ashgrove-dispatcher.toml"))
        .expect("dispatcher lifecycle");
    assert!(lifecycle.contains(&format!("manager_toolchain_id = \"{new_id}\"")));
    Command::new(launcher_bin.join("ash"))
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "new-task985:{new_id}:ash-runtime-support:0.1.0"
        )));

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["remove", old_id, "--force"])
        .write_stdin(format!("{old_id}\n"))
        .envs(roots.env())
        .assert()
        .success();
    assert!(!roots.toolchain(old_id).exists());
    assert!(roots.toolchain(new_id).is_dir());
}

fn packaged_manager_tarball(
    id: &str,
    output: &Path,
    ash_body: &str,
    signature_required: bool,
) -> PathBuf {
    let source = tempfile::tempdir().expect("toolchain source");
    support::create_toolchain_shape(source.path(), id);
    support::write_tool_script(&source.path().join("bin/ash"), ash_body);
    std::fs::copy(
        assert_cmd::cargo::cargo_bin("ashgrove"),
        source.path().join("bin/ashgrove"),
    )
    .expect("copy ashgrove");
    if signature_required {
        let manifest = source.path().join("manifest.toml");
        let mut text = std::fs::read_to_string(&manifest).expect("read manifest");
        text.push_str("\n[trust.release]\nsignature_required = true\n");
        std::fs::write(&manifest, text).expect("write manifest");
    }
    let archive = output.join(format!("{id}.tar.gz"));
    pack_toolchain_dir(id, source.path(), &archive);
    archive
}

fn pack_toolchain_dir(id: &str, source: &Path, archive: &Path) {
    let file = std::fs::File::create(archive).expect("archive");
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    builder.append_dir_all(id, source).expect("append");
    builder.finish().expect("finish");
}

fn write_valid_release_signature_sidecar(id: &str, archive: &Path) {
    let digest = archive_digest(archive);
    std::fs::write(
        release_signature_sidecar_path(archive),
        format!(
            "schema_version = 1\ntoolchain_id = \"{id}\"\ntarball_digest = \"{digest}\"\nsignature = \"task985-signature-evidence\"\n"
        ),
    )
    .expect("signature sidecar");
}

fn archive_digest(archive: &Path) -> String {
    let bytes = std::fs::read(archive).expect("archive bytes");
    format!("sha256:{:x}", Sha256::digest(&bytes))
}

fn release_signature_sidecar_path(archive: &Path) -> PathBuf {
    PathBuf::from(format!("{}.release-signature.toml", archive.display()))
}

fn read_toml(path: impl AsRef<Path>) -> toml::Value {
    let text = std::fs::read_to_string(path).expect("read toml");
    toml::from_str(&text).expect("parse toml")
}

fn assert_digest_field(record: &toml::Value, key: &str) {
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

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf()
}
