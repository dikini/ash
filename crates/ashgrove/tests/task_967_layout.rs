use ashgrove::{
    AshgrovePaths, CollisionStatus, InstallRecord, LauncherDispatchRequest, PublishOutcome,
    RuntimeSupportMetadata, SelectorMetadata, StandardToolMetadata, StdlibMetadata, ToolchainId,
    ToolchainManifest, ToolchainStage, classify_toolchain_collision, install_launcher_shims,
    resolve_launcher_dispatch, stage_stdlib_metadata,
};
use assert_cmd::{Command, assert::OutputAssertExt};
use predicates::prelude::*;

mod support;

#[test]
fn task_967_xdg_paths_are_user_local_and_overridable() {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = AshgrovePaths::from_roots(dir.path().join("home"), None, None, None, None);

    assert!(paths.launcher_bin().ends_with(".local/bin"));
    assert!(
        paths
            .toolchains_dir()
            .ends_with(".local/share/ash/toolchains")
    );
    assert!(paths.config_dir().ends_with(".config/ash"));
    assert!(paths.cache_dir().ends_with(".cache/ash"));
    assert!(paths.state_dir().ends_with(".local/state/ash"));
}

#[test]
fn task_967_toolchain_id_rejects_path_like_values() {
    assert!(ToolchainId::parse("ash-0.1.0+x.source.abcdef123456").is_ok());
    assert!(ToolchainId::parse("../ash-0.1.0").is_err());
    assert!(ToolchainId::parse("ash/slash").is_err());
}

#[test]
fn task_967_toolchain_manifest_and_install_record_are_typed_public_apis() {
    let id = ToolchainId::parse("ash-0.1.0+test.source.metadatapi01").expect("toolchain id");
    let manifest = ToolchainManifest::new(id.clone(), "0.1.0")
        .with_target_triple("x86_64-unknown-linux-gnu")
        .with_stdlib(StdlibMetadata::new("0.1.0", "lib/ash/std"))
        .with_runtime_support(RuntimeSupportMetadata::required(
            "0.1.0",
            "lib/ash/std/src/runtime",
        ))
        .with_tool(StandardToolMetadata::required("ash", "bin/ash"))
        .with_tool(StandardToolMetadata::required("ashgrove", "bin/ashgrove"));

    let manifest_text = manifest.to_toml_string().expect("serialize manifest");
    let parsed_manifest = ToolchainManifest::from_toml_str(&manifest_text).expect("parse manifest");
    parsed_manifest
        .validate_for_toolchain(&id)
        .expect("manifest matches toolchain id");
    assert_eq!(parsed_manifest.toolchain_id(), &id);
    assert_eq!(parsed_manifest.stdlib().version(), "0.1.0");
    assert!(parsed_manifest.required_tool("ash").is_some());

    let record = InstallRecord::source_install(id.clone())
        .with_source_rev("abcdef1234567890abcdef1234567890abcdef12")
        .with_build_profile("release")
        .with_target_triple("x86_64-unknown-linux-gnu")
        .with_reproducible(true);

    let record_text = record.to_toml_string().expect("serialize install record");
    let parsed_record = InstallRecord::from_toml_str(&record_text).expect("parse install record");
    assert_eq!(parsed_record.toolchain_id(), &id);
    assert!(parsed_record.is_reproducible());
    assert_eq!(
        parsed_record.source_rev(),
        Some("abcdef1234567890abcdef1234567890abcdef12")
    );
}

#[test]
fn task_967_selector_metadata_preserves_unknown_trust_and_signing_fields() {
    let roots = support::xdg_fixture();
    let selector_path = roots.config.path().join("ash/toolchains.toml");
    std::fs::create_dir_all(selector_path.parent().expect("selector parent"))
        .expect("selector dir");
    std::fs::write(
        &selector_path,
        r#"
default = "ash-0.1.0+test.source.oldselector"

[projects]
"/tmp/project-a" = "ash-0.1.0+test.source.oldselector"

[trust]
signing = "none"
future_policy = "preserve-me"

[trust.signing_evidence]
opaque = "do-not-drop"
"#,
    )
    .expect("selector fixture");

    let new_default =
        ToolchainId::parse("ash-0.2.0+test.source.newselector").expect("new default id");
    let project = tempfile::tempdir().expect("project");
    let mut metadata = SelectorMetadata::read_from_path(&selector_path).expect("read selector");
    metadata.set_default(new_default.clone());
    metadata.record_project_toolchain(project.path(), new_default.clone());
    metadata
        .write_to_path(&selector_path)
        .expect("rewrite selector");

    let rewritten = std::fs::read_to_string(&selector_path).expect("selector text");
    assert!(rewritten.contains("future_policy = \"preserve-me\""));
    assert!(rewritten.contains("[trust.signing_evidence]"));
    assert!(rewritten.contains("opaque = \"do-not-drop\""));

    let reparsed = SelectorMetadata::read_from_path(&selector_path).expect("reparse selector");
    assert_eq!(reparsed.default(), Some(&new_default));
    assert_eq!(
        reparsed.project_toolchain(project.path()),
        Some(&new_default)
    );
}

