//! Test output formatting (human and JSON).
//!
//! TASK-509: Human and JSON output for test results.

use crate::test_runner::types::{Outcome, TestSource, TestSuiteResult};
use colored::Colorize;

/// Format a test suite result as human-readable output.
pub fn format_human(suite: &TestSuiteResult) -> String {
    let mut out = String::new();

    // Header
    out.push_str(&format!("\n{}\n", "─".repeat(60).dimmed()));
    out.push_str(&format!("  Ash Test Results: {}\n", suite.root.display()));
    out.push_str(&format!("{}\n\n", "─".repeat(60).dimmed()));

    // Individual results
    for test in &suite.tests {
        let outcome_str = format_outcome(test.outcome);
        let source_str = match test.source {
            TestSource::Authored => String::new(),
            ref s => format!(" [{}]", s),
        };
        let duration_str = format_duration(test.duration);
        let kind_str = format!(" ({})", test.kind);

        out.push_str(&format!(
            "  {} {}{}{}  {}\n",
            outcome_str,
            test.name,
            source_str,
            kind_str,
            duration_str.dimmed(),
        ));

        if let Some(ref msg) = test.message {
            out.push_str(&format!("    {}\n", msg.red()));
        }
    }

    // Summary
    out.push_str(&format!("\n{}\n", "─".repeat(60).dimmed()));
    let passed = suite.passed();
    let failed = suite.failed();
    let skipped = suite.skipped();
    let total = suite.total();

    let status = if suite.is_success() {
        "PASSED".green().bold().to_string()
    } else {
        "FAILED".red().bold().to_string()
    };

    out.push_str(&format!(
        "  {} {} tests: {} passed, {} failed, {} skipped ({:.1}ms)\n",
        status,
        total,
        passed.to_string().green(),
        failed.to_string().red(),
        skipped.to_string().yellow(),
        suite.duration.as_secs_f64() * 1000.0,
    ));
    out.push_str(&format!("{}\n", "─".repeat(60).dimmed()));

    out
}

fn format_outcome(outcome: Outcome) -> colored::ColoredString {
    match outcome {
        Outcome::Pass => "PASS".green().bold(),
        Outcome::Fail => "FAIL".red().bold(),
        Outcome::Panic => "PANIC".red().bold().on_yellow(),
        Outcome::Error => "ERROR".red().bold(),
        Outcome::Skip => "SKIP".yellow(),
        Outcome::Xfail => "XFAIL".yellow(),
    }
}

fn format_duration(dur: std::time::Duration) -> String {
    let ms = dur.as_secs_f64() * 1000.0;
    if ms < 1.0 {
        format!("{:.0}µs", dur.as_micros())
    } else if ms < 1000.0 {
        format!("{:.1}ms", ms)
    } else {
        format!("{:.2}s", dur.as_secs_f64())
    }
}

/// Format a test suite result as JSON.
pub fn format_json(suite: &TestSuiteResult) -> Result<String, serde_json::Error> {
    #[derive(serde::Serialize)]
    struct JsonSuiteOutput {
        schema_version: &'static str,
        root: String,
        success: bool,
        total: usize,
        passed: usize,
        failed: usize,
        skipped: usize,
        duration_ms: f64,
        tests: Vec<JsonTest>,
    }

    #[derive(serde::Serialize)]
    struct JsonTest {
        name: String,
        path: String,
        outcome: String,
        source: String,
        kind: String,
        duration_ms: f64,
        message: Option<String>,
        tags: Vec<String>,
        seed: Option<u64>,
        failing_case: Option<usize>,
        world_index: Option<usize>,
    }

    let tests: Vec<JsonTest> = suite
        .tests
        .iter()
        .map(|t| JsonTest {
            name: t.name.clone(),
            path: t.path.to_string_lossy().to_string(),
            outcome: format!("{}", t.outcome).to_lowercase(),
            source: format!("{}", t.source),
            kind: format!("{}", t.kind),
            duration_ms: t.duration.as_secs_f64() * 1000.0,
            message: t.message.clone(),
            tags: t.tags.clone(),
            seed: t.seed,
            failing_case: t.failing_case,
            world_index: t.world_index,
        })
        .collect();

    let output = JsonSuiteOutput {
        schema_version: "ash-test-v1.0",
        root: suite.root.to_string_lossy().to_string(),
        success: suite.is_success(),
        total: suite.total(),
        passed: suite.passed(),
        failed: suite.failed(),
        skipped: suite.skipped(),
        duration_ms: suite.duration.as_secs_f64() * 1000.0,
        tests,
    };

    serde_json::to_string_pretty(&output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_runner::types::{TestKind, TestResult};
    use std::path::PathBuf;
    use std::time::Duration;

    fn make_suite() -> TestSuiteResult {
        let mut suite = TestSuiteResult::new(PathBuf::from("."));
        suite.add(
            TestResult::new("test_a", PathBuf::from("tests/a.ash"))
                .with_outcome(Outcome::Pass)
                .with_kind(TestKind::Unit)
                .with_duration(Duration::from_micros(100)),
        );
        suite.add(
            TestResult::new("test_b", PathBuf::from("tests/b.ash"))
                .with_outcome(Outcome::Fail)
                .with_kind(TestKind::Integration)
                .with_duration(Duration::from_millis(50))
                .with_message("assertion failed: expected 1, got 2"),
        );
        suite.duration = Duration::from_millis(150);
        suite
    }

    #[test]
    fn human_output_contains_pass_and_fail() {
        let suite = make_suite();
        let output = format_human(&suite);
        assert!(output.contains("PASS"));
        assert!(output.contains("FAIL"));
        assert!(output.contains("assertion failed"));
        assert!(output.contains("2 tests"));
    }

    #[test]
    fn json_output_is_valid() {
        let suite = make_suite();
        let output = format_json(&suite).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["schema_version"], "ash-test-v1.0");
        assert_eq!(parsed["total"], 2);
        assert_eq!(parsed["passed"], 1);
        assert_eq!(parsed["failed"], 1);
    }
}
