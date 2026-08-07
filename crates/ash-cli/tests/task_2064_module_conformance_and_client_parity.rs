//! TASK-2064 conformance and client-parity evidence.
//!
//! The fixture deliberately enters through the Engine-owned linked closure
//! boundary. The CLI and daemon adapters receive the same opaque request and
//! cannot parse, lower, install frames, or select a fallback evaluator.

use std::collections::BTreeMap;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_cli::commands::{
    daemon::submit_admitted_program as submit_daemon_admitted_program,
    run::submit_admitted_program as submit_cli_admitted_program,
};
use ash_core::Value;
use ash_core::Visibility;
use ash_core::core_ash::{CoreAtom, CoreExpr};
use ash_core::core_ash_typecheck::{CoreTypeCheckEnv, TypedCoreProgram, type_check_core_program};
use ash_core::core_ash_validate::{RawCoreProgram, validate_core_program};
use ash_core::cps::{Atom as CpsAtom, ContRef, EffectRow, Term as CpsTerm};
use ash_core::module_graph::{ModuleArtifact, ModuleArtifactOrigin, ModuleKey};
use ash_core::module_interface::{
    ModuleInterfaceBinding, ModuleInterfaceDependency, PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
    PublicModuleInterface,
};
use ash_core::module_lowering::{ModuleCoreArtifact, ModuleCpsArtifact};
use ash_core::semantic_summary::{SourceAnchor, SourceOrigin};
use ash_engine::{
    CanonicalTerminalEnvelopeV1, Engine, LinkedModuleArtifactInput, LinkedModuleClosure,
    LinkedModuleClosureBuildError, linked_module_closure_from_checked_definition_lowering,
    linked_module_closure_from_checked_entry_lowering,
};
use ash_parser::{CanonicalExpandedModuleGraph, CanonicalModuleGraphResolver};
use ash_typeck::canonical_checked_module_finalizer::finalize_canonical_module_collection;
use ash_typeck::canonical_module_collection::collect_canonical_expanded_module_graph;
use ash_typeck::module_core_cps_lowering::{
    build_checked_public_module_interface_closure, lower_checked_metadata_only_module,
    lower_complete_checked_module_definition_closure, lower_complete_checked_module_entry_closure,
};
use ash_typeck::resolve_parsed_imports_from_collection;
use proptest::prelude::*;

static NEXT_SOURCE_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

fn checked_core_literal(value: i64) -> TypedCoreProgram {
    let validated =
        validate_core_program(RawCoreProgram::new(CoreExpr::Atom(CoreAtom::LitInt(value))))
            .expect("literal Core validates");
    type_check_core_program(validated, &CoreTypeCheckEnv::default())
        .expect("literal Core type-checks")
}

fn source_anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "TASK-2064 conformance fixture".to_owned(),
        },
        None,
        label,
    )
}

fn literal_cps(value: i64) -> CpsTerm {
    CpsTerm::Jump {
        cont: ContRef::Label("__answer".to_owned()),
        arg: CpsAtom::Int(value),
        row: EffectRow::default(),
    }
}

fn linked_inputs(
    child_origin: ModuleArtifactOrigin,
) -> (
    ModuleKey,
    LinkedModuleArtifactInput,
    LinkedModuleArtifactInput,
) {
    let root = ModuleKey::root("task_2064").expect("fixture root key");
    let child = root.child("shared").expect("fixture child key");
    let root_artifact = ModuleArtifact::new(
        root.clone(),
        ModuleArtifactOrigin::File("fixtures/task_2064.ash".to_owned()),
        None,
        vec![child.clone()],
    )
    .expect("root artifact is canonical");
    let child_artifact =
        ModuleArtifact::new(child.clone(), child_origin, Some(root.clone()), Vec::new())
            .expect("child artifact is canonical");
    let dependency =
        ModuleInterfaceDependency::new(child.clone(), PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION);
    let root_interface = PublicModuleInterface::with_dependencies(
        root_artifact.clone(),
        vec![ModuleInterfaceBinding::child(
            "shared",
            child.clone(),
            Visibility::Public,
            child_artifact.origin().clone(),
        )],
        vec![dependency.clone()],
        None,
    )
    .expect("root interface is export-closed");
    let child_interface = PublicModuleInterface::new(child_artifact.clone(), Vec::new())
        .expect("child interface is export-closed");
    let root_core = ModuleCoreArtifact::new_with_interface_metadata(
        root_artifact,
        Vec::new(),
        PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
        vec![dependency],
        checked_core_literal(42),
    );
    let child_core = ModuleCoreArtifact::new(child_artifact, Vec::new(), checked_core_literal(0));
    let root_input = LinkedModuleArtifactInput::with_entry_metadata(
        root_interface,
        root_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&root_core, literal_cps(42)),
        source_anchor("task_2064-root"),
        "root",
        Vec::<String>::new(),
    );
    let child_input = LinkedModuleArtifactInput::new(
        child_interface,
        child_core.clone(),
        ModuleCpsArtifact::from_core_artifact(&child_core, literal_cps(0)),
        source_anchor("task_2064-child"),
    );
    (root, root_input, child_input)
}

fn linked_closure(child_origin: ModuleArtifactOrigin) -> LinkedModuleClosure {
    let (root, root_input, child_input) = linked_inputs(child_origin);
    LinkedModuleClosure::new(root, vec![child_input, root_input])
}

fn source_linked_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_closure_with_root_body(inline_child, "42")
}

fn source_linked_ordinary_root_closure() -> LinkedModuleClosure {
    let fixture = tempfile::tempdir().expect("create ordinary-root parity fixture");
    let path = fixture.path().join("main.ash");
    let source = "crate app; fn main() -> Int { 42 }";
    fs::write(&path, source).expect("write ordinary-root parity source");
    let engine = Engine::new().build().expect("Engine builds");
    engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect("ordinary root uses canonical source route")
        .expect("ordinary root produces checked closure")
}

fn source_linked_modulo_root_closure() -> LinkedModuleClosure {
    let fixture = tempfile::tempdir().expect("create modulo parity fixture");
    let path = fixture.path().join("main.ash");
    let source = "crate app; fn main() -> Int { 7 % 3 }";
    fs::write(&path, source).expect("write modulo parity source");
    let engine = Engine::new().build().expect("Engine builds");
    engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect("modulo root uses canonical source route")
        .expect("modulo root produces checked closure")
}

fn source_linked_record_field_call_root_closure() -> LinkedModuleClosure {
    let fixture = tempfile::tempdir().expect("create record-field-call parity fixture");
    let path = fixture.path().join("main.ash");
    let source = "crate app; fn helper() -> Int { 41 } fn main() -> Int { let person = { age: helper() }; person.age }";
    fs::write(&path, source).expect("write record-field-call parity source");
    let engine = Engine::new().build().expect("Engine builds");
    engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect("record field call root uses canonical source route")
        .expect("record field call root produces checked closure")
}

fn source_linked_nested_record_field_call_root_closure() -> LinkedModuleClosure {
    let fixture = tempfile::tempdir().expect("create nested record-field-call parity fixture");
    let path = fixture.path().join("main.ash");
    let source = "crate app; fn helper() -> Int { 41 } fn main() -> Int { let person = { inner: { age: helper() } }; person.inner.age }";
    fs::write(&path, source).expect("write nested record-field-call parity source");
    let engine = Engine::new().build().expect("Engine builds");
    engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect("nested record field call root uses canonical source route")
        .expect("nested record field call root produces checked closure")
}

#[test]
fn canonical_source_route_fences_bodyless_builtin_execution_without_core_entry() {
    let fixture = tempfile::tempdir().expect("create builtin canonical-route fixture");
    let path = fixture.path().join("main.ash");
    let source = "crate app; pub mod api { pub builtin fn identity(value: Int) -> Int; } use crate::api::identity as remote; fn main() -> Int { remote(7) }";
    fs::write(&path, source).expect("write builtin canonical-route source");

    let engine = Engine::new().build().expect("Engine builds");
    let closure = engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect("builtin signatures remain transportable through the canonical source route")
        .expect("canonical route returns a closure for the checked ordinary root");
    let error = engine
        .admit_linked_module_closure(closure)
        .expect_err("bodyless builtin calls must not invent an Engine callable entry");
    let diagnostic = error.to_string();
    assert!(
        diagnostic.contains("lowered")
            || diagnostic.contains("callable")
            || diagnostic.contains("builtin"),
        "canonical builtin fence should identify the missing callable realization: {diagnostic}"
    );
}

fn source_linked_record_field_expression_call_root_closure() -> LinkedModuleClosure {
    let fixture = tempfile::tempdir().expect("create record-field-expression-call parity fixture");
    let path = fixture.path().join("main.ash");
    let source = "crate app; fn helper() -> Int { 40 } fn main() -> Int { let person = { age: helper() + 1 }; person.age }";
    fs::write(&path, source).expect("write record-field-expression-call parity source");
    let engine = Engine::new().build().expect("Engine builds");
    engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect("record field expression call root uses canonical source route")
        .expect("record field expression call root produces checked closure")
}

fn source_linked_declaration_order_closure(source: &str) -> LinkedModuleClosure {
    let fixture = tempfile::tempdir().expect("create declaration-order parity fixture");
    let path = fixture.path().join("main.ash");
    fs::write(&path, source).expect("write declaration-order parity source");
    let engine = Engine::new().build().expect("Engine builds");
    engine
        .canonical_module_closure_from_source(&path, source, "main")
        .expect("declaration-order source uses canonical route")
        .expect("declaration-order source produces checked closure")
}

