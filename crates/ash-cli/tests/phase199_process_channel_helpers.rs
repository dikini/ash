//! TASK-1945: productive process/channel helper stdlib surfaces.

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn process_channel_helper_fixture_imports_current_stdlib_api() {
    let relative = "examples/11-process-channel-helpers/process_channel_helpers.ash";
    let output = Command::new(assert_cmd::cargo::cargo_bin("ash"))
        .current_dir(repo_root())
        .args(["check", relative])
        .output()
        .expect("ash CLI should launch");

    assert!(
        output.status.success(),
        "process/channel helper fixture failed `ash check`: {relative}\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
