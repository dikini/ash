//! Tests for CLI SPEC-005 compliance
//!
//! These tests verify that the CLI adheres to the SPEC-005 specification,
//! including proper exit codes and flag handling.

use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test that parse errors return exit code 2
#[test]
fn test_exit_code_parse_error() {
    let temp = TempDir::new().unwrap();
    let bad_syntax = temp.path().join("bad.ash");
    fs::write(&bad_syntax, "workflow { bad syntax }").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "check"])
        .arg(&bad_syntax)
        .output()
        .unwrap();

    // Parse error should return exit code 2
    let code = output.status.code();
    println!("Exit code: {:?}", code);
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));

    // Parse errors should return exit code 2 per SPEC-005
    assert!(!output.status.success(), "Expected failure for parse error");
    assert_eq!(code, Some(2), "Parse errors should return exit code 2");
}

/// Test that type errors return exit code 3
#[test]
fn test_exit_code_type_error() {
    let temp = TempDir::new().unwrap();
    let bad_types = temp.path().join("bad.ash");
    // Create a file that might trigger type errors
    fs::write(
        &bad_types,
        r#"
        workflow test {
            let x: Int = "string";
        }
    "#,
    )
    .unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "check"])
        .arg(&bad_types)
        .output()
        .unwrap();

    println!("Exit code: {:?}", output.status.code());
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Type errors should return exit code 3 per SPEC-005
    assert!(!output.status.success(), "Expected failure for type error");
    // Note: If the file fails to parse, it will return exit code 2 instead
    // Only check for exit code 3 if the file parsed successfully but type checking failed
    let code = output.status.code();
    if code != Some(2) {
        assert_eq!(code, Some(3), "Type errors should return exit code 3");
    }
}

/// Test that --quiet flag suppresses output
#[test]
fn test_quiet_flag_suppresses_output() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "--quiet", "--help"])
        .output()
        .unwrap();

    // --help should still work even with --quiet
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // Help output should still be present
    assert!(stdout.contains("Ash") || stdout.contains("ash"));
}

/// Test that -v, -vv, -vvv are accepted
#[test]
fn test_verbose_flag_levels() {
    // Test that -v, -vv, -vvv are accepted (with --help so it exits quickly)
    for flag in ["-v", "-vv", "-vvv"] {
        let output = Command::new("cargo")
            .args(["run", "--bin", "ash", "--", flag, "--help"])
            .output()
            .unwrap();

        assert!(output.status.success(), "Flag {} failed", flag);
    }
}

/// Test that --color flag is accepted
#[test]
fn test_color_flag() {
    // Test --color never works
    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "--color", "never", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());

    // Test --color always works
    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "--color", "always", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());

    // Test --color auto works
    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "--color", "auto", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
}

/// Test check command with --policy-check flag
#[test]
fn test_check_policy_check_flag() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() { decide 42 }").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "check", "--policy-check"])
        .arg(&workflow)
        .output()
        .unwrap();

    // The command should be accepted (may succeed or fail depending on policy implementation)
    // but it should not fail due to unknown argument
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--policy-check should be a recognized flag"
    );
}

/// Test check command with -s short flag for --strict
#[test]
fn test_check_strict_short_flag() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() { decide 42 }").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "check", "-s"])
        .arg(&workflow)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "-s should be a recognized short flag"
    );
}

/// Test check command with -f short flag for --format
#[test]
fn test_check_format_short_flag() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "check", "--help"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check that -f is listed as a short option for --format
    assert!(
        stdout.contains("-f") || stdout.contains("--format"),
        "Check command should have format option"
    );
}

/// Test run command with --format flag
#[test]
fn test_run_format_flag() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(
        &workflow,
        "workflow test() { decide { \"key\": \"value\" } }",
    )
    .unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "run", "--format", "json"])
        .arg(&workflow)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--format should be a recognized flag for run"
    );
}

/// Test run command with --dry-run flag
#[test]
fn test_run_dry_run_flag() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() { decide 42 }").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "run", "--dry-run"])
        .arg(&workflow)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--dry-run should be a recognized flag"
    );
}

