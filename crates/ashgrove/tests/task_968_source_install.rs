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
fn task_968_source_install_builds_real_source_root_and_records_git_metadata() {
    let source = support::source_workspace_fixture();
    let roots = support::xdg_fixture();
    let expected_id = source.toolchain_id();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "source",
        "--path",
        source.path().to_str().expect("utf8"),
        "--switch",
    ])
    .envs(roots.env())
    .assert()
    .success();

    let installed = roots.toolchain(&expected_id);
    assert!(installed.join("bin/ash").is_file());
    assert!(installed.join("bin/ashgrove").is_file());
    assert!(installed.join("lib/ash/std/ash.toml").is_file());
    assert!(installed.join("lib/ash/std/src/lib.ash").is_file());
    assert!(installed.join("manifest.toml").is_file());
    assert!(installed.join("install-record.toml").is_file());

    let manifest = std::fs::read_to_string(installed.join("manifest.toml")).expect("manifest");
    assert!(manifest.contains(&format!("toolchain_id = \"{expected_id}\"")));
    assert!(manifest.contains("source_kind = \"source\""));
    assert!(manifest.contains("name = \"ash\""));
    assert!(manifest.contains("name = \"ashgrove\""));

    let record =
        std::fs::read_to_string(installed.join("install-record.toml")).expect("install record");
    assert!(record.contains("source_kind = \"source\""));
    assert!(record.contains(&format!("source_path = \"{}\"", source.path().display())));
    assert!(record.contains("source_url = \"https://example.invalid/ash.git\""));
    assert!(record.contains(&format!("source_rev = \"{}\"", source.revision())));
    assert!(record.contains("allow_dirty_source = false"));
    assert!(record.contains("allow_unidentified_source = false"));
    assert!(record.contains("reproducible = true"));
}

#[test]
fn task_968_source_install_keeps_clean_source_root_clean_without_lockfile() {
    let source = support::source_workspace_fixture();
    assert!(!source.path().join("Cargo.lock").exists());
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "source",
        "--path",
        source.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .success();

    assert!(!source.path().join("Cargo.lock").exists());
    assert_eq!(support::git_status(source.path()), "");
}

#[test]
fn task_968_source_install_rejects_git_status_failure_for_identified_source() {
    let source = support::source_workspace_fixture();
    let fake_bin = tempfile::tempdir().expect("fake bin");
    support::write_tool_script(
        &fake_bin.path().join("git"),
        "case \"$*\" in\n  *\"rev-parse HEAD\"*) printf '%s\\n' '0123456789abcdef0123456789abcdef01234567'; exit 0 ;;\n  *\"status --porcelain\"*) printf '%s\\n' 'status failed' >&2; exit 1 ;;\n  *) exit 1 ;;\nesac\n",
    );
    let roots = support::xdg_fixture();
    let path = format!(
        "{}:{}",
        fake_bin.path().display(),
        std::env::var("PATH").expect("PATH")
    );

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "source",
        "--path",
        source.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .env("PATH", path)
    .assert()
    .failure()
    .stderr(predicates::str::contains("git status failed"));
}

#[test]
fn task_968_source_install_rejects_corrupt_git_metadata_even_with_unidentified_override() {
    let source = support::unidentified_source_workspace_fixture();
    std::fs::create_dir(source.path().join(".git")).expect("corrupt git metadata marker");
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "source",
        "--path",
        source.path().to_str().expect("utf8"),
        "--allow-unidentified-source",
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("git revision failed"));
}

