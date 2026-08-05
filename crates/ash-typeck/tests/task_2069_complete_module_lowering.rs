//! TASK-2069 activation contract.
//!
//! This target is intentionally limited to TASK-2069's activation boundary.

use std::fs;
use std::path::Path;

use ash_core::module_graph::ModuleKey;
use ash_parser::{CanonicalExpandedModuleGraph, CanonicalModuleGraphResolver};
use ash_typeck::canonical_checked_module_finalizer::finalize_canonical_module_collection;
use ash_typeck::canonical_module_collection::collect_canonical_expanded_module_graph;
use ash_typeck::module_core_cps_lowering::ModuleCoreCpsLoweringError;
use ash_typeck::resolve_parsed_imports_from_collection;

#[test]
fn task_2069_activation_contract_declares_non_authorizing_handoff() {
    let task_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../docs/plan/tasks/TASK-2069-complete-module-lowering-and-engine-transport-fencing.md",
    );
    let task = fs::read_to_string(&task_path).expect("TASK-2069 task file exists");

    assert!(task.contains("**Status:** In progress"));
    assert!(task.contains(
        "**Semantic coverage map:** [TASK-2069 record](../SEMANTIC-RULE-COVERAGE.md#task-2069-complete-module-lowering-and-engine-transport-fencing)"
    ));

    for evidence_id in [
        "TEST-MOD-REAL-005-FULL-DEFINITION-BODY-LOWERING",
        "TEST-MOD-REAL-005-FINALIZED-BODY-AUTHORITY",
        "TEST-MOD-REAL-005-ENGINE-CHECKED-TRANSPORT",
        "TEST-MOD-REAL-005-BODY-LOWERING-REJECTION",
        "TEST-MOD-REAL-005-PROVENANCE-REWRITE",
        "TEST-MOD-REAL-005-SCANNER-AUTHORITY-REJECTION",
        "TEST-MOD-REAL-005-CANONICAL-CACHE-KEY",
        "TEST-MOD-REAL-005-FILE-INLINE-LOWERING-PARITY",
    ] {
        assert!(
            task.contains(evidence_id),
            "TASK-2069 must reserve evidence identifier {evidence_id}"
        );
    }
}