fn source_linked_metadata_stub_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_id = NEXT_SOURCE_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-metadata-stub-route-{}-{}-{}",
        std::process::id(),
        fixture_id,
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create metadata-stub fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    let root_source = if inline_child {
        "pub mod api { pub role Reviewer { capabilities: [], obligations: [audit_log] } pub policy RateLimit { marker: Int } } pub mod client { use crate::api::Reviewer as reviewer; use crate::api::RateLimit as rate; pub fn entry() -> Int { 1 } } use crate::client::entry as remote; fn main() -> Int { remote() }".to_owned()
    } else {
        "pub mod api; pub mod client; use crate::client::entry as remote; fn main() -> Int { remote() }".to_owned()
    };
    fs::write(&root_path, &root_source).expect("write metadata-stub fixture root");
    if !inline_child {
        fs::write(
            fixture_root.join("src/api.ash"),
            "pub role Reviewer { capabilities: [], obligations: [audit_log] } pub policy RateLimit { marker: Int }",
        )
        .expect("write metadata-stub provider");
        fs::write(
            fixture_root.join("src/client.ash"),
            "use crate::api::Reviewer as reviewer; use crate::api::RateLimit as rate; pub fn entry() -> Int { 1 }",
        )
        .expect("write metadata-stub client");
    }

    let engine = Engine::new().build().expect("Engine builds");
    let closure = engine
        .canonical_module_closure_from_source(&root_path, &root_source, "main")
        .expect("metadata-stub source route is accepted")
        .expect("structural metadata-stub route uses canonical closure");
    let _ = fs::remove_dir_all(fixture_root);
    closure
}

fn source_linked_let_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_closure_with_root_body(inline_child, "let value = 40; value + 2")
}

fn source_linked_if_let_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_closure_with_root_body(inline_child, "if let true = true then { 42 } else { 0 }")
}

fn source_linked_short_circuit_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_closure_with_root_body(inline_child, "if true && true then { 42 } else { 0 }")
}

fn source_linked_short_circuit_let_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_closure_with_root_body(
        inline_child,
        "let ready = true && true; if let true = ready then { 42 } else { 0 }",
    )
}

fn source_linked_short_circuit_if_let_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_closure_with_root_body(
        inline_child,
        "if let true = true && true then { 42 } else { 0 }",
    )
}

fn source_linked_short_circuit_argument_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_closure_with_provider_declarations(
        inline_child,
        "root",
        "if let true = remote(true && false) then { 42 } else { 0 }",
        "pub fn serve(value: Bool) -> Bool { value }",
    )
}

fn source_linked_short_circuit_match_argument_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_closure_with_provider_declarations(
        inline_child,
        "root",
        "match remote(true && false) { true => 42, false => 0 }",
        "pub fn serve(value: Bool) -> Bool { value }",
    )
}

fn source_linked_nested_short_circuit_argument_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_closure_with_provider_declarations(
        inline_child,
        "root",
        "if let true = remote(remote(true && false)) then { 42 } else { 0 }",
        "pub fn serve(value: Bool) -> Bool { value }",
    )
}

fn source_linked_record_short_circuit_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_closure_with_root_body(
        inline_child,
        "let value = { ready: true && false }; if let true = value.ready then { 42 } else { 0 }",
    )
}

fn source_linked_nested_short_circuit_boolean_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_closure_with_root_body(
        inline_child,
        "if let true = (true && false) || true then { 42 } else { 0 }",
    )
}

fn source_linked_closure_with_root_body(
    inline_child: bool,
    root_body: &str,
) -> LinkedModuleClosure {
    source_linked_closure_with_entry_body_and_visibility(inline_child, "root", root_body, "pub")
}

fn source_linked_main_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_closure_with_entry_body(inline_child, "main", "remote()")
}

fn source_linked_crate_visible_callable_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_visibility_callable_closure(inline_child, "pub(crate)")
}

fn source_linked_super_visible_callable_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_visibility_callable_closure(inline_child, "pub(super)")
}

fn source_linked_restricted_visible_callable_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_visibility_callable_closure(inline_child, "pub(in crate)")
}

fn source_linked_parameterized_callable_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_closure_with_provider_declarations(
        inline_child,
        "main",
        "remote(40)",
        "pub fn serve(value: Int) -> Int { value + 2 }",
    )
}

fn source_linked_multiple_imported_callable_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_id = NEXT_SOURCE_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-multiple-imported-callables-{}-{}-{}",
        std::process::id(),
        fixture_id,
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create multi-call fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    let child_source = "pub fn serve() -> Int { 40 } pub fn other() -> Int { 2 }";
    let root_source = if inline_child {
        format!(
            "pub mod api {{ {child_source} }} use crate::api::serve as first; use crate::api::other as second; fn root() -> Int {{ first() + second() }}"
        )
    } else {
        "pub mod api; use crate::api::serve as first; use crate::api::other as second; fn root() -> Int { first() + second() }".to_owned()
    };
    fs::write(&root_path, root_source).expect("write multi-call fixture root");
    if !inline_child {
        fs::write(fixture_root.join("src/api.ash"), child_source)
            .expect("write multi-call fixture child");
    }

    let root = ModuleKey::root("app").expect("multi-call root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("multi-call fixture resolves through canonical graph"),
    )
    .expect("multi-call fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("multi-call fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("multi-call fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("multi-call fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("multi-call fixture interfaces project");
    let child = root.child("api").expect("multi-call child key");
    let selected_entries = BTreeMap::from([
        (root.clone(), "root".to_owned()),
        (child, "serve".to_owned()),
    ]);
    let lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("multi-call fixture definition closure lowers");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task-2064-multiple-imported-callables"),
            )
        })
        .collect();
    linked_module_closure_from_checked_definition_lowering(
        root.clone(),
        lowered,
        &selected_entries,
        interfaces,
        &source_anchors,
    )
    .expect("multi-call checked lowering converts atomically to an Engine closure")
}

fn source_linked_visibility_callable_closure(
    inline_child: bool,
    provider_visibility: &str,
) -> LinkedModuleClosure {
    source_linked_closure_with_provider_declarations(
        inline_child,
        "main",
        "remote()",
        &format!("{provider_visibility} fn serve() -> Int {{ 2 }} fn helper() -> Int {{ 3 }}"),
    )
}

fn source_linked_closure_with_entry_body(
    inline_child: bool,
    entry_name: &str,
    entry_body: &str,
) -> LinkedModuleClosure {
    source_linked_closure_with_entry_body_and_visibility(
        inline_child,
        entry_name,
        entry_body,
        "pub",
    )
}

fn source_linked_closure_with_entry_body_and_visibility(
    inline_child: bool,
    entry_name: &str,
    entry_body: &str,
    provider_visibility: &str,
) -> LinkedModuleClosure {
    source_linked_closure_with_provider_declarations(
        inline_child,
        entry_name,
        entry_body,
        &format!("{provider_visibility} fn serve() -> Int {{ 2 }} fn helper() -> Int {{ 3 }}"),
    )
}

fn source_linked_closure_with_provider_declarations(
    inline_child: bool,
    entry_name: &str,
    entry_body: &str,
    provider_declarations: &str,
) -> LinkedModuleClosure {
    let fixture_id = NEXT_SOURCE_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-source-route-{}-{}-{}-{}-{}",
        std::process::id(),
        fixture_id,
        if entry_body == "42" {
            "literal"
        } else if entry_body == "let value = 40; value + 2" {
            "let"
        } else {
            "remote"
        },
        entry_name,
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create source fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    let root_source = if inline_child {
        format!(
            "pub mod api {{ {provider_declarations} }} use crate::api::serve as remote; fn {entry_name}() -> Int {{ {entry_body} }}"
        )
    } else {
        format!(
            "pub mod api; use crate::api::serve as remote; fn {entry_name}() -> Int {{ {entry_body} }}"
        )
    };
    fs::write(&root_path, root_source).expect("write source fixture root");
    if !inline_child {
        fs::write(fixture_root.join("src/api.ash"), provider_declarations)
            .expect("write source fixture child");
    }

    let root = ModuleKey::root("app").expect("source fixture root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("source fixture resolves through canonical graph"),
    )
    .expect("source fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("source fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("source fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("source fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("source fixture interfaces project");
    let child = root.child("api").expect("source fixture child key");
    let selected_entries = BTreeMap::from([
        (root.clone(), entry_name.to_owned()),
        (child, "serve".to_owned()),
    ]);
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &selected_entries,
    )
    .expect("source fixture selected-entry lowering succeeds");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-source-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("checked lowering converts atomically to an Engine closure")
}

#[test]
fn checked_entry_closure_rejects_interface_without_lowered_module() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2069-extra-interface-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create extra-interface fixture");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(&root_path, "pub mod api; fn root() -> Int { 1 }")
        .expect("write extra-interface root");
    fs::write(
        fixture_root.join("src/api.ash"),
        "pub fn serve() -> Int { 2 }",
    )
    .expect("write extra-interface child");

    let root = ModuleKey::root("app").expect("extra-interface root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("extra-interface fixture resolves"),
    )
    .expect("extra-interface fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("extra-interface fixture collects");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("extra-interface fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("extra-interface fixture finalizes");
    let mut interfaces =
        build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
            .expect("extra-interface fixture interfaces project");
    let selected_entries = BTreeMap::from([
        (root.clone(), "root".to_owned()),
        (
            root.child("api").expect("child key is canonical"),
            "serve".to_owned(),
        ),
    ]);
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &selected_entries,
    )
    .expect("extra-interface fixture lowers");
    let extra = ModuleKey::root("extra_interface").expect("extra interface key is canonical");
    let extra_artifact = ModuleArtifact::new(
        extra.clone(),
        ModuleArtifactOrigin::File("fixtures/extra_interface.ash".to_owned()),
        None,
        Vec::new(),
    )
    .expect("extra interface artifact is canonical");
    interfaces.push(
        PublicModuleInterface::new(extra_artifact, Vec::new())
            .expect("extra interface is locally valid"),
    );
    let source_anchors = interfaces
        .iter()
        .filter(|interface| interface.artifact().key() != &extra)
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task-2069-extra-interface"),
            )
        })
        .collect();

    let error = linked_module_closure_from_checked_entry_lowering(
        root,
        lowered,
        interfaces,
        &source_anchors,
    )
    .expect_err("an interface without a lowered module must reject atomically");
    assert!(matches!(
        error,
        LinkedModuleClosureBuildError::UnexpectedInterface { module } if module == extra
    ));
    let _ = fs::remove_dir_all(fixture_root);
}

