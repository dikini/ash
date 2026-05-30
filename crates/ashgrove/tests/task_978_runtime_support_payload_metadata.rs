use assert_cmd::Command;

mod support;

#[test]
fn task_978_source_and_tarball_installs_have_equivalent_runtime_support_metadata() {
    let roots = support::xdg_fixture();
    let source = support::source_workspace_fixture();
    let tarball_id = "ash-0.1.0+tarball.runtime978equiv";
    let output = tempfile::tempdir().expect("tarball output");
    let archive = support::produced_toolchain_tarball_in(tarball_id, output.path());

    let mut source_cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    source_cmd
        .args([
            "install",
            "--from",
            "source",
            "--path",
            source.path().to_str().expect("utf8 source path"),
        ])
        .envs(roots.env())
        .assert()
        .success();

    let mut tarball_cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    tarball_cmd
        .args([
            "install",
            "--from",
            "tarball",
            "--path",
            archive.to_str().expect("utf8 archive path"),
        ])
        .envs(roots.env())
        .assert()
        .success();

    let source_metadata = runtime_support_metadata(&roots, &source.toolchain_id());
    let tarball_metadata = runtime_support_metadata(&roots, tarball_id);

    assert_eq!(source_metadata, tarball_metadata);
    assert_eq!(
        source_metadata
            .get("identity")
            .and_then(toml::Value::as_str),
        Some("ash-runtime-support:0.1.0")
    );
    assert_eq!(
        source_metadata.get("path").and_then(toml::Value::as_str),
        Some("lib/ash/std/src/runtime")
    );
    assert_eq!(
        source_metadata
            .get("required")
            .and_then(toml::Value::as_bool),
        Some(true)
    );
    assert!(
        roots
            .toolchain(&source.toolchain_id())
            .join("lib/ash/std/src/runtime")
            .is_dir()
    );
    assert!(
        roots
            .toolchain(tarball_id)
            .join("lib/ash/std/src/runtime")
            .is_dir()
    );
}

#[test]
fn task_978_tarball_missing_required_runtime_support_fails_closed() {
    let roots = support::xdg_fixture();
    let id = "ash-0.1.0+tarball.runtime978missing";
    let archive = support::toolchain_tarball_fixture_with_mutation(id, |root| {
        remove_runtime_support_metadata(&root.join("manifest.toml"));
    });

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "tarball",
        "--path",
        archive.path().to_str().expect("utf8 archive path"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("runtime support metadata"));

    assert!(!roots.toolchain(id).exists());
}

fn runtime_support_metadata(
    roots: &support::XdgFixture,
    id: &str,
) -> toml::map::Map<String, toml::Value> {
    let text = std::fs::read_to_string(roots.toolchain(id).join("manifest.toml"))
        .expect("installed manifest");
    let manifest: toml::Value = toml::from_str(&text).expect("manifest toml");
    manifest
        .get("runtime_support")
        .and_then(toml::Value::as_table)
        .cloned()
        .expect("runtime support metadata")
}

fn remove_runtime_support_metadata(path: &std::path::Path) {
    let text = std::fs::read_to_string(path).expect("read manifest");
    let mut filtered = Vec::new();
    let mut skipping = false;
    for line in text.lines() {
        if line.trim() == "[runtime_support]" {
            skipping = true;
            continue;
        }
        if skipping && line.trim_start().starts_with('[') {
            skipping = false;
        }
        if !skipping {
            filtered.push(line);
        }
    }
    std::fs::write(path, format!("{}\n", filtered.join("\n"))).expect("write manifest");
}
