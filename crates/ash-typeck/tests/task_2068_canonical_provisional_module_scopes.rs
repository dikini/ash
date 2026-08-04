//! TASK-2068 RED contracts for canonical provisional module scopes.
//!
//! This target owns a deliberately narrow Type-layer route for inherited
//! `crate::` function aliases. It must derive structural scope facts from the
//! canonical parser graph before resolving every path edge and its visibility.
//! It is not a compatibility binder, public interface, lowering, admission,
//! or runtime route.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::module::ModuleDecl;
use ash_parser::surface::{Definition, FnDef, Visibility};
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver, Span};
use ash_typeck::{
    CanonicalProvisionalModuleScopes, CanonicalStructuralImportError,
    resolve_simple_parsed_imports_with_scopes,
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
            "ash-task-2068-provisional-scopes-{label}-{}-{serial}",
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

fn file_backed_graph(
    root_source: &str,
    api_source: &str,
    label: &str,
) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", root_source);
    tree.write("src/api.ash", api_source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("file-backed fixture source must resolve through the canonical parser graph");
    (root_key, graph)
}

fn file_inline_cycle_graph(
    file_a_source: &str,
    inline_b_source: &str,
    label: &str,
) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write(
        "src/main.ash",
        &format!("pub mod a; pub mod b {{ {inline_b_source} }}"),
    );
    tree.write("src/a.ash", file_a_source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("file/inline cycle fixture resolves through the canonical parser graph");
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
        &format!("pub mod a; pub mod b {{ {inline_b_source} }} pub mod c;"),
    );
    tree.write("src/a.ash", file_a_source);
    tree.write("src/c.ash", file_c_source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("file/inline tail-cycle fixture resolves through the canonical parser graph");
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
        .expect("fixture function remains parser-owned by its canonical module")
}

fn module_declaration<'a>(
    graph: &'a CanonicalModuleGraph,
    module: &ModuleKey,
    name: &str,
) -> &'a ModuleDecl {
    graph
        .module_unit(module)
        .expect("fixture parent module has an acquired canonical unit")
        .body()
        .module_decls()
        .iter()
        .find(|declaration| declaration.name.as_ref() == name)
        .expect("fixture structural module declaration remains parser-owned")
}

fn first_use_span(graph: &CanonicalModuleGraph, module: &ModuleKey) -> Span {
    graph
        .module_unit(module)
        .expect("fixture importing module has an acquired canonical unit")
        .body()
        .uses()
        .first()
        .expect("fixture importing module contains a parsed use declaration")
        .span
}

fn scopes(graph: &CanonicalModuleGraph) -> CanonicalProvisionalModuleScopes {
    CanonicalProvisionalModuleScopes::from_graph(graph)
        .expect("a structurally complete parser graph derives immutable provisional scopes")
}

fn resolve(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<ash_typeck::CanonicalResolvedSimpleImports, CanonicalStructuralImportError> {
    resolve_simple_parsed_imports_with_scopes(graph, scopes)
}

fn cycle_error(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> ash_typeck::CanonicalImportCycle {
    match resolve(graph, scopes).expect_err("the selected structural imports form a cycle") {
        CanonicalStructuralImportError::ImportCycle { edges } => edges,
        other => panic!("expected outer scoped structural cycle diagnostic, got {other:?}"),
    }
}

fn normalized_cycle_projection(
    cycle: &ash_typeck::CanonicalImportCycle,
) -> Vec<(ModuleKey, ModuleKey, String, String, String, Visibility)> {
    cycle
        .edges()
        .iter()
        .map(|edge| {
            (
                edge.importing_module().clone(),
                edge.defining_module().clone(),
                edge.defining_identity().module_key().to_string(),
                edge.defining_identity().name().to_string(),
                edge.local_name().to_string(),
                edge.visibility().clone(),
            )
        })
        .collect()
}

#[test]
fn multilevel_structural_import_preserves_canonical_definition_identity_and_parser_provenance() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub mod text {
                    pub fn normalize(value: Int) -> Int { value }
                }
            }
            use crate::api::text::normalize as normalize_text;
            fn entry(value: Int) -> Int { normalize_text(value) }
        "#,
        "positive-multilevel",
    );
    let api_key = root_key.child("api").expect("fixture key is canonical");
    let text_key = api_key.child("text").expect("fixture key is canonical");
    let target = function(&graph, &text_key, "normalize");
    let text_origin = graph
        .module_unit(&text_key)
        .expect("target unit is graph-owned")
        .artifact()
        .origin()
        .clone();
    let use_span = first_use_span(&graph, &root_key);
    let scopes = scopes(&graph);

    assert!(scopes.contains_module(&root_key));
    assert!(scopes.contains_module(&api_key));
    assert!(scopes.contains_module(&text_key));
    assert_eq!(
        graph.children(&root_key),
        Some([api_key.clone()].as_slice())
    );
    assert_eq!(
        graph.children(&api_key),
        Some([text_key.clone()].as_slice())
    );

    let plan = resolve(&graph, &scopes)
        .expect("the scope-backed route resolves real direct structural graph edges");
    let binding = plan
        .binding(&root_key, "normalize_text")
        .expect("the explicit local alias is staged only after the structural walk succeeds");
    let edge = plan
        .import_edges()
        .first()
        .expect("the cross-module structural alias retains one dependency edge");

    assert_eq!(binding.defining_identity().module_key(), &text_key);
    assert_eq!(binding.defining_identity().name(), "normalize");
    assert_eq!(binding.declaration_span(), target.span);
    assert_eq!(binding.origin(), &text_origin);
    assert_eq!(binding.visibility(), &Visibility::Public);
    assert_eq!(edge.importing_module(), &root_key);
    assert_eq!(edge.defining_module(), &text_key);
    assert_eq!(edge.defining_identity(), binding.defining_identity());
    assert_eq!(edge.local_name(), "normalize_text");
    assert_eq!(edge.declaration_span(), target.span);
    assert_eq!(edge.use_span(), use_span);
    assert_eq!(edge.origin(), &text_origin);
    assert_eq!(edge.visibility(), &Visibility::Public);
}

