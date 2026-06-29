//! TASK-1737 audit proof for the current expanded-surface boundary bypass.

use ash_engine::Engine;

fn write_module(source: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("module.ash");
    std::fs::write(&path, source).expect("write module");
    (dir, path)
}

#[test]
fn check_module_file_currently_bypasses_expansion_for_pub_fn_body() {
    let (_dir, path) = write_module(
        r"
pub fn unresolved_section() -> Int {
    (<*>)
}
",
    );
    let engine = Engine::new().build().expect("engine builds");

    let result = engine.check_module_file(&path);

    assert!(
        result.is_ok(),
        "TASK-1737 audit proof: check_module_file currently does not route pub fn bodies through expanded-surface validation; TASK-1738 must flip this expectation"
    );
}