#[test]
fn task_967_default_command_preserves_selector_trust_fields() {
    let roots = support::xdg_fixture();
    let id = "ash-0.1.0+test.source.trustdefault";
    support::install_fake_toolchain(&roots, id);
    let selector_path = roots.config.path().join("ash/toolchains.toml");
    std::fs::create_dir_all(selector_path.parent().expect("selector parent"))
        .expect("selector dir");
    std::fs::write(
        &selector_path,
        r#"
[trust]
future_policy = "preserve-me"
"#,
    )
    .expect("selector fixture");

    Command::cargo_bin("ashgrove")
        .expect("ashgrove binary")
        .args(["default", id])
        .envs(roots.env())
        .assert()
        .success();

    let rewritten = std::fs::read_to_string(selector_path).expect("selector text");
    assert!(rewritten.contains("future_policy = \"preserve-me\""));
    assert!(rewritten.contains(id));
}

#[test]
fn task_967_staging_publish_and_collision_helpers_are_deterministic() {
    let roots = support::xdg_fixture();
    let paths = AshgrovePaths::from_roots(
        roots.home.path().to_path_buf(),
        Some(roots.data.path().to_path_buf()),
        Some(roots.config.path().to_path_buf()),
        Some(roots.cache.path().to_path_buf()),
        Some(roots.state.path().to_path_buf()),
    );
    let id = ToolchainId::parse("ash-0.1.0+test.source.stagepub0001").expect("toolchain id");
    let source = support::source_fixture(id.as_str());

    let stage = ToolchainStage::create(&paths, id.clone()).expect("create staging dir");
    stage
        .copy_toolchain_payload(source.path())
        .expect("copy payload into staging");
    assert_eq!(
        stage.publish().expect("publish staged toolchain"),
        PublishOutcome::Published
    );

    assert_eq!(
        classify_toolchain_collision(&paths, &id, source.path()).expect("classify identical"),
        CollisionStatus::Identical
    );

    let conflicting = support::source_fixture(id.as_str());
    std::fs::write(
        conflicting.path().join("manifest.toml"),
        format!(
            "toolchain_id = \"{}\"\nversion = \"0.1.0-conflicting\"\n",
            id.as_str()
        ),
    )
    .expect("conflicting manifest");
    assert_eq!(
        classify_toolchain_collision(&paths, &id, conflicting.path()).expect("classify conflict"),
        CollisionStatus::Conflict
    );
}

#[test]
fn task_967_stdlib_metadata_stages_manifest_or_fails_closed() {
    let root = tempfile::tempdir().expect("toolchain root");
    std::fs::create_dir_all(root.path().join("lib/ash/std/src")).expect("stdlib src");

    let metadata = StdlibMetadata::new("0.1.0-alpha.1", "lib/ash/std");
    stage_stdlib_metadata(root.path(), &metadata).expect("stage stdlib metadata");

    let manifest =
        std::fs::read_to_string(root.path().join("lib/ash/std/ash.toml")).expect("stdlib manifest");
    assert!(manifest.contains("name = \"std\""));
    assert!(manifest.contains("version = \"0.1.0-alpha.1\""));

    let missing_stdlib = tempfile::tempdir().expect("missing stdlib root");
    let error = stage_stdlib_metadata(missing_stdlib.path(), &metadata)
        .expect_err("missing stdlib src must fail closed");
    assert!(error.to_string().contains("stdlib"));
}

#[test]
fn task_967_stdlib_metadata_rejects_paths_outside_toolchain_root() {
    let root = tempfile::tempdir().expect("toolchain root");
    std::fs::create_dir_all(root.path().join("lib/ash/std/src")).expect("stdlib src");

    let traversal = StdlibMetadata::new("0.1.0-alpha.1", "../escape");
    let error = stage_stdlib_metadata(root.path(), &traversal)
        .expect_err("traversal stdlib path must fail closed");
    assert!(error.to_string().contains("stdlib metadata path"));

    let absolute = StdlibMetadata::new("0.1.0-alpha.1", root.path().display().to_string());
    let error = stage_stdlib_metadata(root.path(), &absolute)
        .expect_err("absolute stdlib path must fail closed");
    assert!(error.to_string().contains("stdlib metadata path"));
}

