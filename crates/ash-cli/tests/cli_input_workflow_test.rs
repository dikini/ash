//! Tests for CLI workflow execution
//!
//! NOTE: The --input flag was removed in TASK-324. Input handling will be
//! redesigned in a future phase. These tests verify basic workflow execution
//! without input parameters.

use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test that a workflow without parameters executes successfully
#[test]
fn test_workflow_without_parameters() {
    let temp = TempDir::new().unwrap();

    // Create a simple workflow
    let workflow = r#"
        workflow main {
            ret "Hello, World";
        }
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
        "Workflow should execute successfully. stderr: {}",
        stderr
    );
    assert!(
        stdout.contains("Hello, World"),
        "Expected 'Hello, World' in output, got: {}",
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