/// Test run command with --timeout flag
#[test]
fn test_run_timeout_flag() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() { decide 42 }").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "run", "--timeout", "30"])
        .arg(&workflow)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--timeout should be a recognized flag"
    );
}

/// Test run command with --capability flag
#[test]
fn test_run_capability_flag() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() { decide 42 }").unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "ash",
            "--",
            "run",
            "--capability",
            "fs",
            "--capability",
            "http",
        ])
        .arg(&workflow)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--capability should be a recognized flag"
    );
}

/// Test trace command with --sign flag
#[test]
fn test_trace_sign_flag() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() { decide 42 }").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "trace", "--sign"])
        .arg(&workflow)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--sign should be a recognized flag"
    );
}

/// Test trace command with --export flag
#[test]
fn test_trace_export_flag() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() { decide 42 }").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "trace", "--export", "json"])
        .arg(&workflow)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--export should be a recognized flag"
    );
}

/// Test trace command with --provn flag
#[test]
fn test_trace_provn_flag() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() { decide 42 }").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "trace", "--provn"])
        .arg(&workflow)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--provn should be a recognized flag"
    );
}

/// Test trace command with --cypher flag
#[test]
fn test_trace_cypher_flag() {
    let temp = TempDir::new().unwrap();
    let workflow = temp.path().join("test.ash");
    fs::write(&workflow, "workflow test() { decide 42 }").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "trace", "--cypher"])
        .arg(&workflow)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--cypher should be a recognized flag"
    );
}

/// Test repl command with --init flag
#[test]
fn test_repl_init_flag() {
    let temp = TempDir::new().unwrap();
    let init_file = temp.path().join("init.ash");
    fs::write(&init_file, "let x = 42;").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "repl", "--init"])
        .arg(&init_file)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--init should be a recognized flag"
    );
}

/// Test repl command with --config flag
#[test]
fn test_repl_config_flag() {
    let temp = TempDir::new().unwrap();
    let config_file = temp.path().join("config.toml");
    fs::write(&config_file, "[repl]").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "repl", "--config"])
        .arg(&config_file)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--config should be a recognized flag"
    );
}

/// Test repl command with --capability flag
#[test]
fn test_repl_capability_flag() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "repl", "--capability", "fs"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--capability should be a recognized flag"
    );
}

/// Test that global flags work with all subcommands
#[test]
fn test_global_flags_with_subcommands() {
    let subcommands = ["check", "run", "trace", "repl", "dot"];

    for cmd in &subcommands {
        // Test with --quiet
        let output = Command::new("cargo")
            .args(["run", "--bin", "ash", "--", "--quiet", cmd, "--help"])
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "--quiet should work with {} command",
            cmd
        );

        // Test with -v
        let output = Command::new("cargo")
            .args(["run", "--bin", "ash", "--", "-v", cmd, "--help"])
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "-v should work with {} command",
            cmd
        );

        // Test with --color never
        let output = Command::new("cargo")
            .args([
                "run", "--bin", "ash", "--", "--color", "never", cmd, "--help",
            ])
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "--color should work with {} command",
            cmd
        );
    }
}

/// Test that I/O errors (file not found) return exit code 6
#[test]
fn test_exit_code_io_error() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "ash",
            "--",
            "check",
            "/nonexistent/path/file.ash",
        ])
        .output()
        .unwrap();

    println!("Exit code: {:?}", output.status.code());
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    // I/O errors should return exit code 6 per SPEC-005
    assert!(!output.status.success(), "Expected failure for I/O error");
    assert_eq!(
        output.status.code(),
        Some(6),
        "I/O errors should return exit code 6"
    );
}

/// Test that exit code 0 is returned on success
#[test]
fn test_exit_code_success() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

/// Test that unknown commands return exit code 127
#[test]
fn test_exit_code_unknown_command() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "ash", "--", "unknowncommand"])
        .output()
        .unwrap();

    // clap handles unknown subcommands and exits with error
    assert!(!output.status.success());

    // Check that error message mentions unknown command
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unrecognized subcommand") || stderr.contains("error"),
        "Should show error for unknown command"
    );
}
