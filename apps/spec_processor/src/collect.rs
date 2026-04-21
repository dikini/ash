//! Recursive directory walker that classifies files by naming convention.

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

/// Classified collection of files found beneath a repository root.
#[derive(Debug, Clone, Default)]
pub struct FileTree {
    /// Files matching `**/SPEC-*.md`.
    pub spec_files: Vec<String>,
    /// Files matching `**/PLAN-*.md`.
    pub plan_files: Vec<String>,
    /// Files matching `**/TASK-*.md`.
    pub task_files: Vec<String>,
    /// Files matching `**/*.ash`.
    pub example_files: Vec<String>,
    /// Files matching `**/CHANGELOG.md`.
    pub changelog_files: Vec<String>,
    /// Files matching `**/NOTE-*.md`.
    pub note_files: Vec<String>,
}

/// Recursively walk `root` and classify every file by naming convention.
///
/// Returns relative paths (forward-slash separated) from `root`.
///
/// # Errors
///
/// Returns `std::io::Error` if `root` does not exist or any directory entry
/// cannot be read.
#[must_use = "scanning the file tree is expensive and the result should be used"]
pub fn scan_tree(root: &Path) -> Result<FileTree, std::io::Error> {
    let mut tree = FileTree::default();
    walk(root, root, &mut tree)?;
    // Sort each bucket for deterministic output.
    tree.spec_files.sort();
    tree.plan_files.sort();
    tree.task_files.sort();
    tree.example_files.sort();
    tree.changelog_files.sort();
    tree.note_files.sort();
    Ok(tree)
}

/// Recursive visitor.
fn walk(root: &Path, dir: &Path, tree: &mut FileTree) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            walk(root, &path, tree)?;
            continue;
        }

        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        classify(root, &path, &file_name_str, &file_name, tree);
    }
    Ok(())
}

/// Push a relative path (forward-slash normalised) into the correct bucket.
fn classify(
    root: &Path,
    full_path: &Path,
    file_name_str: &str,
    file_name: &OsStr,
    tree: &mut FileTree,
) {
    let Ok(relative) = full_path.strip_prefix(root) else {
        return;
    };

    // Normalise to forward slashes.
    let relative_str = relative.to_string_lossy().replace('\\', "/");

    if file_name == OsStr::new("CHANGELOG.md") {
        tree.changelog_files.push(relative_str);
    } else if matches_prefix(file_name_str, "SPEC-", ".md") {
        tree.spec_files.push(relative_str);
    } else if matches_prefix(file_name_str, "PLAN-", ".md") {
        tree.plan_files.push(relative_str);
    } else if matches_prefix(file_name_str, "TASK-", ".md") {
        tree.task_files.push(relative_str);
    } else if matches_prefix(file_name_str, "NOTE-", ".md") {
        tree.note_files.push(relative_str);
    } else if Path::new(file_name_str)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ash"))
    {
        tree.example_files.push(relative_str);
    }
    // Unrecognised files are silently ignored.
}

/// Returns `true` when `name` starts with `prefix` and ends with `suffix`.
fn matches_prefix(name: &str, prefix: &str, suffix: &str) -> bool {
    name.starts_with(prefix) && name.ends_with(suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_prefix_basic() {
        assert!(matches_prefix("SPEC-001-core.md", "SPEC-", ".md"));
        assert!(matches_prefix("PLAN-090.md", "PLAN-", ".md"));
        assert!(!matches_prefix("README.md", "SPEC-", ".md"));
    }
}
