//! Tests for CLI entry-source execution.
//!
//! NOTE: Phase 57 redefines `ash run` around the canonical entry-source
//! contract. These tests therefore exercise `main() -> Result<(), RuntimeError>`
//! entry sources rather than the non-entry execution path.

use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test that a canonical entry source executes successfully.
#[test]
fn test_entry_source_without_parameters() {
    let temp = TempDir::new().unwrap();

    // Create a simple canonical entry source.
    let entry_source = r#"
        use result::Result
        use runtime::RuntimeError

        fn main() -> Result<(), RuntimeError> { Ok { value: {} } }
    "#;
    let entry_path = temp.path().join("greet.ash");
    fs::write(&entry_path, entry_source).unwrap();

    // Run without --input. Use Cargo's test-provided binary path rather than
    // nesting `cargo run` inside `cargo test --workspace`; nested Cargo builds
    // can stall the broad closeout gate under workspace-level test runs.
    let output = Command::new(env!("CARGO_BIN_EXE_ash"))
        .arg("run")
        .arg(&entry_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(
        output.status.success(),
        "Entry source should execute successfully. stderr: {}",
        stderr
    );
    assert!(
        stdout.is_empty(),
        "Expected no stdout output, got: {}",
        stdout
    );
}

/// Test that an entry source with parameters can still be executed
/// (parameter binding via CLI is not yet supported per TASK-324)
#[test]
#[ignore = "TASK-324: CLI input binding removed. Entry sources with parameters need interpreter support."]
fn test_entry_source_with_parameters_ignored() {
    let temp = TempDir::new().unwrap();

    // Create a target entry function with parameters
    let entry_source = r#"
        fn main(name: String) -> String {
            "Hello, " + name
        }
    "#;
    let entry_path = temp.path().join("greet.ash");
    fs::write(&entry_path, entry_source).unwrap();

    // Run without --input (CLI input binding removed)
    let output = Command::new(env!("CARGO_BIN_EXE_ash"))
        .arg("run")
        .arg(&entry_path)
        .output()
        .expect("Failed to execute");

    // Should fail because parameter is not provided
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("stderr: {}", stderr);
}