#[test]
fn stale_scopes_for_the_same_file_path_and_module_topology_reject_before_any_binding_is_staged() {
    let tree = TempTree::new("stale-scope-same-artifact");
    let root_path = tree.write(
        "src/main.ash",
        r#"
            pub mod api {
                pub fn normalize(value: Int) -> Int { value }
            }
            use crate::api::normalize as normalize_text;
        "#,
    );
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let resolver = CanonicalModuleGraphResolver::new();
    let old_graph = resolver
        .resolve_root(root_key.clone(), &root_path)
        .expect("the original source resolves through the canonical graph");
    let old_scopes = scopes(&old_graph);
    let api_key = root_key
        .child("api")
        .expect("fixture child key is canonical");

    tree.write(
        "src/main.ash",
        r#"
            pub mod api {
                pub fn replacement(value: Int) -> Int { value }
            }
            use crate::api::normalize as normalize_text;
        "#,
    );
    let new_graph = resolver
        .resolve_root(root_key.clone(), &root_path)
        .expect("the changed same-topology source resolves through the canonical graph");

    assert_eq!(
        old_graph
            .module_unit(&root_key)
            .expect("old root unit is graph-owned")
            .artifact(),
        new_graph
            .module_unit(&root_key)
            .expect("new root unit is graph-owned")
            .artifact(),
        "the stale-scope regression keeps the root artifact and file path unchanged"
    );
    assert_eq!(
        old_graph
            .module_unit(&api_key)
            .expect("old child unit is graph-owned")
            .artifact(),
        new_graph
            .module_unit(&api_key)
            .expect("new child unit is graph-owned")
            .artifact(),
        "the stale-scope regression keeps the child artifact and topology unchanged"
    );

    let error = resolve(&new_graph, &old_scopes).expect_err(
        "a scope snapshot must not authorize an updated graph merely because artifacts still match",
    );
    assert!(
        matches!(error, CanonicalStructuralImportError::ScopeGraphMismatch),
        "stale scope facts reject before an old normalize binding can be returned: {error:?}"
    );
}

#[test]
fn private_structural_child_blocks_a_public_function_before_the_alias_is_staged() {
    let (root_key, graph) = parsed_graph(
        r#"
            mod hidden {
                pub fn normalize(value: Int) -> Int { value }
            }
            pub mod client {
                use crate::hidden::normalize as normalize_text;
            }
        "#,
        "private-child-public-target",
    );
    let hidden_key = root_key.child("hidden").expect("fixture key is canonical");
    let client_key = root_key.child("client").expect("fixture key is canonical");
    let hidden_declaration = module_declaration(&graph, &root_key, "hidden");
    let use_span = first_use_span(&graph, &client_key);
    let scopes = scopes(&graph);

    let error = resolve(&graph, &scopes).expect_err(
        "a public function cannot bypass the inherited visibility of its structural child",
    );
    match error {
        CanonicalStructuralImportError::Inaccessible {
            declaration_span,
            use_span: rejected_use_span,
            defining_module,
            attempted_path,
            violated_visibility,
        } => {
            assert_eq!(declaration_span, hidden_declaration.span);
            assert_eq!(rejected_use_span, use_span);
            assert_eq!(defining_module, hidden_key);
            assert_eq!(
                attempted_path,
                vec!["crate".into(), "hidden".into(), "normalize".into()]
            );
            assert_eq!(violated_visibility, Visibility::Inherited);
        }
        other => panic!("expected inherited child visibility rejection, got {other:?}"),
    }
}