#[test]
fn task_967_launcher_dispatch_prefers_project_pin_over_user_default() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.defaultpin01");
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.projectpin01");
    let project = support::project_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        "[toolchain]\nash = \"ash-0.1.0+test.source.projectpin01\"\n",
    )
    .expect("project manifest");
    let paths = support::ashgrove_paths(&roots);
    SelectorMetadata::from_toml_str("default = \"ash-0.1.0+test.source.defaultpin01\"\n")
        .expect("selector")
        .write_to_path(&roots.config.path().join("ash/toolchains.toml"))
        .expect("write selector");

    let dispatch = resolve_launcher_dispatch(
        &paths,
        LauncherDispatchRequest::new("ash").with_project(project.path()),
    )
    .expect("resolve dispatch");

    assert_eq!(
        dispatch.toolchain_id().as_str(),
        "ash-0.1.0+test.source.projectpin01"
    );
    assert_eq!(dispatch.tool_name(), "ash");
    assert!(dispatch.tool_path().ends_with("bin/ash"));
}

#[test]
fn task_967_launcher_dispatch_falls_back_to_user_default() {
    let roots = support::xdg_fixture();
    support::install_fake_toolchain(&roots, "ash-0.1.0+test.source.defaultonly1");
    let project = support::project_fixture();
    let paths = support::ashgrove_paths(&roots);
    SelectorMetadata::from_toml_str("default = \"ash-0.1.0+test.source.defaultonly1\"\n")
        .expect("selector")
        .write_to_path(&roots.config.path().join("ash/toolchains.toml"))
        .expect("write selector");

    let dispatch = resolve_launcher_dispatch(
        &paths,
        LauncherDispatchRequest::new("ashgrove").with_project(project.path()),
    )
    .expect("resolve dispatch");

    assert_eq!(
        dispatch.toolchain_id().as_str(),
        "ash-0.1.0+test.source.defaultonly1"
    );
    assert!(dispatch.tool_path().ends_with("bin/ashgrove"));
}

#[test]
fn task_967_launcher_shims_install_under_temp_home_and_execute_user_default_tool() {
    let roots = support::xdg_fixture();
    let id = "ash-0.1.0+test.source.shimdefault";
    support::install_fake_toolchain(&roots, id);
    support::write_tool_script(
        &roots.toolchain(id).join("bin/ash"),
        "printf 'ash:%s:%s\\n' \"$ASHGROVE_RUNNING_TOOLCHAIN\" \"$1\"\n",
    );
    let paths = support::ashgrove_paths(&roots);
    SelectorMetadata::from_toml_str(&format!("default = \"{id}\"\n"))
        .expect("selector")
        .write_to_path(&roots.config.path().join("ash/toolchains.toml"))
        .expect("write selector");
    let dispatcher = assert_cmd::cargo::cargo_bin("ashgrove");

    install_launcher_shims(&paths, &dispatcher).expect("install launcher shims");

    let ash_shim = paths.launcher_bin().join("ash");
    let ashgrove_shim = paths.launcher_bin().join("ashgrove");
    assert!(ash_shim.is_file());
    assert!(ashgrove_shim.is_file());
    assert!(ash_shim.starts_with(roots.home.path()));
    assert!(ashgrove_shim.starts_with(roots.home.path()));

    std::process::Command::new(ash_shim)
        .arg("payload")
        .envs(roots.env())
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("ash:{id}:payload")));
}

#[test]
fn task_967_launcher_dispatch_preserves_selected_tool_exit_code_without_wrapper_error() {
    let roots = support::xdg_fixture();
    let id = "ash-0.1.0+test.source.exitcode001";
    support::install_fake_toolchain(&roots, id);
    support::write_tool_script(&roots.toolchain(id).join("bin/ash"), "exit 73\n");
    let paths = support::ashgrove_paths(&roots);
    SelectorMetadata::from_toml_str(&format!("default = \"{id}\"\n"))
        .expect("selector")
        .write_to_path(&roots.config.path().join("ash/toolchains.toml"))
        .expect("write selector");
    install_launcher_shims(&paths, &assert_cmd::cargo::cargo_bin("ashgrove"))
        .expect("install launcher shims");

    std::process::Command::new(paths.launcher_bin().join("ash"))
        .envs(roots.env())
        .assert()
        .code(73)
        .stderr(predicate::str::is_empty());
}

