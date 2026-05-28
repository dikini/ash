use ashgrove::{AshgrovePaths, ToolchainId};

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