#[test]
fn two_module_structural_cycle_retains_ordered_canonical_edge_provenance() {
    let (root_key, graph) = file_inline_cycle_graph(
        r#"
            use crate::b::b_fn as from_b;
            pub fn a_fn() -> Int { 1 }
        "#,
        r#"
            use crate::a::a_fn as from_a;
            pub fn b_fn() -> Int { 2 }
        "#,
        "two-module-cycle",
    );
    let a_key = root_key.child("a").expect("fixture key is canonical");
    let b_key = root_key.child("b").expect("fixture key is canonical");
    let a_use_span = first_use_span(&graph, &a_key);
    let b_use_span = first_use_span(&graph, &b_key);
    let a_fn = function(&graph, &a_key, "a_fn");
    let b_fn = function(&graph, &b_key, "b_fn");
    let a_origin = graph
        .module_unit(&a_key)
        .expect("file module is graph-owned")
        .artifact()
        .origin()
        .clone();
    let b_origin = graph
        .module_unit(&b_key)
        .expect("inline module is graph-owned")
        .artifact()
        .origin()
        .clone();
    let scopes = scopes(&graph);

    let cycle = cycle_error(&graph, &scopes);
    let edges = cycle.edges();

    assert_eq!(
        edges.len(),
        2,
        "the diagnostic retains the closing two-edge cycle"
    );
    assert_eq!(edges[0].importing_module(), &a_key);
    assert_eq!(edges[0].defining_module(), &b_key);
    assert_eq!(edges[0].defining_identity().module_key(), &b_key);
    assert_eq!(edges[0].defining_identity().name(), "b_fn");
    assert_eq!(edges[0].local_name(), "from_b");
    assert_eq!(edges[0].use_span(), a_use_span);
    assert_eq!(edges[0].declaration_span(), b_fn.span);
    assert_eq!(edges[0].origin(), &b_origin);
    assert_eq!(edges[0].visibility(), &Visibility::Public);
    assert_eq!(edges[1].importing_module(), &b_key);
    assert_eq!(edges[1].defining_module(), &a_key);
    assert_eq!(edges[1].defining_identity().module_key(), &a_key);
    assert_eq!(edges[1].defining_identity().name(), "a_fn");
    assert_eq!(edges[1].local_name(), "from_a");
    assert_eq!(edges[1].use_span(), b_use_span);
    assert_eq!(edges[1].declaration_span(), a_fn.span);
    assert_eq!(edges[1].origin(), &a_origin);
    assert_eq!(edges[1].visibility(), &Visibility::Public);
}

#[test]
fn tail_dependency_cycle_reports_only_its_ordered_closing_edges() {
    let (root_key, graph) = file_inline_tail_cycle_graph(
        r#"
            use crate::b::b_fn as from_b;
            pub fn a_fn() -> Int { 1 }
        "#,
        r#"
            use crate::c::c_fn as from_c;
            pub fn b_fn() -> Int { 2 }
        "#,
        r#"
            use crate::b::b_fn as from_b;
            pub fn c_fn() -> Int { 3 }
        "#,
        "tail-cycle",
    );
    let a_key = root_key.child("a").expect("fixture key is canonical");
    let b_key = root_key.child("b").expect("fixture key is canonical");
    let c_key = root_key.child("c").expect("fixture key is canonical");
    let b_use_span = first_use_span(&graph, &b_key);
    let c_use_span = first_use_span(&graph, &c_key);
    let b_fn = function(&graph, &b_key, "b_fn");
    let c_fn = function(&graph, &c_key, "c_fn");
    let b_origin = graph
        .module_unit(&b_key)
        .expect("inline module is graph-owned")
        .artifact()
        .origin()
        .clone();
    let c_origin = graph
        .module_unit(&c_key)
        .expect("file module is graph-owned")
        .artifact()
        .origin()
        .clone();
    let scopes = scopes(&graph);

    let cycle = cycle_error(&graph, &scopes);
    let edges = cycle.edges();

    assert_eq!(
        edges.len(),
        2,
        "the leading a -> b edge is not part of b -> c -> b"
    );
    assert!(
        edges.iter().all(|edge| edge.importing_module() != &a_key),
        "the unrelated leading dependency cannot appear in the closing-cycle report"
    );
    assert_eq!(edges[0].importing_module(), &b_key);
    assert_eq!(edges[0].defining_module(), &c_key);
    assert_eq!(edges[0].defining_identity().name(), "c_fn");
    assert_eq!(edges[0].local_name(), "from_c");
    assert_eq!(edges[0].use_span(), b_use_span);
    assert_eq!(edges[0].declaration_span(), c_fn.span);
    assert_eq!(edges[0].origin(), &c_origin);
    assert_eq!(edges[0].visibility(), &Visibility::Public);
    assert_eq!(edges[1].importing_module(), &c_key);
    assert_eq!(edges[1].defining_module(), &b_key);
    assert_eq!(edges[1].defining_identity().name(), "b_fn");
    assert_eq!(edges[1].local_name(), "from_b");
    assert_eq!(edges[1].use_span(), c_use_span);
    assert_eq!(edges[1].declaration_span(), b_fn.span);
    assert_eq!(edges[1].origin(), &b_origin);
    assert_eq!(edges[1].visibility(), &Visibility::Public);
}