#[test]
fn complete_checked_module_definition_bodies_preserve_module_provenance() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-complete-body-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "fn normalize() -> Int { 1 }").expect("write fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("fixture source resolves through the canonical parser graph"),
    )
    .expect("fixture source expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the supported function fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the import-free fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the complete function body");
    let finalized_module = finalized
        .module(&module_key)
        .expect("finalization retains the fixture module");

    let (core, cps) =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collection,
            &expanded,
            &module_key,
            "normalize",
        )
        .expect("a complete checked function body lowers to module Core/CPS");

    assert_eq!(core.module_artifact().key(), finalized_module.module_key());
    assert_eq!(core.module_artifact().origin(), finalized_module.origin());
    assert_eq!(cps.module_artifact().key(), finalized_module.module_key());
    assert_eq!(cps.module_artifact().origin(), finalized_module.origin());
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn complete_checked_module_definition_bodies_use_finalized_callable_body() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-finalized-body-authority-{}",
        std::process::id()
    ));
    let finalized_root = fixture_root.join("finalized");
    let collected_root = fixture_root.join("collected");
    fs::create_dir_all(finalized_root.join("src")).expect("create finalized source directory");
    fs::create_dir_all(collected_root.join("src")).expect("create collected source directory");
    let finalized_path = finalized_root.join("src/main.ash");
    let collected_path = collected_root.join("src/main.ash");
    fs::write(&finalized_path, "fn normalize() -> Int { 1 }").expect("write finalized fixture");
    fs::write(&collected_path, "fn normalize() -> Int { 2 }").expect("write collected fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let finalized_expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &finalized_path)
            .expect("finalized source resolves through the canonical parser graph"),
    )
    .expect("finalized source expands through the canonical expanded graph");
    let finalized_collection = collect_canonical_expanded_module_graph(&finalized_expanded)
        .expect("TASK-2075 collection accepts the finalized fixture");
    let finalized_imports = resolve_parsed_imports_from_collection(
        finalized_expanded.parsed_graph(),
        &finalized_collection,
    )
    .expect("TASK-2072 import handoff accepts the finalized fixture");
    let finalized = finalize_canonical_module_collection(
        &finalized_expanded,
        &finalized_collection,
        &finalized_imports,
    )
    .expect("TASK-2073 finalization checks the finalized fixture");

    let collected_expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &collected_path)
            .expect("collected source resolves through the canonical parser graph"),
    )
    .expect("collected source expands through the canonical expanded graph");
    let collected_collection = collect_canonical_expanded_module_graph(&collected_expanded)
        .expect("TASK-2075 collection accepts the collected fixture");

    let (core, _cps) =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collected_collection,
            &finalized_expanded,
            &module_key,
            "normalize",
        )
        .expect("lowering succeeds with the finalized graph and collected body");

    let checked_core_debug = format!("{:?}", core.checked_core_program());
    assert!(checked_core_debug.contains("LitInt(1)"));
    assert!(!checked_core_debug.contains("LitInt(2)"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn unsupported_checked_module_definition_body_is_rejected_before_artifact_creation() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-body-rejection-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "fn normalize() -> Int { 1 + 1 }").expect("write fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("fixture source resolves through the canonical parser graph"),
    )
    .expect("fixture source expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the supported function fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the import-free fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the complete function body");

    let error =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collection,
            &expanded,
            &module_key,
            "normalize",
        )
        .expect_err("unsupported surface body must not create a module artifact");

    assert!(matches!(
        error,
        ModuleCoreCpsLoweringError::SurfaceLowering { .. }
    ));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn missing_checked_module_definition_is_rejected_before_artifact_creation() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-missing-definition-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "fn normalize() -> Int { 1 }").expect("write fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("fixture source resolves through the canonical parser graph"),
    )
    .expect("fixture source expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the supported function fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the import-free fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the complete function body");

    let error =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collection,
            &expanded,
            &module_key,
            "missing",
        )
        .expect_err("a missing checked definition must not create a module artifact");

    assert!(matches!(
        error,
        ModuleCoreCpsLoweringError::MissingCheckedDefinition { .. }
    ));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn rewritten_module_provenance_is_rejected_before_artifact_creation() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-provenance-rewrite-{}",
        std::process::id()
    ));
    let finalized_root = fixture_root.join("finalized");
    let rewritten_root = fixture_root.join("rewritten");
    fs::create_dir_all(finalized_root.join("src")).expect("create finalized source directory");
    fs::create_dir_all(rewritten_root.join("src")).expect("create rewritten source directory");
    let finalized_path = finalized_root.join("src/main.ash");
    let rewritten_path = rewritten_root.join("src/main.ash");
    fs::write(&finalized_path, "fn normalize() -> Int { 1 }").expect("write finalized fixture");
    fs::write(&rewritten_path, "fn normalize() -> Int { 1 }").expect("write rewritten fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let finalized_expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &finalized_path)
            .expect("finalized source resolves through the canonical parser graph"),
    )
    .expect("finalized source expands through the canonical expanded graph");
    let finalized_collection = collect_canonical_expanded_module_graph(&finalized_expanded)
        .expect("TASK-2075 collection accepts the finalized fixture");
    let finalized_imports = resolve_parsed_imports_from_collection(
        finalized_expanded.parsed_graph(),
        &finalized_collection,
    )
    .expect("TASK-2072 import handoff accepts the finalized fixture");
    let finalized = finalize_canonical_module_collection(
        &finalized_expanded,
        &finalized_collection,
        &finalized_imports,
    )
    .expect("TASK-2073 finalization checks the finalized fixture");

    let rewritten_expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &rewritten_path)
            .expect("rewritten source resolves through the canonical parser graph"),
    )
    .expect("rewritten source expands through the canonical expanded graph");
    let rewritten_collection = collect_canonical_expanded_module_graph(&rewritten_expanded)
        .expect("TASK-2075 collection accepts the rewritten fixture");

    let error =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &rewritten_collection,
            &rewritten_expanded,
            &module_key,
            "normalize",
        )
        .expect_err("rewritten source provenance must not create a module artifact");

    assert!(matches!(
        error,
        ModuleCoreCpsLoweringError::ProvenanceMismatch { .. }
    ));
    let _ = fs::remove_dir_all(fixture_root);
}