fn source_linked_transitive_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-transitive-route-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create transitive source fixture");
    let root_path = fixture_root.join("src/main.ash");
    if inline_child {
        fs::write(
            &root_path,
            "pub mod dep { pub fn leaf() -> Int { 1 } } pub mod api { use crate::dep::leaf as remote; pub fn serve() -> Int { remote() } } use crate::api::serve as remote; fn root() -> Int { remote() }",
        )
        .expect("write inline transitive fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod dep; pub mod api; use crate::api::serve as remote; fn root() -> Int { remote() }",
        )
        .expect("write file transitive fixture root");
        fs::write(
            fixture_root.join("src/dep.ash"),
            "pub fn leaf() -> Int { 1 }",
        )
        .expect("write file transitive dependency");
        fs::write(
            fixture_root.join("src/api.ash"),
            "use crate::dep::leaf as remote; pub fn serve() -> Int { remote() }",
        )
        .expect("write file transitive provider");
    }

    let root = ModuleKey::root("app").expect("transitive fixture root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("transitive fixture resolves through canonical graph"),
    )
    .expect("transitive fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("transitive fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("transitive fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("transitive fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("transitive fixture interfaces project");
    let dep = root.child("dep").expect("transitive dependency key");
    let api = root.child("api").expect("transitive provider key");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (dep, "leaf".to_owned()),
            (api, "serve".to_owned()),
        ]),
    )
    .expect("transitive fixture selected-entry lowering succeeds");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-transitive-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("transitive checked lowering converts atomically to an Engine closure")
}

fn source_linked_reexport_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-reexport-route-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create re-export fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    if inline_child {
        fs::write(
            &root_path,
            "pub mod provider { pub fn serve() -> Int { 2 } } pub mod api { pub use crate::provider::serve; fn local() -> Int { 3 } } use crate::api::serve as remote; fn root() -> Int { remote() }",
        )
        .expect("write inline re-export fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod provider; pub mod api; use crate::api::serve as remote; fn root() -> Int { remote() }",
        )
        .expect("write file re-export fixture root");
        fs::write(
            fixture_root.join("src/provider.ash"),
            "pub fn serve() -> Int { 2 }",
        )
        .expect("write file re-export provider");
        fs::write(
            fixture_root.join("src/api.ash"),
            "pub use crate::provider::serve; fn local() -> Int { 3 }",
        )
        .expect("write file re-export facade");
    }

    let root = ModuleKey::root("app").expect("re-export fixture root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("re-export fixture resolves through canonical graph"),
    )
    .expect("re-export fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("re-export fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("re-export fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("re-export fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("re-export fixture interfaces project");
    let provider = root.child("provider").expect("provider key is canonical");
    let api = root.child("api").expect("api key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (provider, "serve".to_owned()),
            (api.clone(), "local".to_owned()),
        ]),
    )
    .expect("re-export fixture selected-entry lowering succeeds");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-reexport-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("re-export checked lowering converts atomically to an Engine closure")
}

fn source_linked_transitive_reexport_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-transitive-reexport-route-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src"))
        .expect("create transitive re-export fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    if inline_child {
        fs::write(
            &root_path,
            "pub mod provider { pub fn serve() -> Int { 2 } } pub mod api { pub use crate::provider::serve; } pub mod client { pub use crate::api::serve as forwarded; pub fn entry() -> Int { forwarded() } } use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write inline transitive re-export fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod provider; pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write transitive re-export fixture root");
        fs::write(
            fixture_root.join("src/provider.ash"),
            "pub fn serve() -> Int { 2 }",
        )
        .expect("write transitive re-export provider");
        fs::write(
            fixture_root.join("src/api.ash"),
            "pub use crate::provider::serve;",
        )
        .expect("write transitive re-export facade");
        fs::write(
            fixture_root.join("src/client.ash"),
            "pub use crate::api::serve as forwarded; pub fn entry() -> Int { forwarded() }",
        )
        .expect("write transitive re-export client");
    }

    let root = ModuleKey::root("app").expect("transitive re-export fixture root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("transitive re-export fixture resolves through canonical graph"),
    )
    .expect("transitive re-export fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("transitive re-export fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("transitive re-export fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("transitive re-export fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("transitive re-export fixture interfaces project");
    let provider = root
        .child("provider")
        .expect("transitive provider key is canonical");
    let api = root.child("api").expect("transitive api key is canonical");
    let client = root
        .child("client")
        .expect("transitive client key is canonical");
    let exported_identity = |module: &ModuleKey, name: &str| {
        interfaces
            .iter()
            .find(|interface| interface.artifact().key() == module)
            .and_then(|interface| {
                interface
                    .bindings()
                    .iter()
                    .find(|binding| binding.visible_name() == name)
            })
            .map(|binding| binding.defining_identity())
    };
    let provider_identity = exported_identity(&provider, "serve")
        .expect("provider publishes the original callable identity");
    assert_eq!(
        exported_identity(&api, "serve"),
        Some(provider_identity),
        "the first public re-export must preserve the provider identity"
    );
    assert_eq!(
        exported_identity(&client, "forwarded"),
        Some(provider_identity),
        "the second public re-export must preserve the original provider identity"
    );
    let mut lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("transitive re-export fixture definition lowering succeeds");
    lowered.push(
        lower_checked_metadata_only_module(&finalized, &collection, &expanded, &imports, &api)
            .expect("transitive re-export facade retains a metadata-only carrier"),
    );
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task-2064-transitive-reexport-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_definition_lowering(
        root.clone(),
        lowered,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (provider, "serve".to_owned()),
            (client, "entry".to_owned()),
            (api, String::new()),
        ]),
        interfaces,
        &source_anchors,
    )
    .expect("transitive re-export checked lowering converts atomically to an Engine closure")
}

fn source_linked_nested_structural_callable_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-nested-structural-callable-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src"))
        .expect("create nested structural callable fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    if inline_child {
        fs::write(
            &root_path,
            "pub mod api { pub mod nested { pub fn serve() -> Int { 2 } } } pub mod facade { pub use crate::api::nested::serve as forwarded; } pub mod client { use crate::facade::forwarded as remote; pub fn entry() -> Int { remote() } } use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write inline nested structural callable fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; pub mod facade; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write nested structural callable fixture root");
        fs::write(fixture_root.join("src/api.ash"), "pub mod nested;")
            .expect("write nested structural facade");
        fs::write(
            fixture_root.join("src/nested.ash"),
            "pub fn serve() -> Int { 2 }",
        )
        .expect("write nested structural callable provider");
        fs::write(
            fixture_root.join("src/facade.ash"),
            "pub use crate::api::nested::serve as forwarded;",
        )
        .expect("write nested structural re-export facade");
        fs::write(
            fixture_root.join("src/client.ash"),
            "use crate::facade::forwarded as remote; pub fn entry() -> Int { remote() }",
        )
        .expect("write nested structural client");
    }

    let root = ModuleKey::root("app").expect("nested structural fixture root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("nested structural fixture resolves through canonical graph"),
    )
    .expect("nested structural fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("nested structural fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("nested structural fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("nested structural fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("nested structural fixture interfaces project");
    let api = root
        .child("api")
        .expect("nested structural api key is canonical");
    let nested = api
        .child("nested")
        .expect("nested structural child key is canonical");
    let facade = root
        .child("facade")
        .expect("nested structural re-export facade key is canonical");
    let client = root
        .child("client")
        .expect("nested structural client key is canonical");
    let mut lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("nested structural definition lowering succeeds");
    lowered.push(
        lower_checked_metadata_only_module(&finalized, &collection, &expanded, &imports, &api)
            .expect("nested structural facade retains a metadata-only carrier"),
    );
    lowered.push(
        lower_checked_metadata_only_module(&finalized, &collection, &expanded, &imports, &facade)
            .expect("nested structural re-export facade retains a metadata-only carrier"),
    );
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task-2064-nested-structural-callable-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_definition_lowering(
        root.clone(),
        lowered,
        &BTreeMap::from([
            (root, "root".to_owned()),
            (api, String::new()),
            (nested, "serve".to_owned()),
            (facade, String::new()),
            (client, "entry".to_owned()),
        ]),
        interfaces,
        &source_anchors,
    )
    .expect("nested structural callable lowering reaches the Engine closure")
}

fn source_linked_structural_module_alias_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_structural_module_alias_closure_with_depth(inline_child, false)
}

fn source_linked_nested_structural_module_alias_closure(inline_child: bool) -> LinkedModuleClosure {
    source_linked_structural_module_alias_closure_with_depth(inline_child, true)
}

