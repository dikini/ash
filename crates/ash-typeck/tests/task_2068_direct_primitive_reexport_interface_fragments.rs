//! TASK-2068 RED contracts for direct primitive public re-export fragments.
//!
//! This is the only opt-in public re-export route. The generic inherited-use
//! planner and compatibility binder remain fail-closed for `pub use`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::module::ModuleDecl;
use ash_parser::surface::{Definition, FnDef, Visibility};
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver};
use ash_typeck::{
    CanonicalDirectPrimitiveInterfaceImportError, CanonicalPrimitiveInterfaceError, Type,
    check_direct_primitive_interface_fragments, resolve_direct_primitive_interface_imports,
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
            "ash-task-2068-reexport-fragments-{label}-{}-{serial}",
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

fn inline_provider_graph(
    module_visibility: &str,
    provider_body: &str,
    root_body: &str,
    label: &str,
) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write(
        "src/main.ash",
        &format!("{module_visibility} mod api {{ {provider_body} }} {root_body}"),
    );
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("inline provider fixture must resolve through the canonical parser graph");
    (root_key, graph)
}

fn file_provider_graph(
    root_source: &str,
    provider_source: &str,
    label: &str,
) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", root_source);
    tree.write("src/api.ash", provider_source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("file-backed provider fixture must resolve through the canonical parser graph");
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

fn root_module_declaration<'a>(
    graph: &'a CanonicalModuleGraph,
    root_key: &ModuleKey,
    name: &str,
) -> &'a ModuleDecl {
    graph
        .module_unit(root_key)
        .expect("fixture root unit is graph-owned")
        .body()
        .module_decls()
        .iter()
        .find(|declaration| declaration.name.as_ref() == name)
        .expect("fixture direct child declaration remains parser-owned")
}

fn first_use_span(graph: &CanonicalModuleGraph, module: &ModuleKey) -> ash_parser::Span {
    graph
        .module_unit(module)
        .expect("fixture module has an acquired canonical unit")
        .body()
        .uses()
        .first()
        .expect("fixture module contains a parsed public use declaration")
        .span
}

fn int_to_int() -> Type {
    Type::Fn(vec![Type::Int], Box::new(Type::Int))
}

