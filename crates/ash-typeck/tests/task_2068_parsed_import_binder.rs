//! TASK-2068 RED contracts for canonical parsed-use binding.
//!
//! These tests define only the first bounded TASK-2068 slice: collect parser
//! graph declarations provisionally, then bind explicit parsed `crate::…`
//! aliases.  They deliberately do not authorize final interfaces, bodies,
//! re-exports, Core/CPS lowering, or an Engine route.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::surface::{Definition, Visibility};
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver};
use ash_typeck::{
    CanonicalModuleBindError, bind_simple_parsed_uses, resolve_simple_parsed_imports,
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
            "ash-task-2068-{label}-{}-{serial}",
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

fn parsed_graph(source: &str, label: &str) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("fixture source must resolve through the canonical parser graph");
    (root_key, graph)
}

fn file_inline_import_graph(
    file_a_source: &str,
    inline_b_source: &str,
    label: &str,
) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write(
        "src/main.ash",
        &format!("mod a; mod b {{ {inline_b_source} }}"),
    );
    tree.write("src/a.ash", file_a_source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("paired file/inline fixture must resolve through the canonical parser graph");
    (root_key, graph)
}

fn file_inline_tail_cycle_graph(
    file_a_source: &str,
    inline_b_source: &str,
    file_c_source: &str,
    label: &str,
) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write(
        "src/main.ash",
        &format!("mod a; mod b {{ {inline_b_source} }} mod c;"),
    );
    tree.write("src/a.ash", file_a_source);
    tree.write("src/c.ash", file_c_source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("tail-cycle file/inline fixture must resolve through the canonical parser graph");
    (root_key, graph)
}

fn function_span(graph: &CanonicalModuleGraph, module: &ModuleKey, name: &str) -> ash_parser::Span {
    let unit = graph
        .module_unit(module)
        .expect("fixture module has an acquired canonical unit");
    unit.body()
        .definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == name => Some(function.span),
            _ => None,
        })
        .expect("fixture function remains in the parser-owned module unit")
}

fn first_use_span(graph: &CanonicalModuleGraph, module: &ModuleKey) -> ash_parser::Span {
    graph
        .module_unit(module)
        .expect("fixture module has an acquired canonical unit")
        .body()
        .uses()
        .first()
        .expect("fixture module contains one parsed use declaration")
        .span
}

#[test]
fn explicit_parsed_alias_preserves_the_provisional_function_identity_and_parser_anchors() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub fn greet() -> Int { 7 }
                fn secret() -> Int { 13 }
            }
            use crate::api::greet as welcome;
            fn entry() -> Int { welcome() }
        "#,
        "explicit-alias",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture child key is canonical");
    let greet_span = function_span(&graph, &api_key, "greet");
    let api_origin = graph
        .module_unit(&api_key)
        .expect("fixture API unit is retained by the parser graph")
        .artifact()
        .origin()
        .clone();
    let use_span = first_use_span(&graph, &root_key);

    let plan = resolve_simple_parsed_imports(&graph)
        .expect("the canonical planner resolves a public explicit parsed alias");
    let bindings = bind_simple_parsed_uses(&graph)
        .expect("the compatibility binder delegates the public alias to the canonical planner");
    let welcome = plan
        .binding(&root_key, "welcome")
        .expect("the plan retains the root's explicit alias binding");
    let edge = plan
        .import_edges()
        .first()
        .expect("an inter-module explicit import produces one provenance-preserving edge");

    assert_eq!(
        welcome.defining_identity().module_key(),
        &api_key,
        "an alias preserves the declaration's canonical defining module"
    );
    assert_eq!(
        welcome.defining_identity().name(),
        "greet",
        "an alias never rewrites the declaration's defining name"
    );
    assert_eq!(
        welcome.declaration_span(),
        greet_span,
        "the binding retains the parser-owned declaration anchor"
    );
    assert_eq!(
        welcome.origin(),
        &api_origin,
        "the binding retains the source-acquisition provenance of its definition"
    );
    assert_eq!(
        welcome.visibility(),
        &Visibility::Public,
        "the provisional binding retains the parser visibility for later checks"
    );
    assert!(
        bindings.binding(&root_key, "greet").is_none(),
        "a public child declaration must not be flattened into its parent absent an explicit use"
    );
    assert_eq!(
        bindings.binding(&root_key, "welcome"),
        Some(welcome),
        "the compatibility binder returns the same resolved binding as the planner"
    );
    assert_eq!(plan.import_edges().len(), 1);
    assert_eq!(edge.importing_module(), &root_key);
    assert_eq!(edge.defining_module(), &api_key);
    assert_eq!(edge.defining_identity().module_key(), &api_key);
    assert_eq!(edge.defining_identity().name(), "greet");
    assert_eq!(edge.local_name(), "welcome");
    assert_eq!(edge.use_span(), use_span);
    assert_eq!(edge.declaration_span(), greet_span);
    assert_eq!(edge.origin(), &api_origin);
    assert_eq!(edge.visibility(), &Visibility::Public);
}

