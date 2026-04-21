//! Cross-reference validator for spec Markdown files.
//!
//! Extracts Markdown links from spec files and verifies that the target files
//! exist on disk.  External URLs and anchor-only links are ignored.

use crate::finding::SpecFinding;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

/// Compiled link-extraction pattern — static, initialised once.
///
/// Capture group 1: optional `!` prefix (image marker).
/// Capture group 2: link text inside `[...]`.
/// Capture group 3: target URL inside `(...)`.
static LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(!?)\[([^\]]*)\]\(([^)]+)\)").unwrap());

/// Check all spec files for broken cross-references.
///
/// * `spec_files` — list of paths (relative to `repo_root`) to SPEC-*.md files
/// * `repo_root`  — the repository root, used for resolving relative links
///
/// # Errors
///
/// Returns `Err` if `repo_root` does not exist or is not a directory.
/// Individual unreadable spec files are reported as `UnreadableSpec` warnings
/// in the returned findings rather than causing an error.
///
/// # Panics
///
/// The compiled regex guarantees a third capture group for every match,
/// so the indexing `cap[3]` cannot panic in practice.
pub fn check_spec_links(
    spec_files: &[String],
    repo_root: &Path,
) -> Result<Vec<SpecFinding>, std::io::Error> {
    if !repo_root.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            format!(
                "repo root '{}' is not a directory or does not exist",
                repo_root.display()
            ),
        ));
    }

    let mut findings = Vec::new();

    for file in spec_files {
        let path = repo_root.join(file);
        let Ok(content) = std::fs::read_to_string(&path) else {
            findings.push(
                SpecFinding::warning("UnreadableSpec", format!("cannot read spec file '{file}'"))
                    .with_file(file.clone()),
            );
            continue;
        };

        for cap in LINK_RE.captures_iter(&content) {
            // Skip image syntax ![alt](url) — the `!` prefix is captured in group 1.
            if !cap[1].is_empty() {
                continue;
            }

            let target = &cap[3];

            // Skip external URLs and anchor-only links.
            if target.starts_with("http") || target.starts_with('#') {
                continue;
            }

            // Remove anchor fragment, if any.  split('#') always yields >= 1 element.
            let target_path = target.split('#').next().unwrap();

            // Skip empty targets (e.g. "[]()").
            if target_path.is_empty() {
                continue;
            }

            // Resolve relative to the spec file's directory.
            let spec_dir = path.parent().unwrap_or_else(|| Path::new("."));
            let resolved = spec_dir.join(target_path);

            if !resolved.exists() {
                findings.push(
                    SpecFinding::warning(
                        "BrokenLink",
                        format!("Link target '{target}' does not exist"),
                    )
                    .with_file(file.clone()),
                );
            }
        }
    }

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Tier;
    use std::fs;

    #[test]
    fn detects_broken_link() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Create a target file that exists.
        fs::write(root.join("exists.md"), "hello").unwrap();

        // Write a spec with valid, broken, external, and anchor links.
        let spec_content = "\
# Spec

[valid](exists.md)
[broken](missing.md)
[remote](https://example.com)
[anchor](#intro)
";
        let spec_path = root.join("SPEC-001.md");
        fs::write(&spec_path, spec_content).unwrap();

        let spec_files = vec!["SPEC-001.md".to_string()];
        let findings = check_spec_links(&spec_files, root).expect("repo root is valid");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, "BrokenLink");
        assert!(findings[0].description.contains("missing.md"));
        assert_eq!(findings[0].file.as_deref(), Some("SPEC-001.md"));
        assert_eq!(findings[0].tier, Tier::Warning);
    }

    #[test]
    fn no_findings_when_all_links_valid() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        fs::write(root.join("a.md"), "").unwrap();
        fs::write(root.join("b.md"), "").unwrap();

        let spec_content = "[a](a.md) [b](b.md)\n";
        fs::write(root.join("SPEC-002.md"), spec_content).unwrap();

        let spec_files = vec!["SPEC-002.md".to_string()];
        let findings = check_spec_links(&spec_files, root).expect("repo root is valid");

        assert!(findings.is_empty());
    }

    #[test]
    fn skips_unknown_files_gracefully() {
        let dir = tempfile::tempdir().unwrap();
        let spec_files = vec!["nonexistent.md".to_string()];
        let findings = check_spec_links(&spec_files, dir.path()).expect("repo root is valid");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, "UnreadableSpec");
        assert_eq!(findings[0].file.as_deref(), Some("nonexistent.md"));
        assert_eq!(findings[0].tier, Tier::Warning);
    }

    #[test]
    fn error_when_repo_root_does_not_exist() {
        let bogus = Path::new("/no/such/directory/for/testing");
        let result = check_spec_links(&[], bogus);
        assert!(
            result.is_err(),
            "should return Err for non-existent repo root"
        );
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotADirectory);
    }
}
