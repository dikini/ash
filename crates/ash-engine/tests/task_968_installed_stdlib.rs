//! Phase 127 installed stdlib root tests.

#[test]
fn task_968_installed_stdlib_root_override_is_used_for_imports() {
    let project = tempfile::tempdir().expect("project");
    let std_root = tempfile::tempdir().expect("std");
    std::fs::write(
        std_root.path().join("installed_only.ash"),
        "pub type InstalledOnly = InstalledOnly;\n",
    )
    .expect("stdlib module");
    let main = project.path().join("main.ash");
    std::fs::write(
        &main,
        "use installed_only::InstalledOnly\nworkflow main { ret 0 }\n",
    )
    .expect("main");

    let loaded = ash_engine::module_loader::with_module_roots(
        Vec::new(),
        Some(std_root.path().to_path_buf()),
        || ash_engine::module_loader::load_ordinary_file(&main),
    )
    .expect("installed stdlib root should resolve");

    assert!(
        loaded
            .imported_type_defs
            .iter()
            .any(|def| def.name == "InstalledOnly")
    );
}