#[test]
fn same_module_simple_import_binds_without_creating_an_import_dependency_edge() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub fn f() -> Int { 7 }
            use crate::f as g;
        "#,
        "same-module-import",
    );

    let plan = resolve_simple_parsed_imports(&graph)
        .expect("a same-module public simple import is resolved within the canonical root");
    let delegated = bind_simple_parsed_uses(&graph)
        .expect("the compatibility binder delegates a same-module import to the planner");
    let binding = plan
        .binding(&root_key, "g")
        .expect("the same-module alias is retained in the resolved plan");

    assert!(
        plan.import_edges().is_empty(),
        "an import whose defining module is the importer adds no graph dependency edge"
    );
    assert_eq!(binding.defining_identity().module_key(), &root_key);
    assert_eq!(binding.defining_identity().name(), "f");
    assert_eq!(delegated.binding(&root_key, "g"), Some(binding));
}

#[test]
fn private_explicit_import_reports_an_anchored_inaccessible_error_without_a_partial_binding_set() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                fn greet() -> Int { 7 }
            }
            use crate::api::greet as welcome;
        "#,
        "private-alias",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture child key is canonical");
    let greet_span = function_span(&graph, &api_key, "greet");

    let error = bind_simple_parsed_uses(&graph)
        .expect_err("a private declaration must reject the complete binding operation atomically");

    match error {
        CanonicalModuleBindError::Inaccessible {
            declaration_span,
            defining_module,
            attempted_path,
            violated_visibility,
        } => {
            assert_eq!(
                declaration_span, greet_span,
                "the inaccessible diagnostic must point to the private declaration"
            );
            assert_eq!(
                defining_module, api_key,
                "the inaccessible diagnostic retains canonical defining identity"
            );
            assert_eq!(
                attempted_path,
                vec!["crate".into(), "api".into(), "greet".into()],
                "the inaccessible diagnostic preserves the parsed access path"
            );
            assert_eq!(
                violated_visibility,
                Visibility::Inherited,
                "the diagnostic reports the exact parser visibility boundary"
            );
        }
        CanonicalModuleBindError::Unresolved { .. } => {
            panic!("a resolved private declaration must never be downgraded to an unresolved name")
        }
        other => panic!("expected anchored inaccessible diagnostic, got {other:?}"),
    }
}

#[test]
fn public_use_is_rejected_as_an_unsupported_reexport_before_any_binding_set_is_published() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub fn greet() -> Int { 7 }
            }
            pub use crate::api::greet as welcome;
        "#,
        "public-use",
    );
    let use_span = graph
        .module_unit(&root_key)
        .expect("fixture root unit is retained by the parser graph")
        .body()
        .uses()
        .first()
        .expect("fixture has one parsed public use declaration")
        .span;

    let error = bind_simple_parsed_uses(&graph).expect_err(
        "a public use is a re-export and must reject atomically outside TASK-2068's first slice",
    );

    match error {
        CanonicalModuleBindError::Unsupported { span, reason } => {
            assert_eq!(
                span, use_span,
                "the unsupported re-export diagnostic retains the parsed use anchor"
            );
            assert!(
                !reason.is_empty(),
                "the unsupported re-export diagnostic supplies a stable reason"
            );
        }
        other => panic!("expected unsupported public-use diagnostic, got {other:?}"),
    }
}

