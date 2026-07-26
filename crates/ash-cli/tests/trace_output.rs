//! Integration tests for `ash trace` observable output.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

const MISSING_TYPED_LOWERING_ERROR: &str = "application execution failed: checked Core/CPS admission rejected: no validated production typed lowering is available";

#[test]
fn trace_stdout_rejects_generic_source_without_emitting_a_document() {
    let temp = tempdir().expect("tempdir");
    let entry_path = temp.path().join("main.ash");
    fs::write(&entry_path, "fn main() { 0 }\n").expect("write entry");

    Command::cargo_bin("ash")
        .expect("ash binary exists")
        .arg("trace")
        .arg(&entry_path)
        .assert()
        .code(5)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(MISSING_TYPED_LOWERING_ERROR));
}

#[test]
fn trace_output_file_is_not_created_when_generic_source_is_not_admitted() {
    let temp = tempdir().expect("tempdir");
    let entry_path = temp.path().join("main.ash");
    let output_path = temp.path().join("trace.json");
    fs::write(&entry_path, "fn main() { 1 }\n").expect("write entry");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("trace")
        .arg(&entry_path)
        .arg("--output")
        .arg(&output_path);

    cmd.assert()
        .code(5)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(MISSING_TYPED_LOWERING_ERROR));

    assert!(
        !output_path.exists(),
        "closed admission must not emit a partial trace file"
    );
}
