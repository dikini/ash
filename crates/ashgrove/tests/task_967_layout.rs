use ashgrove::{
    AshgrovePaths, CollisionStatus, InstallRecord, PublishOutcome, SelectorMetadata,
    StandardToolMetadata, StdlibMetadata, ToolchainId, ToolchainManifest, ToolchainStage,
    classify_toolchain_collision, stage_stdlib_metadata,
};
use assert_cmd::Command;

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
    let manifest = ToolchainManifest::new(id.clone(), "0.1.0-alpha.1")
        .with_target_triple("x86_64-unknown-linux-gnu")
        .with_stdlib(StdlibMetadata::new("0.1.0-alpha.1", "lib/ash/std"))
        .with_tool(StandardToolMetadata::required("ash", "bin/ash"))
        .with_tool(StandardToolMetadata::required("ashgrove", "bin/ashgrove"));

    let manifest_text = manifest.to_toml_string().expect("serialize manifest");
    let parsed_manifest = ToolchainManifest::from_toml_str(&manifest_text).expect("parse manifest");
    parsed_manifest
        .validate_for_toolchain(&id)
        .expect("manifest matches toolchain id");
    assert_eq!(parsed_manifest.toolchain_id(), &id);
    assert_eq!(parsed_manifest.stdlib().version(), "0.1.0-alpha.1");
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
