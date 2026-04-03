//! Integration tests for `ash run` observable output.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

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
fn run_trace_missing_main_reports_entry_error_and_exit_one() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("missing-main-trace.ash");
    fs::write(
        &workflow_path,
        r#"
        use result::Result
        use runtime::RuntimeError

        workflow other() -> Result<(), RuntimeError> { done; }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg("--trace").arg(&workflow_path);

    cmd.assert()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "entry file has no 'main' workflow",
        ));
}

#[test]
fn run_trace_entry_workflow_accepts_trailing_cli_args_after_double_dash() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("entry-args-trace.ash");
    fs::write(
        &workflow_path,
        r#"
        use result::Result
        use runtime::RuntimeError
        use runtime::Args

        workflow main(args: cap Args) -> Result<(), RuntimeError> {
            observe Args 0 as _;
            done;
        }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run")
        .arg("--trace")
        .arg(&workflow_path)
        .arg("--")
        .arg("hello");

    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn run_ordinary_non_entry_workflow_with_return_type_executes_normally() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("helper.ash");
    fs::write(&workflow_path, "workflow helper() -> Int { ret 11; }\n").expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .success()
        .stdout(predicate::eq("11\n"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn run_trace_ordinary_non_entry_workflow_with_return_type_executes_normally() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("helper-trace.ash");
    fs::write(&workflow_path, "workflow helper() -> Int { ret 11; }\n").expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg("--trace").arg(&workflow_path);

    cmd.assert()
        .success()
        .stdout(predicate::eq("11\n"))
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
fn run_entry_workflow_with_output_creates_empty_output_file() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("entry-output.ash");
    let output_path = temp.path().join("result.txt");
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
    cmd.arg("run")
        .arg(&workflow_path)
        .arg("--output")
        .arg(&output_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert_eq!(fs::read_to_string(output_path).expect("read output"), "");
}

#[test]
fn run_entry_runtime_error_with_output_does_not_create_output_file() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("entry-error-output.ash");
    let output_path = temp.path().join("result.txt");
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
    cmd.arg("run")
        .arg(&workflow_path)
        .arg("--output")
        .arg(&output_path);

    cmd.assert()
        .code(42)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    assert!(
        !output_path.exists(),
        "output file should not be created for non-zero entry exits"
    );
}

#[test]
fn run_entry_workflow_accepts_trailing_cli_args_after_double_dash() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("entry-args.ash");
    fs::write(
        &workflow_path,
        r#"
        use result::Result
        use runtime::RuntimeError
        use runtime::Args

        workflow main(args: cap Args) -> Result<(), RuntimeError> {
            observe Args 0 as _;
            done;
        }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path).arg("--").arg("hello");

    cmd.assert()
        .success()
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
fn run_dry_run_missing_main_reports_entry_error() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("missing-main-dry-run.ash");
    fs::write(
        &workflow_path,
        r#"
        workflow other() -> Result<(), RuntimeError> { done; }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg("--dry-run").arg(&workflow_path);

    cmd.assert()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "entry file has no 'main' workflow",
        ));
}

#[test]
fn run_missing_main_reports_entry_error_and_exit_one() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("missing-main.ash");
    fs::write(
        &workflow_path,
        r#"
        use result::Result
        use runtime::RuntimeError

        workflow other() -> Result<(), RuntimeError> { done; }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "entry file has no 'main' workflow",
        ));
}

#[test]
fn run_wrong_return_type_reports_error_and_exit_one() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("wrong-type.ash");
    fs::write(
        &workflow_path,
        r#"
        use result::Result
        use runtime::RuntimeError

        workflow main() -> Int { ret 42; }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("'main' has wrong return type"))
        .stderr(predicate::str::contains(
            "expected: Result<(), RuntimeError>",
        ))
        .stderr(predicate::str::contains("found: Int"));
}

#[test]
fn run_non_capability_parameter_reports_error_and_exit_one() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("wrong-param.ash");
    fs::write(
        &workflow_path,
        r#"
        use result::Result
        use runtime::RuntimeError
        use runtime::Args

        workflow main(args: Args) -> Result<(), RuntimeError> { done; }
        "#,
    )
    .expect("write workflow");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "parameter 'args' must be capability type",
        ));
}

#[test]
fn run_file_not_found_reports_entry_specific_message() {
    let temp = tempdir().expect("tempdir");
    let workflow_path = temp.path().join("nope.ash");

    let mut cmd = Command::cargo_bin("ash").expect("ash binary exists");
    cmd.arg("run").arg(&workflow_path);

    cmd.assert()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(format!(
            "file not found: {}",
            workflow_path.display()
        )));
}