#[test]
fn same_module_structural_alias_binds_without_a_self_edge_or_cycle() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod a {
                pub fn a_fn() -> Int { 1 }
                use crate::a::a_fn as local_a;
                use crate::b::b_fn as from_b;
            }
            pub mod b {
                pub fn b_fn() -> Int { 2 }
            }
        "#,
        "same-module-no-edge",
    );
    let a_key = root_key.child("a").expect("fixture key is canonical");
    let b_key = root_key.child("b").expect("fixture key is canonical");
    let scopes = scopes(&graph);

    let plan = resolve(&graph, &scopes)
        .expect("a same-module alias alongside an acyclic cross-module edge is cycle-free");
    let local = plan
        .binding(&a_key, "local_a")
        .expect("the same-module alias is staged as a local binding");
    let remote = plan
        .binding(&a_key, "from_b")
        .expect("the cross-module alias is staged as a local binding");

    assert_eq!(local.defining_identity().module_key(), &a_key);
    assert_eq!(local.defining_identity().name(), "a_fn");
    assert_eq!(remote.defining_identity().module_key(), &b_key);
    assert_eq!(remote.defining_identity().name(), "b_fn");
    assert_eq!(plan.import_edges().len(), 1);
    assert_eq!(plan.import_edges()[0].importing_module(), &a_key);
    assert_eq!(plan.import_edges()[0].defining_module(), &b_key);
    assert_eq!(plan.import_edges()[0].local_name(), "from_b");
}

#[test]
fn structural_visibility_failure_precedes_cycle_detection() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod a {
                mod hidden {
                    use crate::b::b_fn as from_b;
                    pub fn a_fn() -> Int { 1 }
                }
            }
            pub mod b {
                pub fn b_fn() -> Int { 2 }
                use crate::a::hidden::a_fn as from_a;
            }
        "#,
        "visibility-before-cycle",
    );
    let a_key = root_key.child("a").expect("fixture key is canonical");
    let b_key = root_key.child("b").expect("fixture key is canonical");
    let hidden_key = a_key.child("hidden").expect("fixture key is canonical");
    let hidden_declaration = module_declaration(&graph, &a_key, "hidden");
    let b_use_span = first_use_span(&graph, &b_key);
    let scopes = scopes(&graph);

    let error = resolve(&graph, &scopes).expect_err(
        "an inaccessible child that would otherwise close a cycle keeps diagnostic precedence",
    );
    match error {
        CanonicalStructuralImportError::Inaccessible {
            declaration_span,
            use_span,
            defining_module,
            attempted_path,
            violated_visibility,
        } => {
            assert_eq!(declaration_span, hidden_declaration.span);
            assert_eq!(use_span, b_use_span);
            assert_eq!(defining_module, hidden_key);
            assert_eq!(
                attempted_path,
                vec!["crate".into(), "a".into(), "hidden".into(), "a_fn".into()]
            );
            assert_eq!(violated_visibility, Visibility::Inherited);
        }
        CanonicalStructuralImportError::ImportCycle { .. } => {
            panic!("cycle detection must not run after a structural visibility failure")
        }
        other => panic!("expected anchored pre-cycle visibility rejection, got {other:?}"),
    }
}

#[test]
fn inaccessible_structural_child_and_target_report_visibility_anchors_before_any_binding_is_published()
 {
    let (root_key, hidden_child_graph) = parsed_graph(
        r#"
            mod api {
                pub mod text {
                    pub fn normalize(value: Int) -> Int { value }
                }
            }
            pub mod client {
                use crate::api::text::normalize as normalize_text;
            }
        "#,
        "hidden-intermediate-child",
    );
    let api_key = root_key.child("api").expect("fixture key is canonical");
    let client_key = root_key.child("client").expect("fixture key is canonical");
    let api_declaration = module_declaration(&hidden_child_graph, &root_key, "api");
    let child_use_span = first_use_span(&hidden_child_graph, &client_key);
    let child_scopes = scopes(&hidden_child_graph);

    let child_error = resolve(&hidden_child_graph, &child_scopes).expect_err(
        "an inaccessible intermediate structural module must reject before staging a client alias",
    );
    match child_error {
        CanonicalStructuralImportError::Inaccessible {
            declaration_span,
            use_span,
            defining_module,
            attempted_path,
            violated_visibility,
        } => {
            assert_eq!(declaration_span, api_declaration.span);
            assert_eq!(use_span, child_use_span);
            assert_eq!(defining_module, api_key);
            assert_eq!(
                attempted_path,
                vec![
                    "crate".into(),
                    "api".into(),
                    "text".into(),
                    "normalize".into()
                ]
            );
            assert_eq!(violated_visibility, Visibility::Inherited);
        }
        CanonicalStructuralImportError::Unresolved { .. } => {
            panic!("an existing inaccessible child must not be downgraded to unresolved")
        }
        other => panic!("expected anchored child visibility rejection, got {other:?}"),
    }

    let (root_key, hidden_target_graph) = parsed_graph(
        r#"
            pub mod api {
                pub mod text {
                    fn normalize(value: Int) -> Int { value }
                }
            }
            pub mod client {
                use crate::api::text::normalize as normalize_text;
            }
        "#,
        "hidden-final-target",
    );
    let api_key = root_key.child("api").expect("fixture key is canonical");
    let text_key = api_key.child("text").expect("fixture key is canonical");
    let client_key = root_key.child("client").expect("fixture key is canonical");
    let target = function(&hidden_target_graph, &text_key, "normalize");
    let target_use_span = first_use_span(&hidden_target_graph, &client_key);
    let target_scopes = scopes(&hidden_target_graph);

    let target_error = resolve(&hidden_target_graph, &target_scopes)
        .expect_err("an inaccessible final function must reject before staging a client alias");
    match target_error {
        CanonicalStructuralImportError::Inaccessible {
            declaration_span,
            use_span,
            defining_module,
            attempted_path,
            violated_visibility,
        } => {
            assert_eq!(declaration_span, target.span);
            assert_eq!(use_span, target_use_span);
            assert_eq!(defining_module, text_key);
            assert_eq!(
                attempted_path,
                vec![
                    "crate".into(),
                    "api".into(),
                    "text".into(),
                    "normalize".into()
                ]
            );
            assert_eq!(violated_visibility, Visibility::Inherited);
        }
        CanonicalStructuralImportError::Unresolved { .. } => {
            panic!("an existing inaccessible function must not be downgraded to unresolved")
        }
        other => panic!("expected anchored final-target visibility rejection, got {other:?}"),
    }
}

