//! TASK-2069 activation contract.
//!
//! This target is intentionally limited to TASK-2069's activation boundary.

use std::fs;
use std::path::Path;

use ash_core::module_graph::ModuleKey;
use ash_core::module_interface::ModuleInterfaceDefiningIdentity;
use ash_parser::{CanonicalExpandedModuleGraph, CanonicalModuleGraphResolver};
use ash_typeck::canonical_checked_module_finalizer::finalize_canonical_module_collection;
use ash_typeck::canonical_module_collection::{
    CanonicalNamespace, collect_canonical_expanded_module_graph,
};
use ash_typeck::module_core_cps_lowering::{
    LoweredCheckedModuleDefinition, ModuleCoreCpsLoweringError,
    lower_complete_checked_module_definition_closure,
};
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
        "TEST-MOD-REAL-005-FULL-DEFINITION-BODY-CLOSURE",
        "TEST-MOD-REAL-005-FINALIZED-BODY-AUTHORITY",
        "TEST-MOD-REAL-005-IMPORT-TRANSPORT",
        "TEST-MOD-REAL-005-UNSUPPORTED-IMPORT-TRANSPORT",
        "TEST-MOD-REAL-005-ENGINE-CHECKED-TRANSPORT",
        "TEST-MOD-REAL-005-BODY-LOWERING-REJECTION",
        "TEST-MOD-REAL-005-PROVENANCE-REWRITE",
        "TEST-MOD-REAL-005-CLOSURE-ATOMICITY",
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

#[test]
fn complete_checked_module_definition_closure_lowers_all_bodies_in_declaration_order() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-definition-closure-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "fn normalize() -> Int { 1 } fn second() -> Int { 2 }",
    )
    .expect("write fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key, &root_path)
            .expect("fixture source resolves through the canonical parser graph"),
    )
    .expect("fixture source expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the supported function fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the import-free fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks both complete function bodies");

    let lowered: Vec<LoweredCheckedModuleDefinition> =
        lower_complete_checked_module_definition_closure(
            &finalized,
            &collection,
            &expanded,
            &imports,
        )
        .expect("complete checked definition closure lowers all supported bodies");

    assert_eq!(lowered.len(), 2);
    assert_eq!(lowered[0].declaration_name(), "normalize");
    assert_eq!(lowered[1].declaration_name(), "second");
    assert!(format!("{:?}", lowered[0].core()).contains("LitInt(1)"));
    assert!(format!("{:?}", lowered[1].core()).contains("LitInt(2)"));
    assert!(!format!("{:?}", lowered[0].cps()).is_empty());
    assert!(!format!("{:?}", lowered[1].cps()).is_empty());

    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn complete_checked_module_definition_closure_rejects_unsupported_body_without_partial_result() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-definition-closure-rejection-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "fn normalize() -> Int { 1 } fn broken() -> Int { 1 + 1 }",
    )
    .expect("write fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key, &root_path)
            .expect("fixture source resolves through the canonical parser graph"),
    )
    .expect("fixture source expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the supported function fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the import-free fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks both function bodies");

    let result = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    );

    assert!(matches!(
        result,
        Err(ModuleCoreCpsLoweringError::SurfaceLowering { .. })
    ));

    let _ = fs::remove_dir_all(fixture_root);
}

