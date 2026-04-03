//! Tests for CLI entry workflow execution.
//!
//! NOTE: Phase 57 redefines `ash run` around the canonical entry workflow
//! contract. These tests therefore exercise `main() -> Result<(), RuntimeError>`
//! entry sources rather than the legacy generic workflow execution path.

use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test that a canonical entry workflow executes successfully
#[test]
fn test_workflow_without_parameters() {
    let temp = TempDir::new().unwrap();

    // Create a simple canonical entry workflow
    let workflow = r#"
        use result::Result
        use runtime::RuntimeError

        workflow main() -> Result<(), RuntimeError> { done; }
    "#;
    let workflow_path = temp.path().join("greet.ash");
    fs::write(&workflow_path, workflow).unwrap();

    // Run without --input
    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "run"])
        .arg(&workflow_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(
        output.status.success(),
        "Entry workflow should execute successfully. stderr: {}",
        stderr
    );
    assert!(
        stdout.is_empty(),
        "Expected no stdout output, got: {}",
        stdout
    );
}

/// Test that a workflow with parameters can still be executed
/// (parameter binding via CLI is not yet supported per TASK-324)
#[test]
#[ignore = "TASK-324: CLI input binding removed. Workflows with parameters need interpreter support."]
fn test_workflow_with_parameters_ignored() {
    let temp = TempDir::new().unwrap();

    // Create a workflow with parameters
    let workflow = r#"
        workflow greet(name: String) {
            ret "Hello, " + name;
        }
    "#;
    let workflow_path = temp.path().join("greet.ash");
    fs::write(&workflow_path, workflow).unwrap();

    // Run without --input (CLI input binding removed)
    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "run"])
        .arg(&workflow_path)
        .output()
        .expect("Failed to execute");

    // Should fail because parameter is not provided
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("stderr: {}", stderr);
}