#[test]
fn task_968_dirty_source_override_ids_include_content_digest() {
    let first = support::source_workspace_fixture();
    let second = support::source_workspace_fixture();
    std::fs::write(first.path().join("dirty.txt"), "first dirty payload\n").expect("first dirty");
    std::fs::write(second.path().join("dirty.txt"), "second dirty payload\n")
        .expect("second dirty");
    let roots = support::xdg_fixture();

    let mut first_cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    first_cmd
        .args([
            "install",
            "--from",
            "source",
            "--path",
            first.path().to_str().expect("utf8"),
            "--allow-dirty-source",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let mut second_cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    second_cmd
        .args([
            "install",
            "--from",
            "source",
            "--path",
            second.path().to_str().expect("utf8"),
            "--allow-dirty-source",
        ])
        .envs(roots.env())
        .assert()
        .success();

    let installed = std::fs::read_dir(roots.data.path().join("ash/toolchains"))
        .expect("toolchains")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .into_string()
                .expect("utf8")
        })
        .filter(|name| name.contains("+source.") && name.contains(".dirty"))
        .collect::<Vec<_>>();
    assert_eq!(installed.len(), 2, "{installed:?}");
    assert_ne!(installed[0], installed[1]);

    for id in installed {
        let record = std::fs::read_to_string(roots.toolchain(&id).join("install-record.toml"))
            .expect("record");
        assert!(record.contains("allow_dirty_source = true"));
        assert!(record.contains("dirty_source_digest = \"sha256:"));
        assert!(record.contains("reproducible = false"));
    }
}

#[test]
fn task_968_launcher_public_path_passes_selected_toolchain_stdlib_root() {
    let roots = support::xdg_fixture();
    let id = "ash-0.1.0+test.source.stdlibroute";
    support::install_fake_toolchain(&roots, id);
    support::write_tool_script(
        &roots.toolchain(id).join("bin/ash"),
        "printf '%s\\n' \"$ASH_STDLIB_ROOT\"\n",
    );
    let paths = support::ashgrove_paths(&roots);
    ashgrove::SelectorMetadata::from_toml_str(&format!("default = \"{id}\"\n"))
        .expect("selector")
        .write_to_path(&roots.config.path().join("ash/toolchains.toml"))
        .expect("write selector");
    ashgrove::install_launcher_shims(&paths, &assert_cmd::cargo::cargo_bin("ashgrove"))
        .expect("install launcher shims");

    let output = spawn_with_text_busy_retry(paths.launcher_bin().join("ash"), roots.env());
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(
        stdout.contains(&format!(
            "{}/lib/ash/std/src",
            roots.toolchain(id).display()
        )),
        "{stdout}"
    );
}

fn spawn_with_text_busy_retry(
    program: std::path::PathBuf,
    env: Vec<(&'static str, String)>,
) -> std::process::Output {
    let mut last_error = None;
    for _ in 0..20 {
        match std::process::Command::new(&program)
            .envs(env.clone())
            .output()
        {
            Ok(output) => return output,
            Err(error) if error.raw_os_error() == Some(26) => {
                last_error = Some(error);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(error) => panic!("failed to spawn {}: {error}", program.display()),
        }
    }
    panic!(
        "failed to spawn {} after ETXTBSY retries: {}",
        program.display(),
        last_error.expect("text busy error")
    );
}

#[test]
fn task_968_source_install_rejects_dirty_git_source_without_override() {
    let source = support::source_workspace_fixture();
    std::fs::write(source.path().join("uncommitted.txt"), "dirty").expect("dirty file");
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "source",
        "--path",
        source.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("dirty source"));
}

#[test]
fn task_968_source_install_rejects_real_unidentified_source_without_override() {
    let source = support::unidentified_source_workspace_fixture();
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "source",
        "--path",
        source.path().to_str().expect("utf8"),
    ])
    .envs(roots.env())
    .assert()
    .failure()
    .stderr(predicates::str::contains("unidentified source"));
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

#[test]
fn task_968_source_install_rejects_unidentified_archive_without_override() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.cccccccccccc");
    std::fs::remove_file(fixture.path().join(".source-rev")).expect("remove source rev");
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
    .stderr(predicates::str::contains("unidentified source"));
}

#[test]
fn task_968_source_install_rejects_empty_source_rev_without_override() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.cddddddddddd");
    std::fs::write(fixture.path().join(".source-rev"), " \n").expect("empty source rev");
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
    .stderr(predicates::str::contains("unidentified source"));
}