#[test]
fn task_967_install_command_writes_shims_to_stable_user_local_dispatcher_copy() {
    let roots = support::xdg_fixture();
    let id = "ash-0.1.0+test.source.stabledisp1";
    let source = support::source_fixture(id);

    Command::cargo_bin("ashgrove")
        .expect("ashgrove binary")
        .args([
            "install",
            "--from",
            "source",
            "--path",
            &source.path().display().to_string(),
        ])
        .envs(roots.env())
        .assert()
        .success();

    let paths = support::ashgrove_paths(&roots);
    let ash_shim = paths.launcher_bin().join("ash");
    let shim_text = std::fs::read_to_string(&ash_shim).expect("ash shim");
    assert!(shim_text.contains(".ashgrove-dispatcher"));
    assert!(
        !shim_text.contains(
            &assert_cmd::cargo::cargo_bin("ashgrove")
                .display()
                .to_string()
        )
    );
    assert!(paths.launcher_bin().join(".ashgrove-dispatcher").is_file());
}

#[cfg(unix)]
#[test]
fn task_967_launcher_shim_install_does_not_follow_predictable_tmp_symlink() {
    use std::os::unix::fs::symlink;

    let roots = support::xdg_fixture();
    let paths = support::ashgrove_paths(&roots);
    std::fs::create_dir_all(paths.launcher_bin()).expect("launcher bin");
    let outside = tempfile::tempdir().expect("outside");
    let victim = outside.path().join("victim");
    std::fs::write(&victim, "do-not-touch").expect("victim");
    let predictable = paths
        .launcher_bin()
        .join(format!(".ash.tmp-{}", std::process::id()));
    symlink(&victim, &predictable).expect("predictable tmp symlink");

    install_launcher_shims(&paths, &assert_cmd::cargo::cargo_bin("ashgrove"))
        .expect("install launcher shims");

    assert_eq!(
        std::fs::read_to_string(&victim).expect("victim text"),
        "do-not-touch"
    );
}

#[test]
fn task_967_launcher_shim_explicit_env_override_wins_before_project_pin_and_default() {
    let roots = support::xdg_fixture();
    let default_id = "ash-0.1.0+test.source.shimdefault2";
    let project_id = "ash-0.1.0+test.source.shimproject2";
    let override_id = "ash-0.1.0+test.source.shimoverrid2";
    for id in [default_id, project_id, override_id] {
        support::install_fake_toolchain(&roots, id);
        support::write_tool_script(
            &roots.toolchain(id).join("bin/ash"),
            "printf 'selected:%s\\n' \"$ASHGROVE_RUNNING_TOOLCHAIN\"\n",
        );
    }
    let project = support::project_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        format!("[toolchain]\nash = \"{project_id}\"\n"),
    )
    .expect("project manifest");
    let paths = support::ashgrove_paths(&roots);
    SelectorMetadata::from_toml_str(&format!("default = \"{default_id}\"\n"))
        .expect("selector")
        .write_to_path(&roots.config.path().join("ash/toolchains.toml"))
        .expect("write selector");
    install_launcher_shims(&paths, &assert_cmd::cargo::cargo_bin("ashgrove"))
        .expect("install launcher shims");

    std::process::Command::new(paths.launcher_bin().join("ash"))
        .current_dir(project.path())
        .envs(roots.env())
        .env("ASH_TOOLCHAIN", override_id)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("selected:{override_id}")));
}

#[test]
fn task_967_launcher_shim_fails_closed_before_executing_incomplete_toolchain() {
    let roots = support::xdg_fixture();
    let id = "ash-0.1.0+test.source.shimbroken1";
    support::install_fake_toolchain(&roots, id);
    std::fs::remove_file(roots.toolchain(id).join("manifest.toml")).expect("remove manifest");
    let paths = support::ashgrove_paths(&roots);
    SelectorMetadata::from_toml_str(&format!("default = \"{id}\"\n"))
        .expect("selector")
        .write_to_path(&roots.config.path().join("ash/toolchains.toml"))
        .expect("write selector");
    install_launcher_shims(&paths, &assert_cmd::cargo::cargo_bin("ashgrove"))
        .expect("install launcher shims");

    std::process::Command::new(paths.launcher_bin().join("ash"))
        .envs(roots.env())
        .assert()
        .failure()
        .stderr(predicate::str::contains("validate selected toolchain"))
        .stderr(predicate::str::contains(id));
}