#[test]
fn canonical_module_key_visibility_regions_cover_private_restricted_public_and_cross_crate_cases() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod a {
                pub mod b {
                    pub mod deep {}
                }
                pub mod requester {}
            }
            pub mod sibling {}
        "#,
        "visibility-regions",
    );
    let a_key = root_key.child("a").expect("fixture key is canonical");
    let b_key = a_key.child("b").expect("fixture key is canonical");
    let deep_key = b_key.child("deep").expect("fixture key is canonical");
    let requester_key = a_key.child("requester").expect("fixture key is canonical");
    let sibling_key = root_key.child("sibling").expect("fixture key is canonical");
    let external_root = ModuleKey::root("outside").expect("cross-crate key is canonical");
    let external_child = external_root
        .child("client")
        .expect("cross-crate child key is canonical");
    let scopes = scopes(&graph);

    let cases = vec![
        (
            "private permits exactly its defining module",
            Visibility::Inherited,
            b_key.clone(),
            b_key.clone(),
            true,
        ),
        (
            "private rejects a sibling requester and therefore a nonpublic intermediate child",
            Visibility::Inherited,
            b_key.clone(),
            sibling_key.clone(),
            false,
        ),
        (
            "pub self permits exactly its defining module",
            Visibility::Self_,
            b_key.clone(),
            b_key.clone(),
            true,
        ),
        (
            "pub self rejects a descendant",
            Visibility::Self_,
            b_key.clone(),
            deep_key.clone(),
            false,
        ),
        (
            "pub crate permits a same-crate sibling",
            Visibility::Crate,
            b_key.clone(),
            sibling_key.clone(),
            true,
        ),
        (
            "pub crate rejects a cross-crate requester",
            Visibility::Crate,
            b_key.clone(),
            external_child.clone(),
            false,
        ),
        (
            "pub super permits its structural parent",
            Visibility::Super { levels: 1 },
            b_key.clone(),
            a_key.clone(),
            true,
        ),
        (
            "pub super permits a descendant of its structural parent",
            Visibility::Super { levels: 1 },
            b_key.clone(),
            requester_key.clone(),
            true,
        ),
        (
            "pub super rejects an ancestor outside its parent region",
            Visibility::Super { levels: 1 },
            b_key.clone(),
            root_key.clone(),
            false,
        ),
        (
            "pub in permits its named canonical module",
            Visibility::Restricted {
                path: "crate::a".into(),
            },
            b_key.clone(),
            a_key.clone(),
            true,
        ),
        (
            "pub in permits descendants of its named canonical module",
            Visibility::Restricted {
                path: "crate::a".into(),
            },
            b_key.clone(),
            deep_key.clone(),
            true,
        ),
        (
            "pub in rejects a same-crate sibling outside its named region",
            Visibility::Restricted {
                path: "crate::a".into(),
            },
            b_key.clone(),
            sibling_key.clone(),
            false,
        ),
        (
            "public permits a cross-crate requester before route-form filtering",
            Visibility::Public,
            b_key,
            external_child,
            true,
        ),
    ];

    for (name, visibility, defining_module, requesting_module, expected) in cases {
        assert_eq!(
            scopes
                .is_visible_from(&visibility, &defining_module, &requesting_module)
                .expect("the canonical visibility relation resolves all selected regions"),
            expected,
            "{name}"
        );
    }
}

