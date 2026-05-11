//! TASK-850: engine summary dedup/cache invalidation includes computation facts.

use ash_core::semantic_summary::SummaryVersion;
use ash_engine::module_loader::load_ordinary_file;

fn write_file(path: &std::path::Path, source: &str) {
    std::fs::write(path, source).expect("write ash source");
}

#[test]
fn explicit_type_function_imports_with_same_ordinary_types_do_not_dedup_together() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub type Token = Int;
pub sealed type domain Flag { On; Off; }
pub type fn First(x: Flag) -> Flag { case First<x> = x; }
pub type fn Second(x: Flag) -> Flag { case Second<x> = On; }
",
    );
    write_file(
        &caller,
        r"use provider::{First, Second}
workflow main { ret 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("caller imports both computation heads");
    let computation_summaries = loaded
        .imported_semantic_summaries
        .iter()
        .filter(|summary| !summary.exported_type_functions.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(
        computation_summaries.len(),
        2,
        "summaries with different computation facts must not dedup into one summary: {computation_summaries:#?}"
    );
    for summary in computation_summaries {
        assert_eq!(summary.version, SummaryVersion::SPEC062_TYPE_COMPUTATION_V3);
        assert_eq!(summary.exported_types.len(), 0);
        assert_eq!(summary.exported_sealed_domains.len(), 1);
        assert_eq!(summary.exported_type_functions.len(), 1);
    }
}

#[test]
fn dependency_closure_and_selected_head_imports_do_not_dedup_by_ordinary_types_only() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider = dir.path().join("provider.ash");
    let caller = dir.path().join("caller.ash");
    write_file(
        &provider,
        r"
pub sealed type domain Bits { Zero; One; }
pub type fn Id(x: Bits) -> Bits { case Id<x> = x; }
pub type fn UseId(x: Bits) -> Bits { case UseId<x> = Id<x>; }
",
    );
    write_file(
        &caller,
        r"use provider::{Id, UseId}
workflow main { ret 0 }
",
    );

    let loaded = load_ordinary_file(&caller).expect("caller imports direct and dependent heads");
    let mut computation_head_sets = loaded
        .imported_semantic_summaries
        .iter()
        .filter(|summary| !summary.exported_type_functions.is_empty())
        .map(|summary| {
            let mut names = summary
                .exported_type_functions
                .iter()
                .map(|type_function| type_function.exported_name.as_str())
                .collect::<Vec<_>>();
            names.sort_unstable();
            names
        })
        .collect::<Vec<_>>();
    computation_head_sets.sort_unstable();

    assert_eq!(
        computation_head_sets,
        vec![vec!["Id"], vec!["Id", "UseId"]],
        "selected computation summaries must remain separated when their computation facts differ"
    );
}
