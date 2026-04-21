//! Tests for the Report formatter (TASK-594).

use spec_processor::finding::SpecFinding;
use spec_processor::report::Report;

#[test]
fn empty_findings_not_blocked() {
    let report = Report::from_findings(vec![]);
    assert!(!report.blocked);
    assert_eq!(report.info_count, 0);
    assert_eq!(report.warning_count, 0);
    assert_eq!(report.error_count, 0);
    assert!(report.findings.is_empty());
}

#[test]
fn single_error_blocks() {
    let findings = vec![SpecFinding::error("Blocker", "something broke").with_file("src/main.rs")];
    let report = Report::from_findings(findings);
    assert!(report.blocked);
    assert_eq!(report.error_count, 1);
    assert_eq!(report.info_count, 0);
    assert_eq!(report.warning_count, 0);
}

#[test]
fn tier_counts_are_correct() {
    let findings = vec![
        SpecFinding::info("InfoCat", "info msg"),
        SpecFinding::info("InfoCat2", "another info"),
        SpecFinding::warning("WarnCat", "warn msg"),
        SpecFinding::error("ErrCat", "err msg"),
        SpecFinding::error("ErrCat2", "another err"),
    ];
    let report = Report::from_findings(findings);
    assert_eq!(report.info_count, 2);
    assert_eq!(report.warning_count, 1);
    assert_eq!(report.error_count, 2);
    assert!(report.blocked);
}

#[test]
fn no_error_not_blocked() {
    let findings = vec![
        SpecFinding::info("InfoCat", "info msg"),
        SpecFinding::warning("WarnCat", "warn msg"),
    ];
    let report = Report::from_findings(findings);
    assert!(!report.blocked);
}

#[test]
fn format_json_produces_valid_json() {
    let findings = vec![
        SpecFinding::info("InfoCat", "info msg").with_file("a.txt"),
        SpecFinding::error("ErrCat", "err msg").with_task_id("TASK-1"),
    ];
    let report = Report::from_findings(findings);
    let json = report
        .format_json()
        .expect("JSON serialization should succeed");

    // Verify it parses back as valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("should parse back");
    assert_eq!(parsed["blocked"], true);
    assert_eq!(parsed["info_count"], 1);
    assert_eq!(parsed["warning_count"], 0);
    assert_eq!(parsed["error_count"], 1);
    assert_eq!(parsed["findings"].as_array().unwrap().len(), 2);
}

#[test]
fn format_human_contains_expected_text() {
    let findings = vec![
        SpecFinding::info("InfoCat", "all good").with_file("a.txt"),
        SpecFinding::warning("WarnCat", "look out")
            .with_file("b.txt")
            .with_task_id("TASK-42"),
        SpecFinding::error("ErrCat", "oh no").with_file("c.txt"),
    ];
    let report = Report::from_findings(findings);
    let text = report.format_human();

    assert!(text.contains("Spec Processor Report"));
    assert!(text.contains("======================"));
    assert!(text.contains("Info: 1 | Warnings: 1 | Errors: 1"));
    assert!(text.contains("Blocked: true"));
    assert!(text.contains("[INFO] InfoCat (a.txt): all good"));
    assert!(text.contains("[WARN] WarnCat (b.txt): look out [TASK-42]"));
    assert!(text.contains("[ERROR] ErrCat (c.txt): oh no"));
}

#[test]
fn format_human_shows_dash_for_missing_file() {
    let findings = vec![SpecFinding::info("NoFile", "no file here")];
    let report = Report::from_findings(findings);
    let text = report.format_human();
    assert!(text.contains("[INFO] NoFile (-): no file here"));
}