#[test]
fn task_968_source_install_records_source_metadata_and_overrides() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.dddddddddddd");
    std::fs::write(fixture.path().join(".dirty"), "dirty").expect("dirty marker");
    std::fs::write(
        fixture.path().join(".source-url"),
        "https://example.invalid/ash.git\n",
    )
    .expect("source url");
    let roots = support::xdg_fixture();

    let mut cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    cmd.args([
        "install",
        "--from",
        "source",
        "--path",
        fixture.path().to_str().expect("utf8"),
        "--allow-dirty-source",
    ])
    .envs(roots.env())
    .assert()
    .success();

    let record = std::fs::read_to_string(
        roots
            .toolchain("ash-0.1.0+test.source.dddddddddddd")
            .join("install-record.toml"),
    )
    .expect("install record");
    assert!(record.contains("source_kind = \"source\""));
    assert!(record.contains("source_url = \"https://example.invalid/ash.git\""));
    assert!(record.contains("source_rev = \"abcdef1234567890\""));
    assert!(record.contains("build_profile = \"debug\""));
    assert!(record.contains("target_triple = "));
    assert!(record.contains("allow_dirty_source = true"));
    assert!(record.contains("allow_unidentified_source = false"));
    assert!(record.contains("reproducible = false"));
}

#[test]
fn task_968_source_install_records_unidentified_override_as_non_reproducible() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.eeeeeeeeeeee");
    std::fs::remove_file(fixture.path().join(".source-rev")).expect("remove source rev");
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

    let record = std::fs::read_to_string(
        roots
            .toolchain("ash-0.1.0+test.source.eeeeeeeeeeee")
            .join("install-record.toml"),
    )
    .expect("install record");
    assert!(record.contains("allow_unidentified_source = true"));
    assert!(record.contains("reproducible = false"));
    assert!(!record.contains("source_rev = "));
}

#[test]
fn task_968_source_install_rejects_same_id_with_different_source_metadata() {
    let first = support::source_fixture("ash-0.1.0+test.source.ffffffffffff");
    let second = support::source_fixture("ash-0.1.0+test.source.ffffffffffff");
    std::fs::write(
        second.path().join("manifest.toml"),
        "toolchain_id = \"ash-0.1.0+test.source.ffffffffffff\"\nversion = \"0.1.0\"\nsource_kind = \"fixture\"\n[stdlib]\nversion = \"0.1.1\"\npath = \"lib/ash/std\"\n[[standard_tools]]\nname = \"ash\"\npath = \"bin/ash\"\nrequired = true\n[[standard_tools]]\nname = \"ashgrove\"\npath = \"bin/ashgrove\"\nrequired = true\n",
    )
    .expect("different manifest");
    let roots = support::xdg_fixture();

    let mut first_cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    first_cmd
        .args([
            "install",
            "--from",
            "source",
            "--path",
            first.path().to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .success();

    let mut second_cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    second_cmd
        .args([
            "install",
            "--from",
            "source",
            "--path",
            second.path().to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("metadata collision"));
}

#[test]
fn task_968_source_install_rejects_same_id_with_different_source_rev() {
    let first = support::source_fixture("ash-0.1.0+test.source.aaaaaaaa9999");
    let second = support::source_fixture("ash-0.1.0+test.source.aaaaaaaa9999");
    std::fs::write(second.path().join(".source-rev"), "fedcba0987654321\n")
        .expect("different source rev");
    let roots = support::xdg_fixture();

    let mut first_cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    first_cmd
        .args([
            "install",
            "--from",
            "source",
            "--path",
            first.path().to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .success();

    let mut second_cmd = Command::cargo_bin("ashgrove").expect("ashgrove binary");
    second_cmd
        .args([
            "install",
            "--from",
            "source",
            "--path",
            second.path().to_str().expect("utf8"),
        ])
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicates::str::contains("metadata collision"));
}

#[test]
fn task_968_source_install_identical_reinstall_is_deterministic_noop() {
    let fixture = support::source_fixture("ash-0.1.0+test.source.999999999999");
    let roots = support::xdg_fixture();

    for _ in 0..2 {
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
    }

    let installed = roots.toolchain("ash-0.1.0+test.source.999999999999");
    assert!(installed.join("manifest.toml").is_file());
    assert!(
        !roots
            .data
            .path()
            .join("ash/toolchains/.staging/ash-0.1.0+test.source.999999999999")
            .exists()
    );
}