#[test]
fn crate_visible_declaration_is_rejected_as_unsupported_until_restricted_visibility_is_owned() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub(crate) fn greet() -> Int { 7 }
            }
            use crate::api::greet as welcome;
        "#,
        "crate-visible-alias",
    );
    let use_span = graph
        .module_unit(&root_key)
        .expect("fixture root unit is retained by the parser graph")
        .body()
        .uses()
        .first()
        .expect("fixture has one parsed inherited use declaration")
        .span;

    let error = bind_simple_parsed_uses(&graph).expect_err(
        "pub(crate) visibility is outside the bounded slice and cannot publish a partial binding set",
    );

    match error {
        CanonicalModuleBindError::Unsupported { span, reason } => {
            assert_eq!(
                span, use_span,
                "the unsupported restricted-visibility diagnostic retains the parsed use anchor"
            );
            assert!(
                !reason.is_empty(),
                "the unsupported restricted-visibility diagnostic supplies a stable reason"
            );
        }
        CanonicalModuleBindError::Inaccessible { .. } => {
            panic!("unsupported restricted visibility must not be misreported as inaccessible")
        }
        other => panic!("expected unsupported restricted-visibility diagnostic, got {other:?}"),
    }
}

#[test]
fn a_late_private_import_rejects_the_whole_module_without_publishing_an_earlier_alias() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub fn greet() -> Int { 7 }
                fn secret() -> Int { 13 }
            }
            use crate::api::greet as welcome;
            use crate::api::secret as hidden;
        "#,
        "late-private-import",
    );
    let api_key = root_key
        .child("api")
        .expect("fixture child key is canonical");
    let secret_span = function_span(&graph, &api_key, "secret");

    let error = bind_simple_parsed_uses(&graph).expect_err(
        "a late inaccessible import must reject atomically instead of publishing the earlier alias",
    );

    match error {
        CanonicalModuleBindError::Inaccessible {
            declaration_span,
            defining_module,
            attempted_path,
            violated_visibility,
        } => {
            assert_eq!(declaration_span, secret_span);
            assert_eq!(defining_module, api_key);
            assert_eq!(
                attempted_path,
                vec!["crate".into(), "api".into(), "secret".into()]
            );
            assert_eq!(violated_visibility, Visibility::Inherited);
        }
        other => panic!("expected inaccessible late import diagnostic, got {other:?}"),
    }
}

#[test]
fn file_and_inline_simple_import_cycle_is_rejected_with_ordered_parser_anchored_edges() {
    let (root_key, graph) = file_inline_import_graph(
        r#"
            use crate::b::b_fn as b_value;
            pub fn a_fn() -> Int { 1 }
        "#,
        r#"
            use crate::a::a_fn as a_value;
            pub fn b_fn() -> Int { 2 }
        "#,
        "file-inline-cycle",
    );
    let a_key = root_key
        .child("a")
        .expect("fixture file child key is canonical");
    let b_key = root_key
        .child("b")
        .expect("fixture inline child key is canonical");
    let a_use_span = first_use_span(&graph, &a_key);
    let b_use_span = first_use_span(&graph, &b_key);

    let planner_error = resolve_simple_parsed_imports(&graph)
        .expect_err("a file/inline import cycle must reject before a plan is published");
    let delegated_error = bind_simple_parsed_uses(&graph)
        .expect_err("the compatibility binder must not bypass planner cycle rejection");

    assert_eq!(
        delegated_error, planner_error,
        "the compatibility binder reports the same cycle failure as the canonical planner"
    );
    match planner_error {
        CanonicalModuleBindError::ImportCycle { edges } => {
            assert_eq!(edges.len(), 2, "the cycle contains exactly a -> b -> a");
            assert_eq!(edges[0].importing_module(), &a_key);
            assert_eq!(edges[0].defining_module(), &b_key);
            assert_eq!(edges[0].use_span(), a_use_span);
            assert_eq!(edges[1].importing_module(), &b_key);
            assert_eq!(edges[1].defining_module(), &a_key);
            assert_eq!(edges[1].use_span(), b_use_span);
        }
        other => panic!("expected ordered canonical import-cycle diagnostic, got {other:?}"),
    }
}

