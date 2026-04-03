//! Integration tests for `ash run` observable output.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn run_success_stdout_is_only_final_value() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("main.ash");
    fs::write(&workflow_path, "workflow main { ret 0; }\n").expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .success()
        .stdout(predicate::eq("0\n"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn run_output_file_writes_value_without_status_banner() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("main.ash");
    let output_path = temp.path().join("result.txt");
    fs::write(&workflow_path, "workflow main { ret 7; }\n").expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run")
        .arg(&workflow_path)
        .arg("--output")
        .arg(&output_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert_eq!(fs::read_to_string(output_path).expect("read output"), "7");
}

#[test]
fn run_parse_error_is_observably_distinct() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("broken.ash");
    fs::write(&workflow_path, "workflow main { ret ; }\n").expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("parse error"));
}

#[test]
fn run_trace_parse_error_is_observably_distinct() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("broken-trace.ash");
    fs::write(&workflow_path, "workflow main { ret ; }\n").expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg("--trace").arg(&workflow_path);

    cmd.assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("parse error"));
}

#[test]
fn run_string_literal_runtimeerror_does_not_trigger_entry_bootstrap() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("runtimeerror-string.ash");
    fs::write(
        &workflow_path,
        r#"
        workflow main {
            ret "plain RuntimeError text";
        }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .success()
        .stdout(predicate::eq("\"plain RuntimeError text\"\n"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn run_entry_workflow_success_returns_exit_zero_without_value_output() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("entry-success.ash");
    fs::write(
        &workflow_path,
        r#"
        use result::Result
        use runtime::RuntimeError

        workflow main() -> Result<(), RuntimeError> { done; }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn run_entry_workflow_runtime_error_uses_declared_exit_code() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("entry-error.ash");
    fs::write(
        &workflow_path,
        r#"
        use result::Result
        use runtime::RuntimeError

        workflow main() -> Result<(), RuntimeError> {
            ret Err { error: RuntimeError { exit_code: 42, message: "boom" } };
        }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .code(42)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn run_dry_run_accepts_entry_workflow_with_runtime_imports() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("entry-dry-run.ash");
    fs::write(
        &workflow_path,
        r#"
        use result::Result
        use runtime::RuntimeError

        workflow main() -> Result<(), RuntimeError> { done; }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg("--dry-run").arg(&workflow_path);

    cmd.assert()
        .success()
        .stdout(predicate::eq("Dry run successful\n"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn run_import_free_entry_with_capabilities_clause_uses_entry_path() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("entry-capabilities-clause.ash");
    fs::write(
        &workflow_path,
        r#"
        workflow main() -> Result<(), RuntimeError>
        capabilities: []
        { done; }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn run_import_free_entry_with_requires_expression_uses_entry_exit_semantics() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("entry-requires-clause.ash");
    fs::write(
        &workflow_path,
        r#"
        workflow main(capabilities: cap Args) -> Result<(), RuntimeError>
        requires: capabilities == capabilities
        {
            ret Err { error: RuntimeError { exit_code: 42, message: "boom" } };
        }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .code(42)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn run_non_entry_workflow_with_leading_runtime_prelude_executes_normally() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("non-entry-runtime-prelude.ash");
    fs::write(
        &workflow_path,
        r#"
        use result::Result
        use runtime::RuntimeError

        workflow main {
            ret 11;
        }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .success()
        .stdout(predicate::eq("11\n"))
        .stderr(predicate::str::is_empty());
}
