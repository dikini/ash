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