#[test]
fn tail_cycle_reports_only_the_ordered_cycle_edges_with_full_definition_provenance() {
    let (root_key, graph) = file_inline_tail_cycle_graph(
        r#"
            use crate::b::b_fn as b_value;
            pub fn a_fn() -> Int { 1 }
        "#,
        r#"
            use crate::c::c_fn as c_value;
            pub fn b_fn() -> Int { 2 }
        "#,
        r#"
            use crate::b::b_fn as b_value;
            pub fn c_fn() -> Int { 3 }
        "#,
        "tail-cycle",
    );
    let a_key = root_key
        .child("a")
        .expect("fixture file child key is canonical");
    let b_key = root_key
        .child("b")
        .expect("fixture inline child key is canonical");
    let c_key = root_key
        .child("c")
        .expect("fixture file child key is canonical");
    let b_use_span = first_use_span(&graph, &b_key);
    let c_use_span = first_use_span(&graph, &c_key);
    let c_declaration_span = function_span(&graph, &c_key, "c_fn");
    let b_declaration_span = function_span(&graph, &b_key, "b_fn");
    let c_origin = graph
        .module_unit(&c_key)
        .expect("fixture c unit is graph-owned")
        .artifact()
        .origin()
        .clone();
    let b_origin = graph
        .module_unit(&b_key)
        .expect("fixture b unit is graph-owned")
        .artifact()
        .origin()
        .clone();

    let error = resolve_simple_parsed_imports(&graph)
        .expect_err("the tail a -> b must not hide the actual b -> c -> b import cycle");

    match error {
        CanonicalModuleBindError::ImportCycle { edges } => {
            let edges = edges.edges();
            assert_eq!(edges.len(), 2, "only the b -> c -> b cycle is reported");
            assert_eq!(edges[0].importing_module(), &b_key);
            assert_eq!(edges[0].defining_module(), &c_key);
            assert_eq!(edges[0].use_span(), b_use_span);
            assert_eq!(edges[0].declaration_span(), c_declaration_span);
            assert_eq!(edges[0].origin(), &c_origin);
            assert_eq!(edges[0].visibility(), &Visibility::Public);
            assert_eq!(edges[1].importing_module(), &c_key);
            assert_eq!(edges[1].defining_module(), &b_key);
            assert_eq!(edges[1].use_span(), c_use_span);
            assert_eq!(edges[1].declaration_span(), b_declaration_span);
            assert_eq!(edges[1].origin(), &b_origin);
            assert_eq!(edges[1].visibility(), &Visibility::Public);
            assert!(
                edges.iter().all(|edge| edge.importing_module() != &a_key),
                "the non-cyclic a -> b tail cannot be reported as a cycle edge"
            );
        }
        other => panic!("expected full-provenance canonical tail-cycle diagnostic, got {other:?}"),
    }
}

