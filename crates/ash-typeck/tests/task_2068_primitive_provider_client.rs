//! TASK-2068 RED contracts for primitive provider/client checking.
//!
//! This slice consumes a canonical parser graph and its already-resolved simple
//! import plan. It checks one direct provider/client relationship without
//! recreating import resolution or granting final-interface, lowering, or
//! runtime authority.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::surface::{Definition, FnDef, Visibility};
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver, Spanned};
use ash_typeck::{
    CanonicalPrimitiveProviderClientError, Type, check_primitive_provider_client,
    resolve_simple_parsed_imports,
};
use proptest::prelude::*;

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

/// A real parser fixture whose drop implementation removes its source tree.
struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ash-task-2068-provider-client-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create temporary parser fixture tree");
        Self { root }
    }

    fn write(&self, relative: impl AsRef<Path>, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture path has a parent"))
            .expect("create fixture parent directory");
        fs::write(&path, source).expect("write parser fixture source");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn provider_client_graph(
    provider_body: &str,
    root_body: &str,
    label: &str,
) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write(
        "src/main.ash",
        &format!("pub mod api {{ {provider_body} }} {root_body}"),
    );
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("provider/client fixture must resolve through the canonical parser graph");
    (root_key, graph)
}

fn function<'a>(graph: &'a CanonicalModuleGraph, module: &ModuleKey, name: &str) -> &'a FnDef {
    graph
        .module_unit(module)
        .expect("fixture module has an acquired canonical unit")
        .body()
        .definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == name => Some(function),
            _ => None,
        })
        .expect("fixture function remains in its parser-owned module unit")
}

fn first_use_span(graph: &CanonicalModuleGraph, module: &ModuleKey) -> ash_parser::Span {
    graph
        .module_unit(module)
        .expect("fixture module has an acquired canonical unit")
        .body()
        .uses()
        .first()
        .expect("fixture module contains a parsed use declaration")
        .span
}

fn int_to_int() -> Type {
    Type::Fn(vec![Type::Int], Box::new(Type::Int))
}

