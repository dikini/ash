//! TASK-1690: Continuation multiplicity docs consistency.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("ash-core crate lives under crates/ash-core")
        .to_path_buf()
}

fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

#[test]
fn reference_doc_links_required_phase_164_sources_and_commentary() {
    let reference = read(repo_root().join("docs/reference/core-cps-continuation-multiplicity.md"));

    for required in [
        "SPEC-102",
        "PLAN-164",
        "../design/multi-shot-continuations.md",
        "../notes/NOTE-012-MUTUAL-RECURSION-CPS-ASPECTS-DESIGN.md",
        "Non-normative",
        "not a surface Ash syntax proposal",
    ] {
        assert!(
            reference.contains(required),
            "continuation multiplicity reference should contain {required}"
        );
    }
}

#[test]
fn reference_doc_covers_behavior_and_current_core_spelling() {
    let reference = read(repo_root().join("docs/reference/core-cps-continuation-multiplicity.md"));

    for required in [
        "affine",
        "multi-shot-pure",
        "(cont A Ans Row affine)",
        "(cont A Ans {} multi-shot-pure)",
        "(let-cont-call answer resume (lit-int 1) answer)",
        "closed empty row `{}`",
        "empty row by itself does not imply multi-shot",
        "HandlerClause.resume_row = Known(row)",
        "Term::LetContCall",
    ] {
        assert!(
            reference.contains(required),
            "continuation multiplicity reference should document {required}"
        );
    }
}

#[test]
fn reference_doc_names_phase_164_fixtures() {
    let reference = read(repo_root().join("docs/reference/core-cps-continuation-multiplicity.md"));

    for required in [
        "multishot_resume_text_roundtrip.core",
        "affine_empty_row_remains_affine.core",
        "invalid_multishot_nonempty_row.core",
        "invalid_multishot_open_row.core",
        "let_cont_call_text_roundtrip.core",
        "motivational_choice_all_outcomes.core",
        "motivational_backtracking_find_first.core",
        "motivational_nested_choice.core",
        "motivational_discard_resume.core",
        "motivational_affine_choice_all_outcomes_invalid.core",
        "motivational_effectful_multishot_invalid.core",
    ] {
        assert!(
            reference.contains(required),
            "continuation multiplicity reference should list fixture {required}"
        );
    }
}

#[test]
fn existing_core_reference_pages_anchor_phase_164_behavior() {
    let text = read(repo_root().join("docs/reference/core-ash-text-format.md"));
    let lowering = read(repo_root().join("docs/reference/core-ash-lowering.md"));

    for required in [
        "core-cps-continuation-multiplicity.md",
        "let-cont-call",
        "multi-shot-pure",
    ] {
        assert!(
            text.contains(required) || lowering.contains(required),
            "existing Core reference pages should anchor {required}"
        );
    }
}
