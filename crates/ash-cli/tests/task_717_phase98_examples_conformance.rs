//! TASK-717: cross-layer CLI conformance over Phase 98 examples.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_ash_command(args: &[&str], file_path: &Path) -> (Option<i32>, String, String) {
    let output = Command::new("cargo")
        .current_dir(repo_root())
        .args(["run", "--bin", "ash", "--"])
        .args(args)
        .arg(file_path)
        .output()
        .expect("ash CLI should launch");

    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn fail_with_error_example_check_run_trace_agree() {
    let example = repo_root().join("examples/05-phase98/01-fail-with-error.ash");
    assert!(example.exists(), "missing Phase 98 fail/with_error example");

    let (check_code, _, check_stderr) = run_ash_command(&["check"], &example);
    assert_eq!(check_code, Some(0), "ash check failed: {check_stderr}");

    let (run_code, run_stdout, run_stderr) = run_ash_command(&["run"], &example);
    assert_eq!(run_code, Some(0), "ash run failed: {run_stderr}");
    assert!(
        run_stdout.contains('7') || run_stderr.contains('7'),
        "expected recovered value 7 in output, stdout={run_stdout:?}, stderr={run_stderr:?}"
    );

    let (trace_code, trace_stdout, trace_stderr) = run_ash_command(&["trace"], &example);
    assert_eq!(trace_code, Some(0), "ash trace failed: {trace_stderr}");
    assert!(
        trace_stdout.contains('7') || trace_stderr.contains('7'),
        "expected traced value 7 in output, stdout={trace_stdout:?}, stderr={trace_stderr:?}"
    );
}

#[test]
fn proc_examples_are_cli_checkable_and_honestly_render_proc_closures() {
    for relative in [
        "examples/05-phase98/02-proc-par-await-join.ash",
        "examples/05-phase98/03-proc-scatter-gather.ash",
    ] {
        let example = repo_root().join(relative);
        assert!(
            example.exists(),
            "missing Phase 98 proc example: {relative}"
        );

        let (check_code, _, check_stderr) = run_ash_command(&["check"], &example);
        assert_eq!(
            check_code,
            Some(0),
            "ash check failed for {relative}: {check_stderr}"
        );

        let (run_code, run_stdout, run_stderr) = run_ash_command(&["run"], &example);
        assert_eq!(
            run_code,
            Some(0),
            "ash run failed for {relative}: {run_stderr}"
        );
        assert!(
            run_stdout.contains("<closure(1)>") || run_stderr.contains("<closure(1)>"),
            "expected proc example {relative} to render an honest Proc closure, stdout={run_stdout:?}, stderr={run_stderr:?}"
        );

        let (trace_code, trace_stdout, trace_stderr) = run_ash_command(&["trace"], &example);
        assert_eq!(
            trace_code,
            Some(0),
            "ash trace failed for {relative}: {trace_stderr}"
        );
        assert!(
            trace_stdout.contains("<closure(1)>") || trace_stderr.contains("<closure(1)>"),
            "expected proc trace for {relative} to retain the Proc closure surface, stdout={trace_stdout:?}, stderr={trace_stderr:?}"
        );
    }
}

#[test]
fn boundary_reporting_source_example_is_cli_runnable_but_reporting_stays_engine_api_only() {
    let example = repo_root().join("examples/05-phase98/04-workflow-boundary-reporting.ash");
    assert!(
        example.exists(),
        "missing Phase 98 workflow-boundary source example"
    );

    let (check_code, _, check_stderr) = run_ash_command(&["check"], &example);
    assert_eq!(
        check_code,
        Some(0),
        "ash check failed for boundary example: {check_stderr}"
    );

    let (run_code, run_stdout, run_stderr) = run_ash_command(&["run"], &example);
    assert_eq!(
        run_code,
        Some(0),
        "ash run failed for boundary example: {run_stderr}"
    );
    assert!(
        run_stdout.contains('9') || run_stderr.contains('9'),
        "expected boundary source example to execute to 9, stdout={run_stdout:?}, stderr={run_stderr:?}"
    );

    let readme = std::fs::read_to_string(repo_root().join("examples/README.md"))
        .expect("read examples README");
    assert!(readme.contains("05-phase98"));
    assert!(readme.contains("01-fail-with-error.ash"));
    assert!(readme.contains("02-proc-par-await-join.ash"));
    assert!(readme.contains("03-proc-scatter-gather.ash"));
    assert!(readme.contains("04-workflow-boundary-reporting.ash"));
    assert!(
        readme.contains("workflow boundary reporting currently requires the engine admission API"),
        "README should document the honest workflow-boundary reporting limitation"
    );
}