fn source_linked_structural_module_alias_closure_with_depth(
    inline_child: bool,
    nested: bool,
) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-structural-module-alias-{}-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" },
        if nested { "nested" } else { "direct" }
    ));
    fs::create_dir_all(fixture_root.join("src"))
        .expect("create structural module alias fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    if inline_child {
        fs::write(
            &root_path,
            if nested {
                "pub mod api { pub mod nested { pub fn serve() -> Int { 2 } } } pub mod client { use crate::api::nested as remote; pub fn entry() -> Int { remote::serve() } } use crate::client::entry as remote; fn root() -> Int { remote() }"
            } else {
                "pub mod api { pub fn serve() -> Int { 2 } } pub mod client { use crate::api as remote; pub fn entry() -> Int { remote::serve() } } use crate::client::entry as remote; fn root() -> Int { remote() }"
            },
        )
        .expect("write inline structural module alias fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write structural module alias fixture root");
        fs::write(
            fixture_root.join("src/api.ash"),
            if nested {
                "pub mod nested;"
            } else {
                "pub fn serve() -> Int { 2 }"
            },
        )
        .expect("write structural module alias provider");
        if nested {
            fs::write(
                fixture_root.join("src/nested.ash"),
                "pub fn serve() -> Int { 2 }",
            )
            .expect("write nested structural module alias provider");
        }
        fs::write(
            fixture_root.join("src/client.ash"),
            if nested {
                "use crate::api::nested as remote; pub fn entry() -> Int { remote::serve() }"
            } else {
                "use crate::api as remote; pub fn entry() -> Int { remote::serve() }"
            },
        )
        .expect("write structural module alias client");
    }

    let root = ModuleKey::root("app").expect("structural module alias fixture root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("structural module alias fixture resolves through canonical graph"),
    )
    .expect("structural module alias fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("structural module alias fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("structural module alias fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("structural module alias fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("structural module alias fixture interfaces project");
    let api = ModuleKey::root("app")
        .expect("structural module alias root key")
        .child("api")
        .expect("structural module alias api key");
    let mut lowered = lower_complete_checked_module_definition_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
    )
    .expect("structural module alias callable lowering succeeds");
    if nested {
        lowered.push(
            lower_checked_metadata_only_module(&finalized, &collection, &expanded, &imports, &api)
                .expect("nested structural alias parent retains a metadata-only carrier"),
        );
    }
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task-2064-structural-module-alias-route"),
            )
        })
        .collect();
    let mut selected_entries = BTreeMap::from([
        (
            ModuleKey::root("app").expect("structural module alias root key"),
            "root".to_owned(),
        ),
        (api.clone(), String::new()),
        (
            ModuleKey::root("app")
                .expect("structural module alias root key")
                .child("client")
                .expect("structural module alias client key"),
            "entry".to_owned(),
        ),
    ]);
    if nested {
        selected_entries.insert(
            api.child("nested")
                .expect("structural module alias nested key"),
            "serve".to_owned(),
        );
    } else {
        selected_entries.insert(api, "serve".to_owned());
    }
    linked_module_closure_from_checked_definition_lowering(
        root,
        lowered,
        &selected_entries,
        interfaces,
        &source_anchors,
    )
    .expect("structural module alias lowering reaches the Engine closure")
}

fn source_linked_type_reexport_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-type-reexport-route-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create type re-export fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    if inline_child {
        fs::write(
            &root_path,
            "pub mod provider { pub type Marker = Int; pub fn marker() -> Int { 1 } } pub mod api { pub use crate::provider::Marker; fn local() -> Int { 1 } } pub mod client { use crate::api::Marker; pub fn typed(value: Marker) -> Int { 1 } pub fn entry() -> Int { 1 } } use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write inline type re-export fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod provider; pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write file type re-export fixture root");
        fs::write(
            fixture_root.join("src/provider.ash"),
            "pub type Marker = Int; pub fn marker() -> Int { 1 }",
        )
        .expect("write file type re-export provider");
        fs::write(
            fixture_root.join("src/api.ash"),
            "pub use crate::provider::Marker; fn local() -> Int { 1 }",
        )
        .expect("write file type re-export facade");
        fs::write(
            fixture_root.join("src/client.ash"),
            "use crate::api::Marker; pub fn typed(value: Marker) -> Int { 1 } pub fn entry() -> Int { 1 }",
        )
        .expect("write file type re-export client");
    }

    let root = ModuleKey::root("app").expect("type re-export fixture root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("type re-export fixture resolves through canonical graph"),
    )
    .expect("type re-export fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("type re-export fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("type re-export fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("type re-export fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("type re-export fixture interfaces project");
    let provider = root.child("provider").expect("provider key is canonical");
    let api = root.child("api").expect("api key is canonical");
    let client = root.child("client").expect("client key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (provider.clone(), "marker".to_owned()),
            (api.clone(), "local".to_owned()),
            (client.clone(), "entry".to_owned()),
        ]),
    )
    .expect("type re-export fixture selected-entry lowering succeeds");
    let client_entry = lowered
        .iter()
        .find(|entry| entry.declaration_name() == "entry")
        .expect("type re-export client entry is lowered");
    assert!(
        client_entry.core().imports().iter().any(|import| {
            import.binding().kind() == ash_core::module_interface::ModuleInterfaceBindingKind::Type
                && matches!(
                    import.binding().defining_identity(),
                    ash_core::module_interface::ModuleInterfaceDefiningIdentity::Declaration(
                        identity
                    ) if identity.module == provider
                )
        }),
        "a type re-export must retain its defining provider identity as a typed Core transport fact: {:?}",
        client_entry.core().imports()
    );
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-type-reexport-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("type re-export checked lowering converts atomically to an Engine closure")
}

fn source_linked_notation_import_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-notation-import-route-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create notation import fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    let provider_source =
        "pub infixl 6 <+> = combine\npub fn combine(left: Int, right: Int) -> Int { 1 }";
    let client_source = "use crate::api::(<+>); pub fn entry() -> Int { 1 }";
    if inline_child {
        fs::write(
            &root_path,
            format!(
                "pub mod api {{ {provider_source} }} pub mod client {{ {client_source} }} use crate::client::entry as remote; fn root() -> Int {{ remote() }}"
            ),
        )
        .expect("write inline notation import fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write file notation import fixture root");
        fs::write(fixture_root.join("src/api.ash"), provider_source)
            .expect("write file notation import provider");
        fs::write(fixture_root.join("src/client.ash"), client_source)
            .expect("write file notation import client");
    }

    let root = ModuleKey::root("app").expect("notation import fixture root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("notation import fixture resolves through canonical graph"),
    )
    .expect("notation import fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("notation import fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("notation import fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("notation import fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("notation import fixture interfaces project");
    let api = root
        .child("api")
        .expect("notation provider key is canonical");
    let client = root
        .child("client")
        .expect("notation client key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (api.clone(), "combine".to_owned()),
            (client.clone(), "entry".to_owned()),
        ]),
    )
    .expect("notation import fixture selected-entry lowering succeeds");
    let client_entry = lowered
        .iter()
        .find(|entry| entry.declaration_name() == "entry")
        .expect("notation import client entry is lowered");
    assert!(
        client_entry.core().imports().iter().any(|import| {
            import.binding().kind()
                == ash_core::module_interface::ModuleInterfaceBindingKind::SyntaxNotation
                && matches!(
                    import.binding().defining_identity(),
                    ash_core::module_interface::ModuleInterfaceDefiningIdentity::Declaration(
                        identity
                    ) if identity.module == api
                )
        }),
        "explicit notation imports must remain syntax-phase Core transport metadata"
    );
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-notation-import-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("notation import checked lowering converts atomically to an Engine closure")
}

fn source_linked_implementation_metadata_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-implementation-metadata-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src"))
        .expect("create implementation metadata fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    if inline_child {
        fs::write(
            &root_path,
            "pub mod api { pub interface Eq<A> { equiv(A, A) -> Bool } pub impl Eq<Int> { equiv(a, b) = a == b } pub fn marker() -> Int { 1 } } use crate::api::marker as remote; fn root() -> Int { remote() }",
        )
        .expect("write inline implementation metadata fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; use crate::api::marker as remote; fn root() -> Int { remote() }",
        )
        .expect("write file implementation metadata fixture root");
        fs::write(
            fixture_root.join("src/api.ash"),
            "pub interface Eq<A> { equiv(A, A) -> Bool } pub impl Eq<Int> { equiv(a, b) = a == b } pub fn marker() -> Int { 1 }",
        )
        .expect("write file implementation metadata fixture child");
    }

    let root = ModuleKey::root("app").expect("implementation metadata root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("implementation metadata fixture resolves through canonical graph"),
    )
    .expect("implementation metadata fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("implementation metadata fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("implementation metadata fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("implementation metadata fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("implementation metadata fixture interfaces project");
    let api = root
        .child("api")
        .expect("implementation metadata child key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (api, "marker".to_owned()),
        ]),
    )
    .expect("implementation metadata fixture selected-entry lowering succeeds");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-implementation-metadata-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("implementation metadata checked lowering converts atomically to an Engine closure")
}

fn source_linked_interface_import_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-interface-import-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src"))
        .expect("create interface import fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    if inline_child {
        fs::write(
            &root_path,
            "pub mod api { pub interface Eq<A> { equiv(A, A) -> Bool } pub fn marker() -> Int { 1 } } pub mod client { use crate::api::Eq; pub fn entry() -> Int { 1 } } use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write inline interface import fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write file interface import fixture root");
        fs::write(
            fixture_root.join("src/api.ash"),
            "pub interface Eq<A> { equiv(A, A) -> Bool } pub fn marker() -> Int { 1 }",
        )
        .expect("write file interface import provider");
        fs::write(
            fixture_root.join("src/client.ash"),
            "use crate::api::Eq; pub fn entry() -> Int { 1 }",
        )
        .expect("write file interface import client");
    }

    let root = ModuleKey::root("app").expect("interface import root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("interface import fixture resolves through canonical graph"),
    )
    .expect("interface import fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("interface import fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("interface import fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("interface import fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("interface import fixture interfaces project");
    let api = root
        .child("api")
        .expect("interface import provider key is canonical");
    let client = root
        .child("client")
        .expect("interface import client key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (api, "marker".to_owned()),
            (client, "entry".to_owned()),
        ]),
    )
    .expect("interface import fixture selected-entry lowering succeeds");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-interface-import-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("interface import checked lowering converts atomically to an Engine closure")
}

fn source_linked_type_import_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-type-import-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create type import fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    if inline_child {
        fs::write(
            &root_path,
            "pub mod api { pub type Marker = Int; pub fn marker() -> Int { 1 } } pub mod client { use crate::api::Marker; pub fn typed(value: Marker) -> Int { 1 } pub fn entry() -> Int { 1 } } use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write inline type import fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write file type import fixture root");
        fs::write(
            fixture_root.join("src/api.ash"),
            "pub type Marker = Int; pub fn marker() -> Int { 1 }",
        )
        .expect("write file type import provider");
        fs::write(
            fixture_root.join("src/client.ash"),
            "use crate::api::Marker; pub fn typed(value: Marker) -> Int { 1 } pub fn entry() -> Int { 1 }",
        )
        .expect("write file type import client");
    }

    let root = ModuleKey::root("app").expect("type import root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("type import fixture resolves through canonical graph"),
    )
    .expect("type import fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("type import fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("type import fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("type import fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("type import fixture interfaces project");
    let api = root
        .child("api")
        .expect("type import provider key is canonical");
    let client = root
        .child("client")
        .expect("type import client key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (api, "marker".to_owned()),
            (client, "entry".to_owned()),
        ]),
    )
    .expect("type import fixture selected-entry lowering succeeds");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-type-import-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("type import checked lowering converts atomically to an Engine closure")
}