#[test]
fn task_967_launcher_dispatch_fails_closed_for_missing_selected_toolchain() {
    let roots = support::xdg_fixture();
    let project = support::project_fixture();
    std::fs::write(
        project.path().join("ash.toml"),
        "[toolchain]\nash = \"ash-0.1.0+test.source.missingpin001\"\n",
    )
    .expect("project manifest");
    let paths = support::ashgrove_paths(&roots);
    SelectorMetadata::from_toml_str("default = \"ash-0.1.0+test.source.missingdefault1\"\n")
        .expect("selector")
        .write_to_path(&roots.config.path().join("ash/toolchains.toml"))
        .expect("write selector");

    let error = resolve_launcher_dispatch(
        &paths,
        LauncherDispatchRequest::new("ash").with_project(project.path()),
    )
    .expect_err("missing project pin must fail closed");

    assert!(error.to_string().contains("project pin"));
    assert!(error.to_string().contains("is not installed"));
    assert!(error.to_string().contains("install"));
}

#[cfg(unix)]
#[test]
fn task_967_launcher_dispatch_rejects_symlink_tool_escaping_toolchain_root() {
    use std::os::unix::fs::symlink;

    let roots = support::xdg_fixture();
    let id = "ash-0.1.0+test.source.symlinkescape";
    support::install_fake_toolchain(&roots, id);
    let outside = tempfile::tempdir().expect("outside");
    let escaped_tool = outside.path().join("ash");
    std::fs::write(&escaped_tool, "#!/bin/sh\n").expect("escaped tool");
    let tool_path = roots.toolchain(id).join("bin/ash");
    std::fs::remove_file(&tool_path).expect("remove original tool");
    symlink(&escaped_tool, &tool_path).expect("escaping symlink");
    let paths = support::ashgrove_paths(&roots);
    SelectorMetadata::from_toml_str(&format!("default = \"{id}\"\n"))
        .expect("selector")
        .write_to_path(&roots.config.path().join("ash/toolchains.toml"))
        .expect("write selector");

    let error = resolve_launcher_dispatch(&paths, LauncherDispatchRequest::new("ash"))
        .expect_err("escaping tool symlink must fail closed");

    let diagnostic = format!("{error:#}");
    assert!(diagnostic.contains("standard tool path"));
    assert!(diagnostic.contains("toolchain root"));
}

#[cfg(unix)]
#[test]
fn task_967_launcher_dispatch_rejects_symlink_toolchain_root_before_canonicalizing() {
    use std::os::unix::fs::symlink;

    let roots = support::xdg_fixture();
    let id = "ash-0.1.0+test.source.rootsymlink";
    let outside = tempfile::tempdir().expect("outside");
    support::create_toolchain_shape(outside.path(), id);
    std::fs::create_dir_all(roots.data.path().join("ash/toolchains")).expect("toolchains root");
    symlink(outside.path(), roots.toolchain(id)).expect("toolchain root symlink");
    let paths = support::ashgrove_paths(&roots);
    SelectorMetadata::from_toml_str(&format!("default = \"{id}\"\n"))
        .expect("selector")
        .write_to_path(&roots.config.path().join("ash/toolchains.toml"))
        .expect("write selector");

    let error = resolve_launcher_dispatch(&paths, LauncherDispatchRequest::new("ash"))
        .expect_err("symlink toolchain root must fail closed");

    let diagnostic = format!("{error:#}");
    assert!(diagnostic.contains("selected toolchain"));
    assert!(diagnostic.contains("symlink"));
}

#[test]
fn task_967_launcher_dispatch_rejects_manifest_tool_path_traversal() {
    let roots = support::xdg_fixture();
    let id = "ash-0.1.0+test.source.toolpathtrav";
    support::install_fake_toolchain(&roots, id);
    std::fs::write(
        roots.toolchain(id).join("manifest.toml"),
        format!(
            "toolchain_id = \"{id}\"\nversion = \"0.1.0\"\nsource_kind = \"fixture\"\n[stdlib]\nversion = \"0.1.0\"\npath = \"lib/ash/std\"\n[runtime_support]\nidentity = \"ash-runtime-support:0.1.0\"\npath = \"lib/ash/std/src/runtime\"\nrequired = true\n[[standard_tools]]\nname = \"ash\"\npath = \"../escape/ash\"\nrequired = true\n[[standard_tools]]\nname = \"ashgrove\"\npath = \"bin/ashgrove\"\nrequired = true\n"
        ),
    )
    .expect("manifest");
    let paths = support::ashgrove_paths(&roots);
    SelectorMetadata::from_toml_str(&format!("default = \"{id}\"\n"))
        .expect("selector")
        .write_to_path(&roots.config.path().join("ash/toolchains.toml"))
        .expect("write selector");

    let error = resolve_launcher_dispatch(&paths, LauncherDispatchRequest::new("ash"))
        .expect_err("manifest tool traversal must fail closed");

    let diagnostic = format!("{error:#}");
    assert!(diagnostic.contains("standard tool path"));
    assert!(diagnostic.contains("toolchain root"));
}
