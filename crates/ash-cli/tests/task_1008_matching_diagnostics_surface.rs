#![allow(non_snake_case)]

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn cli_and_lsp_surface_matching_diagnostics_from_typeck_when_available() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("binder_mismatch.ash");
    fs::write(
        &source,
        r"
        fn main() {
            let 0 = 1;
            {};
        }
        ",
    )
    .expect("write source");

    let output = Command::cargo_bin("ash")
        .expect("ash binary")
        .args(["check", "--format", "json"])
        .arg(&source)
        .output()
        .expect("run ash check");

    assert!(
        !output.status.success(),
        "checked source with refutable binder should fail before runtime\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let json: Value = serde_json::from_str(&stdout).expect("json check output");
    let diagnostics = json["diagnostics"].as_array().expect("diagnostics array");
    let joined = diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic["message"].as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        joined.contains("non-irrefutable pattern in let"),
        "{joined}"
    );
    assert!(joined.contains("irrefutable"), "{joined}");
    assert!(joined.contains("use match or if let"), "{joined}");
}