#[test]
fn adding_a_back_edge_to_an_otherwise_acyclic_file_inline_import_graph_rejects_atomically() {
    let (_, acyclic_graph) = file_inline_import_graph(
        r#"
            use crate::b::b_fn as b_value;
            pub fn a_fn() -> Int { 1 }
        "#,
        "pub fn b_fn() -> Int { 2 }",
        "acyclic-before-mutation",
    );
    let acyclic_plan = resolve_simple_parsed_imports(&acyclic_graph)
        .expect("the initial one-way file-to-inline import graph is acyclic");
    assert_eq!(acyclic_plan.import_edges().len(), 1);

    let (root_key, mutated_graph) = file_inline_import_graph(
        r#"
            use crate::b::b_fn as b_value;
            pub fn a_fn() -> Int { 1 }
        "#,
        r#"
            use crate::a::a_fn as a_value;
            pub fn b_fn() -> Int { 2 }
        "#,
        "back-edge-mutation",
    );
    let a_key = root_key
        .child("a")
        .expect("fixture file child key is canonical");
    let b_key = root_key
        .child("b")
        .expect("fixture inline child key is canonical");
    let a_use_span = first_use_span(&mutated_graph, &a_key);
    let b_use_span = first_use_span(&mutated_graph, &b_key);

    let planner_error = resolve_simple_parsed_imports(&mutated_graph)
        .expect_err("the added b -> a edge must reject before any mutated plan is published");
    let delegated_error = bind_simple_parsed_uses(&mutated_graph)
        .expect_err("the compatibility binder must preserve atomic back-edge rejection");

    assert_eq!(delegated_error, planner_error);
    match planner_error {
        CanonicalModuleBindError::ImportCycle { edges } => {
            assert_eq!(edges.len(), 2);
            assert_eq!(edges[0].importing_module(), &a_key);
            assert_eq!(edges[0].defining_module(), &b_key);
            assert_eq!(edges[0].use_span(), a_use_span);
            assert_eq!(edges[1].importing_module(), &b_key);
            assert_eq!(edges[1].defining_module(), &a_key);
            assert_eq!(edges[1].use_span(), b_use_span);
        }
        other => {
            panic!("expected atomic import-cycle rejection after back-edge mutation, got {other:?}")
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn generated_explicit_aliases_preserve_the_same_defining_identity(
        suffix in "[a-z0-9_]{0,12}",
    ) {
        let alias = format!("alias_{suffix}");
        let source = format!(
            "pub mod api {{ pub fn greet() -> Int {{ 7 }} }} use crate::api::greet as {alias};"
        );
        let (root_key, graph) = parsed_graph(&source, "generated-alias");
        let api_key = root_key.child("api").expect("fixture child key is canonical");

        let bindings = bind_simple_parsed_uses(&graph)
            .expect("every generated lowercase explicit alias is parser-valid and public");
        let binding = bindings
            .binding(&root_key, &alias)
            .expect("the generated alias must be published after successful atomic binding");

        prop_assert_eq!(binding.defining_identity().module_key(), &api_key);
        prop_assert_eq!(binding.defining_identity().name(), "greet");
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
fn canonical_simple_import_planner_and_compatibility_binder_have_no_authority_or_bypass() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let planner_path = manifest_dir.join("src/canonical_simple_import_planner.rs");
    let planner_source = fs::read_to_string(&planner_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated canonical simple-import planner at {}: {error}",
            planner_path.display()
        )
    });
    let binder_path = manifest_dir.join("src/canonical_module_binder.rs");
    let binder_source = fs::read_to_string(&binder_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 retains a compatibility binder delegating at {}: {error}",
            binder_path.display()
        )
    });

    for forbidden_identifier in [
        "ModuleGraph",
        "ModuleIdentity",
        "ModuleId",
        "LegacyModuleResolver",
        "ModuleResolver",
        "NameBinder",
        "PublicModuleInterface",
        "FinalizedModuleInterface",
        "InterfaceImportResolver",
        "CheckedInterfaceStore",
        "ModuleUnitResolver",
        "TypeEnv",
        "RawCoreProgram",
        "CoreExpr",
        "CpsProgram",
        "Core",
        "Cps",
        "CPS",
        "Engine",
    ] {
        assert!(
            !contains_exact_identifier(&planner_source, forbidden_identifier),
            "the canonical simple-import planner must not depend on legacy, identity, final-interface, checker, lowering, acquisition, or runtime authority: {forbidden_identifier}"
        );
    }
    for forbidden_bypass in [
        "interface_import_resolver",
        "module_interface_finalization",
        "module_core_cps_lowering",
        "std::fs",
        "read_to_string",
        "parse_surface",
        "resolve_root",
        "from_legacy",
        "into_legacy",
    ] {
        assert!(
            !planner_source.contains(forbidden_bypass),
            "the canonical simple-import planner must consume only CanonicalModuleGraph, ModuleKey, and typed parser payloads: {forbidden_bypass}"
        );
    }
    assert!(
        contains_exact_identifier(&binder_source, "resolve_simple_parsed_imports"),
        "bind_simple_parsed_uses must delegate to the canonical simple-import planner"
    );
    for independent_resolver in [
        "fn resolve_use",
        "fn collect_provisional_functions",
        "fn is_accessible_from",
    ] {
        assert!(
            !binder_source.contains(independent_resolver),
            "the compatibility binder must not retain an independent import resolver: {independent_resolver}"
        );
    }
}
