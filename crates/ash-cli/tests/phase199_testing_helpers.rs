//! TASK-1944: productive testing helper stdlib surfaces.

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_ash_check(relative: &str) -> (bool, String, String) {
    let output = Command::new(assert_cmd::cargo::cargo_bin("ash"))
        .current_dir(repo_root())
        .args(["check", relative])
        .output()
        .expect("ash CLI should launch");

    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn testing_helper_fixture_imports_current_stdlib_api() {
    let relative = "examples/10-testing-helpers/testing_helpers.ash";
    let (success, stdout, stderr) = run_ash_check(relative);
    assert!(
        success,
        "testing helper fixture failed `ash check`: {relative}\nstdout={stdout}\nstderr={stderr}"
    );
}
