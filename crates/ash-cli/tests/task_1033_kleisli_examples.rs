#![allow(missing_docs)]

use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn kleisli_examples_cli_rejects_removed_concrete_wrappers() {
    let project = tempdir().expect("project");
    let main = project.path().join("kleisli_examples.ash");
    std::fs::write(
        &main,
        r#"use algebra::kleisli::{id_option}

workflow main { ret 0 }
"#,
    )
    .expect("write example");

    let output = Command::cargo_bin("ash")
        .expect("ash binary")
        .arg("check")
        .arg(&main)
        .output()
        .expect("run ash check");

    assert!(
        !output.status.success(),
        "ash check unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("id_option"), "{combined}");
}