#[test]
fn primitive_direct_provider_client_checks_against_the_resolved_plan_and_retains_all_provenance() {
    let (root_key, graph) = provider_client_graph(
        "pub fn greet(value: Int) -> Int { value + 1 }",
        r#"
            use crate::api::greet as welcome;
            pub fn entry(value: Int) -> Int { welcome(value) }
        "#,
        "positive",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture provider key is canonical");
    let root_artifact = graph
        .module_unit(&root_key)
        .expect("root artifact is graph-owned")
        .artifact()
        .clone();
    let provider_artifact = graph
        .module_unit(&api_key)
        .expect("provider artifact is graph-owned")
        .artifact()
        .clone();
    let greet = function(&graph, &api_key, "greet");
    let entry = function(&graph, &root_key, "entry");
    let use_span = first_use_span(&graph, &root_key);
    let plan = resolve_simple_parsed_imports(&graph)
        .expect("the public provider import must resolve before provider/client checking");

    let checked = check_primitive_provider_client(&graph, &plan)
        .expect("a direct primitive provider/client call checks against the resolved plan");
    let root = checked.root_module();
    let provider = checked
        .provider_module(&api_key)
        .expect("the directly imported provider is retained under its canonical key");
    let binding = checked
        .import_binding("welcome")
        .expect("the checked client retains the resolved imported binding");
    let root_entry = root
        .function("entry")
        .expect("the checked root retains its client function");
    let provider_greet = provider
        .function("greet")
        .expect("the checked provider retains its public function");

    assert_eq!(root.artifact(), &root_artifact);
    assert_eq!(root.artifact().key(), &root_key);
    assert_eq!(provider.artifact(), &provider_artifact);
    assert_eq!(provider.artifact().key(), &api_key);
    assert_eq!(provider_greet.defining_identity().module_key(), &api_key);
    assert_eq!(provider_greet.defining_identity().name(), "greet");
    assert_eq!(provider_greet.declaration_span(), greet.span);
    assert_eq!(provider_greet.body_span(), greet.body.span());
    assert_eq!(provider_greet.signature(), &int_to_int());
    assert_eq!(provider_greet.body_type(), &Type::Int);
    assert_eq!(root_entry.defining_identity().module_key(), &root_key);
    assert_eq!(root_entry.defining_identity().name(), "entry");
    assert_eq!(root_entry.declaration_span(), entry.span);
    assert_eq!(root_entry.body_span(), entry.body.span());
    assert_eq!(root_entry.signature(), &int_to_int());
    assert_eq!(root_entry.body_type(), &Type::Int);

    assert_eq!(checked.import_bindings().len(), 1);
    assert_eq!(binding.local_name(), "welcome");
    assert_eq!(binding.use_span(), use_span);
    assert_eq!(binding.defining_identity().module_key(), &api_key);
    assert_eq!(binding.defining_identity().name(), "greet");
    assert_eq!(binding.declaration_span(), greet.span);
    assert_eq!(binding.origin(), provider_artifact.origin());
    assert_eq!(binding.visibility(), &Visibility::Public);
    assert_eq!(binding.signature(), &int_to_int());
}

#[test]
fn nonprimitive_provider_signature_is_rejected_before_client_checking() {
    let (root_key, graph) = provider_client_graph(
        "pub fn greet(value: Option<Int>) -> Option<Int> { value }",
        r#"
            use crate::api::greet as welcome;
            pub fn entry(value: Int) -> Int { welcome(value) }
        "#,
        "nonprimitive-provider",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture provider key is canonical");
    let greet = function(&graph, &api_key, "greet");
    let plan = resolve_simple_parsed_imports(&graph)
        .expect("planner syntax collection remains independent of provider signature checking");

    let error = check_primitive_provider_client(&graph, &plan)
        .expect_err("a nonprimitive provider signature is outside the primitive provider slice");

    match error {
        CanonicalPrimitiveProviderClientError::ProviderSignatureOutsidePrimitiveSlice {
            provider_module,
            function,
            declaration_span,
        } => {
            assert_eq!(provider_module, api_key);
            assert_eq!(function, "greet");
            assert_eq!(declaration_span, greet.span);
        }
        other => panic!("expected primitive-provider signature boundary error, got {other:?}"),
    }
}

#[test]
fn incompatible_provider_result_is_reported_as_an_anchored_client_body_error() {
    let (root_key, graph) = provider_client_graph(
        "pub fn greet(value: Int) -> Bool { true }",
        r#"
            use crate::api::greet as welcome;
            pub fn entry(value: Int) -> Int { welcome(value) }
        "#,
        "provider-result-mutation",
    );
    let entry = function(&graph, &root_key, "entry");
    let plan = resolve_simple_parsed_imports(&graph)
        .expect("the provider result mutation preserves an otherwise valid parsed import plan");

    let error = check_primitive_provider_client(&graph, &plan)
        .expect_err("a Bool provider result cannot satisfy the client's declared Int body");

    match error {
        CanonicalPrimitiveProviderClientError::ClientBodyCheck {
            root_module,
            function,
            declaration_span,
            ..
        } => {
            assert_eq!(root_module, root_key);
            assert_eq!(function, "entry");
            assert_eq!(declaration_span, entry.span);
        }
        other => panic!("expected anchored client body-check error, got {other:?}"),
    }
}

#[test]
fn plan_from_a_same_key_different_artifact_is_rejected_before_provider_client_checking() {
    let (_, planned_graph) = provider_client_graph(
        "pub fn greet(value: Int) -> Int { value + 1 }",
        r#"
            use crate::api::greet as welcome;
            pub fn entry(value: Int) -> Int { welcome(value) }
        "#,
        "planned-artifact",
    );
    let plan = resolve_simple_parsed_imports(&planned_graph)
        .expect("the original graph produces a resolved import plan");
    let (root_key, changed_graph) = provider_client_graph(
        "pub fn greet(value: Int) -> Int { value + 2 }",
        r#"
            use crate::api::greet as welcome;
            pub fn entry(value: Int) -> Int { welcome(value) }
        "#,
        "changed-artifact",
    );
    assert_eq!(changed_graph.root_key(), &root_key);
    assert_ne!(
        planned_graph
            .module_unit(planned_graph.root_key())
            .expect("planned root unit exists")
            .artifact(),
        changed_graph
            .module_unit(&root_key)
            .expect("changed root unit exists")
            .artifact(),
        "the graphs intentionally share ModuleKey identity while retaining distinct source artifacts"
    );

    let error = check_primitive_provider_client(&changed_graph, &plan)
        .expect_err("a plan cannot be replayed against a same-key graph with different artifacts");

    assert!(matches!(
        error,
        CanonicalPrimitiveProviderClientError::PlannerArtifactMismatch { .. }
    ));
}

#[test]
fn root_local_name_collision_with_an_import_alias_rejects_before_environment_overwrite() {
    let (root_key, graph) = provider_client_graph(
        "pub fn greet(value: Int) -> Int { value + 1 }",
        r#"
            fn welcome(value: Int) -> Int { value }
            use crate::api::greet as welcome;
            pub fn entry(value: Int) -> Int { welcome(value) }
        "#,
        "local-import-collision",
    );
    let welcome = function(&graph, &root_key, "welcome");
    let use_span = first_use_span(&graph, &root_key);
    let plan = resolve_simple_parsed_imports(&graph)
        .expect("a parser plan can carry a colliding alias before client environment admission");

    let error = check_primitive_provider_client(&graph, &plan)
        .expect_err("a root local declaration must not be overwritten by an import alias");

    match error {
        CanonicalPrimitiveProviderClientError::LocalImportCollision {
            root_module,
            local_name,
            local_declaration_span,
            use_span: collision_use_span,
        } => {
            assert_eq!(root_module, root_key);
            assert_eq!(local_name, "welcome");
            assert_eq!(local_declaration_span, welcome.span);
            assert_eq!(collision_use_span, use_span);
        }
        other => panic!("expected pre-environment local/import collision, got {other:?}"),
    }
}

#[test]
fn provider_imports_are_rejected_as_an_unsupported_provider_shape() {
    let (root_key, graph) = provider_client_graph(
        r#"
            use crate::api::greet as self_greet;
            pub fn greet(value: Int) -> Int { value + 1 }
        "#,
        r#"
            use crate::api::greet as welcome;
            pub fn entry(value: Int) -> Int { welcome(value) }
        "#,
        "provider-import",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture provider key is canonical");
    let provider_use_span = first_use_span(&graph, &api_key);
    let plan = resolve_simple_parsed_imports(&graph)
        .expect("the parsed self-import remains planner-valid until provider-shape admission");

    let error = check_primitive_provider_client(&graph, &plan)
        .expect_err("provider imports are outside the direct primitive provider shape");

    match error {
        CanonicalPrimitiveProviderClientError::UnsupportedProviderShape {
            provider_module,
            span,
            reason,
        } => {
            assert_eq!(provider_module, api_key);
            assert_eq!(span, provider_use_span);
            assert!(!reason.is_empty());
        }
        other => panic!("expected provider-import shape rejection, got {other:?}"),
    }
}

#[test]
fn provider_deep_child_is_rejected_as_an_unsupported_provider_shape() {
    let (root_key, graph) = provider_client_graph(
        r#"
            mod nested { pub fn hidden() -> Int { 0 } }
            pub fn greet(value: Int) -> Int { value + 1 }
        "#,
        r#"
            use crate::api::greet as welcome;
            pub fn entry(value: Int) -> Int { welcome(value) }
        "#,
        "provider-deep-child",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture provider key is canonical");
    let child_declaration_span = graph
        .module_unit(&api_key)
        .expect("provider unit is graph-owned")
        .body()
        .module_decls()
        .first()
        .expect("fixture provider contains a nested declaration")
        .span;
    let plan = resolve_simple_parsed_imports(&graph)
        .expect("the root/provider simple import is independent of the provider child shape");

    let error = check_primitive_provider_client(&graph, &plan)
        .expect_err("deep provider children are outside the direct primitive provider shape");

    match error {
        CanonicalPrimitiveProviderClientError::UnsupportedProviderShape {
            provider_module,
            span,
            reason,
        } => {
            assert_eq!(provider_module, api_key);
            assert_eq!(span, child_declaration_span);
            assert!(!reason.is_empty());
        }
        other => panic!("expected provider-child shape rejection, got {other:?}"),
    }
}

#[test]
fn late_broken_root_body_rejects_the_whole_provider_client_result_atomically() {
    let (root_key, graph) = provider_client_graph(
        "pub fn greet(value: Int) -> Int { value + 1 }",
        r#"
            use crate::api::greet as welcome;
            pub fn entry(value: Int) -> Int { welcome(value) }
            fn broken() -> Int { true }
        "#,
        "late-root-body-failure",
    );
    let broken = function(&graph, &root_key, "broken");
    let plan = resolve_simple_parsed_imports(&graph)
        .expect("the late root body failure preserves a valid parsed import plan");

    let error = check_primitive_provider_client(&graph, &plan)
        .expect_err("a late root body failure must not publish a partial provider/client result");

    match error {
        CanonicalPrimitiveProviderClientError::ClientBodyCheck {
            root_module,
            function,
            declaration_span,
            ..
        } => {
            assert_eq!(root_module, root_key);
            assert_eq!(function, "broken");
            assert_eq!(declaration_span, broken.span);
        }
        other => panic!("expected atomic late client body-check error, got {other:?}"),
    }
}

#[test]
fn unselected_nested_module_rejects_the_direct_provider_client_topology() {
    let (_, graph) = provider_client_graph(
        "pub fn greet(value: Int) -> Int { value + 1 }",
        r#"
            mod spare { mod deep { pub fn ignored() -> Int { 0 } } }
            use crate::api::greet as welcome;
            pub fn entry(value: Int) -> Int { welcome(value) }
        "#,
        "unselected-nested-topology",
    );
    let plan = resolve_simple_parsed_imports(&graph)
        .expect("the selected direct api import remains planner-valid before topology admission");

    let error = check_primitive_provider_client(&graph, &plan).expect_err(
        "an unrelated nested module is outside the root-client plus selected-direct-provider domain",
    );

    match error {
        CanonicalPrimitiveProviderClientError::InvalidTopology { reason } => {
            assert!(
                reason.contains("selected")
                    && reason.contains("direct")
                    && reason.contains("provider"),
                "the topology diagnostic must explain that only selected direct providers are admitted: {reason}"
            );
        }
        other => panic!("expected unselected nested-module topology rejection, got {other:?}"),
    }
}

#[test]
fn out_of_domain_nested_topology_rejects_before_a_malformed_selected_provider_is_checked() {
    let (_, graph) = provider_client_graph(
        "pub fn greet(value: Option<Int>) -> Option<Int> { value }",
        r#"
            mod spare { mod deep { pub fn ignored() -> Int { 0 } } }
            use crate::api::greet as welcome;
            pub fn entry(value: Int) -> Int { welcome(value) }
        "#,
        "topology-before-provider-error",
    );
    let plan = resolve_simple_parsed_imports(&graph)
        .expect("the malformed provider signature does not prevent parser-plan construction");

    let error = check_primitive_provider_client(&graph, &plan).expect_err(
        "full-graph topology must reject before a selected provider emits a narrower shape error",
    );

    match error {
        CanonicalPrimitiveProviderClientError::InvalidTopology { reason } => {
            assert!(
                reason.contains("selected")
                    && reason.contains("direct")
                    && reason.contains("provider"),
                "the topology diagnostic must state the selected direct provider domain: {reason}"
            );
        }
        other => {
            panic!("expected global topology rejection before provider checking, got {other:?}")
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn generated_primitive_provider_client_values_preserve_binding_signature_and_checked_root(
        increment in 0i64..10_000,
        argument in 0i64..10_000,
    ) {
        let provider = format!("pub fn greet(value: Int) -> Int {{ value + {increment} }}");
        let root = format!(
            "use crate::api::greet as welcome; pub fn entry() -> Int {{ welcome({argument}) }}"
        );
        let (root_key, graph) = provider_client_graph(&provider, &root, "generated-primitive");
        let api_key = root_key.child("api").expect("fixture provider key is canonical");
        let plan = resolve_simple_parsed_imports(&graph)
            .expect("generated primitive provider import must resolve");

        let checked = check_primitive_provider_client(&graph, &plan)
            .expect("generated primitive provider/client pair must check");
        let binding = checked
            .import_binding("welcome")
            .expect("generated checked client retains the planned import");
        let entry = checked
            .root_module()
            .function("entry")
            .expect("generated checked root retains entry");

        prop_assert!(checked.provider_module(&api_key).is_some());
        prop_assert_eq!(binding.defining_identity().module_key(), &api_key);
        prop_assert_eq!(binding.defining_identity().name(), "greet");
        prop_assert_eq!(binding.signature(), &int_to_int());
        prop_assert_eq!(entry.body_type(), &Type::Int);
    }
}

fn contains_exact_identifier(source: &str, identifier: &str) -> bool {
    source.match_indices(identifier).any(|(start, _)| {
        let before_is_identifier = source[..start]
            .chars()
            .next_back()
            .is_some_and(|character| character == '_' || character.is_ascii_alphanumeric());
        let after_is_identifier = source[start + identifier.len()..]
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphanumeric());
        !before_is_identifier && !after_is_identifier
    })
}

#[test]
fn primitive_provider_client_checker_has_no_legacy_final_interface_binder_or_runtime_bypass() {
    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/canonical_primitive_provider_client.rs");
    let source = fs::read_to_string(&source_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated primitive provider/client checker at {}: {error}",
            source_path.display()
        )
    });

    for forbidden_identifier in [
        "ModuleIdentity",
        "ModuleId",
        "ModuleGraph",
        "LegacyModuleResolver",
        "ModuleResolver",
        "NameBinder",
        "bind_simple_parsed_uses",
        "CanonicalBoundModuleSet",
        "PublicModuleInterface",
        "FinalizedModuleInterface",
        "InterfaceImportResolver",
        "CheckedInterfaceStore",
        "TypeEnvModuleInterfaceCollection",
        "RawCoreProgram",
        "CoreExpr",
        "CpsProgram",
        "Engine",
    ] {
        assert!(
            !contains_exact_identifier(&source, forbidden_identifier),
            "the primitive provider/client checker must not depend on legacy, binder, final-interface, lowering, or runtime authority: {forbidden_identifier}"
        );
    }
    for forbidden_bypass in [
        "module_interface_finalization",
        "interface_import_resolver",
        "canonical_module_binder",
        "module_core_cps_lowering",
        "std::fs",
        "read_to_string",
        "parse_surface",
        "resolve_root",
        "from_legacy",
        "into_legacy",
    ] {
        assert!(
            !source.contains(forbidden_bypass),
            "the primitive provider/client checker must consume only a canonical graph, resolved planner facts, TypeEnv, and typed parser/checker payloads: {forbidden_bypass}"
        );
    }
    assert!(
        !source.contains("pub fn new("),
        "provider/client output constructors remain private to preserve checked provenance"
    );
}