fn source_linked_newtype_import_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-newtype-import-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create newtype import fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    if inline_child {
        fs::write(
            &root_path,
            "pub mod api { pub newtype OrderId = OrderId(Int); pub fn marker() -> Int { 1 } } pub mod client { use crate::api::OrderId; pub fn typed(value: OrderId) -> Int { 1 } pub fn entry() -> Int { 1 } } use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write inline newtype import fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write file newtype import fixture root");
        fs::write(
            fixture_root.join("src/api.ash"),
            "pub newtype OrderId = OrderId(Int); pub fn marker() -> Int { 1 }",
        )
        .expect("write file newtype import provider");
        fs::write(
            fixture_root.join("src/client.ash"),
            "use crate::api::OrderId; pub fn typed(value: OrderId) -> Int { 1 } pub fn entry() -> Int { 1 }",
        )
        .expect("write file newtype import client");
    }

    let root = ModuleKey::root("app").expect("newtype import root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("newtype import fixture resolves through canonical graph"),
    )
    .expect("newtype import fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("newtype import fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("newtype import fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("newtype import fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("newtype import fixture interfaces project");
    let api = root
        .child("api")
        .expect("newtype import provider key is canonical");
    let client = root
        .child("client")
        .expect("newtype import client key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (api, "marker".to_owned()),
            (client, "entry".to_owned()),
        ]),
    )
    .expect("newtype import fixture selected-entry lowering succeeds");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-newtype-import-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("newtype import checked lowering converts atomically to an Engine closure")
}

fn source_linked_resource_import_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-resource-import-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create resource import fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    if inline_child {
        fs::write(
            &root_path,
            "pub mod api { pub resource type Store { value: Int } pub fn marker() -> Int { 1 } } pub mod client { use crate::api::Store; pub fn typed(value: Store) -> Int { 1 } pub fn entry() -> Int { 1 } } use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write inline resource import fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write file resource import fixture root");
        fs::write(
            fixture_root.join("src/api.ash"),
            "pub resource type Store { value: Int } pub fn marker() -> Int { 1 }",
        )
        .expect("write file resource import provider");
        fs::write(
            fixture_root.join("src/client.ash"),
            "use crate::api::Store; pub fn typed(value: Store) -> Int { 1 } pub fn entry() -> Int { 1 }",
        )
        .expect("write file resource import client");
    }

    let root = ModuleKey::root("app").expect("resource import root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("resource import fixture resolves through canonical graph"),
    )
    .expect("resource import fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("resource import fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("resource import fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("resource import fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("resource import fixture interfaces project");
    let api = root
        .child("api")
        .expect("resource import provider key is canonical");
    let client = root
        .child("client")
        .expect("resource import client key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (api, "marker".to_owned()),
            (client, "entry".to_owned()),
        ]),
    )
    .expect("resource import fixture selected-entry lowering succeeds");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-resource-import-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("resource import checked lowering converts atomically to an Engine closure")
}

fn source_linked_effect_row_import_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-effect-row-import-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src"))
        .expect("create effect-row import fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    if inline_child {
        fs::write(
            &root_path,
            "pub mod api { pub effect alias Audit = { evidence audit_log }; pub fn marker() -> Int { 1 } } pub mod client { use crate::api::Audit; pub fn typed() -> Int where row { Audit } { 1 } pub fn entry() -> Int { 1 } } use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write inline effect-row import fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write file effect-row import fixture root");
        fs::write(
            fixture_root.join("src/api.ash"),
            "pub effect alias Audit = { evidence audit_log }; pub fn marker() -> Int { 1 }",
        )
        .expect("write file effect-row import provider");
        fs::write(
            fixture_root.join("src/client.ash"),
            "use crate::api::Audit; pub fn typed() -> Int where row { Audit } { 1 } pub fn entry() -> Int { 1 }",
        )
        .expect("write file effect-row import client");
    }

    let root = ModuleKey::root("app").expect("effect-row import root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("effect-row import fixture resolves through canonical graph"),
    )
    .expect("effect-row import fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("effect-row import fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("effect-row import fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("effect-row import fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("effect-row import fixture interfaces project");
    let api = root
        .child("api")
        .expect("effect-row import provider key is canonical");
    let client = root
        .child("client")
        .expect("effect-row import client key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (api, "marker".to_owned()),
            (client, "entry".to_owned()),
        ]),
    )
    .expect("effect-row import fixture selected-entry lowering succeeds");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-effect-row-import-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("effect-row import checked lowering converts atomically to an Engine closure")
}

fn source_linked_type_function_import_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-type-function-import-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src"))
        .expect("create type-function import fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    let api_source = "pub sealed type domain Tree { Leaf; } pub type fn Identity(xs: Tree) -> Tree { case Identity<xs> = xs; } pub fn marker() -> Int { 1 }";
    let client_source = "use crate::api::Tree; use crate::api::Identity; pub type fn Wrapper(xs: Tree) -> Tree { case Wrapper<xs> = Identity<xs>; } pub fn entry() -> Int { 1 }";
    if inline_child {
        fs::write(
            &root_path,
            format!(
                "pub mod api {{ {api_source} }} pub mod client {{ {client_source} }} use crate::client::entry as remote; fn root() -> Int {{ remote() }}"
            ),
        )
        .expect("write inline type-function import fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write file type-function import fixture root");
        fs::write(fixture_root.join("src/api.ash"), api_source)
            .expect("write file type-function import provider");
        fs::write(fixture_root.join("src/client.ash"), client_source)
            .expect("write file type-function import client");
    }

    let root = ModuleKey::root("app").expect("type-function import root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("type-function import fixture resolves through canonical graph"),
    )
    .expect("type-function import fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("type-function import fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("type-function import fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("type-function import fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("type-function import fixture interfaces project");
    let api = root
        .child("api")
        .expect("type-function import provider key is canonical");
    let client = root
        .child("client")
        .expect("type-function import client key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (api, "marker".to_owned()),
            (client, "entry".to_owned()),
        ]),
    )
    .expect("type-function import fixture selected-entry lowering succeeds");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-type-function-import-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("type-function import checked lowering converts atomically to an Engine closure")
}

fn source_linked_promoted_kind_import_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-promoted-kind-import-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src"))
        .expect("create promoted-kind import fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    let api_source = "pub type Tree = Leaf | Branch(Tree); pub data kind TreeKind from type Tree; pub fn marker() -> Int { 1 }";
    let client_source = "use crate::api::TreeKind; pub fn entry() -> Int { 1 }";
    if inline_child {
        fs::write(
            &root_path,
            format!(
                "pub mod api {{ {api_source} }} pub mod client {{ {client_source} }} use crate::client::entry as remote; fn root() -> Int {{ remote() }}"
            ),
        )
        .expect("write inline promoted-kind import fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write file promoted-kind import fixture root");
        fs::write(fixture_root.join("src/api.ash"), api_source)
            .expect("write file promoted-kind import provider");
        fs::write(fixture_root.join("src/client.ash"), client_source)
            .expect("write file promoted-kind import client");
    }

    let root = ModuleKey::root("app").expect("promoted-kind import root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("promoted-kind import fixture resolves through canonical graph"),
    )
    .expect("promoted-kind import fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("promoted-kind import fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("promoted-kind import fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("promoted-kind import fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("promoted-kind import fixture interfaces project");
    let api = root
        .child("api")
        .expect("promoted-kind import provider key is canonical");
    let client = root
        .child("client")
        .expect("promoted-kind import client key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (api, "marker".to_owned()),
            (client, "entry".to_owned()),
        ]),
    )
    .expect("promoted-kind import fixture selected-entry lowering succeeds");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-promoted-kind-import-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("promoted-kind import checked lowering converts atomically to an Engine closure")
}

fn source_linked_proposition_import_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-proposition-import-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src"))
        .expect("create proposition import fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    let api_source = "pub type Tree = Leaf | Branch(Tree); pub prop NonEmpty<Xs: Tree>; pub fn marker() -> Int { 1 }";
    let client_source = "use crate::api::NonEmpty; pub fn entry() -> Int { 1 }";
    if inline_child {
        fs::write(
            &root_path,
            format!(
                "pub mod api {{ {api_source} }} pub mod client {{ {client_source} }} use crate::client::entry as remote; fn root() -> Int {{ remote() }}"
            ),
        )
        .expect("write inline proposition import fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write file proposition import fixture root");
        fs::write(fixture_root.join("src/api.ash"), api_source)
            .expect("write file proposition import provider");
        fs::write(fixture_root.join("src/client.ash"), client_source)
            .expect("write file proposition import client");
    }

    let root = ModuleKey::root("app").expect("proposition import root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("proposition import fixture resolves through canonical graph"),
    )
    .expect("proposition import fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("proposition import fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("proposition import fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("proposition import fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("proposition import fixture interfaces project");
    let api = root
        .child("api")
        .expect("proposition import provider key is canonical");
    let client = root
        .child("client")
        .expect("proposition import client key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (api, "marker".to_owned()),
            (client, "entry".to_owned()),
        ]),
    )
    .expect("proposition import fixture selected-entry lowering succeeds");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-proposition-import-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("proposition import checked lowering converts atomically to an Engine closure")
}

