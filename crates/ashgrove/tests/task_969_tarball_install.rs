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
}