#[test]
fn direct_public_primitive_reexport_builds_export_closed_fragments_without_flattening() {
    let (root_key, graph) = inline_provider_graph(
        "pub",
        "pub fn greet(value: Int) -> Int { value + 1 }",
        "pub use crate::api::greet as welcome;",
        "positive-inline",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture direct child key is canonical");
    let root_artifact = graph
        .module_unit(&root_key)
        .expect("root artifact is graph-owned")
        .artifact()
        .clone();
    let api_artifact = graph
        .module_unit(&api_key)
        .expect("provider artifact is graph-owned")
        .artifact()
        .clone();
    let api_declaration = root_module_declaration(&graph, &root_key, "api");
    let greet = function(&graph, &api_key, "greet");
    let use_span = first_use_span(&graph, &root_key);
    let plan = resolve_direct_primitive_interface_imports(&graph)
        .expect("the opt-in public direct re-export planner accepts the bounded positive route");

    let fragments = check_direct_primitive_interface_fragments(&graph, &plan)
        .expect("the direct primitive public re-export builds an export-closed fragment");
    let child = fragments
        .public_child("api")
        .expect("the fragment retains the explicitly public direct child");
    let reexport = fragments
        .reexport("welcome")
        .expect("the fragment retains the one explicit root-visible re-export");

    assert_eq!(fragments.root_artifact(), &root_artifact);
    assert_eq!(child.module_key(), &api_key);
    assert_eq!(child.declaration_span(), api_declaration.span);
    assert_eq!(child.origin(), api_artifact.origin());
    assert_eq!(child.visibility(), &Visibility::Public);
    assert_eq!(reexport.visible_name(), "welcome");
    assert_eq!(reexport.defining_identity().module_key(), &api_key);
    assert_eq!(reexport.defining_identity().name(), "greet");
    assert_eq!(reexport.declaration_span(), greet.span);
    assert_eq!(reexport.origin(), api_artifact.origin());
    assert_eq!(reexport.signature(), &int_to_int());
    assert_eq!(reexport.use_span(), use_span);
    assert_eq!(reexport.visibility(), &Visibility::Public);
    assert!(
        fragments.reexport("greet").is_none(),
        "pub mod api exposes only its child identity; it cannot flatten api::greet into root"
    );
}

#[test]
fn direct_public_planner_rejects_implicit_reexport_name_before_any_plan_is_published() {
    let (root_key, graph) = inline_provider_graph(
        "pub",
        "pub fn greet(value: Int) -> Int { value + 1 }",
        "pub use crate::api::greet;",
        "implicit-reexport-name",
    );
    let use_span = first_use_span(&graph, &root_key);

    let error = resolve_direct_primitive_interface_imports(&graph).expect_err(
        "an implicit re-export name is outside the opt-in direct primitive fragment and publishes no plan",
    );

    match error {
        CanonicalDirectPrimitiveInterfaceImportError::Unsupported { span, reason } => {
            assert_eq!(span, use_span);
            assert_eq!(reason, "an explicit re-export alias is required");
        }
        other => panic!("expected anchored implicit-name rejection before planning, got {other:?}"),
    }
}

#[test]
fn nonpublic_direct_structural_path_rejects_before_fragment_publication() {
    let (root_key, graph) = inline_provider_graph(
        "",
        "pub fn greet(value: Int) -> Int { value + 1 }",
        "pub use crate::api::greet as welcome;",
        "private-path",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture direct child key is canonical");
    let declaration_span = root_module_declaration(&graph, &root_key, "api").span;

    let error = resolve_direct_primitive_interface_imports(&graph).expect_err(
        "a private direct structural path cannot be published through a public re-export",
    );

    match error {
        CanonicalDirectPrimitiveInterfaceImportError::NonPublicStructuralPath {
            root_module,
            child_module,
            declaration_span: actual_span,
        } => {
            assert_eq!(root_module, root_key);
            assert_eq!(child_module, api_key);
            assert_eq!(actual_span, declaration_span);
        }
        other => panic!("expected anchored non-public structural-path rejection, got {other:?}"),
    }
}

#[test]
fn private_direct_reexport_target_rejects_with_its_declaration_anchor() {
    let (root_key, graph) = inline_provider_graph(
        "pub",
        "fn greet(value: Int) -> Int { value + 1 }",
        "pub use crate::api::greet as welcome;",
        "private-target",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture direct child key is canonical");
    let greet = function(&graph, &api_key, "greet");

    let error = resolve_direct_primitive_interface_imports(&graph)
        .expect_err("a private target cannot be published through a public direct re-export");

    match error {
        CanonicalDirectPrimitiveInterfaceImportError::PrivateTarget {
            defining_module,
            declaration_span,
            ..
        } => {
            assert_eq!(defining_module, api_key);
            assert_eq!(declaration_span, greet.span);
        }
        other => panic!("expected anchored private direct target rejection, got {other:?}"),
    }
}

#[test]
fn nonprimitive_direct_reexport_target_is_rejected_before_publication() {
    let (_, graph) = inline_provider_graph(
        "pub",
        "pub fn greet(value: Option<Int>) -> Option<Int> { value }",
        "pub use crate::api::greet as welcome;",
        "nonprimitive-target",
    );
    let plan = resolve_direct_primitive_interface_imports(&graph)
        .expect("direct-public planning is structural and preserves the selected public target");

    let error = check_direct_primitive_interface_fragments(&graph, &plan)
        .expect_err("a nonprimitive target is outside the primitive interface-fragment slice");

    assert!(matches!(
        error,
        CanonicalPrimitiveInterfaceError::NonPrimitiveTarget { .. }
    ));
}

#[test]
fn direct_public_planner_rejects_root_without_any_public_reexport() {
    let (root_key, graph) = inline_provider_graph(
        "pub",
        "pub fn greet(value: Int) -> Int { value + 1 }",
        "",
        "empty-root-public-reexport",
    );
    let root_body_span = graph
        .module_unit(&root_key)
        .expect("fixture root unit is graph-owned")
        .body()
        .span();

    let error = resolve_direct_primitive_interface_imports(&graph).expect_err(
        "an export-closed direct primitive fragment requires one root public re-export",
    );

    match error {
        CanonicalDirectPrimitiveInterfaceImportError::MissingPublicReexport {
            root_module,
            span,
        } => {
            assert_eq!(root_module, root_key);
            assert_eq!(span, root_body_span);
        }
        other => panic!("expected anchored empty root public-reexport rejection, got {other:?}"),
    }
}

#[test]
fn root_definition_outside_direct_reexport_domain_rejects_before_fragment_publication() {
    let (root_key, graph) = inline_provider_graph(
        "pub",
        "pub fn greet(value: Int) -> Int { value + 1 }",
        r#"
            pub fn outside(value: Option<Int>) -> Option<Int> { value }
            pub use crate::api::greet as welcome;
        "#,
        "root-shape-rejection",
    );
    let outside = function(&graph, &root_key, "outside");
    let plan = resolve_direct_primitive_interface_imports(&graph)
        .expect("the structural direct public plan precedes exact root-shape validation");

    let error = check_direct_primitive_interface_fragments(&graph, &plan).expect_err(
        "a root definition outside the exact direct re-export fragment must reject atomically",
    );

    match error {
        CanonicalPrimitiveInterfaceError::UnsupportedRootShape {
            root_module, span, ..
        } => {
            assert_eq!(root_module, root_key);
            assert_eq!(span, outside.span);
        }
        other => panic!("expected anchored root-shape rejection, got {other:?}"),
    }
}

#[test]
fn root_public_child_identity_and_reexport_alias_collision_rejects_before_publication() {
    let (root_key, graph) = inline_provider_graph(
        "pub",
        "pub fn greet(value: Int) -> Int { value + 1 }",
        "pub use crate::api::greet as api;",
        "child-alias-collision",
    );
    let child_declaration_span = root_module_declaration(&graph, &root_key, "api").span;
    let use_span = first_use_span(&graph, &root_key);
    let plan = resolve_direct_primitive_interface_imports(&graph)
        .expect("the structural direct public plan precedes root namespace collision validation");

    let error = check_direct_primitive_interface_fragments(&graph, &plan)
        .expect_err("a public re-export alias cannot overwrite the retained public child identity");

    match error {
        CanonicalPrimitiveInterfaceError::RootVisibleChildCollision {
            root_module,
            visible_name,
            child_declaration_span: actual_child_declaration_span,
            use_span: collision_use_span,
        } => {
            assert_eq!(root_module, root_key);
            assert_eq!(visible_name, "api");
            assert_eq!(actual_child_declaration_span, child_declaration_span);
            assert_eq!(collision_use_span, use_span);
        }
        other => panic!("expected anchored public child identity alias collision, got {other:?}"),
    }
}

#[test]
fn root_public_reexport_name_collision_rejects_before_fragment_publication() {
    let (root_key, graph) = inline_provider_graph(
        "pub",
        "pub fn greet(value: Int) -> Int { value + 1 }",
        r#"
            pub fn welcome(value: Int) -> Int { value }
            pub use crate::api::greet as welcome;
        "#,
        "root-visible-collision",
    );
    let welcome = function(&graph, &root_key, "welcome");
    let use_span = first_use_span(&graph, &root_key);
    let plan = resolve_direct_primitive_interface_imports(&graph)
        .expect("the planner records a public re-export before root visible-name publication");

    let error = check_direct_primitive_interface_fragments(&graph, &plan).expect_err(
        "a root-visible re-export cannot overwrite an existing public root declaration",
    );

    match error {
        CanonicalPrimitiveInterfaceError::RootVisibleNameCollision {
            root_module,
            visible_name,
            local_declaration_span,
            use_span: collision_use_span,
        } => {
            assert_eq!(root_module, root_key);
            assert_eq!(visible_name, "welcome");
            assert_eq!(local_declaration_span, welcome.span);
            assert_eq!(collision_use_span, use_span);
        }
        other => panic!("expected atomic root visible-name collision, got {other:?}"),
    }
}

#[test]
fn plan_from_a_same_key_different_artifact_is_rejected_before_fragment_checking() {
    let (_, planned_graph) = file_provider_graph(
        "pub mod api; pub use crate::api::greet as welcome;",
        "pub fn greet(value: Int) -> Int { value + 1 }",
        "planned-file-artifact",
    );
    let plan = resolve_direct_primitive_interface_imports(&planned_graph)
        .expect("the original file-backed provider graph produces a direct public plan");
    let (root_key, changed_graph) = file_provider_graph(
        "pub mod api; pub use crate::api::greet as welcome;",
        "pub fn greet(value: Int) -> Int { value + 2 }",
        "changed-file-artifact",
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
        "the graphs share ModuleKey identity while retaining source-specific artifacts"
    );

    let error = check_direct_primitive_interface_fragments(&changed_graph, &plan)
        .expect_err("a direct public plan cannot be replayed against another artifact snapshot");

    assert!(matches!(
        error,
        CanonicalPrimitiveInterfaceError::PlanArtifactMismatch { .. }
    ));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn generated_direct_public_aliases_preserve_identity_provenance_signature_and_use_span(
        suffix in "[a-z0-9_]{0,12}",
    ) {
        let alias = format!("alias_{suffix}");
        let root = format!("pub use crate::api::greet as {alias};");
        let (root_key, graph) = inline_provider_graph(
            "pub",
            "pub fn greet(value: Int) -> Int { value + 1 }",
            &root,
            "generated-public-alias",
        );
        let api_key = root_key.child("api").expect("fixture child key is canonical");
        let provider_origin = graph
            .module_unit(&api_key)
            .expect("provider unit is graph-owned")
            .artifact()
            .origin()
            .clone();
        let use_span = first_use_span(&graph, &root_key);
        let plan = resolve_direct_primitive_interface_imports(&graph)
            .expect("every generated direct public alias is parser-valid and planned");
        let fragments = check_direct_primitive_interface_fragments(&graph, &plan)
            .expect("every generated direct public alias builds a primitive fragment");
        let reexport = fragments
            .reexport(&alias)
            .expect("the generated root-visible alias is retained exactly once");

        prop_assert_eq!(reexport.defining_identity().module_key(), &api_key);
        prop_assert_eq!(reexport.defining_identity().name(), "greet");
        prop_assert_eq!(reexport.origin(), &provider_origin);
        prop_assert_eq!(reexport.signature(), &int_to_int());
        prop_assert_eq!(reexport.use_span(), use_span);
    }
}

#[test]
fn late_invalid_public_reexport_rejects_atomically() {
    let (root_key, graph) = inline_provider_graph(
        "pub",
        r#"
            pub fn greet(value: Int) -> Int { value + 1 }
            pub fn exotic(value: Option<Int>) -> Option<Int> { value }
        "#,
        r#"
            pub use crate::api::greet as early;
            pub use crate::api::exotic as late;
        "#,
        "late-invalid-reexport",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture child key is canonical");
    let exotic = function(&graph, &api_key, "exotic");
    let plan = resolve_direct_primitive_interface_imports(&graph).expect(
        "the public structural/visibility plan stages both re-exports before primitive checking",
    );

    let error = check_direct_primitive_interface_fragments(&graph, &plan)
        .expect_err("a late invalid target must return only an error, never an earlier fragment");

    match error {
        CanonicalPrimitiveInterfaceError::NonPrimitiveTarget {
            defining_module,
            declaration_span,
            ..
        } => {
            assert_eq!(defining_module, api_key);
            assert_eq!(declaration_span, exotic.span);
        }
        other => panic!("expected atomic late invalid re-export rejection, got {other:?}"),
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
fn direct_primitive_interface_fragment_checker_has_no_compatibility_or_runtime_authority() {
    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/canonical_primitive_interface_fragments.rs");
    let source = fs::read_to_string(&source_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated direct primitive interface fragment checker at {}: {error}",
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
        "Default",
        "eval",
        "evaluate",
        "execute",
    ] {
        assert!(
            !contains_exact_identifier(&source, forbidden_identifier),
            "the direct primitive interface fragment checker must not depend on compatibility, final-interface, evaluator, lowering, or runtime authority: {forbidden_identifier}"
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
            "the fragment checker may consume only canonical graph, opt-in planner, primitive checked facts, and ordinary Type/provenance data: {forbidden_bypass}"
        );
    }
    assert!(
        !source.contains("pub fn new("),
        "the immutable fragment result must not expose a public constructor"
    );
}
