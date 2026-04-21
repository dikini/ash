//! Integration tests for the spec cross-reference validator.

use spec_processor::spec_links::check_spec_links;
use std::fs;

/// Sets up a temp repo with a spec file containing four link types:
///   - valid internal link   →  should NOT produce a finding
///   - broken internal link  →  should produce a warning
///   - external URL           →  should be ignored
///   - anchor link            →  should be ignored
#[test]
fn integration_broken_link_detected() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Create files that the spec *can* link to.
    fs::write(root.join("target_exists.md"), "# Target").unwrap();
    fs::create_dir_all(root.join("subdir")).unwrap();
    fs::write(root.join("subdir/nested.md"), "# Nested").unwrap();

    // Spec with mixed link types.
    let spec = "\
# SPEC-099 — Integration test

See [existing](target_exists.md) and [nested](subdir/nested.md).
Also check [missing](no_such_file.md).
Visit [website](https://example.com) or jump to [intro](#intro).
";
    fs::write(root.join("SPEC-099.md"), spec).unwrap();

    let findings =
        check_spec_links(&["SPEC-099.md".to_string()], root).expect("repo root is a valid tempdir");

    assert_eq!(
        findings.len(),
        1,
        "expected exactly one broken-link finding"
    );
    let f = &findings[0];
    assert_eq!(f.category, "BrokenLink");
    assert!(
        f.description.contains("no_such_file.md"),
        "description should name the missing file"
    );
    assert_eq!(f.file.as_deref(), Some("SPEC-099.md"));
    assert_eq!(
        f.tier,
        spec_processor::finding::Tier::Warning,
        "broken links should be warnings"
    );
}

/// When every link resolves, no findings should be emitted.
#[test]
fn integration_all_links_valid() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join("a.md"), "").unwrap();
    fs::write(root.join("b.md"), "").unwrap();

    let spec = "[a](a.md) [b](b.md) [ext](https://host) [jump](#top)\n";
    fs::write(root.join("SPEC-100.md"), spec).unwrap();

    let findings =
        check_spec_links(&["SPEC-100.md".to_string()], root).expect("repo root is a valid tempdir");
    assert!(findings.is_empty(), "no findings when all links resolve");
}

/// Spec inside a subdirectory: relative links resolve from that subdirectory.
#[test]
fn integration_relative_links_from_subdirectory() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    fs::create_dir_all(root.join("docs/specs")).unwrap();
    fs::write(root.join("docs/specs/sibling.md"), "# Sibling").unwrap();

    let spec = "[sibling](sibling.md) [missing](gone.md)\n";
    fs::write(root.join("docs/specs/SPEC-101.md"), spec).unwrap();

    let findings = check_spec_links(&["docs/specs/SPEC-101.md".to_string()], root)
        .expect("repo root is a valid tempdir");

    assert_eq!(findings.len(), 1);
    assert!(findings[0].description.contains("gone.md"));
    assert_eq!(findings[0].file.as_deref(), Some("docs/specs/SPEC-101.md"));
}

/// Links with anchor fragments should strip the fragment before checking.
#[test]
fn integration_anchor_fragment_stripped() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join("glossary.md"), "# Glossary\n## Terms\n").unwrap();

    let spec = "[terms](glossary.md#Terms) [broken](absent.md#Section)\n";
    fs::write(root.join("SPEC-102.md"), spec).unwrap();

    let findings =
        check_spec_links(&["SPEC-102.md".to_string()], root).expect("repo root is a valid tempdir");

    assert_eq!(findings.len(), 1);
    assert!(findings[0].description.contains("absent.md"));
}

/// Empty link targets `[]()` should be silently skipped.
#[test]
fn integration_empty_target_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let spec = "[empty]()\n";
    fs::write(root.join("SPEC-103.md"), spec).unwrap();

    let findings =
        check_spec_links(&["SPEC-103.md".to_string()], root).expect("repo root is a valid tempdir");
    assert!(
        findings.is_empty(),
        "empty targets should not produce findings"
    );
}

/// Listing a nonexistent spec file should produce an `UnreadableSpec` warning.
#[test]
fn integration_unreadable_spec_produces_warning() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let findings = check_spec_links(&["no_such_file.md".to_string()], root)
        .expect("repo root is a valid tempdir");

    assert_eq!(findings.len(), 1);
    let f = &findings[0];
    assert_eq!(f.category, "UnreadableSpec");
    assert!(f.description.contains("no_such_file.md"));
    assert_eq!(f.file.as_deref(), Some("no_such_file.md"));
    assert_eq!(f.tier, spec_processor::finding::Tier::Warning);
}

/// Image syntax ![alt](url) should NOT produce a `BrokenLink` finding,
/// but regular [text](url) links to missing targets should still be detected.
#[test]
fn integration_image_links_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let spec = "![alt](missing.png)\n[link](missing.md)\n";
    fs::write(root.join("SPEC-104.md"), spec).unwrap();

    let findings =
        check_spec_links(&["SPEC-104.md".to_string()], root).expect("repo root is a valid tempdir");

    assert_eq!(
        findings.len(),
        1,
        "image link should be skipped, regular link should be flagged"
    );
    assert_eq!(findings[0].category, "BrokenLink");
    assert!(findings[0].description.contains("missing.md"));
}

/// When `repo_root` does not exist, the function should return `Err`,
/// not a list of findings.
#[test]
fn integration_error_on_invalid_repo_root() {
    let bogus = std::path::Path::new("/no/such/directory/ash/testing");
    let result = check_spec_links(&["SPEC-999.md".to_string()], bogus);
    assert!(
        result.is_err(),
        "should return Err when repo_root does not exist"
    );
    let err = result.unwrap_err();
    assert_eq!(
        err.kind(),
        std::io::ErrorKind::NotADirectory,
        "error kind should be NotADirectory"
    );
}