fn source_linked_evidence_import_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-evidence-import-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create evidence import fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    let api_source = "pub law reflexive(value: Int): value == value\npub fn marker() -> Int { 1 }";
    let client_source = "use crate::api::reflexive; pub fn entry() -> Int { 1 }";
    if inline_child {
        fs::write(
            &root_path,
            format!(
                "pub mod api {{ {api_source} }} pub mod client {{ {client_source} }} use crate::client::entry as remote; fn root() -> Int {{ remote() }}"
            ),
        )
        .expect("write inline evidence import fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write file evidence import fixture root");
        fs::write(fixture_root.join("src/api.ash"), api_source)
            .expect("write file evidence import provider");
        fs::write(fixture_root.join("src/client.ash"), client_source)
            .expect("write file evidence import client");
    }

    let root = ModuleKey::root("app").expect("evidence import root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("evidence import fixture resolves through canonical graph"),
    )
    .expect("evidence import fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("evidence import fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("evidence import fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("evidence import fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("evidence import fixture interfaces project");
    let api = root
        .child("api")
        .expect("evidence import provider key is canonical");
    let client = root
        .child("client")
        .expect("evidence import client key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (api, "marker".to_owned()),
            (client.clone(), "entry".to_owned()),
        ]),
    )
    .expect("evidence import fixture selected-entry lowering succeeds");
    let client_entry = lowered
        .iter()
        .find(|entry| entry.declaration_name() == "entry")
        .expect("evidence import client entry is lowered");
    assert!(
        client_entry.core().imports().iter().any(|import| {
            import.binding().kind()
                == ash_core::module_interface::ModuleInterfaceBindingKind::Evidence
        }),
        "the evidence import must remain an explicit non-authorizing Core transport fact"
    );
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-evidence-import-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("evidence import checked lowering converts atomically to an Engine closure")
}

fn source_linked_macro_import_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-macro-import-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create macro import fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    let api_source = "pub macro inc(x: Int) -> Int => x + 1; pub fn marker() -> Int { 1 }";
    let client_source = "use crate::api::inc; pub fn entry() -> Int { inc!(1) }";
    if inline_child {
        fs::write(
            &root_path,
            format!(
                "pub mod api {{ {api_source} }} pub mod client {{ {client_source} }} use crate::client::entry as remote; fn root() -> Int {{ remote() }}"
            ),
        )
        .expect("write inline macro import fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write file macro import fixture root");
        fs::write(fixture_root.join("src/api.ash"), api_source)
            .expect("write file macro import provider");
        fs::write(fixture_root.join("src/client.ash"), client_source)
            .expect("write file macro import client");
    }

    let root = ModuleKey::root("app").expect("macro import root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("macro import fixture resolves through canonical graph"),
    )
    .expect("macro import fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("macro import fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("macro import fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("macro import fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("macro import fixture interfaces project");
    let api = root
        .child("api")
        .expect("macro import provider key is canonical");
    let client = root
        .child("client")
        .expect("macro import client key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (api, "marker".to_owned()),
            (client, "entry".to_owned()),
        ]),
    )
    .expect("macro import fixture selected-entry lowering succeeds");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-macro-import-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("macro import checked lowering converts atomically to an Engine closure")
}

fn source_linked_constructor_import_closure(inline_child: bool) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-constructor-import-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" }
    ));
    fs::create_dir_all(fixture_root.join("src"))
        .expect("create constructor import fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    if inline_child {
        fs::write(
            &root_path,
            "pub mod api { pub type Tree = Leaf | Branch(Tree); pub fn marker() -> Int { 1 } } pub mod client { use crate::api::Tree::Branch; pub fn entry() -> Int { 1 } } use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write inline constructor import fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod api; pub mod client; use crate::client::entry as remote; fn root() -> Int { remote() }",
        )
        .expect("write file constructor import fixture root");
        fs::write(
            fixture_root.join("src/api.ash"),
            "pub type Tree = Leaf | Branch(Tree); pub fn marker() -> Int { 1 }",
        )
        .expect("write file constructor import provider");
        fs::write(
            fixture_root.join("src/client.ash"),
            "use crate::api::Tree::Branch; pub fn entry() -> Int { 1 }",
        )
        .expect("write file constructor import client");
    }

    let root = ModuleKey::root("app").expect("constructor import root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("constructor import fixture resolves through canonical graph"),
    )
    .expect("constructor import fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("constructor import fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("constructor import fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("constructor import fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("constructor import fixture interfaces project");
    let api = root
        .child("api")
        .expect("constructor import provider key is canonical");
    let client = root
        .child("client")
        .expect("constructor import client key is canonical");
    let lowered = lower_complete_checked_module_entry_closure(
        &finalized,
        &collection,
        &expanded,
        &imports,
        &BTreeMap::from([
            (root.clone(), "root".to_owned()),
            (api, "marker".to_owned()),
            (client, "entry".to_owned()),
        ]),
    )
    .expect("constructor import fixture selected-entry lowering succeeds");
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-constructor-import-route"),
            )
        })
        .collect();
    linked_module_closure_from_checked_entry_lowering(root, lowered, interfaces, &source_anchors)
        .expect("constructor import checked lowering converts atomically to an Engine closure")
}

fn source_linked_self_alias_closure(
    inline_child: bool,
    recursive_selected_entry: bool,
) -> LinkedModuleClosure {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-self-alias-{}-{}-{}",
        std::process::id(),
        if inline_child { "inline" } else { "file" },
        if recursive_selected_entry {
            "recursive"
        } else {
            "acyclic"
        }
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create self-alias fixture directory");
    let root_path = fixture_root.join("src/main.ash");
    let child_source = if recursive_selected_entry {
        "use self::root as remote; pub fn root() -> Int { remote() }"
    } else {
        "use self::helper as remote; fn helper() -> Int { 21 } pub fn root() -> Int { remote() + 21 }"
    };
    if inline_child {
        fs::write(
            &root_path,
            format!(
                "pub mod client {{ {child_source} }} use crate::client::root as remote; fn entry() -> Int {{ remote() }}"
            ),
        )
        .expect("write inline self-alias fixture");
    } else {
        fs::write(
            &root_path,
            "pub mod client; use crate::client::root as remote; fn entry() -> Int { remote() }",
        )
        .expect("write file self-alias fixture");
        fs::write(fixture_root.join("src/client.ash"), child_source)
            .expect("write file self-alias child fixture");
    }

    let root = ModuleKey::root("app").expect("self-alias root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root.clone(), &root_path)
            .expect("self-alias fixture resolves through canonical graph"),
    )
    .expect("self-alias fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("self-alias fixture collection succeeds");
    let imports = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("self-alias fixture imports resolve");
    let finalized = finalize_canonical_module_collection(&expanded, &collection, &imports)
        .expect("self-alias fixture finalization succeeds");
    let interfaces = build_checked_public_module_interface_closure(&finalized, &expanded, &imports)
        .expect("self-alias fixture interfaces project");
    let client = root.child("client").expect("self-alias client key");
    let client_dependencies = interfaces
        .iter()
        .find(|interface| interface.artifact().key() == &client)
        .expect("self-alias client interface")
        .dependencies();
    assert_eq!(
        client_dependencies,
        &[],
        "same-module aliases must not manufacture external dependency edges"
    );
    let selected_entries = BTreeMap::from([
        (
            ModuleKey::root("app").expect("self-alias root key"),
            "entry".to_owned(),
        ),
        (client.clone(), "root".to_owned()),
    ]);
    let lowered = if recursive_selected_entry {
        lower_complete_checked_module_entry_closure(
            &finalized,
            &collection,
            &expanded,
            &imports,
            &selected_entries,
        )
        .expect("recursive self-alias selected-entry lowering succeeds")
    } else {
        lower_complete_checked_module_definition_closure(
            &finalized,
            &collection,
            &expanded,
            &imports,
        )
        .expect("self-alias fixture complete lowering succeeds")
    };
    let _ = fs::remove_dir_all(fixture_root);
    let source_anchors = interfaces
        .iter()
        .map(|interface| {
            (
                interface.artifact().key().clone(),
                source_anchor("task_2064-self-alias-route"),
            )
        })
        .collect();
    if recursive_selected_entry {
        linked_module_closure_from_checked_entry_lowering(
            root,
            lowered,
            interfaces,
            &source_anchors,
        )
        .expect("recursive self-alias selected lowering converts atomically to an Engine closure")
    } else {
        linked_module_closure_from_checked_definition_lowering(
            root,
            lowered,
            &selected_entries,
            interfaces,
            &source_anchors,
        )
        .expect("self-alias checked lowering converts atomically to an Engine closure")
    }
}

async fn client_terminals(
    engine: &Engine,
    closure: LinkedModuleClosure,
) -> (CanonicalTerminalEnvelopeV1, CanonicalTerminalEnvelopeV1) {
    let admitted = engine
        .admit_linked_module_closure(closure)
        .expect("complete linked fixture admits");
    let (request, _cancellation) = engine
        .new_admitted_program_request(&admitted, None)
        .expect("Engine issues one opaque request");
    let cli = submit_cli_admitted_program(engine, &request)
        .await
        .expect("CLI adapter executes the request");
    let daemon = submit_daemon_admitted_program(engine, &request)
        .await
        .expect("daemon adapter executes the request");
    (cli, daemon)
}

#[tokio::test]
async fn file_and_inline_module_units_share_one_client_terminal() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) = client_terminals(
        &engine,
        linked_closure(ModuleArtifactOrigin::File("fixtures/shared.ash".to_owned())),
    )
    .await;
    let (inline_cli, inline_daemon) = client_terminals(
        &engine,
        linked_closure(ModuleArtifactOrigin::Inline {
            parent: ModuleKey::root("task_2064").expect("fixture root key"),
            declaration_offset: 19,
        }),
    )
    .await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
    );
}

#[tokio::test]
async fn real_parser_checked_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) = client_terminals(&engine, source_linked_closure(false)).await;
    let (inline_cli, inline_daemon) = client_terminals(&engine, source_linked_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
    );
}

#[tokio::test]
async fn public_role_and_policy_imports_remain_metadata_only_on_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_metadata_stub_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_metadata_stub_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_checked_let_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) = client_terminals(&engine, source_linked_let_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_let_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
    );
}

#[tokio::test]
async fn real_parser_checked_if_let_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_if_let_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_if_let_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
    );
}

#[tokio::test]
async fn real_parser_checked_short_circuit_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_short_circuit_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_short_circuit_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
    );
}

#[tokio::test]
async fn real_parser_checked_short_circuit_let_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_short_circuit_let_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_short_circuit_let_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
    );
}

