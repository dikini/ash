//! Changelog completeness checker.
//!
//! Compares completed tasks from a `PLAN-INDEX` document against a `CHANGELOG.md`
//! and reports any task that is marked as completed but missing from the changelog.

use crate::finding::{SpecFinding, TASK_RE};
use std::collections::HashSet;

/// Check that every completed task in `PLAN-INDEX` appears in `CHANGELOG.md`.
///
/// Returns a [`SpecFinding`] (tier 1 — warning) for each task that is marked
/// complete (indicated by `✅` or the word `Complete` on the same line) but
/// whose `TASK-NNN` identifier does not appear anywhere in the changelog
/// content.
///
/// When `changelog_path` is `Some`, each finding will carry the file path via
/// the `file` field so callers can attribute findings to a specific file.
#[must_use = "check_changelog performs analysis and the findings should be inspected"]
pub fn check_changelog(
    plan_index_content: &str,
    changelog_content: &str,
    changelog_path: Option<&str>,
) -> Vec<SpecFinding> {
    let completed = extract_completed_tasks(plan_index_content);

    completed
        .into_iter()
        .filter(|task_id| !changelog_content.contains(task_id.as_str()))
        .map(|task_id| {
            let mut finding = SpecFinding::warning(
                "ChangelogMissing",
                format!("{task_id} is marked complete but missing from CHANGELOG.md"),
            )
            .with_task_id(task_id);

            if let Some(path) = changelog_path {
                finding = finding.with_file(path);
            }

            finding
        })
        .collect()
}

/// Extract unique `TASK-NNN` identifiers from lines that indicate completion.
///
/// A line is considered "completed" if it contains `✅` or the word `Complete`.
fn extract_completed_tasks(content: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for line in content.lines() {
        if line.contains('\u{2705}') || line.contains("Complete") {
            for m in TASK_RE.find_iter(line) {
                let id = m.as_str().to_string();
                if seen.insert(id.clone()) {
                    result.push(id);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Tier;

    #[test]
    fn all_completed_in_changelog_no_findings() {
        let plan_index = "- ✅ TASK-100 foo\n- ✅ TASK-200 bar\n";
        let changelog = "Changes: TASK-100, TASK-200\n";
        let findings = check_changelog(plan_index, changelog, None);
        assert!(findings.is_empty());
    }

    #[test]
    fn missing_task_produces_warning() {
        let plan_index = "- ✅ TASK-100 foo\n- ✅ TASK-200 bar\n";
        let changelog = "Changes: TASK-100\n";
        let findings = check_changelog(plan_index, changelog, None);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, "ChangelogMissing");
        assert_eq!(findings[0].task_id.as_deref(), Some("TASK-200"));
        assert_eq!(findings[0].tier, Tier::Warning);
    }

    #[test]
    fn no_completed_tasks_no_findings() {
        let plan_index = "- TASK-100 foo\n- TASK-200 bar\n";
        let changelog = "No changes yet\n";
        let findings = check_changelog(plan_index, changelog, None);
        assert!(findings.is_empty());
    }

    #[test]
    fn complete_keyword_also_detected() {
        let plan_index = "- Complete: TASK-300 baz\n";
        let changelog = "Nothing here\n";
        let findings = check_changelog(plan_index, changelog, None);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].task_id.as_deref(), Some("TASK-300"));
    }

    #[test]
    fn duplicate_tasks_reported_once() {
        let plan_index = "- ✅ TASK-100 a\n- ✅ TASK-100 b\n";
        let changelog = "No changes\n";
        let findings = check_changelog(plan_index, changelog, None);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn extract_completed_ignores_incomplete_lines() {
        let plan_index = "- TASK-999 pending\n";
        let tasks = extract_completed_tasks(plan_index);
        assert!(tasks.is_empty());
    }

    #[test]
    fn checkmark_unicode_detected() {
        // Verify the ✅ character (U+2705) is matched.
        let plan_index = "- \u{2705} TASK-456 done\n";
        let changelog = "Empty\n";
        let findings = check_changelog(plan_index, changelog, None);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].task_id.as_deref(), Some("TASK-456"));
    }
}
