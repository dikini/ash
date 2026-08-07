//! TASK-2069 activation contract.
//!
//! This target is intentionally limited to TASK-2069's activation boundary.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ash_core::module_graph::ModuleKey;
use ash_core::module_interface::ModuleInterfaceDefiningIdentity;
use ash_core::module_interface::ModuleInterfaceDependency;
use ash_core::module_lowering::{ModuleImportVisibility, ResolvedModuleImport};
use ash_parser::{CanonicalExpandedModuleGraph, CanonicalModuleGraphResolver};
use ash_typeck::canonical_checked_module_finalizer::finalize_canonical_module_collection;
use ash_typeck::canonical_module_collection::collect_canonical_expanded_module_graph;
use ash_typeck::module_core_cps_lowering::{
    LoweredCheckedModuleDefinition, ModuleCoreCpsLoweringError,
    build_checked_public_module_interface_closure, lower_complete_checked_module_definition_bodies,
    lower_complete_checked_module_definition_closure, lower_complete_checked_module_entry_closure,
    lower_complete_checked_module_route_closure,
};
use ash_typeck::resolve_parsed_imports_from_collection;

#[derive(Debug, Clone, PartialEq)]
struct NormalizedLoweredDefinition {
    declaration_name: String,
    module_key: ModuleKey,
    interface_schema_version: u32,
    dependencies: Vec<ModuleInterfaceDependency>,
    imports: Vec<ResolvedModuleImport>,
    core: ash_core::core_ash_typecheck::TypedCoreProgram,
    cps: ash_core::cps::Term,
}

fn normalize_lowered_definition_closure(
    root_source: &str,
    child_source: &str,
    inline_child: bool,
    label: &str,
) -> Vec<NormalizedLoweredDefinition> {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-lowering-parity-{label}-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create parity fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, root_source).expect("write parity fixture root");
    if !inline_child {
        fs::write(fixture_root.join("src/api.ash"), child_source)
            .expect("write parity fixture child");
    }

    let root = ModuleKey::root("app").expect("parity fixture root key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("parity fixture resolves through the canonical parser graph"),
    )
    .expect("parity fixture expands through the canonical parser graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collects the parity fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the parity fixture imports");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalizes the parity fixture");
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("TASK-2069 lowers both file and inline parity fixtures");

    let normalized = lowered
        .into_iter()
        .map(|definition| NormalizedLoweredDefinition {
            declaration_name: definition.declaration_name().to_owned(),
            module_key: definition.core().module_artifact().key().clone(),
            interface_schema_version: definition.core().interface_schema_version(),
            dependencies: definition.core().dependencies().to_vec(),
            imports: definition.core().imports().to_vec(),
            core: definition.core().checked_core_program().clone(),
            cps: definition.cps().cps_program().clone(),
        })
        .collect();
    let _ = fs::remove_dir_all(fixture_root);
    normalized
}