#[tokio::test]
async fn real_parser_checked_short_circuit_if_let_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_short_circuit_if_let_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_short_circuit_if_let_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
    );
}

#[tokio::test]
async fn real_parser_checked_short_circuit_argument_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_short_circuit_argument_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_short_circuit_argument_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(0))
    );
}

#[tokio::test]
async fn real_parser_checked_short_circuit_match_argument_file_and_inline_routes_reach_both_clients()
 {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) = client_terminals(
        &engine,
        source_linked_short_circuit_match_argument_closure(false),
    )
    .await;
    let (inline_cli, inline_daemon) = client_terminals(
        &engine,
        source_linked_short_circuit_match_argument_closure(true),
    )
    .await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(0))
    );
}

#[tokio::test]
async fn real_parser_checked_nested_short_circuit_argument_file_and_inline_routes_reach_both_clients()
 {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) = client_terminals(
        &engine,
        source_linked_nested_short_circuit_argument_closure(false),
    )
    .await;
    let (inline_cli, inline_daemon) = client_terminals(
        &engine,
        source_linked_nested_short_circuit_argument_closure(true),
    )
    .await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(0))
    );
}

#[tokio::test]
async fn real_parser_checked_record_short_circuit_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_record_short_circuit_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_record_short_circuit_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(0))
    );
}

#[tokio::test]
async fn real_parser_checked_nested_short_circuit_boolean_file_and_inline_routes_reach_both_clients()
 {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) = client_terminals(
        &engine,
        source_linked_nested_short_circuit_boolean_closure(false),
    )
    .await;
    let (inline_cli, inline_daemon) = client_terminals(
        &engine,
        source_linked_nested_short_circuit_boolean_closure(true),
    )
    .await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
    );
}

#[tokio::test]
async fn real_parser_checked_ordinary_root_route_reaches_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (cli, daemon) = client_terminals(&engine, source_linked_ordinary_root_closure()).await;

    assert_eq!(cli, daemon);
    assert_eq!(cli, CanonicalTerminalEnvelopeV1::returned(Value::Int(42)));
}

#[tokio::test]
async fn real_parser_checked_modulo_root_route_reaches_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (cli, daemon) = client_terminals(&engine, source_linked_modulo_root_closure()).await;

    assert_eq!(cli, daemon);
    assert_eq!(cli, CanonicalTerminalEnvelopeV1::returned(Value::Int(1)));
}

#[tokio::test]
async fn real_parser_checked_record_field_call_root_route_reaches_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (cli, daemon) =
        client_terminals(&engine, source_linked_record_field_call_root_closure()).await;

    assert_eq!(cli, daemon);
    assert_eq!(cli, CanonicalTerminalEnvelopeV1::returned(Value::Int(41)));
}

#[tokio::test]
async fn real_parser_checked_nested_record_field_call_root_route_reaches_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (cli, daemon) = client_terminals(
        &engine,
        source_linked_nested_record_field_call_root_closure(),
    )
    .await;

    assert_eq!(cli, daemon);
    assert_eq!(cli, CanonicalTerminalEnvelopeV1::returned(Value::Int(41)));
}

#[tokio::test]
async fn real_parser_checked_record_field_expression_call_root_route_reaches_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (cli, daemon) = client_terminals(
        &engine,
        source_linked_record_field_expression_call_root_closure(),
    )
    .await;

    assert_eq!(cli, daemon);
    assert_eq!(cli, CanonicalTerminalEnvelopeV1::returned(Value::Int(41)));
}

#[tokio::test]
async fn real_parser_declaration_order_variants_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let sources = [
        "crate app; fn helper() -> Int { 40 } fn main() -> Int { helper() + 1 }",
        "crate app; fn main() -> Int { helper() + 1 } fn helper() -> Int { 40 }",
    ];

    let mut terminals = Vec::new();
    for source in sources {
        let (cli, daemon) =
            client_terminals(&engine, source_linked_declaration_order_closure(source)).await;
        assert_eq!(cli, daemon);
        terminals.push(cli);
    }

    assert_eq!(terminals[0], terminals[1]);
    assert_eq!(
        terminals[0],
        CanonicalTerminalEnvelopeV1::returned(Value::Int(41))
    );
}

#[tokio::test]
async fn real_parser_checked_imported_callable_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) = client_terminals(
        &engine,
        source_linked_closure_with_root_body(false, "remote()"),
    )
    .await;
    let (inline_cli, inline_daemon) = client_terminals(
        &engine,
        source_linked_closure_with_root_body(true, "remote()"),
    )
    .await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(2))
    );
}

#[tokio::test]
async fn real_parser_parameterized_imported_callable_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_parameterized_callable_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_parameterized_callable_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
    );
}

#[tokio::test]
async fn real_parser_multiple_imported_callables_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) = client_terminals(
        &engine,
        source_linked_multiple_imported_callable_closure(false),
    )
    .await;
    let (inline_cli, inline_daemon) = client_terminals(
        &engine,
        source_linked_multiple_imported_callable_closure(true),
    )
    .await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
    );
}

#[tokio::test]
async fn real_parser_crate_visible_callable_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_crate_visible_callable_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_crate_visible_callable_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(2))
    );
}

#[tokio::test]
async fn real_parser_super_visible_callable_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_super_visible_callable_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_super_visible_callable_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(2))
    );
}

#[tokio::test]
async fn real_parser_restricted_visible_callable_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) = client_terminals(
        &engine,
        source_linked_restricted_visible_callable_closure(false),
    )
    .await;
    let (inline_cli, inline_daemon) = client_terminals(
        &engine,
        source_linked_restricted_visible_callable_closure(true),
    )
    .await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(2))
    );
}

#[tokio::test]
async fn real_parser_checked_main_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_main_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_main_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(2))
    );
}

#[tokio::test]
async fn real_parser_checked_transitive_import_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_transitive_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_transitive_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_checked_reexport_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_reexport_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_reexport_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(2))
    );
}

#[tokio::test]
async fn real_parser_checked_transitive_reexport_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_transitive_reexport_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_transitive_reexport_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(2))
    );
}

#[tokio::test]
async fn real_parser_checked_nested_structural_callable_file_and_inline_routes_reach_both_clients()
{
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) = client_terminals(
        &engine,
        source_linked_nested_structural_callable_closure(false),
    )
    .await;
    let (inline_cli, inline_daemon) = client_terminals(
        &engine,
        source_linked_nested_structural_callable_closure(true),
    )
    .await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(2))
    );
}

#[tokio::test]
async fn real_parser_structural_module_alias_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) = client_terminals(
        &engine,
        source_linked_structural_module_alias_closure(false),
    )
    .await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_structural_module_alias_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(2))
    );
}

#[tokio::test]
async fn real_parser_nested_structural_module_alias_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) = client_terminals(
        &engine,
        source_linked_nested_structural_module_alias_closure(false),
    )
    .await;
    let (inline_cli, inline_daemon) = client_terminals(
        &engine,
        source_linked_nested_structural_module_alias_closure(true),
    )
    .await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(2))
    );
}

#[tokio::test]
async fn real_canonical_source_nested_reexport_reaches_both_clients() {
    let temporary = tempfile::tempdir().expect("temporary canonical source directory exists");
    let file_path = temporary.path().join("main.ash");
    let file_source = "pub mod api; pub mod facade; pub mod client; use crate::client::entry as remote; fn main() -> Int { remote() }";
    fs::write(&file_path, file_source).expect("write canonical file-backed root");
    fs::write(temporary.path().join("api.ash"), "pub mod nested;")
        .expect("write canonical api child");
    fs::write(
        temporary.path().join("nested.ash"),
        "pub type Marker = Int; pub fn serve() -> Int { 2 }",
    )
    .expect("write canonical nested child");
    fs::write(
        temporary.path().join("facade.ash"),
        "pub use crate::api::nested::serve as forwarded; pub use crate::api::nested::Marker;",
    )
    .expect("write canonical facade child");
    fs::write(
        temporary.path().join("client.ash"),
        "use crate::facade::forwarded; use crate::facade::Marker; pub fn typed(value: Marker) -> Int { forwarded() } pub fn entry() -> Int { forwarded() }",
    )
    .expect("write canonical client child");

    let inline_path = temporary.path().join("inline.ash");
    let inline_source = "pub mod api { pub mod nested { pub type Marker = Int; pub fn serve() -> Int { 2 } } } pub mod facade { pub use crate::api::nested::serve as forwarded; pub use crate::api::nested::Marker; } pub mod client { use crate::facade::forwarded; use crate::facade::Marker; pub fn typed(value: Marker) -> Int { forwarded() } pub fn entry() -> Int { forwarded() } } use crate::client::entry as remote; fn main() -> Int { remote() }";
    fs::write(&inline_path, inline_source).expect("write canonical inline root");

    let engine = Engine::new().build().expect("Engine builds");
    let file_closure = engine
        .canonical_module_closure_from_source(&file_path, file_source, "main")
        .expect("file source reaches canonical closure")
        .expect("file source contains structural modules");
    let inline_closure = engine
        .canonical_module_closure_from_source(&inline_path, inline_source, "main")
        .expect("inline source reaches canonical closure")
        .expect("inline source contains structural modules");

    let (file_cli, file_daemon) = client_terminals(&engine, file_closure).await;
    let (inline_cli, inline_daemon) = client_terminals(&engine, inline_closure).await;
    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(2))
    );
}

#[tokio::test]
async fn real_parser_public_type_reexport_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_type_reexport_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_type_reexport_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_explicit_notation_import_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_notation_import_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_notation_import_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_public_implementation_metadata_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) = client_terminals(
        &engine,
        source_linked_implementation_metadata_closure(false),
    )
    .await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_implementation_metadata_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_imported_interface_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_interface_import_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_interface_import_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_imported_type_signature_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_type_import_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_type_import_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_imported_newtype_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_newtype_import_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_newtype_import_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_imported_resource_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_resource_import_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_resource_import_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_imported_effect_row_signature_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_effect_row_import_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_effect_row_import_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_imported_type_function_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_type_function_import_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_type_function_import_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_imported_promoted_kind_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_promoted_kind_import_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_promoted_kind_import_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_imported_proposition_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_proposition_import_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_proposition_import_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_imported_evidence_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_evidence_import_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_evidence_import_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_imported_macro_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_macro_import_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_macro_import_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(2))
    );
}

