//! Integration tests for the file collector.

use std::path::PathBuf;

use spec_processor::collect;

fn repo_a() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("repo_a")
}

#[test]
fn scan_tree_classifies_spec_file() {
    let tree = collect::scan_tree(&repo_a()).expect("scan should succeed");
    assert!(
        tree.spec_files
            .iter()
            .any(|p| p == "docs/spec/SPEC-001-core.md"),
        "expected SPEC-001-core.md in spec_files, got {:?}",
        tree.spec_files,
    );
}

#[test]
fn scan_tree_classifies_plan_file() {
    let tree = collect::scan_tree(&repo_a()).expect("scan should succeed");
    assert!(
        tree.plan_files.iter().any(|p| p == "docs/plan/PLAN-090.md"),
        "expected PLAN-090.md in plan_files, got {:?}",
        tree.plan_files,
    );
}

#[test]
fn scan_tree_classifies_task_file() {
    let tree = collect::scan_tree(&repo_a()).expect("scan should succeed");
    assert!(
        tree.task_files
            .iter()
            .any(|p| p == "docs/plan/tasks/TASK-590-file-collector.md"),
        "expected TASK-590 in task_files, got {:?}",
        tree.task_files,
    );
}

#[test]
fn scan_tree_classifies_example_file() {
    let tree = collect::scan_tree(&repo_a()).expect("scan should succeed");
    assert!(
        tree.example_files.iter().any(|p| p == "examples/hello.ash"),
        "expected hello.ash in example_files, got {:?}",
        tree.example_files,
    );
}

#[test]
fn scan_tree_classifies_changelog() {
    let tree = collect::scan_tree(&repo_a()).expect("scan should succeed");
    assert!(
        tree.changelog_files
            .iter()
            .any(|p| p == "docs/CHANGELOG.md"),
        "expected CHANGELOG.md in changelog_files, got {:?}",
        tree.changelog_files,
    );
}

#[test]
fn scan_tree_no_cross_contamination() {
    let tree = collect::scan_tree(&repo_a()).expect("scan should succeed");
    // There should be exactly 0 note files in repo_a.
    assert!(
        tree.note_files.is_empty(),
        "expected no note files, got {:?}",
        tree.note_files,
    );
}

#[test]
fn scan_tree_nonexistent_dir_is_error() {
    let result = collect::scan_tree(std::path::Path::new("/no/such/directory/__test__"));
    assert!(
        result.is_err(),
        "scanning a nonexistent directory should fail"
    );
}

#[test]
fn scan_tree_file_counts() {
    let tree = collect::scan_tree(&repo_a()).expect("scan should succeed");
    assert_eq!(tree.spec_files.len(), 1, "spec_files count");
    assert_eq!(tree.plan_files.len(), 1, "plan_files count");
    assert_eq!(tree.task_files.len(), 1, "task_files count");
    assert_eq!(tree.example_files.len(), 1, "example_files count");
    assert_eq!(tree.changelog_files.len(), 1, "changelog_files count");
    assert_eq!(tree.note_files.len(), 0, "note_files count");
}