#[test]
fn local_function_name_collision_rejects_without_overwriting_the_declaration() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub mod text {
                    pub fn normalize(value: Int) -> Int { value }
                }
            }
            fn normalize(value: Int) -> Int { value + 1 }
            use crate::api::text::normalize as normalize;
        "#,
        "local-declaration-collision",
    );
    let local = function(&graph, &root_key, "normalize");
    let use_span = first_use_span(&graph, &root_key);
    let scopes = scopes(&graph);

    let error = resolve(&graph, &scopes)
        .expect_err("an import alias must not shadow or overwrite an ordinary local declaration");
    match error {
        CanonicalStructuralImportError::LocalDeclarationCollision {
            importing_module,
            name,
            declaration_span,
            use_span: collision_use_span,
        } => {
            assert_eq!(importing_module, root_key);
            assert_eq!(name, "normalize");
            assert_eq!(declaration_span, local.span);
            assert_eq!(collision_use_span, use_span);
        }
        other => panic!("expected anchored local declaration collision, got {other:?}"),
    }
}

#[test]
fn file_and_inline_structural_paths_have_equal_normalized_scope_visibility_and_binding_projections()
{
    let (inline_root, inline_graph) = parsed_graph(
        r#"
            pub mod api {
                pub mod text {
                    pub fn normalize(value: Int) -> Int { value }
                }
            }
            use crate::api::text::normalize as normalize_text;
        "#,
        "inline-projection-parity",
    );
    let (file_root, file_graph) = file_backed_graph(
        r#"
            pub mod api;
            use crate::api::text::normalize as normalize_text;
        "#,
        r#"
            pub mod text {
                pub fn normalize(value: Int) -> Int { value }
            }
        "#,
        "file-projection-parity",
    );
    let inline_scopes = scopes(&inline_graph);
    let file_scopes = scopes(&file_graph);
    let inline_plan = resolve(&inline_graph, &inline_scopes)
        .expect("inline structural source resolves through provisional scopes");
    let file_plan = resolve(&file_graph, &file_scopes)
        .expect("file structural source resolves through provisional scopes");
    let inline_api = inline_root.child("api").expect("fixture key is canonical");
    let inline_text = inline_api.child("text").expect("fixture key is canonical");
    let file_api = file_root.child("api").expect("fixture key is canonical");
    let file_text = file_api.child("text").expect("fixture key is canonical");
    let inline_binding = inline_plan
        .binding(&inline_root, "normalize_text")
        .expect("inline plan stages the expected local alias");
    let file_binding = file_plan
        .binding(&file_root, "normalize_text")
        .expect("file plan stages the expected local alias");

    assert_eq!(
        inline_scopes.normalized_scope_projection(),
        file_scopes.normalized_scope_projection(),
        "normalization compares graph-derived module identities, children, functions, and visibility without source-layout provenance"
    );
    assert_eq!(
        inline_scopes
            .is_visible_from(&Visibility::Public, &inline_text, &inline_root)
            .expect("inline public target visibility is structurally decidable"),
        file_scopes
            .is_visible_from(&Visibility::Public, &file_text, &file_root)
            .expect("file public target visibility is structurally decidable")
    );
    assert_eq!(
        inline_binding.defining_identity().module_key(),
        &inline_text
    );
    assert_eq!(file_binding.defining_identity().module_key(), &file_text);
    assert_eq!(inline_binding.defining_identity().name(), "normalize");
    assert_eq!(file_binding.defining_identity().name(), "normalize");
    assert_eq!(inline_binding.visibility(), file_binding.visibility());
    assert_eq!(
        inline_plan.import_edges()[0].local_name(),
        file_plan.import_edges()[0].local_name(),
        "equivalent sources stage the same local binding projection"
    );
}

#[test]
fn a_late_invalid_structural_path_rejects_the_complete_scope_binding_result_atomically() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub mod text {
                    pub fn normalize(value: Int) -> Int { value }
                }
            }
            use crate::api::text::normalize as normalize_text;
            use crate::api::missing::normalize as broken;
        "#,
        "late-invalid-structural-path",
    );
    let uses = graph
        .module_unit(&root_key)
        .expect("root unit is graph-owned")
        .body()
        .uses();
    let late_use_span = uses
        .get(1)
        .expect("fixture has a late malformed structural path")
        .span;
    let scopes = scopes(&graph);

    let error = resolve(&graph, &scopes)
        .expect_err("a late invalid structural child must not publish the earlier valid alias");
    match error {
        CanonicalStructuralImportError::Unresolved {
            use_span,
            attempted_path,
        } => {
            assert_eq!(use_span, late_use_span);
            assert_eq!(
                attempted_path,
                vec![
                    "crate".into(),
                    "api".into(),
                    "missing".into(),
                    "normalize".into()
                ]
            );
        }
        other => panic!("expected atomic late structural-path rejection, got {other:?}"),
    }
}

