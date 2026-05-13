//! TASK-742: CLI conformance for Phase 104 capability implementation examples.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_ash_check(file_path: &Path) -> (Option<i32>, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_ash"))
        .current_dir(repo_root())
        .args(["check"])
        .arg(file_path)
        .output()
        .expect("ash CLI should launch");

    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn phase104_capability_examples_are_cli_checkable() {
    for relative in [
        "examples/06-capability-implementations/01-mock-internal-kv.ash",
        "examples/06-capability-implementations/02-caching-kv-adapter.ash",
        "examples/06-capability-implementations/03-recording-replay-sketch.ash",
    ] {
        let example = repo_root().join(relative);
        assert!(example.exists(), "missing Phase 104 example: {relative}");

        let (code, stdout, stderr) = run_ash_check(&example);
        assert_eq!(
            code,
            Some(0),
            "ash check failed for {relative}: stdout={stdout:?} stderr={stderr:?}"
        );
    }
}

#[test]
fn phase104_examples_readme_documents_runtime_boundary_honestly() {
    let readme = std::fs::read_to_string(
        repo_root().join("examples/06-capability-implementations/README.md"),
    )
    .expect("read Phase 104 examples README");

    for expected in [
        "01-mock-internal-kv.ash",
        "02-caching-kv-adapter.ash",
        "03-recording-replay-sketch.ash",
        "ash check",
        "runtime API support",
        "not complete yet",
    ] {
        assert!(
            readme.contains(expected),
            "Phase 104 README should mention {expected:?}"
        );
    }

    let top_level = std::fs::read_to_string(repo_root().join("examples/README.md"))
        .expect("read top-level examples README");
    assert!(top_level.contains("06-capability-implementations"));
}