#[tokio::test]
async fn real_parser_imported_constructor_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_constructor_import_closure(false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_constructor_import_closure(true)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(1))
    );
}

#[tokio::test]
async fn real_parser_same_module_alias_file_and_inline_routes_reach_both_clients() {
    let engine = Engine::new().build().expect("Engine builds");
    let (file_cli, file_daemon) =
        client_terminals(&engine, source_linked_self_alias_closure(false, false)).await;
    let (inline_cli, inline_daemon) =
        client_terminals(&engine, source_linked_self_alias_closure(true, false)).await;

    assert_eq!(file_cli, file_daemon);
    assert_eq!(inline_cli, inline_daemon);
    assert_eq!(file_cli, inline_cli);
    assert_eq!(
        file_cli,
        CanonicalTerminalEnvelopeV1::returned(Value::Int(42))
    );
}

#[test]
fn recursive_same_module_alias_to_selected_entry_rejects_as_callable_cycle() {
    let engine = Engine::new().build().expect("Engine builds");
    let error = engine
        .admit_linked_module_closure(source_linked_self_alias_closure(false, true))
        .expect_err("recursive selected-entry alias must fail before client execution");

    assert!(
        error
            .to_string()
            .contains("callable linking encountered a cycle"),
        "recursive selected-entry alias should use the callable-cycle fence: {error}"
    );
}

#[test]
fn real_source_structure_rejects_missing_duplicate_and_cyclic_children() {
    let cases = [
        (
            "missing",
            "pub mod missing; fn root() -> Int { 42 }",
            Vec::<(&str, &str)>::new(),
        ),
        (
            "duplicate",
            "pub mod api; pub mod api; fn root() -> Int { 42 }",
            Vec::<(&str, &str)>::new(),
        ),
        (
            "cycle",
            "pub mod a; fn root() -> Int { 42 }",
            vec![("a.ash", "pub mod b;"), ("b.ash", "pub mod a;")],
        ),
    ];

    for (label, root_source, children) in cases {
        let fixture_root = std::env::temp_dir().join(format!(
            "ash-task-2064-structure-{label}-{}",
            std::process::id()
        ));
        fs::create_dir_all(fixture_root.join("src")).expect("create structure fixture");
        let root_path = fixture_root.join("src/main.ash");
        fs::write(&root_path, root_source).expect("write structure root");
        for (name, source) in children {
            fs::write(fixture_root.join("src").join(name), source).expect("write structure child");
        }
        let root = ModuleKey::root("structure").expect("structure root key");
        let result = CanonicalModuleGraphResolver::new().resolve_root(root, &root_path);
        assert!(result.is_err(), "{label} structural input must reject");
        let _ = fs::remove_dir_all(fixture_root);
    }
}

#[test]
fn real_source_visibility_rejects_private_import_before_lowering() {
    let fixture_root =
        std::env::temp_dir().join(format!("ash-task-2064-visibility-{}", std::process::id()));
    fs::create_dir_all(fixture_root.join("src")).expect("create visibility fixture");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "pub mod api { fn hidden() -> Int { 1 } } use crate::api::hidden; fn root() -> Int { 42 }",
    )
    .expect("write visibility fixture");

    let root = ModuleKey::root("visibility").expect("visibility root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root, &root_path)
            .expect("visibility fixture resolves through canonical graph"),
    )
    .expect("visibility fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("visibility fixture collection succeeds");
    let result = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection);

    assert!(
        result.is_err(),
        "an inherited private child declaration must not cross the module boundary"
    );
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn real_source_ambiguous_glob_import_rejects_before_finalization() {
    let fixture_root =
        std::env::temp_dir().join(format!("ash-task-2064-ambiguity-{}", std::process::id()));
    fs::create_dir_all(fixture_root.join("src")).expect("create ambiguity fixture");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "pub mod left { pub type Marker = Int; } pub mod right { pub type Marker = Int; } pub mod client { use crate::left::*; use crate::right::*; fn entry() -> Int { 1 } } fn root() -> Int { 1 }",
    )
    .expect("write ambiguity fixture");

    let root = ModuleKey::root("ambiguity").expect("ambiguity root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root, &root_path)
            .expect("ambiguity fixture resolves through canonical graph"),
    )
    .expect("ambiguity fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("ambiguity fixture collection succeeds");
    let error = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect_err("glob imports with the same visible name must reject atomically");

    assert!(
        error
            .to_string()
            .contains("duplicate parsed import binding")
            || error.to_string().contains("ambiguous parsed import path"),
        "ambiguity rejection should retain the import diagnostic family: {error}"
    );
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn real_source_invalid_public_reexport_rejects_private_provider_boundary() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-invalid-reexport-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create invalid re-export fixture");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "pub mod provider { type Hidden = Int; } pub mod facade { pub use crate::provider::Hidden; } fn root() -> Int { 1 }",
    )
    .expect("write invalid re-export fixture");

    let root = ModuleKey::root("invalid_reexport").expect("invalid re-export root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root, &root_path)
            .expect("invalid re-export fixture resolves structurally"),
    )
    .expect("invalid re-export fixture expands structurally");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("invalid re-export fixture collection succeeds");
    let error = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect_err("a public re-export cannot cross a private provider boundary");

    assert!(
        error
            .to_string()
            .contains("inaccessible parsed import path"),
        "invalid public re-export should retain the visibility diagnostic family: {error}"
    );
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn real_source_public_module_does_not_implicitly_flatten_child_exports() {
    let fixture_root = std::env::temp_dir().join(format!(
        "ash-task-2064-no-implicit-flattening-{}",
        std::process::id()
    ));
    fs::create_dir_all(fixture_root.join("src")).expect("create flattening fixture");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "pub mod provider { pub fn serve() -> Int { 1 } } use crate::serve; fn main() -> Int { 1 }",
    )
    .expect("write flattening fixture");

    let root = ModuleKey::root("no_implicit_flattening").expect("flattening root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root, &root_path)
            .expect("flattening fixture resolves structurally"),
    )
    .expect("flattening fixture expands structurally");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("flattening fixture collection succeeds");
    let error = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect_err("pub mod must not implicitly flatten the child's declarations");

    assert!(
        error.to_string().contains("unresolved parsed import path"),
        "implicit flattening should reject as an unresolved parent path: {error}"
    );
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn real_source_import_cycle_rejects_before_finalization() {
    let fixture_root =
        std::env::temp_dir().join(format!("ash-task-2064-import-cycle-{}", std::process::id()));
    fs::create_dir_all(fixture_root.join("src")).expect("create import-cycle fixture");
    let root_path = fixture_root.join("src/main.ash");
    fs::write(
        &root_path,
        "pub mod a { pub fn local() -> Int { 1 } use crate::b::value; } pub mod b { pub fn value() -> Int { 2 } use crate::a::local; } fn root() -> Int { 42 }",
    )
    .expect("write import-cycle fixture");

    let root = ModuleKey::root("import_cycle").expect("import-cycle root key");
    let expanded = CanonicalExpandedModuleGraph::try_expand(
        CanonicalModuleGraphResolver::new()
            .resolve_root(root, &root_path)
            .expect("import-cycle fixture resolves through canonical graph"),
    )
    .expect("import-cycle fixture expands");
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("import-cycle fixture collection succeeds");
    let result = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection);

    assert!(
        result.is_err(),
        "a cross-module parsed import cycle must reject atomically"
    );
    let _ = fs::remove_dir_all(fixture_root);
}

#[test]
fn incomplete_linked_conformance_rejects_before_either_client_is_reachable() {
    let (root, root_input, _child_input) =
        linked_inputs(ModuleArtifactOrigin::File("fixtures/shared.ash".to_owned()));
    let engine = Engine::new().build().expect("Engine builds");
    assert!(
        engine
            .admit_linked_module_closure(LinkedModuleClosure::new(root, vec![root_input]))
            .is_err(),
        "an incomplete linked closure must reject before either client is reachable"
    );
}

#[test]
fn provenance_mutation_rejects_before_client_submission() {
    let (root, root_input, child_input) =
        linked_inputs(ModuleArtifactOrigin::File("fixtures/shared.ash".to_owned()));
    let forged_artifact = ModuleArtifact::new(
        root.child("shared").expect("fixture child key"),
        ModuleArtifactOrigin::File("fixtures/forged-shared.ash".to_owned()),
        Some(root.clone()),
        Vec::new(),
    )
    .expect("forged artifact remains structurally shaped");
    let forged_cps = ModuleCpsArtifact::new(forged_artifact, Vec::new(), literal_cps(0));
    let mutated_child = LinkedModuleArtifactInput::new(
        child_input.interface().clone(),
        child_input.core().expect("child Core exists").clone(),
        forged_cps,
        source_anchor("task_2064-mutated-child"),
    );
    let engine = Engine::new().build().expect("Engine builds");
    assert!(
        engine
            .admit_linked_module_closure(LinkedModuleClosure::new(
                root,
                vec![mutated_child, root_input],
            ))
            .is_err(),
        "a mutated defining origin must reject before client submission"
    );
}

proptest! {
    #[test]
    fn inline_source_metadata_mutation_preserves_normalized_terminal(offset in 0usize..4096) {
        let engine = Engine::new().build().expect("Engine builds");
        let closure = linked_closure(ModuleArtifactOrigin::Inline {
            parent: ModuleKey::root("task_2064").expect("fixture root key"),
            declaration_offset: offset,
        });
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("property runtime builds");
        let (cli, daemon) = runtime.block_on(client_terminals(&engine, closure));
        prop_assert_eq!(cli.clone(), daemon);
        prop_assert_eq!(cli, CanonicalTerminalEnvelopeV1::returned(Value::Int(42)));
    }
}
