//! TASK-882: SPEC-H engine transport non-interference aggregation.

use ash_core::semantic_summary::SummaryVersion;
use ash_core::type_ir::{PropositionEvidenceRule, PropositionOutcome, TypeProposition};
use ash_engine::module_loader::load_ordinary_file;

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash fixture");
}

#[test]
fn task_882_engine_h9_transports_v5_proposition_requirements_without_engine_solving() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub fn needs(x: Int) -> Int where Int == Int { x }
",
    );
    write_file(
        &caller,
        r"use provider::{needs}
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("named import should load provider");
    let proposition_summary = loaded
        .imported_semantic_summaries
        .iter()
        .find(|summary| !summary.exported_proposition_facts.is_empty())
        .expect("transported proposition fact summary should be present");

    assert_eq!(
        proposition_summary.version,
        SummaryVersion::SPEC064_PROPOSITIONS_V5,
        "H9: proposition-carrying imported summary must use V5"
    );
    assert_eq!(proposition_summary.exported_proposition_facts.len(), 1);
    assert!(matches!(
        &proposition_summary.exported_proposition_facts[0].proposition,
        TypeProposition::Equality(eq) if eq.lhs == eq.rhs
    ));
    assert!(matches!(
        &proposition_summary.exported_proposition_facts[0].outcome,
        Some(PropositionOutcome::Satisfied(evidence))
            if evidence.rule == PropositionEvidenceRule::DefinitionalEquality
    ));
}

#[test]
fn task_882_engine_h12_pub_use_and_glob_transport_do_not_duplicate_or_interpret_proposition_facts()
{
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let facade = dir.path().join("facade.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub fn needs(x: Int) -> Int where Int == Int { x }
",
    );
    write_file(&facade, "pub use provider::*;\n");
    write_file(
        &caller,
        r"use facade::*
fn main() { 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("pub-use glob import should load");
    let facts = loaded
        .imported_semantic_summaries
        .iter()
        .flat_map(|summary| summary.exported_proposition_facts.iter())
        .collect::<Vec<_>>();

    assert_eq!(
        facts.len(),
        1,
        "engine should transport exactly one proposition payload and must not solve/deduplicate by semantics: {facts:?}"
    );
    assert!(matches!(
        &facts[0].proposition,
        TypeProposition::Equality(eq) if eq.lhs == eq.rhs
    ));
}
