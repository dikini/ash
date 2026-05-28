use assert_cmd::Command;

mod support;

#[test]
fn task_970_default_list_current_and_update_switch_are_selector_only() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.oldoldoldold");
    support::install_fake_toolchain(&roots, "ash-0.2.0+test.source.newnewnewnew");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args(["default", "ash-0.1.0+test.source.oldoldoldold"])
        .envs(roots.env())
        .assert()
        .success();

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .arg("current")
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains("oldoldoldold"));

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .arg("list")
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicates::str::contains("oldoldoldold"))
        .stdout(predicates::str::contains("newnewnewnew"));

    Command::cargo_bin("ashgrove")
        .expect("ashgrove")
        .args([
            "update",
            "--to",
            "ash-0.2.0+test.source.newnewnewnew",
            "--from",
            "existing",
            "--switch",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let current = std::fs::read_to_string(roots.config.path().join("ash/toolchains.toml")).unwrap();
    assert!(current.contains("newnewnewnew"));
    assert!(
        roots
            .toolchain("ash-0.1.0+test.source.oldoldoldold")
            .join("manifest.toml")
            .is_file()
    );
}
