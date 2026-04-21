//! Integration tests for meta-validation (the spec processor auditing itself).

use std::fs;

use spec_processor::finding::Tier;
use spec_processor::meta_validation::check_meta;

/// The processor's own source directory — all real source files should be
/// non-empty.
#[test]
fn all_processor_source_files_are_non_empty() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = std::path::Path::new(manifest_dir).join("src");
    let repo_root = std::path::Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("manifest_dir should be inside apps/spec_processor/");

    let findings = check_meta(&src_dir, repo_root);

    let empty: Vec<_> = findings
        .iter()
        .filter(|f| f.category == "SourceFileEmpty")
        .collect();

    assert!(
        empty.is_empty(),
        "no source file should be empty, but found: {empty:?}"
    );
}

/// If a referenced design doc is missing, `check_meta` must emit a
/// `BrokenDocRef` warning.
#[test]
fn missing_design_doc_produces_broken_doc_ref() {
    let dir = tempfile::tempdir().unwrap();
    let repo_root = dir.path();

    // Use the real src dir so source-file checks pass.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = std::path::Path::new(manifest_dir).join("src");

    // The temp dir has no docs/ tree, so both referenced docs will be missing.
    let findings = check_meta(&src_dir, repo_root);

    let broken: Vec<_> = findings
        .iter()
        .filter(|f| f.category == "BrokenDocRef")
        .collect();

    assert!(
        !broken.is_empty(),
        "missing docs should produce at least one BrokenDocRef finding"
    );

    for f in &broken {
        assert_eq!(f.tier, Tier::Warning, "BrokenDocRef should be a Warning");
    }
}

/// If a source module has no corresponding test file, `check_meta` must emit
/// a `MissingTestFile` info finding.
#[test]
fn missing_test_file_produces_info_finding() {
    let dir = tempfile::tempdir().unwrap();
    let base = dir.path();

    // Create a src/ with one module but no corresponding test file.
    let src_dir = base.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("fictional_module.rs"), "// not empty\n").unwrap();

    // The tests/ directory is empty (or absent).
    let tests_dir = base.join("tests");
    fs::create_dir_all(&tests_dir).unwrap();

    let findings = check_meta(&src_dir, base);

    let missing: Vec<_> = findings
        .iter()
        .filter(|f| f.category == "MissingTestFile")
        .collect();

    assert_eq!(
        missing.len(),
        1,
        "expected exactly one MissingTestFile finding for 'fictional_module.rs'"
    );
    assert_eq!(missing[0].tier, Tier::Info);
    assert!(missing[0].description.contains("fictional_module"));
}

/// Running `check_meta` against the real processor source tree should produce
/// no Error or Warning tier findings (Info-level findings are acceptable,
/// e.g. `MissingTestFile` for modules that legitimately lack tests).
#[test]
fn clean_state_produces_no_error_or_warning_findings() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = std::path::Path::new(manifest_dir).join("src");
    let repo_root = std::path::Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("manifest_dir should be inside apps/spec_processor/");

    let findings = check_meta(&src_dir, repo_root);

    let errors_warnings: Vec<_> = findings
        .iter()
        .filter(|f| f.tier >= Tier::Warning)
        .collect();

    assert!(
        errors_warnings.is_empty(),
        "real processor tree should have no error/warning findings, got: {errors_warnings:?}"
    );
}
