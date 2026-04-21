//! Integration tests for the changelog completeness checker.

use spec_processor::changelog::check_changelog;
use spec_processor::finding::Tier;

#[test]
fn all_completed_tasks_in_changelog_yields_no_findings() {
    let plan_index = "\
- ✅ TASK-100 implement foo
- ✅ TASK-200 implement bar
";
    let changelog = "\
### 0.2.0
- TASK-100: implement foo
- TASK-200: implement bar
";

    let findings = check_changelog(plan_index, changelog, None);
    assert!(findings.is_empty());
}

#[test]
fn missing_task_from_changelog_produces_warning() {
    let plan_index = "\
- ✅ TASK-100 implement foo
- ✅ TASK-200 implement bar
";
    let changelog = "\
### 0.2.0
- TASK-100: implement foo
";

    let findings = check_changelog(plan_index, changelog, None);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].category, "ChangelogMissing");
    assert_eq!(findings[0].tier, Tier::Warning);
    assert_eq!(findings[0].task_id.as_deref(), Some("TASK-200"));
    assert!(findings[0].description.contains("TASK-200"));
}

#[test]
fn no_completed_tasks_yields_no_findings() {
    let plan_index = "\
- TASK-100 pending
- TASK-200 in progress
";
    let changelog = "No entries yet.\n";

    let findings = check_changelog(plan_index, changelog, None);
    assert!(findings.is_empty());
}

#[test]
fn empty_inputs_yields_no_findings() {
    let findings = check_changelog("", "", None);
    assert!(findings.is_empty());
}

#[test]
fn multiple_missing_tasks_all_reported() {
    let plan_index = "\
- ✅ TASK-100 done
- ✅ TASK-101 done
- ✅ TASK-102 done
";
    let changelog = "TASK-100 is here\n";

    let findings = check_changelog(plan_index, changelog, None);
    assert_eq!(findings.len(), 2);
    let missing_ids: Vec<&str> = findings
        .iter()
        .map(|f| f.task_id.as_deref().unwrap())
        .collect();
    assert!(missing_ids.contains(&"TASK-101"));
    assert!(missing_ids.contains(&"TASK-102"));
}

#[test]
fn findings_structurally_correct() {
    let plan_index = "- ✅ TASK-999 orphan\n";
    let changelog = "Nothing\n";

    let findings = check_changelog(plan_index, changelog, None);
    assert_eq!(findings.len(), 1);
    let f = &findings[0];
    assert_eq!(f.tier, Tier::Warning);
    assert_eq!(f.category, "ChangelogMissing");
    assert!(f.description.contains("TASK-999"));
    assert_eq!(f.task_id.as_deref(), Some("TASK-999"));
    assert!(f.file.is_none());
}

#[test]
fn changelog_path_attached_to_findings() {
    let plan_index = "- ✅ TASK-999 orphan\n";
    let changelog = "Nothing\n";

    let findings = check_changelog(plan_index, changelog, Some("CHANGELOG.md"));
    assert_eq!(findings.len(), 1);
    let f = &findings[0];
    assert_eq!(f.file.as_deref(), Some("CHANGELOG.md"));
    assert_eq!(f.task_id.as_deref(), Some("TASK-999"));
    assert_eq!(f.category, "ChangelogMissing");
}
