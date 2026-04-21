//! Integration tests for the capability boundary audit.

use std::fs;

use spec_processor::capability_boundary::{check_capabilities, expected_capabilities};
use spec_processor::finding::Tier;

/// A missing stdlib file for an expected capability must produce a
/// `ToolingGap` warning.
#[test]
fn missing_stdlib_file_produces_tooling_gap() {
    let dir = tempfile::tempdir().unwrap();
    // Temp dir is empty — no stdlib files at all.
    let findings = check_capabilities(dir.path());

    let expected_count = expected_capabilities()
        .iter()
        .filter(|c| c.expected)
        .count();

    let tooling_gaps: Vec<_> = findings
        .iter()
        .filter(|f| f.category == "ToolingGap")
        .collect();

    assert_eq!(
        tooling_gaps.len(),
        expected_count,
        "every expected capability should produce a ToolingGap when its file is missing"
    );

    for gap in &tooling_gaps {
        assert_eq!(gap.tier, Tier::Warning);
    }
}

/// A present, substantive stdlib file for an expected capability should
/// produce **no** `ToolingGap` finding.
#[test]
fn present_stdlib_file_produces_no_finding() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Create every expected stdlib file with a substantive declaration.
    for cap in expected_capabilities() {
        if cap.expected {
            let path = root.join(&cap.stdlib_file);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&path, "pub builtin fn do_thing() -> None\n").unwrap();
        }
    }

    let findings = check_capabilities(root);

    let tooling_gaps: Vec<_> = findings
        .iter()
        .filter(|f| f.category == "ToolingGap")
        .collect();

    assert!(
        tooling_gaps.is_empty(),
        "no ToolingGap findings expected when all files exist with declarations, got: {tooling_gaps:?}"
    );
}

/// An empty stdlib file (only whitespace/comments) should produce a
/// `ToolingGap` warning.
#[test]
fn empty_stdlib_file_produces_tooling_gap() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Create every expected stdlib file, but all are effectively empty.
    for cap in expected_capabilities() {
        if cap.expected {
            let path = root.join(&cap.stdlib_file);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&path, "// TODO: implement later\n\n").unwrap();
        }
    }

    let findings = check_capabilities(root);

    let expected_count = expected_capabilities()
        .iter()
        .filter(|c| c.expected)
        .count();

    let tooling_gaps: Vec<_> = findings
        .iter()
        .filter(|f| f.category == "ToolingGap")
        .collect();

    assert_eq!(
        tooling_gaps.len(),
        expected_count,
        "every expected capability should produce a ToolingGap when its file is empty"
    );

    for gap in &tooling_gaps {
        assert_eq!(gap.tier, Tier::Warning);
        assert!(
            gap.description.contains("no public fn/type declarations"),
            "description should explain the file is effectively empty: {}",
            gap.description
        );
    }
}

/// Capabilities marked `expected == false` should each produce an
/// informational `CapabilityPending` finding.
#[test]
fn unexpected_capability_produces_info_finding() {
    let dir = tempfile::tempdir().unwrap();
    // Even with an empty dir, the only findings for expected=false caps
    // should be CapabilityPending info items.
    let findings = check_capabilities(dir.path());

    let pending: Vec<_> = findings
        .iter()
        .filter(|f| f.category == "CapabilityPending")
        .collect();

    let unexpected_count = expected_capabilities()
        .iter()
        .filter(|c| !c.expected)
        .count();

    assert_eq!(
        pending.len(),
        unexpected_count,
        "each expected=false capability should produce one CapabilityPending finding"
    );

    for p in &pending {
        assert_eq!(p.tier, Tier::Info);
        assert!(
            p.description.contains("not yet available"),
            "info finding should note the capability is pending: {}",
            p.description
        );
    }
}

/// The boundary declaration should be auditable (public and return the
/// expected number of entries).
#[test]
fn expected_capabilities_is_auditable() {
    let caps = expected_capabilities();
    // file_io, stdio, regex, json, markdown, process, generic_interfaces
    assert_eq!(
        caps.len(),
        7,
        "boundary declaration should list 7 capabilities"
    );

    let names: Vec<&str> = caps.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"file_io"), "missing file_io");
    assert!(names.contains(&"stdio"), "missing stdio");
    assert!(names.contains(&"regex"), "missing regex");
    assert!(names.contains(&"json"), "missing json");
    assert!(names.contains(&"markdown"), "missing markdown");
    assert!(names.contains(&"process"), "missing process");
    assert!(
        names.contains(&"generic_interfaces"),
        "missing generic_interfaces"
    );
}