#[test]
fn task_2069_activation_contract_declares_non_authorizing_handoff() {
    let task_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../docs/plan/tasks/TASK-2069-complete-module-lowering-and-engine-transport-fencing.md",
    );
    let task = fs::read_to_string(&task_path).expect("TASK-2069 task file exists");

    assert!(task.contains("**Status:** Complete for the frozen callable-module completion domain"));
    assert!(task.contains(
        "**Semantic coverage map:** [TASK-2069 record](../SEMANTIC-RULE-COVERAGE.md#task-2069-complete-module-lowering-and-engine-transport-fencing)"
    ));

    for evidence_id in [
        "TEST-MOD-REAL-005-FULL-DEFINITION-BODY-LOWERING",
        "TEST-MOD-REAL-005-FULL-DEFINITION-BODY-CLOSURE",
        "TEST-MOD-REAL-005-FINALIZED-BODY-AUTHORITY",
        "TEST-MOD-REAL-005-PRIMITIVE-EXPRESSION-LOWERING",
        "TEST-MOD-REAL-005-IMPORT-TRANSPORT",
        "TEST-MOD-REAL-005-REPRESENTABLE-IMPORT-TRANSPORT",
        "TEST-MOD-REAL-005-ROLE-POLICY-METADATA-STUB-TRANSPORT",
        "TEST-MOD-REAL-005-SELECTED-ENTRY-PARENT-SCOPED-REJECTION",
        "TEST-MOD-REAL-005-SINGLE-BODY-PARENT-SCOPED-REJECTION",
        "TEST-MOD-REAL-005-ENGINE-CHECKED-TRANSPORT",
        "TEST-MOD-REAL-005-BODY-LOWERING-REJECTION",
        "TEST-MOD-REAL-005-PARSEABLE-CALLABLE-LOWERING-FAIL-CLOSED",
        "TEST-MOD-REAL-005-PARSEABLE-CALLABLE-IMPORT-FAIL-CLOSED",
        "TEST-MOD-REAL-005-PROVENANCE-REWRITE",
        "TEST-MOD-REAL-005-CLOSURE-ATOMICITY",
        "TEST-MOD-REAL-005-CHECKED-INTERFACE-CLOSURE",
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
fn checked_arithmetic_body_lowers_to_core_primitive_and_cps() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-arithmetic-body-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "fn normalize() -> Int { 1 + 2 }").expect("write fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("fixture source resolves through the canonical parser graph"),
    )
    .expect("fixture source expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the arithmetic fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the arithmetic fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the arithmetic body");

    let (core, cps) =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collection,
            &expanded,
            &module_key,
            "normalize",
        )
        .expect("checked primitive arithmetic should lower to Core/CPS");

    assert!(format!("{:?}", core.checked_core_program()).contains("LetPrim"));
    assert!(format!("{:?}", core.checked_core_program()).contains("Add"));
    assert!(!format!("{:?}", cps.cps_program()).is_empty());
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_modulo_body_lowers_to_core_primitive_and_cps() {
    let fixture_root =
        std::env::temp_dir().join(format!("ash-task-2069-modulo-body-{}", std::process::id()));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "fn normalize() -> Int { 7 % 3 }").expect("write fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("fixture source resolves through the canonical parser graph"),
    )
    .expect("fixture source expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the modulo fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the modulo fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the modulo body");

    let (core, cps) =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collection,
            &expanded,
            &module_key,
            "normalize",
        )
        .expect("checked modulo arithmetic should lower to Core/CPS");

    assert!(format!("{:?}", core.checked_core_program()).contains("Rem"));
    assert!(!format!("{:?}", cps.cps_program()).is_empty());
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_parameterized_body_retains_finalized_parameter_bindings() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-parameterized-body-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "fn add(left: Int, right: Int) -> Int { left + right }",
    )
    .expect("write parameterized fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("parameterized fixture resolves through the canonical parser graph"),
    )
    .expect("parameterized fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the parameterized fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the parameterized fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the parameterized body");

    let (core, cps) =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collection,
            &expanded,
            &module_key,
            "add",
        )
        .expect("checked parameterized body should lower to Core/CPS");

    let core_debug = format!("{:?}", core.checked_core_program());
    assert!(core_debug.contains("Var(\"left\")"));
    assert!(core_debug.contains("Var(\"right\")"));
    assert!(core_debug.contains("Add"));
    assert!(format!("{:?}", cps.cps_program()).contains("Add"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_imported_callable_body_uses_resolved_signature_environment() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-imported-call-body-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "pub mod api { pub fn serve() -> Int { 7 } } use crate::api::serve as remote; fn root() -> Int { remote() + 40 }",
    )
    .expect("write imported-call fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("imported-call fixture resolves through the canonical parser graph"),
    )
    .expect("imported-call fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the imported-call fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the imported callable");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the imported call");
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("checked imported callable body should lower to Core/CPS");
    let root_definition = lowered
        .iter()
        .find(|definition| {
            definition.core().module_artifact().key() == &module_key
                && definition.declaration_name() == "root"
        })
        .expect("root definition is lowered");

    let core_debug = format!("{:?}", root_definition.core().checked_core_program());
    assert!(core_debug.contains("Call"));
    assert!(core_debug.contains("Var(\"remote\")"));
    assert!(core_debug.contains("Add"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_imported_builtin_callable_is_transportable_without_a_body() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-imported-builtin-call-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "pub mod api { pub builtin fn identity(value: Int) -> Int; } use crate::api::identity as remote; fn root() -> Int { remote(7) }",
    )
    .expect("write imported-builtin fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("imported-builtin fixture resolves through the canonical parser graph"),
    )
    .expect("imported-builtin fixture expands through the canonical parser graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collects the imported-builtin fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the imported builtin");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the imported builtin call");
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("bodyless imported builtins should remain lossless callable transport metadata");
    let root_definition = lowered
        .iter()
        .find(|definition| {
            definition.core().module_artifact().key() == &module_key
                && definition.declaration_name() == "root"
        })
        .expect("root definition is lowered");

    assert_eq!(root_definition.core().imports().len(), 1);
    assert_eq!(
        root_definition.core().imports()[0].binding().kind(),
        ash_core::module_interface::ModuleInterfaceBindingKind::Callable
    );
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_imported_handler_is_transportable_as_non_authorizing_callable_metadata() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-imported-handler-call-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "pub mod api { pub interface Clock<T> { sleep(Int) -> Int } pub type TestClock = SystemClock(Int); impl Clock<TestClock> { sleep(milliseconds) = milliseconds } pub handler identity(comp: () -> { TestClock::sleep } Int) -> Int { on comp { TestClock::sleep(ms, resume) => ms, done(value) => value } } } use crate::api::identity as remote; fn root() -> Int { 7 }",
    )
    .expect("write imported-handler fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("imported-handler fixture resolves through the canonical parser graph"),
    )
    .expect("imported-handler fixture expands through the canonical parser graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collects the imported-handler fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the imported handler");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the imported handler call");
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("imported handlers should remain lossless callable transport metadata");
    let root_definition = lowered
        .iter()
        .find(|definition| {
            definition.core().module_artifact().key() == &module_key
                && definition.declaration_name() == "root"
        })
        .expect("root definition is lowered");

    assert_eq!(root_definition.core().imports().len(), 1);
    assert_eq!(
        root_definition.core().imports()[0].binding().kind(),
        ash_core::module_interface::ModuleInterfaceBindingKind::Callable
    );
    let api = module_key.child("api").expect("api key is canonical");
    let handler_definition = lowered
        .iter()
        .find(|definition| {
            definition.core().module_artifact().key() == &api
                && definition.declaration_name() == "identity"
        })
        .expect("checked handler definition is lowered");
    assert!(format!("{:?}", handler_definition.core().checked_core_program()).contains("Handle"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_implementation_members_remain_outside_standalone_lowering() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-implementation-handler-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "pub interface Clock<T> { sleep(Int) -> Int } pub type TestClock = SystemClock(Int); pub impl Clock<TestClock> { sleep(milliseconds) = milliseconds handler logging_fs(comp: () -> { TestClock::sleep } Int) -> Int { on comp { TestClock::sleep(value, resume) => value, done(result) => result } } }",
    )
    .expect("write implementation-handler fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("implementation-handler fixture resolves through the canonical parser graph"),
    )
    .expect("implementation-handler fixture expands through the canonical parser graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collects the implementation-handler fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the implementation-handler fixture imports");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalizes parent-scoped implementation members");
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("parent-scoped implementation methods and handlers lower through checked facts");

    assert!(
        lowered.iter().all(|definition| {
            definition.declaration_name() != "sleep"
                && definition.declaration_name() != "logging_fs"
        }),
        "parent-scoped implementation members must not become standalone entries"
    );
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_local_let_arithmetic_body_lowers_to_core_and_cps() {
    let fixture_root =
        std::env::temp_dir().join(format!("ash-task-2069-let-body-{}", std::process::id()));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "fn normalize() -> Int { let x = 1; x + 2 }").expect("write fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("fixture source resolves through the canonical parser graph"),
    )
    .expect("fixture source expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the let fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the let fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the let body");

    let (core, cps) =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collection,
            &expanded,
            &module_key,
            "normalize",
        )
        .expect("checked local let arithmetic should lower to Core/CPS");

    let core_debug = format!("{:?}", core.checked_core_program());
    assert!(core_debug.contains("LetVal"));
    assert!(core_debug.contains("Add"));
    assert!(format!("{:?}", cps.cps_program()).contains("LetVal"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_nonliteral_let_initializer_lowers_to_core_and_cps() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-let-initializer-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "fn normalize() -> Int { let x = 1 + 2; x }")
        .expect("write nonliteral-let fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("nonliteral-let fixture resolves through the canonical parser graph"),
    )
    .expect("nonliteral-let fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the nonliteral-let fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the nonliteral-let fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the nonliteral-let body");

    let (core, cps) =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collection,
            &expanded,
            &module_key,
            "normalize",
        )
        .expect("nonliteral let initializer should lower to Core/CPS");

    let core_debug = format!("{:?}", core.checked_core_program());
    assert!(core_debug.contains("LetPrim"));
    assert!(core_debug.contains("LetVal"));
    assert!(core_debug.contains("Add"));
    assert!(format!("{:?}", cps.cps_program()).contains("Add"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_builtin_call_body_lowers_through_core_and_cps() {
    let fixture_root =
        std::env::temp_dir().join(format!("ash-task-2069-builtin-call-{}", std::process::id()));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "builtin fn record() -> Int; fn root() -> Int { record() }",
    )
    .expect("write builtin-call fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("builtin-call fixture resolves through the canonical parser graph"),
    )
    .expect("builtin-call fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the builtin-call fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the builtin-call fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the builtin call");

    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("builtin call should lower to Core/CPS");
    let root_definition = lowered
        .iter()
        .find(|definition| definition.declaration_name() == "root")
        .expect("builtin-call root definition is lowered");
    let core = root_definition.core();
    let cps = root_definition.cps();

    let core_debug = format!("{:?}", core.checked_core_program());
    assert!(core_debug.contains("Call"));
    assert!(core_debug.contains("Var(\"record\")"));
    assert!(format!("{:?}", cps.cps_program()).contains("record"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_local_call_initializer_lowers_to_core_let_call_and_cps() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-local-call-initializer-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "fn helper() -> Int { 1 } fn root() -> Int { let x = helper(); x + 2 }",
    )
    .expect("write local-call initializer fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("local-call initializer fixture resolves through the canonical parser graph"),
    )
    .expect("local-call initializer fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the local-call initializer fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the local-call initializer fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the local-call initializer");
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("local call initializer should lower to Core/CPS");
    let root_definition = lowered
        .iter()
        .find(|definition| definition.declaration_name() == "root")
        .expect("local-call root definition is lowered");

    let core_debug = format!("{:?}", root_definition.core().checked_core_program());
    assert!(core_debug.contains("LetCall"));
    assert!(core_debug.contains("Var(\"helper\")"));
    assert!(format!("{:?}", root_definition.cps().cps_program()).contains("helper"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_nested_callable_argument_lowers_to_core_and_cps() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-nested-call-argument-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "fn helper() -> Int { 1 } fn increment(value: Int) -> Int { value + 1 } fn root() -> Int { increment(helper()) }",
    )
    .expect("write nested-call argument fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("nested-call argument fixture resolves through the canonical parser graph"),
    )
    .expect("nested-call argument fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the nested-call argument fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the nested-call argument fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the nested-call argument");
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("nested callable argument should lower to Core/CPS");
    let root_definition = lowered
        .iter()
        .find(|definition| definition.declaration_name() == "root")
        .expect("nested-call root definition is lowered");

    let core_debug = format!("{:?}", root_definition.core().checked_core_program());
    assert!(core_debug.contains("LetCall"));
    assert!(core_debug.contains("Var(\"helper\")"));
    assert!(core_debug.contains("Var(\"increment\")"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_boolean_if_body_lowers_to_core_if_and_cps() {
    let fixture_root =
        std::env::temp_dir().join(format!("ash-task-2069-boolean-if-{}", std::process::id()));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "fn absolute(value: Int) -> Int { if value < 0 then 0 - value else value }",
    )
    .expect("write boolean-if fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("boolean-if fixture resolves through the canonical parser graph"),
    )
    .expect("boolean-if fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the boolean-if fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the boolean-if fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the boolean-if body");

    let (core, cps) =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collection,
            &expanded,
            &module_key,
            "absolute",
        )
        .expect("boolean if should lower to Core/CPS");

    let core_debug = format!("{:?}", core.checked_core_program());
    assert!(core_debug.contains("If"));
    assert!(core_debug.contains("Lt"));
    assert!(format!("{:?}", cps.cps_program()).contains("Lt"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_boolean_if_let_body_lowers_to_core_if_and_cps() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-boolean-if-let-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "fn selected(value: Bool) -> Int { if let true = value then { 1 } else { 0 } } fn selected_false(value: Bool) -> Int { if let false = value then { 1 } else { 0 } }",
    )
    .expect("write boolean-if-let fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("boolean-if-let fixture resolves through the canonical parser graph"),
    )
    .expect("boolean-if-let fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the boolean-if-let fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the boolean-if-let fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the boolean-if-let body");

    let (core, cps) =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collection,
            &expanded,
            &module_key,
            "selected",
        )
        .expect("boolean if-let should lower to Core/CPS");

    let core_debug = format!("{:?}", core.checked_core_program());
    assert!(core_debug.contains("If"));
    assert!(format!("{:?}", cps.cps_program()).contains("If"));

    let (false_core, _) =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collection,
            &expanded,
            &module_key,
            "selected_false",
        )
        .expect("false boolean if-let should lower to Core/CPS");
    assert!(format!("{:?}", false_core.checked_core_program()).contains("Not"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_short_circuit_boolean_body_lowers_to_core_if_and_cps() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-short-circuit-boolean-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create short-circuit fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "fn both(value: Bool) -> Bool { value && true } fn either(value: Bool) -> Bool { value || false } fn consume(value: Bool) -> Bool { value } fn called_and() -> Bool { both(true) && false } fn called_or() -> Bool { either(false) || true } fn argument_short_circuit() -> Bool { consume(true && false) } fn argument_short_circuit_call() -> Bool { consume(both(true) && false) } fn nested_argument_short_circuit() -> Bool { consume(consume(true && false)) } fn record_short_circuit() -> Bool { let value = { ready: true && false }; value.ready } fn nested_short_circuit_boolean() -> Bool { (true && false) || true } fn bound(value: Bool) -> Bool { let ready = value && true; ready } fn bound_call() -> Bool { let ready = both(true) && false; ready } fn condition() -> Bool { true } fn conditional() -> Int { if condition() && true then { 1 } else { 0 } } fn conditional_let() -> Int { if let true = true && true then { 1 } else { 0 } } fn conditional_let_false() -> Int { if let false = true || false then { 1 } else { 0 } } fn match_argument() -> Int { match consume(true && false) { true => 1, false => 0 } }",
    )
    .expect("write short-circuit fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("short-circuit fixture resolves through the canonical parser graph"),
    )
    .expect("short-circuit fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the short-circuit fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the short-circuit fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the short-circuit bodies");

    for name in ["both", "either"] {
        let (core, cps) =
            ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
                &finalized,
                &collection,
                &expanded,
                &module_key,
                name,
            )
            .expect("short-circuit boolean body should lower to Core/CPS");
        assert!(format!("{:?}", core.checked_core_program()).contains("If"));
        assert!(format!("{:?}", cps.cps_program()).contains("If"));
    }

    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("the complete short-circuit closure should lower local callable signatures");
    for name in [
        "called_and",
        "called_or",
        "argument_short_circuit",
        "argument_short_circuit_call",
        "nested_argument_short_circuit",
        "record_short_circuit",
        "nested_short_circuit_boolean",
        "bound_call",
        "conditional_let",
        "conditional_let_false",
        "match_argument",
    ] {
        let definition = lowered
            .iter()
            .find(|definition| definition.declaration_name() == name)
            .expect("call-bearing short-circuit definition is retained in the closure");
        assert!(format!("{:?}", definition.core().checked_core_program()).contains("If"));
        assert!(format!("{:?}", definition.cps().cps_program()).contains("If"));
    }

    let (core, cps) =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collection,
            &expanded,
            &module_key,
            "bound",
        )
        .expect("short-circuit let initializer should lower to Core/CPS");
    let core_debug = format!("{:?}", core.checked_core_program());
    assert!(core_debug.contains("If"));
    assert!(core_debug.contains("ready"));
    assert!(format!("{:?}", cps.cps_program()).contains("If"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_block_expression_discards_intermediate_values_without_binding_collisions() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-block-expression-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "fn root() -> Int { 1; 2; 3 }").expect("write block-expression fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("block-expression fixture resolves through the canonical parser graph"),
    )
    .expect("block-expression fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the block-expression fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the block-expression fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the block expression");

    let (core, cps) =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collection,
            &expanded,
            &module_key,
            "root",
        )
        .expect("block expression should lower to Core/CPS");

    let core_debug = format!("{:?}", core.checked_core_program());
    assert!(core_debug.matches("LetVal").count() >= 2);
    assert!(core_debug.contains("LitInt(3)"));
    assert!(!format!("{:?}", cps.cps_program()).is_empty());
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_record_field_projection_lowers_to_core_and_cps() {
    let fixture_root =
        std::env::temp_dir().join(format!("ash-task-2069-record-field-{}", std::process::id()));
    fs::create_dir_all(fixture_root.join("src")).expect("create record-field fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "fn project() -> Int { let person = { age: 41 }; let age = person.age; age }",
    )
    .expect("write record-field fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("record-field fixture resolves through the canonical parser graph"),
    )
    .expect("record-field fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the record-field fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the record-field fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the record-field body");

    let (core, cps) =
        ash_typeck::module_core_cps_lowering::lower_complete_checked_module_definition_bodies(
            &finalized,
            &collection,
            &expanded,
            &module_key,
            "project",
        )
        .expect("record field projection should lower to Core/CPS");

    let core_debug = format!("{:?}", core.checked_core_program());
    assert!(core_debug.contains("Record"));
    assert!(core_debug.contains("RecordGet(\"age\")"));
    assert!(format!("{:?}", cps.cps_program()).contains("RecordGet"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_record_field_call_lowers_to_core_and_cps() {
    let fixture_root =
        std::env::temp_dir().join(format!("ash-task-2069-record-call-{}", std::process::id()));
    fs::create_dir_all(fixture_root.join("src")).expect("create record-call fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "fn helper() -> Int { 41 } fn project() -> Int { let person = { age: helper() }; let age = person.age; age }",
    )
    .expect("write record-call fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("record-call fixture resolves through the canonical parser graph"),
    )
    .expect("record-call fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the record-call fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the record-call fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the record-call body");

    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("record field call should lower to Core/CPS");
    let project = lowered
        .iter()
        .find(|definition| definition.declaration_name() == "project")
        .expect("record-call project definition is lowered");

    let core_debug = format!("{:?}", project.core().checked_core_program());
    assert!(core_debug.contains("LetCall"));
    assert!(core_debug.contains("Record"));
    assert!(core_debug.contains("RecordGet(\"age\")"));
    assert!(format!("{:?}", project.cps().cps_program()).contains("RecordGet"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_nested_record_field_call_lowers_to_core_and_cps() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-nested-record-call-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src"))
        .expect("create nested record-call fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "fn helper() -> Int { 41 } fn project() -> Int { let person = { inner: { age: helper() } }; let age = person.inner.age; age }",
    )
    .expect("write nested record-call fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("nested record-call fixture resolves through the canonical parser graph"),
    )
    .expect("nested record-call fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the nested record-call fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the nested record-call fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the nested record-call body");

    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("nested record field call should lower to Core/CPS");
    let project = lowered
        .iter()
        .find(|definition| definition.declaration_name() == "project")
        .expect("nested record-call project definition is lowered");

    let core_debug = format!("{:?}", project.core().checked_core_program());
    assert!(core_debug.contains("LetCall"));
    assert!(core_debug.matches("Record").count() >= 2);
    assert!(core_debug.contains("RecordGet(\"inner\")"));
    assert!(core_debug.contains("RecordGet(\"age\")"));
    assert!(format!("{:?}", project.cps().cps_program()).contains("RecordGet"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_record_field_expression_with_call_lowers_to_core_and_cps() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-record-field-expression-call-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src"))
        .expect("create record-field-expression-call fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "fn helper() -> Int { 40 } fn project() -> Int { let person = { age: helper() + 1 }; let age = person.age; age }",
    )
    .expect("write record-field-expression-call fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect(
                "record-field-expression-call fixture resolves through the canonical parser graph",
            ),
    )
    .expect("record-field-expression-call fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the record-field-expression-call fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the record-field-expression-call fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the record-field-expression-call body");

    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("record field expression with call should lower to Core/CPS");
    let project = lowered
        .iter()
        .find(|definition| definition.declaration_name() == "project")
        .expect("record-field-expression-call project definition is lowered");

    let core_debug = format!("{:?}", project.core().checked_core_program());
    assert!(core_debug.contains("LetCall"));
    assert!(core_debug.contains("Add"));
    assert!(core_debug.contains("RecordGet(\"age\")"));
    assert!(format!("{:?}", project.cps().cps_program()).contains("RecordGet"));
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_match_call_scrutinee_lowers_to_core_and_cps() {
    let fixture_root =
        std::env::temp_dir().join(format!("ash-task-2069-match-call-{}", std::process::id()));
    fs::create_dir_all(fixture_root.join("src")).expect("create match-call fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "fn ready() -> Bool { true } fn project() -> Int { match ready() { true => 1, false => 0 } }",
    )
    .expect("write match-call fixture");

    let module_key = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(module_key.clone(), &root_path)
            .expect("match-call fixture resolves through the canonical parser graph"),
    )
    .expect("match-call fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the match-call fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the match-call fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization checks the match-call body");

    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("call scrutinee should lower to Core/CPS");
    let project = lowered
        .iter()
        .find(|definition| definition.declaration_name() == "project")
        .expect("match-call project definition is lowered");

    let core_debug = format!("{:?}", project.core().checked_core_program());
    assert!(core_debug.contains("LetCall"));
    assert!(core_debug.contains("If"));
    assert!(core_debug.contains("Var(\"ready\")"));
    assert!(format!("{:?}", project.cps().cps_program()).contains("ready"));
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
    fs::write(
        &root_path,
        "fn normalize() -> Int { match 1 { 1 => 1, _ => 0 } }",
    )
    .expect("write fixture");

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
fn selected_checked_module_entry_closure_has_one_artifact_per_module() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-selected-entry-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "pub mod api; fn root() -> Int { 1 }").expect("write fixture root");
    fs::write(
        fixture_root.join("src/api.ash"),
        "pub fn serve() -> Int { 2 } fn helper() -> Int { 3 }",
    )
    .expect("write fixture child");

    let root = ModuleKey::root("app").expect("fixture module key is canonical");
    let child = root.child("api").expect("fixture child key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("fixture source resolves through the canonical parser graph"),
    )
    .expect("fixture source expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the selected-entry fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the selected-entry fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization accepts the selected-entry fixture");

    let selected_entries = BTreeMap::from([
        (root.clone(), "root".to_owned()),
        (child, "serve".to_owned()),
    ]);
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &selected_entries,
    )
    .expect("selected entries lower into one artifact per module");

    assert_eq!(lowered.len(), 2);
    assert_eq!(
        lowered
            .iter()
            .map(|definition| definition.core().module_artifact().key())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        lowered.len(),
        "selected entry transport must not duplicate canonical module keys"
    );
    assert_eq!(
        lowered
            .iter()
            .map(LoweredCheckedModuleDefinition::declaration_name)
            .collect::<Vec<_>>(),
        ["root", "serve"]
    );

    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn selected_checked_module_entry_closure_rejects_parent_scoped_member() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-selected-parent-member-{}-{}",
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
        "pub interface Eq<A> { equiv(A, A) -> Bool } pub impl Eq<Int> { equiv(a, b) = a == b } fn root() -> Int { 1 }",
    )
    .expect("write fixture root");

    let root = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("fixture source resolves through the canonical parser graph"),
    )
    .expect("fixture source expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the selected-member fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the selected-member fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization accepts the selected-member fixture");

    let error = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([(root.clone(), "equiv".to_owned())]),
    )
    .expect_err("parent-scoped implementation members are not standalone entries");

    assert!(matches!(
        error,
        ModuleCoreCpsLoweringError::UnsupportedDefinition {
            module,
            name,
            kind: ash_typeck::canonical_module_collection::CanonicalDeclarationKind::Function,
        } if module == root && name == "equiv"
    ));

    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn single_checked_module_body_lowering_rejects_parent_scoped_member() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-single-parent-member-{}-{}",
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
        "pub interface Eq<A> { equiv(A, A) -> Bool } pub impl Eq<Int> { equiv(a, b) = a == b } fn root() -> Int { 1 }",
    )
    .expect("write fixture root");

    let root = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("fixture source resolves through the canonical parser graph"),
    )
    .expect("fixture source expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the single-member fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the single-member fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization accepts the single-member fixture");

    let error = lower_complete_checked_module_definition_bodies(
        &finalized,
        &collection,
        &expanded,
        &root,
        "equiv",
    )
    .expect_err("parent-scoped implementation members are not standalone bodies");

    assert!(matches!(
        error,
        ModuleCoreCpsLoweringError::UnsupportedDefinition {
            module,
            name,
            kind: ash_typeck::canonical_module_collection::CanonicalDeclarationKind::Function,
        } if module == root && name == "equiv"
    ));

    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn selected_checked_module_entry_closure_rejects_missing_module_selection() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-selected-entry-missing-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "pub mod api; fn root() -> Int { 1 }").expect("write fixture root");
    fs::write(
        fixture_root.join("src/api.ash"),
        "pub fn serve() -> Int { 2 }",
    )
    .expect("write fixture child");

    let root = ModuleKey::root("app").expect("fixture module key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("fixture source resolves through the canonical parser graph"),
    )
    .expect("fixture source expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collection accepts the selected-entry fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 import handoff accepts the selected-entry fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalization accepts the selected-entry fixture");
    let error = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([(root.clone(), "root".to_owned())]),
    )
    .expect_err("every finalized module must have an explicit selected entry");

    assert!(matches!(
        error,
        ModuleCoreCpsLoweringError::MissingSelectedEntry { module } if module == root.child("api").expect("fixture child key is canonical")
    ));

    let unexpected = root.child("forged").expect("forged child key is canonical");
    let error = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (root.child("api").unwrap(), "serve".to_owned()),
            (unexpected.clone(), "serve".to_owned()),
        ]),
    )
    .expect_err("selection must not name a module outside the checked closure");
    assert!(matches!(
        error,
        ModuleCoreCpsLoweringError::UnexpectedSelectedEntry { module } if module == unexpected
    ));

    let api = root.child("api").expect("fixture child key is canonical");
    let route_lowered = lower_complete_checked_module_route_closure(
        root.clone(),
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (api.clone(), String::new()),
        ]),
    )
    .expect("route lowering supplies a neutral carrier for an unselected structural child");
    assert_eq!(route_lowered.len(), 3);
    assert!(
        route_lowered.iter().any(|definition| {
            definition.core().module_artifact().key() == &api && !definition.is_callable_entry()
        }),
        "metadata-only structural children must remain non-callable route carriers"
    );
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
        "fn normalize() -> Int { 1 } fn broken() -> Int { match 1 { 1 => 1, _ => 0 } }",
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
    assert_eq!(client_lowered.core().interface_schema_version(), 1);
    assert_eq!(client_lowered.cps().interface_schema_version(), 1);
    assert_eq!(client_lowered.core().dependencies().len(), 1);
    assert_eq!(
        client_lowered.core().dependencies()[0].module,
        root.child("api").expect("api module key is canonical")
    );
    assert_eq!(
        client_lowered.cps().dependencies(),
        client_lowered.core().dependencies()
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
fn checked_lowering_carries_transitive_reachable_dependency_snapshot() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-transitive-dependencies-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "pub mod api; pub mod dep; pub mod client;").expect("write fixture root");
    fs::write(
        fixture_root.join("src/api.ash"),
        "use crate::dep::leaf as remote; pub fn serve() -> Int { remote() }",
    )
    .expect("write transitive provider fixture");
    fs::write(
        fixture_root.join("src/dep.ash"),
        "pub fn leaf() -> Int { 1 }",
    )
    .expect("write transitive dependency fixture");
    fs::write(
        fixture_root.join("src/client.ash"),
        "use crate::api::serve as remote; pub fn entry() -> Int { remote() }",
    )
    .expect("write importing fixture");

    let root = ModuleKey::root("app").expect("fixture root key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("fixture resolves through the canonical parser graph"),
    )
    .expect("fixture expands through the canonical parser graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collects the transitive fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the transitive fixture imports");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalizes the transitive fixture");
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("TASK-2069 lowers the transitive fixture");

    let api = root.child("api").expect("api key is canonical");
    let dep = root.child("dep").expect("dep key is canonical");
    let client = root.child("client").expect("client key is canonical");
    let client_lowered = lowered
        .iter()
        .find(|definition| definition.core().module_artifact().key() == &client)
        .expect("client definition is lowered");
    assert_eq!(
        client_lowered
            .core()
            .dependencies()
            .iter()
            .map(|dependency| dependency.module.clone())
            .collect::<Vec<_>>(),
        vec![api.clone(), dep.clone()]
    );
    assert_eq!(
        client_lowered.cps().dependencies(),
        client_lowered.core().dependencies()
    );

    let api_lowered = lowered
        .iter()
        .find(|definition| definition.core().module_artifact().key() == &api)
        .expect("api definition is lowered");
    assert_eq!(
        api_lowered
            .core()
            .dependencies()
            .iter()
            .map(|dependency| dependency.module.clone())
            .collect::<Vec<_>>(),
        vec![dep]
    );

    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn role_and_policy_imports_reach_core_as_metadata_only_stubs() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-metadata-import-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create metadata fixture");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "pub mod api; pub mod client;").expect("write metadata fixture root");
    fs::write(
        fixture_root.join("src/api.ash"),
        "pub role Reviewer { capabilities: [], obligations: [audit_log] } pub policy RateLimit { marker: Int }",
    )
    .expect("write metadata provider fixture");
    fs::write(
        fixture_root.join("src/client.ash"),
        "use crate::api::Reviewer as reviewer; use crate::api::RateLimit as rate; fn entry() -> Int { 1 }",
    )
    .expect("write metadata importing fixture");

    let root = ModuleKey::root("app").expect("metadata fixture root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root, &root_path)
            .expect("metadata fixture resolves through the canonical parser graph"),
    )
    .expect("metadata fixture expands");
    let collection =
        collect_canonical_expanded_module_graph(&expanded).expect("metadata fixture collects");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("metadata fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("metadata fixture finalizes");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("metadata namespaces remain transportable as non-authorizing stubs");
    let api = ModuleKey::root("app")
        .expect("metadata fixture root key")
        .child("api")
        .expect("metadata api key");
    let api_interface = interfaces
        .iter()
        .find(|interface| interface.artifact().key() == &api)
        .expect("api interface is projected");
    for name in ["Reviewer", "RateLimit"] {
        let binding = api_interface
            .bindings()
            .iter()
            .find(|binding| binding.visible_name() == name)
            .expect("metadata declaration is retained in the provider interface");
        assert!(!binding.is_runtime_callable());
    }

    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("metadata-only imports do not block unrelated callable lowering");
    let entry = lowered
        .iter()
        .find(|definition| definition.declaration_name() == "entry")
        .expect("client entry is lowered");
    assert_eq!(entry.core().imports().len(), 2);
    assert!(
        entry
            .core()
            .imports()
            .iter()
            .all(|import| !import.binding().is_runtime_callable())
    );
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn representable_type_import_namespace_reaches_core_cps_metadata() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-type-import-{}-{}",
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

    let root = ModuleKey::root("app").expect("fixture root key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("fixture resolves through the canonical parser graph"),
    )
    .expect("fixture expands through the canonical parser graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collects the type-import fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the type import");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalizes the type-import fixture");
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("representable type imports should reach Core/CPS metadata");
    let client = root.child("client").expect("client key is canonical");
    let client_lowered = lowered
        .iter()
        .find(|definition| definition.core().module_artifact().key() == &client)
        .expect("client definition is lowered");
    assert_eq!(client_lowered.core().imports().len(), 1);
    assert_eq!(
        client_lowered.core().imports()[0].binding().kind(),
        ash_core::module_interface::ModuleInterfaceBindingKind::Type
    );
    assert_eq!(
        client_lowered.cps().imports(),
        client_lowered.core().imports()
    );
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn qualified_constructor_import_reaches_core_cps_metadata() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-qualified-constructor-import-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create constructor fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "pub mod api; pub mod client;").expect("write constructor fixture root");
    fs::write(
        fixture_root.join("src/api.ash"),
        "pub type Tree = Leaf | Branch(Tree);",
    )
    .expect("write constructor provider fixture");
    fs::write(
        fixture_root.join("src/client.ash"),
        "use crate::api::Tree::Branch; fn entry() -> Int { 1 }",
    )
    .expect("write constructor client fixture");

    let root = ModuleKey::root("app").expect("fixture root key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("constructor fixture resolves through the canonical parser graph"),
    )
    .expect("constructor fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collects the constructor fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the qualified constructor import");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalizes the constructor fixture");
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("qualified constructor imports remain lossless Core/CPS metadata");

    let client = root.child("client").expect("client key is canonical");
    let client_lowered = lowered
        .iter()
        .find(|definition| definition.core().module_artifact().key() == &client)
        .expect("client definition is lowered");
    assert_eq!(client_lowered.core().imports().len(), 1);
    assert_eq!(
        client_lowered.core().imports()[0].binding().kind(),
        ash_core::module_interface::ModuleInterfaceBindingKind::Constructor
    );
    assert_eq!(
        client_lowered.cps().imports(),
        client_lowered.core().imports()
    );
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn parent_scoped_visibility_import_reaches_core_cps_metadata() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-parent-scoped-import-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(fixture_root.join("src"))
        .expect("create parent-scoped import fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "pub mod parent { pub mod child { pub(super) fn value() -> Int { 7 } } use crate::parent::child::value as local; pub fn run() -> Int { local() } } fn main() -> Int { 1 }",
    )
        .expect("write parent-scoped import fixture root");

    let root = ModuleKey::root("app").expect("parent-scoped fixture root key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("parent-scoped fixture resolves through the canonical parser graph"),
    )
    .expect("parent-scoped fixture expands through the canonical expanded graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collects the parent-scoped fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the parent-scoped import");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalizes the parent-scoped fixture");
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("parent-scoped imports remain lossless Core/CPS metadata");

    let parent = root.child("parent").expect("parent key is canonical");
    let parent_lowered = lowered
        .iter()
        .find(|definition| definition.core().module_artifact().key() == &parent)
        .expect("parent definition is lowered");
    let imported = parent_lowered
        .core()
        .imports()
        .iter()
        .find(|import| import.local_name() == "local")
        .expect("parent-scoped import is retained in Core metadata");
    assert_eq!(
        imported.binding().visibility(),
        ash_core::Visibility::Private
    );
    match imported.binding().defining_identity() {
        ModuleInterfaceDefiningIdentity::Declaration(identity) => assert_eq!(
            identity.module,
            root.child("parent")
                .unwrap()
                .child("child")
                .expect("child module key is canonical")
        ),
        other => panic!("expected parent-scoped declaration identity, got {other:?}"),
    }
    assert_eq!(
        parent_lowered.cps().imports(),
        parent_lowered.core().imports()
    );
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn public_callable_reexport_chain_reaches_core_cps_metadata() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-callable-reexport-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "pub mod api; pub mod facade; pub mod client;")
        .expect("write fixture root");
    fs::write(
        fixture_root.join("src/api.ash"),
        "pub fn serve() -> Int { 2 }",
    )
    .expect("write callable provider fixture");
    fs::write(
        fixture_root.join("src/facade.ash"),
        "pub use crate::api::serve as remote; fn facade_entry() -> Int { 1 }",
    )
    .expect("write callable facade fixture");
    fs::write(
        fixture_root.join("src/client.ash"),
        "use crate::facade::remote; fn entry() -> Int { 1 }",
    )
    .expect("write callable client fixture");

    let root = ModuleKey::root("app").expect("fixture root key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("fixture resolves through the canonical parser graph"),
    )
    .expect("fixture expands through the canonical parser graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collects the callable re-export fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the callable re-export fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalizes the callable re-export fixture");

    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("public callable re-exports should remain lossless in Core/CPS transport");
    let facade = root.child("facade").expect("facade key is canonical");
    let facade_lowered = lowered
        .iter()
        .find(|definition| definition.core().module_artifact().key() == &facade)
        .expect("facade lowering is present");
    assert_eq!(facade_lowered.core().imports().len(), 1);
    assert_eq!(facade_lowered.core().imports()[0].local_name(), "remote");
    assert_eq!(
        facade_lowered.core().imports()[0].binding().kind(),
        ash_core::module_interface::ModuleInterfaceBindingKind::Callable
    );
    assert_eq!(
        facade_lowered.core().imports()[0].binding().visibility(),
        ash_core::Visibility::Public
    );
    assert_eq!(
        facade_lowered.cps().imports(),
        facade_lowered.core().imports()
    );

    let client = root.child("client").expect("client key is canonical");
    let client_lowered = lowered
        .iter()
        .find(|definition| definition.core().module_artifact().key() == &client)
        .expect("client lowering is present");
    assert_eq!(client_lowered.core().imports().len(), 1);
    assert_eq!(
        client_lowered.core().imports()[0]
            .binding()
            .defining_identity(),
        facade_lowered.core().imports()[0]
            .binding()
            .defining_identity()
    );

    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn reexport_import_retains_the_reexport_visibility_in_core_cps_metadata() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-reexport-visibility-{}-{}",
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
        "pub mod parent { pub mod api { pub fn serve() -> Int { 2 } } pub mod facade { pub(super) use crate::parent::api::serve as remote; pub fn entry() -> Int { 1 } } } fn main() -> Int { 1 }",
    )
    .expect("write re-export visibility fixture");

    let root = ModuleKey::root("app").expect("fixture root key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("re-export visibility fixture resolves through the canonical parser graph"),
    )
    .expect("re-export visibility fixture expands through the canonical parser graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collects the re-export visibility fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the re-export visibility fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalizes the re-export visibility fixture");
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("TASK-2069 lowers the re-export visibility fixture");

    let facade = root
        .child("parent")
        .expect("parent key is canonical")
        .child("facade")
        .expect("facade key is canonical");
    let facade_lowered = lowered
        .iter()
        .find(|definition| definition.core().module_artifact().key() == &facade)
        .expect("facade callable is lowered");
    let imported = facade_lowered
        .core()
        .imports()
        .iter()
        .find(|import| import.local_name() == "remote")
        .expect("re-export import is retained in Core metadata");
    assert_eq!(
        imported.visibility(),
        &ModuleImportVisibility::Super { levels: 1 },
        "Core/CPS transport must retain the re-export visibility, not only the provider declaration visibility"
    );
    assert_eq!(
        facade_lowered.cps().imports(),
        facade_lowered.core().imports()
    );

    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn structural_child_reexport_chain_preserves_defining_callable_identity() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-structural-reexport-chain-{}-{}",
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
        "pub mod api; pub mod facade; pub mod client; fn root() -> Int { 1 }",
    )
    .expect("write structural re-export root");
    fs::write(fixture_root.join("src/api.ash"), "pub mod nested;")
        .expect("write structural provider facade");
    fs::write(
        fixture_root.join("src/nested.ash"),
        "pub fn serve() -> Int { 2 }",
    )
    .expect("write structural callable provider");
    fs::write(
        fixture_root.join("src/facade.ash"),
        "pub use crate::api::nested::serve as forwarded;",
    )
    .expect("write structural re-export facade");
    fs::write(
        fixture_root.join("src/client.ash"),
        "use crate::facade::forwarded; fn entry() -> Int { forwarded() }",
    )
    .expect("write structural re-export client");

    let root = ModuleKey::root("app").expect("fixture root key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("fixture resolves through the canonical parser graph"),
    )
    .expect("fixture expands through the canonical parser graph");
    let collection =
        collect_canonical_expanded_module_graph(&expanded).expect("fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("fixture imports resolve through structural module interfaces");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("fixture finalization succeeds");
    let nested = root
        .child("api")
        .expect("api key is canonical")
        .child("nested")
        .expect("nested key is canonical");
    let facade = root.child("facade").expect("facade key is canonical");
    let client = root.child("client").expect("client key is canonical");
    let lowered = lower_complete_checked_module_route_closure(
        root.clone(),
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (
                root.child("api").expect("api key is canonical"),
                String::new(),
            ),
            (nested.clone(), "serve".to_owned()),
            (facade.clone(), String::new()),
            (client.clone(), "entry".to_owned()),
        ]),
    )
    .expect("fixture lowering succeeds");

    let client_lowered = lowered
        .iter()
        .find(|definition| definition.core().module_artifact().key() == &client)
        .expect("client callable is lowered");
    let client_import = client_lowered
        .core()
        .imports()
        .iter()
        .find(|import| import.local_name() == "forwarded")
        .expect("client retains its imported re-export");
    assert_eq!(
        client_import.binding().defining_identity(),
        &ModuleInterfaceDefiningIdentity::Declaration(
            ash_core::module_interface::ModuleInterfaceDeclarationIdentity::new(
                nested.clone(),
                "serve",
                ash_core::module_interface::ModuleInterfaceBindingKind::Callable,
            )
        )
    );

    let facade_lowered = lowered
        .iter()
        .find(|definition| definition.core().module_artifact().key() == &facade)
        .expect("facade metadata carrier is lowered");
    let facade_import = facade_lowered
        .core()
        .imports()
        .iter()
        .find(|import| import.local_name() == "forwarded")
        .expect("facade retains its structural re-export import");
    assert_eq!(
        facade_import.binding().defining_identity(),
        client_import.binding().defining_identity(),
        "both import hops retain the original callable identity"
    );
    assert_eq!(
        facade_import.visibility(),
        client_import.visibility(),
        "re-export and downstream import retain the checked visibility carrier"
    );
    assert_eq!(
        facade_lowered.cps().imports(),
        facade_lowered.core().imports()
    );
    assert_eq!(
        client_lowered.cps().imports(),
        client_lowered.core().imports()
    );
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn public_type_function_import_reaches_core_cps_metadata() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-type-function-import-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "pub mod api; pub mod client;").expect("write fixture root");
    fs::write(
        fixture_root.join("src/api.ash"),
        "pub sealed type domain TypeList { Nil; Cons<head: Type, tail: TypeList>; }\n\
         pub type fn Identity(xs: TypeList) -> TypeList { case Identity<xs> = xs; }",
    )
    .expect("write type-function provider fixture");
    fs::write(
        fixture_root.join("src/client.ash"),
        "use crate::api::Identity; fn entry() -> Int { 1 }",
    )
    .expect("write type-function client fixture");

    let root = ModuleKey::root("app").expect("fixture root key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("fixture resolves through the canonical parser graph"),
    )
    .expect("fixture expands through the canonical parser graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collects the type-function fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the type-function import");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalizes the type-function fixture");
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("public type-function imports should remain metadata in Core/CPS transport");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("typed public exports should retain their checked identity carrier");
    let api = root.child("api").expect("api key is canonical");
    let api_interface = interfaces
        .iter()
        .find(|interface| interface.artifact().key() == &api)
        .expect("type-function provider interface is retained");
    assert!(
        api_interface
            .bindings()
            .iter()
            .any(|binding| binding.visible_name() == "TypeList")
    );
    let client = root.child("client").expect("client key is canonical");
    let client_lowered = lowered
        .iter()
        .find(|definition| definition.core().module_artifact().key() == &client)
        .expect("client lowering is present");
    assert_eq!(client_lowered.core().imports().len(), 1);
    assert!(matches!(
        client_lowered.core().imports()[0].binding().kind(),
        ash_core::module_interface::ModuleInterfaceBindingKind::TypeFunction
    ));
    assert_eq!(
        client_lowered.cps().imports(),
        client_lowered.core().imports()
    );
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn bodyless_constructor_declarations_do_not_require_body_lowering() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-constructor-import-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create fixture source directory");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "pub mod api; pub mod client;").expect("write fixture root");
    fs::write(
        fixture_root.join("src/api.ash"),
        "pub newtype UserId = UserId(Int);",
    )
    .expect("write constructor provider fixture");
    fs::write(
        fixture_root.join("src/client.ash"),
        "fn entry() -> Int { 1 }",
    )
    .expect("write constructor sibling fixture");

    let root = ModuleKey::root("app").expect("fixture root key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("fixture resolves through the canonical parser graph"),
    )
    .expect("fixture expands through the canonical parser graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collects the bodyless-constructor fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the bodyless-constructor fixture");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalizes the bodyless-constructor fixture");
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("bodyless constructor declarations should not require body lowering");
    let client = root.child("client").expect("client key is canonical");
    let client_lowered = lowered
        .iter()
        .find(|definition| definition.core().module_artifact().key() == &client)
        .expect("client definition is lowered");
    assert!(client_lowered.core().imports().is_empty());
    assert_eq!(
        client_lowered.cps().imports(),
        client_lowered.core().imports()
    );
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_public_interface_closure_projects_finalized_exports_and_import_dependencies() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-interface-closure-{}-{}",
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
        "pub mod api; pub mod client; pub fn root() -> Int { 1 }",
    )
    .expect("write fixture root");
    fs::write(
        fixture_root.join("src/api.ash"),
        "pub fn serve() -> Int { 2 }",
    )
    .expect("write fixture provider");
    fs::write(
        fixture_root.join("src/client.ash"),
        "use crate::api::serve as remote; pub fn entry() -> Int { 3 }",
    )
    .expect("write fixture client");

    let root = ModuleKey::root("app").expect("fixture root key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("fixture resolves through the canonical parser graph"),
    )
    .expect("fixture expands through the canonical parser graph");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("TASK-2075 collects the interface fixture");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("TASK-2072 resolves the interface fixture import");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("TASK-2073 finalizes the interface fixture");

    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("finalized exports should project into checked public interfaces");

    assert_eq!(interfaces.len(), 3);
    let client = root.child("client").expect("client key is canonical");
    let client_interface = interfaces
        .iter()
        .find(|interface| interface.artifact().key() == &client)
        .expect("client interface is retained");
    assert_eq!(client_interface.dependencies().len(), 1);
    assert_eq!(
        client_interface.dependencies()[0].module,
        root.child("api").unwrap()
    );
    assert_eq!(client_interface.bindings()[0].visible_name(), "entry");

    let root_interface = interfaces
        .iter()
        .find(|interface| interface.artifact().key() == &root)
        .expect("root interface is retained");
    assert_eq!(
        root_interface
            .bindings()
            .iter()
            .map(|binding| binding.visible_name())
            .collect::<Vec<_>>(),
        vec!["api", "client", "root"]
    );

    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn checked_public_interface_closure_transports_public_impl_as_non_authorizing_metadata() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-public-implementation-interface-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create implementation fixture");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "pub interface Eq<A> { equiv(A, A) -> Bool } pub impl Eq<Int> { equiv(a, b) = a == b } fn root() -> Int { 1 }",
    )
    .expect("write implementation fixture");

    let root = ModuleKey::root("app").expect("implementation fixture root key is canonical");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("implementation fixture resolves through the canonical parser graph"),
    )
    .expect("implementation fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("implementation fixture collects");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("implementation fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("implementation fixture finalizes");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("public implementation summary projects into the checked interface carrier");
    let interface = interfaces
        .iter()
        .find(|interface| interface.artifact().key() == &root)
        .expect("root implementation interface is retained");
    let binding = interface
        .bindings()
        .iter()
        .find(|binding| {
            binding.visible_name() == "Eq"
                && binding.kind()
                    == ash_core::module_interface::ModuleInterfaceBindingKind::Implementation
        })
        .expect("public implementation metadata is retained");
    assert_eq!(
        binding.kind(),
        ash_core::module_interface::ModuleInterfaceBindingKind::Implementation
    );
    assert!(!binding.is_runtime_callable());
    let _ = fs::remove_dir_all(fixture_root);
}

