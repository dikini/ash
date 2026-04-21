//! Meta-validation: the spec processor audits itself.
//!
//! Runs a small set of self-checks to ensure the processor's own source tree,
//! documentation references, capability declarations, and test coverage are
//! internally consistent. These checks are intentionally lightweight — `cargo
//! check` already validates Rust parseability, so we focus on higher-level
//! structural invariants.

use std::fs;
use std::path::Path;

use crate::capability_boundary;
use crate::finding::SpecFinding;

/// Design documents that the spec processor references and that must exist.
const REFERENCED_DOCS: &[&str] = &[
    "docs/design/DESIGN-SPEC-PROCESSOR.md",
    "docs/plan/PLAN-090-SPEC-PROCESSOR.md",
];

/// Validate the spec processor's own integrity.
///
/// `processor_src_dir` is the `apps/spec_processor/src/` directory.
/// `repo_root` is the repository root (for resolving doc references).
///
/// Returns a list of findings covering:
/// - empty source files (`SourceFileEmpty`, Warning)
/// - missing referenced documentation (`BrokenDocRef`, Warning)
/// - capability boundary inconsistencies (`CapabilityInconsistency`, Warning)
/// - missing test files (`MissingTestFile`, Info)
#[must_use]
pub fn check_meta(processor_src_dir: &Path, repo_root: &Path) -> Vec<SpecFinding> {
    let mut findings = Vec::new();
    findings.extend(check_source_files(processor_src_dir));
    findings.extend(check_doc_refs(repo_root));
    findings.extend(check_capability_consistency());
    findings.extend(check_test_coverage(processor_src_dir));
    findings
}

// ---------------------------------------------------------------------------
// 1. Source file non-emptiness
// ---------------------------------------------------------------------------

/// Verify every `.rs` file in the processor's `src/` directory exists and is
/// non-empty.
fn check_source_files(src_dir: &Path) -> Vec<SpecFinding> {
    let mut findings = Vec::new();

    let Ok(entries) = fs::read_dir(src_dir) else {
        findings.push(
            SpecFinding::error(
                "MetaValidationError",
                format!("cannot read source directory: {}", src_dir.display()),
            )
            .with_file(src_dir.display().to_string()),
        );
        return findings;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();

            let contents = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    findings.push(
                        SpecFinding::warning(
                            "SourceUnreadable",
                            format!("source file '{file_name}' is unreadable: {e}"),
                        )
                        .with_file(format!("src/{file_name}")),
                    );
                    continue;
                }
            };
            if contents.trim().is_empty() {
                findings.push(
                    SpecFinding::warning(
                        "SourceFileEmpty",
                        format!("source file '{file_name}' is empty"),
                    )
                    .with_file(format!("src/{file_name}")),
                );
            }
        }
    }

    findings
}

// ---------------------------------------------------------------------------
// 2. Design document cross-references
// ---------------------------------------------------------------------------

/// Verify that the design/plan documents referenced by the processor exist in
/// the repository.
fn check_doc_refs(repo_root: &Path) -> Vec<SpecFinding> {
    let mut findings = Vec::new();

    for doc_rel in REFERENCED_DOCS {
        let full_path = repo_root.join(doc_rel);
        if !full_path.exists() {
            findings.push(
                SpecFinding::warning(
                    "BrokenDocRef",
                    format!("referenced document '{doc_rel}' does not exist"),
                )
                .with_file(*doc_rel),
            );
        }
    }

    findings
}

// ---------------------------------------------------------------------------
// 3. Capability boundary consistency
// ---------------------------------------------------------------------------

/// Every capability marked `expected == true` must have a non-empty
/// `stdlib_file` path.
fn check_capability_consistency() -> Vec<SpecFinding> {
    let mut findings = Vec::new();

    for cap in capability_boundary::expected_capabilities() {
        if cap.expected && cap.stdlib_file.trim().is_empty() {
            findings.push(SpecFinding::warning(
                "CapabilityInconsistency",
                format!(
                    "capability '{}' is marked expected=true but has an empty stdlib_file path",
                    cap.name
                ),
            ));
        }
    }

    findings
}

// ---------------------------------------------------------------------------
// 4. Test coverage check
// ---------------------------------------------------------------------------

/// For each source module in `src/`, check that a corresponding test file
/// exists under the `tests/` sibling directory. The mapping is:
///
///   `src/example_check.rs`  →  `tests/example_check_tests.rs`
fn check_test_coverage(src_dir: &Path) -> Vec<SpecFinding> {
    let mut findings = Vec::new();

    // tests/ is a sibling of src/
    let tests_dir = src_dir.parent().map(|p| p.join("tests"));

    let Some(tests_dir) = tests_dir else {
        return findings;
    };

    let Ok(entries) = fs::read_dir(src_dir) else {
        findings.push(SpecFinding::warning(
            "TestCoverageUnreadable",
            format!(
                "cannot read source directory for test coverage check: {}",
                src_dir.display()
            ),
        ));
        return findings;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();

            // lib.rs and mod.rs are not expected to have standalone test files
            if stem == "lib" || stem == "mod" {
                continue;
            }

            let expected_test = tests_dir.join(format!("{stem}_tests.rs"));
            if !expected_test.exists() {
                findings.push(SpecFinding::info(
                    "MissingTestFile",
                    format!("no test file found for source module '{stem}.rs'"),
                ));
            }
        }
    }

    findings
}
