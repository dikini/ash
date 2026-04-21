//! Integration tests for the spec-processor pipeline orchestrator.

use std::fs;
use std::path::PathBuf;

use spec_processor::finding::Tier;
use spec_processor::run_pipeline;

fn repo_a() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("repo_a")
}

/// Running the pipeline against the well-formed fixture repository should
/// succeed without blocking (no error-tier findings). Informational and
/// warning-tier findings are expected (e.g. capability gaps).
#[test]
fn pipeline_against_fixture_repo_succeeds() {
    let root = repo_a();
    let report = run_pipeline(&root).expect("pipeline should succeed on fixture repo_a");

    assert!(
        !report.blocked,
        "fixture repo_a should not be blocked, but got errors:\n{}",
        report.format_human()
    );
    assert!(
        !report.findings.is_empty(),
        "fixture repo_a should produce at least some findings (e.g. capability info)"
    );
}

/// Run the full pipeline against the real Ash repository.
///
/// This test is `#[ignore]` because:
/// - It requires the complete repo to be present at `../../..` relative to
///   `CARGO_MANIFEST_DIR`.
/// - Many example `.ash` files intentionally exercise features that may not yet
///   parse cleanly, producing `ExampleFailure` errors.
/// - It is intended for manual CI verification, not automatic gating.
///
/// Run with: `cargo test -p spec_processor -- --ignored pipeline_against_real_repo`
#[test]
#[ignore = "requires full repo; run manually with --ignored"]
fn pipeline_against_real_repo_succeeds() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..");

    let report = run_pipeline(&repo_root).expect("pipeline should succeed on real repo");

    eprintln!("{}", report.format_human());

    // NOTE: Do NOT assert !report.blocked here. The real repo is expected to
    // have findings (e.g. example .ash files using unimplemented features).
    // Reviewers should inspect the printed report for actionable items.
    let _ = report;
}

/// A temp fixture with a broken `.ash` file and a PLAN-INDEX referencing a
/// missing task file should produce at least one error-tier finding, causing
/// the report to be blocked.
#[test]
fn pipeline_blocked_on_errors() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let root = tmp.path();

    // Create a broken .ash file that will fail to parse.
    let examples_dir = root.join("examples");
    fs::create_dir_all(&examples_dir).expect("create examples dir");
    fs::write(
        examples_dir.join("broken.ash"),
        "!!!this is not valid ash!!!\n",
    )
    .expect("write broken .ash");

    // Create PLAN-INDEX referencing a task that has no file.
    let plan_dir = root.join("docs/plan");
    let tasks_dir = plan_dir.join("tasks");
    fs::create_dir_all(&tasks_dir).expect("create tasks dir");
    fs::write(
        plan_dir.join("PLAN-INDEX.md"),
        "# Plan Index\n- TASK-999-nonexistent\n",
    )
    .expect("write PLAN-INDEX");

    // Create a minimal CHANGELOG so the changelog check has something to scan.
    let docs_dir = root.join("docs");
    fs::write(
        docs_dir.join("CHANGELOG.md"),
        "# Changelog\n\nNothing yet.\n",
    )
    .expect("write CHANGELOG");

    let report = run_pipeline(root).expect("pipeline should succeed on temp fixture");

    assert!(
        report.blocked,
        "temp fixture should be blocked due to broken .ash and missing task file, but got:\n{}",
        report.format_human()
    );

    // Verify we got at least one error-tier finding.
    assert!(
        report.findings.iter().any(|f| f.tier == Tier::Error),
        "expected at least one error-tier finding"
    );
}
