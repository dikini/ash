//! Test discovery: find Ash test files in standard roots.
//!
//! TASK-509: Discovers authored test files from conventional directory roots.

use std::path::{Path, PathBuf};

/// Default test discovery roots (relative to a project root).
pub const TEST_ROOTS: &[&str] = &[
    "tests/ash/unit",
    "tests/ash/integration",
    "tests/ash/e2e",
    "tests/ash/property",
    "tests/ash/smallworld",
];

/// Infer the test kind from the directory a test file lives in.
pub fn infer_kind_from_path(path: &Path) -> crate::test_runner::types::TestKind {
    use crate::test_runner::types::TestKind;
    let path_str = path.to_string_lossy();
    if path_str.contains("/unit/") {
        TestKind::Unit
    } else if path_str.contains("/integration/") {
        TestKind::Integration
    } else if path_str.contains("/e2e/") {
        TestKind::E2e
    } else if path_str.contains("/property/") {
        TestKind::Property
    } else if path_str.contains("/smallworld/") {
        TestKind::SmallWorld
    } else {
        TestKind::Unit
    }
}

/// Discover all `.ash` test files under the given root path.
///
/// If `root` points to a single `.ash` file, returns just that file.
/// If `root` points to a directory, walks the conventional test roots.
pub fn discover_tests(root: &Path) -> Vec<PathBuf> {
    if root.is_file() && root.extension().is_some_and(|e| e == "ash") {
        return vec![root.to_path_buf()];
    }

    let mut files = Vec::new();

    // If root is a directory, look for test files
    if root.is_dir() {
        // First check conventional test roots
        for test_root in TEST_ROOTS {
            let test_dir = root.join(test_root);
            if test_dir.is_dir() {
                collect_ash_files(&test_dir, &mut files);
            }
        }

        // Also check for .ash files directly in tests/ash/
        let tests_ash = root.join("tests/ash");
        if tests_ash.is_dir() {
            // Only collect files directly in tests/ash/ (not in subdirs, which are
            // already covered by the conventional roots above)
            if let Ok(entries) = std::fs::read_dir(&tests_ash) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|e| e == "ash") {
                        files.push(path);
                    }
                }
            }
        }
    }

    files.sort();
    files
}

/// Recursively collect `.ash` files from a directory.
fn collect_ash_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_ash_files(&path, files);
            } else if path.extension().is_some_and(|e| e == "ash") {
                files.push(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn discover_single_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("my_test.ash");
        fs::write(&file, "// test").unwrap();
        let found = discover_tests(&file);
        assert_eq!(found, vec![file]);
    }

    #[test]
    fn discover_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let found = discover_tests(dir.path());
        assert!(found.is_empty());
    }

    #[test]
    fn discover_from_conventional_roots() {
        let dir = tempfile::tempdir().unwrap();
        let unit = dir.path().join("tests/ash/unit");
        fs::create_dir_all(&unit).unwrap();
        let f1 = unit.join("basic.ash");
        fs::write(&f1, "// test").unwrap();
        let f2 = unit.join("nested/deep.ash");
        fs::create_dir_all(unit.join("nested")).unwrap();
        fs::write(&f2, "// test").unwrap();

        let found = discover_tests(dir.path());
        assert_eq!(found.len(), 2);
        assert!(found.contains(&f1));
        assert!(found.contains(&f2));
    }

    #[test]
    fn infer_kind_unit() {
        let p = PathBuf::from("tests/ash/unit/foo.ash");
        assert_eq!(
            infer_kind_from_path(&p),
            crate::test_runner::types::TestKind::Unit
        );
    }

    #[test]
    fn infer_kind_e2e() {
        let p = PathBuf::from("tests/ash/e2e/foo.ash");
        assert_eq!(
            infer_kind_from_path(&p),
            crate::test_runner::types::TestKind::E2e
        );
    }

    #[test]
    fn infer_kind_default() {
        let p = PathBuf::from("tests/ash/foo.ash");
        assert_eq!(
            infer_kind_from_path(&p),
            crate::test_runner::types::TestKind::Unit
        );
    }
}