/// `TEST-MOD-REAL-005-FILE-INLINE-LOWERING-PARITY`: source acquisition may
/// change diagnostic origins, but it must not change normalized Core/CPS facts.
#[test]
fn file_and_inline_checked_module_lowering_have_equal_normalized_closures() {
    let file = normalize_lowered_definition_closure(
        "pub mod api; fn root() -> Int { 1 }",
        "fn serve() -> Int { 2 }",
        false,
        "file",
    );
    let inline = normalize_lowered_definition_closure(
        "pub mod api { fn serve() -> Int { 2 } } fn root() -> Int { 1 }",
        "fn serve() -> Int { 2 }",
        true,
        "inline",
    );

    assert_eq!(file, inline);
}

/// Declaration order may affect source spans, but it must not alter the
/// canonical identity-keyed Core/CPS facts for the same definitions.
#[test]
fn declaration_order_variants_have_equal_normalized_lowered_closures() {
    let first = normalize_lowered_definition_closure(
        "crate app; fn helper() -> Int { 40 } fn main() -> Int { helper() + 1 }",
        "",
        true,
        "order-first",
    );
    let second = normalize_lowered_definition_closure(
        "crate app; fn main() -> Int { helper() + 1 } fn helper() -> Int { 40 }",
        "",
        true,
        "order-second",
    );

    let mut first = first;
    let mut second = second;
    first.sort_by(|left, right| {
        (&left.module_key, &left.declaration_name)
            .cmp(&(&right.module_key, &right.declaration_name))
    });
    second.sort_by(|left, right| {
        (&left.module_key, &left.declaration_name)
            .cmp(&(&right.module_key, &right.declaration_name))
    });

    assert_eq!(first, second);
}
