//! PLAN-INDEX coherence checker.
//!
//! Parses `PLAN-INDEX.md`, extracts every `TASK-NNN` reference, and then
//! cross-references them against the task files in `docs/plan/tasks/` to
//! detect:
//!
//! * **Missing task files** — referenced in the index but no matching file.
//! * **Orphaned task files** — exist on disk but never referenced in the index.

use crate::finding::{SpecFinding, TASK_RE};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Parse `PLAN-INDEX.md` and return coherence findings.
///
/// * `plan_index_path` — path to the `PLAN-INDEX.md` file.
/// * `tasks_dir` — path to the directory that holds `TASK-NNN-*.md` files.
///
/// # Errors
///
/// Returns an error if `plan_index_path` cannot be read from disk or if
/// `tasks_dir` cannot be enumerated.
#[must_use = "check performs I/O and analysis; the findings or error should be handled"]
pub fn check(plan_index_path: &Path, tasks_dir: &Path) -> Result<Vec<SpecFinding>, std::io::Error> {
    let content = fs::read_to_string(plan_index_path)?;
    let mut findings = Vec::new();

    let referenced_tasks = extract_task_references(&content);

    // 1. Detect referenced tasks with no matching file.
    for task_id in &referenced_tasks {
        let prefix = format!("{task_id}-");
        let exists = dir_entries(tasks_dir)?.any(|name| {
            Path::new(&name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                && name.starts_with(&prefix)
        });

        if !exists {
            findings.push(
                SpecFinding::error(
                    "MissingTaskFile",
                    format!(
                        "{task_id} is referenced in PLAN-INDEX but has no task file in {}",
                        tasks_dir.display()
                    ),
                )
                .with_file(plan_index_path.to_string_lossy().into_owned())
                .with_task_id(task_id),
            );
        }
    }

    // 2. Detect orphaned task files (on disk but not referenced).
    for name in dir_entries(tasks_dir)? {
        let Some(task_id) = task_id_from_filename(&name) else {
            continue;
        };
        if !referenced_tasks.contains(&task_id) {
            findings.push(
                SpecFinding::warning(
                    "OrphanedTaskFile",
                    format!("{name} exists in tasks/ but is not referenced in PLAN-INDEX"),
                )
                .with_file(tasks_dir.join(&name).to_string_lossy().into_owned())
                .with_task_id(task_id),
            );
        }
    }

    Ok(findings)
}

/// Extract all unique `TASK-NNN` identifiers from the plan-index content.
fn extract_task_references(content: &str) -> HashSet<String> {
    TASK_RE
        .find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Return sorted file names in `dir` (ignores subdirectories).
///
/// # Errors
///
/// Returns an error if the directory cannot be read.
fn dir_entries(dir: &Path) -> Result<impl Iterator<Item = String>, std::io::Error> {
    let mut names: Vec<String> = fs::read_dir(dir)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    Ok(names.into_iter())
}

/// Given a filename like `TASK-591-plan-index-coherence.md`, return `TASK-591`.
fn task_id_from_filename(name: &str) -> Option<String> {
    let stem = Path::new(name).file_stem()?.to_str()?;
    let number_part = stem.split('-').nth(1)?;
    if number_part.bytes().all(|b| b.is_ascii_digit()) {
        Some(format!("TASK-{number_part}"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_finds_multiple_references() {
        let md = "## Phase 9\n- TASK-100-foo\n- TASK-200-bar\nSee TASK-100 again.";
        let refs = extract_task_references(md);
        assert!(refs.contains("TASK-100"));
        assert!(refs.contains("TASK-200"));
        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn extract_ignores_too_short_numbers() {
        // TASK-12 is only 2 digits — the pattern requires 3+.
        let md = "TASK-12 and TASK-123";
        let refs = extract_task_references(md);
        assert!(!refs.contains("TASK-12"));
        assert!(refs.contains("TASK-123"));
    }

    #[test]
    fn task_id_from_filename_works() {
        assert_eq!(
            task_id_from_filename("TASK-591-plan-index-coherence.md"),
            Some("TASK-591".to_string())
        );
        assert_eq!(task_id_from_filename("README.md"), None);
    }
}