/// RED: `TEST-MOD-REAL-005-FULL-DEFINITION-BODY-CLOSURE` and
/// `TEST-MOD-REAL-005-CLOSURE-ATOMICITY`.
#[test]
fn red_complete_checked_module_definition_closure_retains_import_identity_and_rejects_stale_transport_atomically()
 {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-import-closure-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    for variant in ["valid", "missing", "stale"] {
        fs::create_dir_all(fixture_root.join(variant).join("src"))
            .expect("create multi-module fixture source directory");
    }

    let valid_main = fixture_root.join("valid/src/main.ash");
    fs::write(&valid_main, "pub mod api; pub mod client;").expect("write valid fixture root");
    fs::write(
        fixture_root.join("valid/src/api.ash"),
        "pub fn serve() -> Int { 1 }",
    )
    .expect("write valid provider fixture");
    fs::write(
        fixture_root.join("valid/src/client.ash"),
        "use crate::api::serve as remote; fn entry() -> Int { 2 }",
    )
    .expect("write valid importing fixture");

    let missing_main = fixture_root.join("missing/src/main.ash");
    fs::write(&missing_main, "pub mod api; pub mod client;")
        .expect("write missing-transport fixture root");
    fs::write(
        fixture_root.join("missing/src/api.ash"),
        "pub fn serve() -> Int { 1 }",
    )
    .expect("write missing-transport provider fixture");
    fs::write(
        fixture_root.join("missing/src/client.ash"),
        "fn entry() -> Int { 2 }",
    )
    .expect("write missing-transport importing fixture");

    let stale_main = fixture_root.join("stale/src/main.ash");
    fs::write(&stale_main, "pub mod api; pub mod client;")
        .expect("write stale-transport fixture root");
    fs::write(
        fixture_root.join("stale/src/api.ash"),
        "pub fn other() -> Int { 3 }",
    )
    .expect("write stale-transport provider fixture");
    fs::write(
        fixture_root.join("stale/src/client.ash"),
        "use crate::api::other as remote; fn entry() -> Int { 2 }",
    )
    .expect("write stale-transport importing fixture");

    let root = ModuleKey::root("app").expect("fixture module key is canonical");
    let valid_expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &valid_main)
            .expect("valid fixture resolves through the canonical parser graph"),
    )
    .expect("valid fixture expands through the canonical expanded graph");
    let valid_collection = collect_canonical_expanded_module_graph(&valid_expanded)
        .expect("TASK-2075 collects the valid multi-module closure");
    let valid_imports =
        resolve_parsed_imports_from_collection(valid_expanded.parsed_graph(), &valid_collection)
            .expect("TASK-2072 resolves the valid aliased import");
    let finalized =
        finalize_canonical_module_collection(&valid_expanded, &valid_collection, &valid_imports)
            .expect("TASK-2073 finalizes the valid multi-module closure");

    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &valid_collection,
        &valid_expanded,
        &valid_imports,
    )
    .expect("complete lowering retains the checked import transport");
    let client = root
        .child("client")
        .expect("client module key is canonical");
    let client_lowered = lowered
        .iter()
        .find(|definition| definition.core().module_artifact().key() == &client)
        .expect("client definition is present in the lowered closure");
    assert_eq!(client_lowered.core().imports().len(), 1);
    assert_eq!(
        client_lowered.cps().imports(),
        client_lowered.core().imports()
    );
    let imported = &client_lowered.core().imports()[0];
    assert_eq!(imported.local_name(), "remote");
    assert_eq!(
        imported.binding().origin(),
        finalized
            .module(&root.child("api").expect("api module key is canonical"))
            .expect("api finalization is retained")
            .origin()
    );
    match imported.binding().defining_identity() {
        ModuleInterfaceDefiningIdentity::Declaration(identity) => {
            assert_eq!(
                identity.module,
                root.child("api").expect("api module key is canonical")
            );
            assert_eq!(identity.name, "serve");
        }
        other => panic!("expected imported callable declaration identity, got {other:?}"),
    }

    let missing_expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &missing_main)
            .expect("missing-transport fixture resolves through the canonical parser graph"),
    )
    .expect("missing-transport fixture expands through the canonical expanded graph");
    let missing_collection = collect_canonical_expanded_module_graph(&missing_expanded)
        .expect("TASK-2075 collects the missing-transport fixture");
    let missing_imports = resolve_parsed_imports_from_collection(
        missing_expanded.parsed_graph(),
        &missing_collection,
    )
    .expect("missing-transport fixture has an atomically empty import carrier");
    let missing = lower_complete_checked_module_definition_closure(
        &finalized,
        &valid_collection,
        &valid_expanded,
        &missing_imports,
    );
    assert!(
        missing.is_err(),
        "missing import transport must reject before publishing a partial closure"
    );

    let stale_expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &stale_main)
            .expect("stale-transport fixture resolves through the canonical parser graph"),
    )
    .expect("stale-transport fixture expands through the canonical expanded graph");
    let stale_collection = collect_canonical_expanded_module_graph(&stale_expanded)
        .expect("TASK-2075 collects the stale-transport fixture");
    let stale_imports =
        resolve_parsed_imports_from_collection(stale_expanded.parsed_graph(), &stale_collection)
            .expect("TASK-2072 resolves the stale aliased import carrier");
    let stale = lower_complete_checked_module_definition_closure(
        &finalized,
        &valid_collection,
        &valid_expanded,
        &stale_imports,
    );
    assert!(
        stale.is_err(),
        "stale import transport must reject before publishing a partial closure"
    );

    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn unsupported_import_namespace_rejects_before_lowering() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-unsupported-import-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "pub mod api; pub mod client;").expect("write fixture root");
    fs::write(fixture_root.join("src/api.ash"), "pub type Value = Int;")
        .expect("write type provider fixture");
    fs::write(
        fixture_root.join("src/client.ash"),
        "use crate::api::Value as remote; fn entry() -> Int { 1 }",
    )
    .expect("write importing fixture");

    let root = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root, &root_path)
            .expect("fixture resolves through the canonical parser graph"),
    )
    .expect("fixture expands through the canonical parser graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collects the typed-import fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the typed import");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalizes the typed-import fixture");

    let error = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect_err("unsupported import namespaces must not be collapsed into callable metadata");
    assert!(matches!(
        error,
        ModuleCoreCpsLoweringError::UnsupportedImportFact {
            namespace: CanonicalNamespace::TypeDomain,
            ..
        }
    ));
    let _ = fs::remove_dir_all(fixture_root);
}
