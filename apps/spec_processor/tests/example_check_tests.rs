//! Integration tests for the example syntax conformance checker.

use std::path::PathBuf;

use spec_processor::example_check::check_examples;
use spec_processor::finding::Tier;

fn repo_a() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("repo_a")
}

#[test]
fn valid_example_produces_no_findings() {
    let root = repo_a();
    let files = vec!["examples/hello.ash".to_string()];
    let findings = check_examples(&files, &root).expect("engine should build");
    assert!(
        findings.is_empty(),
        "expected no findings for a valid .ash file, got: {findings:?}"
    );
}

#[test]
fn invalid_example_produces_example_failure() {
    let tmp_dir = tempfile::tempdir().expect("temp dir");
    let bad_path = tmp_dir.path().join("bad.ash");
    std::fs::write(&bad_path, "this is not valid ash syntax @@@@!!!\n").expect("write bad file");

    let files = vec!["bad.ash".to_string()];
    let findings = check_examples(&files, tmp_dir.path()).expect("engine should build");

    assert_eq!(
        findings.len(),
        1,
        "expected exactly one finding, got: {findings:?}"
    );
    assert_eq!(findings[0].tier, Tier::Error);
    assert_eq!(findings[0].category, "ExampleFailure");
    assert_eq!(findings[0].file.as_deref(), Some("bad.ash"));
}

#[test]
fn empty_file_list_produces_no_findings() {
    let findings =
        check_examples(&[], PathBuf::from("/tmp").as_path()).expect("engine should build");
    assert!(
        findings.is_empty(),
        "expected no findings for an empty file list, got: {findings:?}"
    );
}