fn assert_unsupported_route_form(source: &str, label: &str) {
    let (root_key, graph) = parsed_graph(source, label);
    let use_span = first_use_span(&graph, &root_key);
    let scopes = scopes(&graph);

    let error = resolve(&graph, &scopes)
        .expect_err("a route form outside the scope-backed structural slice must reject");
    match error {
        CanonicalStructuralImportError::Unsupported { span, reason } => {
            assert_eq!(
                span, use_span,
                "the rejection stays anchored at the parsed use"
            );
            assert!(
                !reason.is_empty(),
                "the bounded route supplies a stable reason"
            );
        }
        other => panic!("expected unsupported route-form rejection, got {other:?}"),
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
fn scope_backed_structural_imports_exclude_other_forms_and_authority() {
    for (label, source) in [
        (
            "public-reexport",
            r#"
                pub mod api { pub mod text { pub fn normalize(value: Int) -> Int { value } } }
                pub use crate::api::text::normalize as normalize_text;
            "#,
        ),
        (
            "group-import",
            r#"
                pub mod api { pub mod text { pub fn normalize(value: Int) -> Int { value } } }
                use crate::api::text::{normalize as normalize_text};
            "#,
        ),
        (
            "glob-import",
            r#"
                pub mod api { pub mod text { pub fn normalize(value: Int) -> Int { value } } }
                use crate::api::text::*;
            "#,
        ),
        (
            "noncrate-import",
            r#"
                pub mod api { pub mod text { pub fn normalize(value: Int) -> Int { value } } }
                use self::api::text::normalize as normalize_text;
            "#,
        ),
        (
            "nonfunction-target",
            r#"
                pub mod api { pub type Number = Int; }
                use crate::api::Number as number;
            "#,
        ),
    ] {
        assert_unsupported_route_form(source, label);
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scopes_path = manifest_dir.join("src/canonical_provisional_module_scopes.rs");
    let scopes_source = fs::read_to_string(&scopes_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated scope-backed resolver at {}: {error}",
            scopes_path.display()
        )
    });
    let planner_path = manifest_dir.join("src/canonical_simple_import_planner.rs");
    let planner_source = fs::read_to_string(&planner_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 retains the generic planner fence at {}: {error}",
            planner_path.display()
        )
    });

    for forbidden_identifier in [
        "ModuleIdentity",
        "ModuleId",
        "ModuleGraph",
        "LegacyModuleResolver",
        "ModuleResolver",
        "NameBinder",
        "VisibilityChecker",
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
            !contains_exact_identifier(&scopes_source, forbidden_identifier),
            "provisional scopes must not consume legacy, string-visibility, final-interface, lowering, admission, or runtime authority: {forbidden_identifier}"
        );
    }
    for forbidden_bypass in [
        "is_visible_in_module",
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
            !scopes_source.contains(forbidden_bypass),
            "provisional scopes must consume only canonical graph units and typed payloads: {forbidden_bypass}"
        );
    }
    assert!(
        contains_exact_identifier(&planner_source, "resolve_simple_parsed_imports"),
        "the pre-existing generic planner remains separate from the opt-in scope-backed route"
    );
}

