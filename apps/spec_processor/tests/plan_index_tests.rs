//! Integration tests for `plan_index::check`.

use std::fs;

use spec_processor::finding::SpecFinding;
use spec_processor::finding::Tier;
use spec_processor::plan_index;

struct Fixture {
    _root: tempfile::TempDir,
    plan_index: std::path::PathBuf,
    tasks_dir: std::path::PathBuf,
}

impl Fixture {
    fn new(plan_content: &str, task_files: &[&str]) -> Self {
        let root = tempfile::TempDir::new().expect("tempdir");
        let plan_index = root.path().join("PLAN-INDEX.md");
        fs::write(&plan_index, plan_content).expect("write plan-index");

        let tasks_dir = root.path().join("tasks");
        fs::create_dir_all(&tasks_dir).expect("mkdir tasks");

        for name in task_files {
            fs::write(tasks_dir.join(name), format!("## {name}\n")).expect("write task");
        }

        Self {
            _root: root,
            plan_index,
            tasks_dir,
        }
    }
}

#[test]
fn all_good_produces_no_findings() {
    let fix = Fixture::new(
        "## Plan\n- TASK-100-do-thing\n- TASK-200-other\n",
        &["TASK-100-do-thing.md", "TASK-200-other.md"],
    );

    let findings = plan_index::check(&fix.plan_index, &fix.tasks_dir).unwrap();
    assert!(
        findings.is_empty(),
        "expected no findings, got {findings:?}"
    );
}

#[test]
fn missing_task_file_is_detected() {
    let fix = Fixture::new(
        "## Plan\n- TASK-100-do-thing\n- TASK-300-missing\n",
        &["TASK-100-do-thing.md"],
    );

    let findings = plan_index::check(&fix.plan_index, &fix.tasks_dir).unwrap();
    let errors: Vec<&SpecFinding> = findings
        .iter()
        .filter(|f| f.category == "MissingTaskFile")
        .collect();

    assert_eq!(
        errors.len(),
        1,
        "expected 1 missing-file error, got {errors:?}"
    );
    assert_eq!(errors[0].task_id.as_deref(), Some("TASK-300"));
    assert_eq!(errors[0].tier, Tier::Error);
}

#[test]
fn orphaned_task_file_is_detected() {
    let fix = Fixture::new(
        "## Plan\n- TASK-100-do-thing\n",
        &["TASK-100-do-thing.md", "TASK-999-orphaned.md"],
    );

    let findings = plan_index::check(&fix.plan_index, &fix.tasks_dir).unwrap();
    let warnings: Vec<&SpecFinding> = findings
        .iter()
        .filter(|f| f.category == "OrphanedTaskFile")
        .collect();

    assert_eq!(
        warnings.len(),
        1,
        "expected 1 orphaned-file warning, got {warnings:?}"
    );
    assert_eq!(warnings[0].task_id.as_deref(), Some("TASK-999"));
    assert_eq!(warnings[0].tier, Tier::Warning);
}

#[test]
fn duplicate_references_deduplicated() {
    let fix = Fixture::new(
        "TASK-400-alpha TASK-400-alpha again\n",
        &["TASK-400-alpha.md"],
    );

    let findings = plan_index::check(&fix.plan_index, &fix.tasks_dir).unwrap();
    assert!(
        findings.is_empty(),
        "duplicates should not produce spurious findings, got {findings:?}"
    );
}