#[test]
fn file_and_inline_cycles_have_equal_normalized_structural_edge_projections() {
    let (inline_root, inline_graph) = parsed_graph(
        r#"
            pub mod a {
                use crate::b::b_fn as from_b;
                pub fn a_fn() -> Int { 1 }
            }
            pub mod b {
                use crate::a::a_fn as from_a;
                pub fn b_fn() -> Int { 2 }
            }
        "#,
        "inline-cycle-projection",
    );
    let (mixed_root, mixed_graph) = file_inline_cycle_graph(
        r#"
            use crate::b::b_fn as from_b;
            pub fn a_fn() -> Int { 1 }
        "#,
        r#"
            use crate::a::a_fn as from_a;
            pub fn b_fn() -> Int { 2 }
        "#,
        "file-inline-cycle-projection",
    );
    let inline_scopes = scopes(&inline_graph);
    let mixed_scopes = scopes(&mixed_graph);
    let inline_cycle = cycle_error(&inline_graph, &inline_scopes);
    let mixed_cycle = cycle_error(&mixed_graph, &mixed_scopes);

    assert_eq!(
        inline_root, mixed_root,
        "the compared fixtures share canonical keys"
    );
    assert_eq!(
        normalized_cycle_projection(&inline_cycle),
        normalized_cycle_projection(&mixed_cycle),
        "cycle normalization retains canonical importer/definer identity, alias, and visibility while excluding source-layout provenance"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn generated_structural_cycles_report_deterministic_ordered_alias_provenance(
        a_suffix in "[a-z][a-z0-9_]{0,12}",
        b_suffix in "[a-z][a-z0-9_]{0,12}",
    ) {
        let a_module = format!("module_a_{a_suffix}");
        let b_module = format!("module_b_{b_suffix}");
        let from_b = format!("from_b_{a_suffix}");
        let from_a = format!("from_a_{b_suffix}");
        let source = format!(
            "pub mod {a_module} {{ use crate::{b_module}::b_fn as {from_b}; pub fn a_fn() -> Int {{ 1 }} }} \
             pub mod {b_module} {{ use crate::{a_module}::a_fn as {from_a}; pub fn b_fn() -> Int {{ 2 }} }}"
        );
        let (root_key, graph) = parsed_graph(&source, "generated-structural-cycle");
        let a_key = root_key
            .child(&a_module)
            .expect("generated first fixture key is canonical");
        let b_key = root_key
            .child(&b_module)
            .expect("generated second fixture key is canonical");
        prop_assert_ne!(
            &a_key,
            &b_key,
            "the generated module names remain distinct"
        );
        let scopes = scopes(&graph);

        match resolve(&graph, &scopes) {
            Err(CanonicalStructuralImportError::ImportCycle { edges }) => {
                let edges = edges.edges();
                prop_assert_eq!(edges.len(), 2);
                prop_assert_eq!(edges[0].importing_module(), &a_key);
                prop_assert_eq!(edges[0].defining_module(), &b_key);
                prop_assert_eq!(edges[0].local_name(), from_b.as_str());
                prop_assert_eq!(edges[1].importing_module(), &b_key);
                prop_assert_eq!(edges[1].defining_module(), &a_key);
                prop_assert_eq!(edges[1].local_name(), from_a.as_str());
            }
            Ok(plan) => prop_assert!(
                false,
                "a generated reachable structural cycle must not publish a plan: {plan:?}"
            ),
            Err(other) => prop_assert!(
                false,
                "a generated reachable structural cycle must retain the outer cycle diagnostic: {other:?}"
            ),
        }
    }
}

#[test]
fn late_cycle_rejects_atomically_without_returning_earlier_cross_module_bindings_or_edges() {
    let (root_key, graph) = file_inline_tail_cycle_graph(
        r#"
            use crate::b::b_fn as from_b;
            pub fn a_fn() -> Int { 1 }
        "#,
        r#"
            use crate::c::c_fn as from_c;
            pub fn b_fn() -> Int { 2 }
        "#,
        r#"
            use crate::b::b_fn as from_b;
            pub fn c_fn() -> Int { 3 }
        "#,
        "late-cycle-atomicity",
    );
    let a_key = root_key.child("a").expect("fixture key is canonical");
    let b_key = root_key.child("b").expect("fixture key is canonical");
    let c_key = root_key.child("c").expect("fixture key is canonical");
    let scopes = scopes(&graph);

    let cycle = cycle_error(&graph, &scopes);
    let edges = cycle.edges();

    assert_eq!(edges.len(), 2);
    assert!(
        edges.iter().all(|edge| edge.importing_module() != &a_key),
        "the early valid a -> b binding and edge remain unpublishable when the later b <-> c cycle fails"
    );
    assert_eq!(edges[0].importing_module(), &b_key);
    assert_eq!(edges[0].defining_module(), &c_key);
    assert_eq!(edges[1].importing_module(), &c_key);
    assert_eq!(edges[1].defining_module(), &b_key);
}

#[test]
fn scoped_cycle_gate_stays_separate_from_generic_binder_and_runtime_authority() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scopes_path = manifest_dir.join("src/canonical_provisional_module_scopes.rs");
    let scopes_source = fs::read_to_string(&scopes_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the scope-owned structural error at {}: {error}",
            scopes_path.display()
        )
    });
    let planner_path = manifest_dir.join("src/canonical_simple_import_planner.rs");
    let planner_source = fs::read_to_string(&planner_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the scope-backed resolver at {}: {error}",
            planner_path.display()
        )
    });
    let binder_path = manifest_dir.join("src/canonical_module_binder.rs");
    let binder_source = fs::read_to_string(&binder_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 retains the generic compatibility binder at {}: {error}",
            binder_path.display()
        )
    });

    assert!(
        contains_exact_identifier(&scopes_source, "CanonicalImportCycle"),
        "the outer structural error retains the canonical ordered-edge wrapper"
    );
    assert!(
        planner_source.contains("CanonicalStructuralImportError::ImportCycle"),
        "only the scope-backed resolver maps staged structural edges into its outer error"
    );
    assert!(
        !binder_source.contains("resolve_simple_parsed_imports_with_scopes"),
        "the generic compatibility binder must remain on its independent grammar and cycle contract"
    );
    assert!(
        !binder_source.contains("CanonicalStructuralImportError"),
        "the generic compatibility binder must not expose the scoped structural error"
    );
    for forbidden_identifier in [
        "PublicModuleInterface",
        "FinalizedModuleInterface",
        "InterfaceImportResolver",
        "CheckedInterfaceStore",
        "RawCoreProgram",
        "CoreExpr",
        "CpsProgram",
        "Engine",
    ] {
        assert!(
            !contains_exact_identifier(&scopes_source, forbidden_identifier),
            "the structural cycle error carrier must not gain final-interface, lowering, admission, or runtime authority: {forbidden_identifier}"
        );
        assert!(
            !contains_exact_identifier(&planner_source, forbidden_identifier),
            "the scope-backed cycle gate must not gain final-interface, lowering, admission, or runtime authority: {forbidden_identifier}"
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
            !scopes_source.contains(forbidden_bypass),
            "the scoped cycle error carrier stays graph-only: {forbidden_bypass}"
        );
        assert!(
            !planner_source.contains(forbidden_bypass),
            "the scoped cycle gate stays graph-only: {forbidden_bypass}"
        );
    }
}
