//! TASK-2068 RED contracts for scoped simple ordinary-function imports.
//!
//! This target admits only inherited simple `crate::` ordinary-function
//! imports through the dedicated scope-backed route. It proves natural-name
//! selection, optional aliases, visibility-before-binding, atomic cycles, and
//! binding-only authority without widening the generic compatibility binder.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::surface::{Definition, FnDef, Visibility};
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver, Span};
use ash_typeck::{
    CanonicalProvisionalModuleScopes, CanonicalStructuralImportError,
    bind_scoped_simple_ordinary_function_imports,
    resolve_scoped_simple_ordinary_function_imports_with_scopes,
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
            "ash-task-2068-scoped-simple-imports-{label}-{}-{serial}",
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
        .expect("file/inline cycle fixture resolves through the canonical parser graph");
    (root_key, graph)
}

fn module_key(root: &ModuleKey, segments: &[&str]) -> ModuleKey {
    segments.iter().fold(root.clone(), |key, segment| {
        key.child(segment)
            .expect("fixture module path remains canonical")
    })
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
    resolve_scoped_simple_ordinary_function_imports_with_scopes(graph, scopes)
}

#[test]
fn scoped_simple_import_without_alias_uses_the_deep_target_name() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub mod text {
                    pub fn normalize(value: Int) -> Int { value }
                }
            }
            use crate::api::text::normalize;
        "#,
        "deep-natural-name",
    );
    let target_module = module_key(&root_key, &["api", "text"]);
    let target = function(&graph, &target_module, "normalize");
    let target_origin = graph
        .module_unit(&target_module)
        .expect("target module is graph-owned")
        .artifact()
        .origin()
        .clone();
    let use_span = first_use_span(&graph, &root_key);
    let scope_snapshot = scopes(&graph);

    let plan = resolve(&graph, &scope_snapshot)
        .expect("a natural deep ordinary-function import is scope-resolvable");
    let bound = bind_scoped_simple_ordinary_function_imports(&graph, &scope_snapshot)
        .expect("the dedicated binder projects the natural-name plan");
    let binding = plan
        .binding(&root_key, "normalize")
        .expect("the final function segment becomes the local name");
    let edge = plan
        .import_edges()
        .first()
        .expect("the deep cross-module import retains one canonical edge");

    assert_eq!(bound.binding(&root_key, "normalize"), Some(binding));
    assert_eq!(binding.defining_identity().module_key(), &target_module);
    assert_eq!(binding.defining_identity().name(), "normalize");
    assert_eq!(binding.declaration_span(), target.span);
    assert_eq!(binding.origin(), &target_origin);
    assert_eq!(binding.visibility(), &Visibility::Public);
    assert_eq!(edge.importing_module(), &root_key);
    assert_eq!(edge.defining_module(), &target_module);
    assert_eq!(edge.local_name(), "normalize");
    assert_eq!(edge.use_span(), use_span);
}

#[test]
fn scoped_simple_explicit_alias_preserves_plan_identity_and_parser_provenance() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub mod text {
                    pub fn normalize(value: Int) -> Int { value }
                }
            }
            pub mod client {
                use crate::api::text::normalize as normalize_text;
            }
        "#,
        "deep-explicit-alias",
    );
    let client_module = module_key(&root_key, &["client"]);
    let target_module = module_key(&root_key, &["api", "text"]);
    let target = function(&graph, &target_module, "normalize");
    let target_origin = graph
        .module_unit(&target_module)
        .expect("target module is graph-owned")
        .artifact()
        .origin()
        .clone();
    let use_span = first_use_span(&graph, &client_module);
    let scope_snapshot = scopes(&graph);

    let plan = resolve(&graph, &scope_snapshot)
        .expect("the scoped resolver accepts an explicit deep alias");
    let bound = bind_scoped_simple_ordinary_function_imports(&graph, &scope_snapshot)
        .expect("the binder may only project the explicit-alias plan");
    let plan_binding = plan
        .binding(&client_module, "normalize_text")
        .expect("the scoped resolver stages the explicit alias");
    let bound_binding = bound
        .binding(&client_module, "normalize_text")
        .expect("the dedicated binder exposes the explicit alias");
    let edge = plan
        .import_edges()
        .first()
        .expect("the cross-module alias retains an import edge");

    assert_eq!(bound_binding, plan_binding);
    assert_eq!(
        plan_binding.defining_identity().module_key(),
        &target_module
    );
    assert_eq!(plan_binding.defining_identity().name(), "normalize");
    assert_eq!(plan_binding.declaration_span(), target.span);
    assert_eq!(plan_binding.origin(), &target_origin);
    assert_eq!(plan_binding.visibility(), &Visibility::Public);
    assert_eq!(edge.defining_identity(), plan_binding.defining_identity());
    assert_eq!(edge.local_name(), "normalize_text");
    assert_eq!(edge.declaration_span(), target.span);
    assert_eq!(edge.origin(), &target_origin);
    assert_eq!(edge.use_span(), use_span);
}

#[test]
fn scoped_simple_imports_accept_root_function_targets_with_and_without_aliases() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub fn root_target(value: Int) -> Int { value }
            pub mod client {
                use crate::root_target;
                use crate::root_target as root_alias;
            }
        "#,
        "root-targets",
    );
    let client_module = module_key(&root_key, &["client"]);
    let target = function(&graph, &root_key, "root_target");
    let scope_snapshot = scopes(&graph);

    let plan = resolve(&graph, &scope_snapshot)
        .expect("root ordinary-function imports accept natural and explicit local names");
    let bound = bind_scoped_simple_ordinary_function_imports(&graph, &scope_snapshot)
        .expect("the binder projects root-target imports without changing their local names");

    for local_name in ["root_target", "root_alias"] {
        let plan_binding = plan
            .binding(&client_module, local_name)
            .expect("each root-target local name is staged by the resolver");
        assert_eq!(
            bound.binding(&client_module, local_name),
            Some(plan_binding)
        );
        assert_eq!(plan_binding.defining_identity().module_key(), &root_key);
        assert_eq!(plan_binding.defining_identity().name(), "root_target");
        assert_eq!(plan_binding.declaration_span(), target.span);
        assert_eq!(plan_binding.visibility(), &Visibility::Public);
    }
    assert_eq!(plan.import_edges().len(), 2);
    assert!(
        plan.import_edges()
            .iter()
            .all(|edge| edge.defining_module() == &root_key),
        "both root imports retain the root defining module rather than a synthetic alias module",
    );
}

#[test]
fn scoped_simple_route_keeps_a_structural_child_final_segment_unresolved() {
    let (root_key, graph) = parsed_graph(
        r#"
            mod api { fn greet() -> Int { 1 } }
            use crate::api;
        "#,
        "structural-child-final-segment",
    );
    let enclosing_use_span = first_use_span(&graph, &root_key);
    let scope_snapshot = scopes(&graph);
    let resolver_error = resolve(&graph, &scope_snapshot)
        .expect_err("the scoped-simple route must retain its pre-group unresolved contract");
    let binder_error = bind_scoped_simple_ordinary_function_imports(&graph, &scope_snapshot)
        .expect_err(
            "the simple binder must not project an unresolved structural-child final segment",
        );

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::Unresolved {
            use_span,
            attempted_path,
        } => {
            assert_eq!(use_span, enclosing_use_span);
            assert_eq!(attempted_path, vec!["crate".into(), "api".into()]);
        }
        other => panic!("expected scoped-simple unresolved final segment, got {other:?}"),
    }
}

#[test]
fn scoped_simple_imports_enforce_all_visibility_regions_before_binding() {
    let permitted_cases = [
        (
            "public",
            r#"
                pub mod host {
                    pub mod provider { pub fn target() -> Int { 1 } }
                    pub mod client { use crate::host::provider::target as local; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            Visibility::Public,
        ),
        (
            "crate",
            r#"
                pub mod host {
                    pub mod provider { pub(crate) fn target() -> Int { 1 } }
                    pub mod client { use crate::host::provider::target as local; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            Visibility::Crate,
        ),
        (
            "super",
            r#"
                pub mod host {
                    pub mod provider { pub(super) fn target() -> Int { 1 } }
                    pub mod client { use crate::host::provider::target as local; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            Visibility::Super { levels: 1 },
        ),
        (
            "restricted",
            r#"
                pub mod host {
                    pub mod provider {
                        pub(in crate::host) fn target() -> Int { 1 }
                    }
                    pub mod client { use crate::host::provider::target as local; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            Visibility::Restricted {
                path: "crate::host".into(),
            },
        ),
        (
            "inherited",
            r#"
                pub mod host {
                    pub mod provider {
                        fn target() -> Int { 1 }
                        use crate::host::provider::target as local;
                    }
                }
            "#,
            &["host", "provider"][..],
            &["host", "provider"][..],
            Visibility::Inherited,
        ),
        (
            "self",
            r#"
                pub mod host {
                    pub mod provider {
                        pub(self) fn target() -> Int { 1 }
                        use crate::host::provider::target as local;
                    }
                }
            "#,
            &["host", "provider"][..],
            &["host", "provider"][..],
            Visibility::Self_,
        ),
    ];

    for (label, source, importer_segments, target_segments, expected_visibility) in permitted_cases
    {
        let (root_key, graph) = parsed_graph(source, label);
        let importing_module = module_key(&root_key, importer_segments);
        let target_module = module_key(&root_key, target_segments);
        let scope_snapshot = scopes(&graph);
        let plan = resolve(&graph, &scope_snapshot)
            .expect("the selected importer lies within its target visibility region");
        let bound = bind_scoped_simple_ordinary_function_imports(&graph, &scope_snapshot)
            .expect("the binder must project every resolver-permitted visibility case");
        let plan_binding = plan
            .binding(&importing_module, "local")
            .expect("the resolver retains the permitted local name");

        assert_eq!(
            bound.binding(&importing_module, "local"),
            Some(plan_binding)
        );
        assert_eq!(
            plan_binding.defining_identity().module_key(),
            &target_module
        );
        assert_eq!(plan_binding.visibility(), &expected_visibility);
    }

    let rejected_cases = [
        (
            "public-behind-private-child",
            r#"
                mod hidden { pub fn target() -> Int { 1 } }
                pub mod client { use crate::hidden::target as local; }
            "#,
            &["client"][..],
            &["hidden"][..],
            Visibility::Inherited,
        ),
        (
            "super-at-root",
            r#"
                pub mod host { pub mod provider { pub(super) fn target() -> Int { 1 } } }
                use crate::host::provider::target as local;
            "#,
            &[][..],
            &["host", "provider"][..],
            Visibility::Super { levels: 1 },
        ),
        (
            "restricted-at-root",
            r#"
                pub mod host {
                    pub mod provider { pub(in crate::host) fn target() -> Int { 1 } }
                }
                use crate::host::provider::target as local;
            "#,
            &[][..],
            &["host", "provider"][..],
            Visibility::Restricted {
                path: "crate::host".into(),
            },
        ),
        (
            "inherited-at-sibling",
            r#"
                pub mod host {
                    pub mod provider { fn target() -> Int { 1 } }
                    pub mod client { use crate::host::provider::target as local; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            Visibility::Inherited,
        ),
        (
            "self-at-sibling",
            r#"
                pub mod host {
                    pub mod provider { pub(self) fn target() -> Int { 1 } }
                    pub mod client { use crate::host::provider::target as local; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            Visibility::Self_,
        ),
    ];

    for (label, source, importer_segments, rejected_segments, expected_visibility) in rejected_cases
    {
        let (root_key, graph) = parsed_graph(source, label);
        let importing_module = module_key(&root_key, importer_segments);
        let rejected_module = module_key(&root_key, rejected_segments);
        let rejected_span = if label == "public-behind-private-child" {
            graph
                .module_unit(&root_key)
                .expect("root unit is graph-owned")
                .body()
                .module_decls()
                .first()
                .expect("fixture contains the hidden structural child")
                .span
        } else {
            function(&graph, &rejected_module, "target").span
        };
        let use_span = first_use_span(&graph, &importing_module);
        let scope_snapshot = scopes(&graph);
        let resolver_error = resolve(&graph, &scope_snapshot)
            .expect_err("an importer outside a visibility boundary must fail before binding");
        let binder_error = bind_scoped_simple_ordinary_function_imports(&graph, &scope_snapshot)
            .expect_err("the binder must preserve the resolver visibility diagnostic");

        assert_eq!(binder_error, resolver_error);
        match binder_error {
            CanonicalStructuralImportError::Inaccessible {
                declaration_span,
                use_span: rejected_use_span,
                defining_module,
                violated_visibility,
                ..
            } => {
                assert_eq!(declaration_span, rejected_span, "{label}");
                assert_eq!(rejected_use_span, use_span, "{label}");
                assert_eq!(defining_module, rejected_module, "{label}");
                assert_eq!(violated_visibility, expected_visibility, "{label}");
            }
            other => panic!("expected a {label} visibility diagnostic, got {other:?}"),
        }
    }

    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api { pub(crate) fn target() -> Int { 1 } }
            pub mod client { use crate::api::target as local; }
        "#,
        "crate-cross-root-predicate",
    );
    let api_key = module_key(&root_key, &["api"]);
    let external_importer = ModuleKey::root("outside")
        .expect("cross-crate fixture key is canonical")
        .child("client")
        .expect("cross-crate fixture key is canonical");
    assert!(
        !scopes(&graph)
            .is_visible_from(&Visibility::Crate, &api_key, &external_importer)
            .expect("the canonical predicate decides the cross-crate pub(crate) boundary"),
        "the scoped route must not manufacture a cross-crate alias for a crate-visible target",
    );
}

#[test]
fn scoped_simple_import_rejects_a_local_function_collision_without_a_binding_set() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api { pub fn target() -> Int { 1 } }
            fn local() -> Int { 2 }
            use crate::api::target as local;
        "#,
        "local-collision",
    );
    let local = function(&graph, &root_key, "local");
    let use_span = first_use_span(&graph, &root_key);
    let scope_snapshot = scopes(&graph);

    let resolver_error = resolve(&graph, &scope_snapshot)
        .expect_err("an alias may not overwrite an ordinary local function");
    let binder_error = bind_scoped_simple_ordinary_function_imports(&graph, &scope_snapshot)
        .expect_err("the binding projection is absent when local collision preflight fails");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::LocalDeclarationCollision {
            importing_module,
            name,
            declaration_span,
            use_span: rejected_use_span,
        } => {
            assert_eq!(importing_module, root_key);
            assert_eq!(name, "local");
            assert_eq!(declaration_span, local.span);
            assert_eq!(rejected_use_span, use_span);
        }
        other => panic!("expected local ordinary-function collision, got {other:?}"),
    }
}

#[test]
fn scoped_simple_import_rejects_duplicate_natural_or_explicit_local_names_atomically() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub fn first() -> Int { 1 }
                pub fn second() -> Int { 2 }
            }
            use crate::api::first as shared;
            use crate::api::second as shared;
        "#,
        "duplicate-binding",
    );
    let scope_snapshot = scopes(&graph);

    let resolver_error = resolve(&graph, &scope_snapshot)
        .expect_err("the second staged local name duplicates the first alias");
    let binder_error = bind_scoped_simple_ordinary_function_imports(&graph, &scope_snapshot)
        .expect_err("duplicate preflight must publish no binding projection");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::DuplicateBinding {
            importing_module,
            name,
            ..
        } => {
            assert_eq!(importing_module, root_key);
            assert_eq!(name.as_ref(), "shared");
        }
        other => panic!("expected atomic duplicate binding rejection, got {other:?}"),
    }
}

#[test]
fn scoped_simple_import_rejects_a_late_cross_module_cycle_before_any_projection() {
    let (root_key, graph) = file_inline_tail_cycle_graph(
        r#"
            use crate::b::b_fn;
            pub fn a_fn() -> Int { 1 }
        "#,
        r#"
            use crate::c::c_fn;
            pub fn b_fn() -> Int { 2 }
        "#,
        r#"
            use crate::b::b_fn;
            pub fn c_fn() -> Int { 3 }
        "#,
        "late-cycle-atomicity",
    );
    let a_key = module_key(&root_key, &["a"]);
    let b_key = module_key(&root_key, &["b"]);
    let c_key = module_key(&root_key, &["c"]);
    let scope_snapshot = scopes(&graph);

    let resolver_error = resolve(&graph, &scope_snapshot)
        .expect_err("the b-to-c-to-b edges close a cycle after the a-to-b natural binding");
    let binder_error = bind_scoped_simple_ordinary_function_imports(&graph, &scope_snapshot)
        .expect_err("the binder must not project the earlier a-to-b binding after a late cycle");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::ImportCycle { edges } => {
            assert_eq!(edges.edges().len(), 2);
            assert!(
                edges
                    .edges()
                    .iter()
                    .all(|edge| edge.importing_module() != &a_key),
                "the earlier valid natural binding remains unavailable after cycle rejection",
            );
            assert_eq!(edges.edges()[0].importing_module(), &b_key);
            assert_eq!(edges.edges()[0].defining_module(), &c_key);
            assert_eq!(edges.edges()[1].importing_module(), &c_key);
            assert_eq!(edges.edges()[1].defining_module(), &b_key);
        }
        other => panic!("expected outer scoped-simple import cycle diagnostic, got {other:?}"),
    }
}

#[test]
fn scoped_simple_imports_have_equal_file_and_inline_binding_projections() {
    let (inline_root, inline_graph) = parsed_graph(
        r#"
            pub mod api {
                pub mod text {
                    pub fn normalize(value: Int) -> Int { value }
                }
            }
            use crate::api::text::normalize;
        "#,
        "inline-projection-parity",
    );
    let (file_root, file_graph) = file_backed_graph(
        r#"
            pub mod api;
            use crate::api::text::normalize;
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
        .expect("the inline graph is scoped-simple resolvable");
    let file_plan = resolve(&file_graph, &file_scopes)
        .expect("the file-backed graph is scoped-simple resolvable");
    let inline_bound = bind_scoped_simple_ordinary_function_imports(&inline_graph, &inline_scopes)
        .expect("the inline plan projects to bindings");
    let file_bound = bind_scoped_simple_ordinary_function_imports(&file_graph, &file_scopes)
        .expect("the file-backed plan projects to bindings");
    let inline_binding = inline_bound
        .binding(&inline_root, "normalize")
        .expect("the inline graph retains the natural local name");
    let file_binding = file_bound
        .binding(&file_root, "normalize")
        .expect("the file graph retains the natural local name");
    let inline_target = module_key(&inline_root, &["api", "text"]);
    let file_target = module_key(&file_root, &["api", "text"]);

    assert_eq!(
        inline_bound.binding(&inline_root, "normalize"),
        inline_plan.binding(&inline_root, "normalize")
    );
    assert_eq!(
        file_bound.binding(&file_root, "normalize"),
        file_plan.binding(&file_root, "normalize")
    );
    assert_eq!(
        inline_binding.defining_identity().module_key(),
        &inline_target
    );
    assert_eq!(file_binding.defining_identity().module_key(), &file_target);
    assert_eq!(inline_binding.defining_identity().name(), "normalize");
    assert_eq!(file_binding.defining_identity().name(), "normalize");
    assert_eq!(inline_binding.visibility(), file_binding.visibility());
    assert_eq!(
        inline_plan.import_edges().len(),
        file_plan.import_edges().len()
    );
    assert_eq!(inline_plan.import_edges()[0].local_name(), "normalize");
    assert_eq!(file_plan.import_edges()[0].local_name(), "normalize");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn scoped_simple_imports_match_the_plan_projection_for_generated_names_positions_and_visibility_regions(
        function_suffix in "[a-z][a-z0-9_]{0,8}",
        alias_suffix in "[a-z][a-z0-9_]{0,8}",
        visibility_case in 0_u8..6,
        target_at_root in any::<bool>(),
        explicit_alias in any::<bool>(),
    ) {
        let host = format!("host_{function_suffix}");
        let provider = format!("provider_{function_suffix}");
        let client = format!("client_{function_suffix}");
        let target = format!("target_{function_suffix}");
        let alias = format!("alias_{alias_suffix}");
        let must_use_alias = matches!(visibility_case, 4 | 5);
        let use_alias = explicit_alias || must_use_alias;
        let local_name = if use_alias { alias.as_str() } else { target.as_str() };
        let use_tail = if use_alias {
            format!(" as {alias}")
        } else {
            String::new()
        };
        let (source, importer_segments, target_segments, expected_visibility, cross_module) =
            match visibility_case {
                0 | 1 if target_at_root => {
                    let visibility = if visibility_case == 0 { "pub" } else { "pub(crate)" };
                    (
                        format!(
                            "{visibility} fn {target}() -> Int {{ 1 }} pub mod {client} {{ use crate::{target}{use_tail}; }}"
                        ),
                        vec![client.as_str()],
                        Vec::new(),
                        if visibility_case == 0 {
                            Visibility::Public
                        } else {
                            Visibility::Crate
                        },
                        true,
                    )
                }
                0 | 1 => {
                    let visibility = if visibility_case == 0 { "pub" } else { "pub(crate)" };
                    (
                        format!(
                            "pub mod {host} {{ pub mod {provider} {{ {visibility} fn {target}() -> Int {{ 1 }} }} pub mod {client} {{ use crate::{host}::{provider}::{target}{use_tail}; }} }}"
                        ),
                        vec![host.as_str(), client.as_str()],
                        vec![host.as_str(), provider.as_str()],
                        if visibility_case == 0 {
                            Visibility::Public
                        } else {
                            Visibility::Crate
                        },
                        true,
                    )
                }
                2 => (
                    format!(
                        "pub mod {host} {{ pub mod {provider} {{ pub(super) fn {target}() -> Int {{ 1 }} }} pub mod {client} {{ use crate::{host}::{provider}::{target}{use_tail}; }} }}"
                    ),
                    vec![host.as_str(), client.as_str()],
                    vec![host.as_str(), provider.as_str()],
                    Visibility::Super { levels: 1 },
                    true,
                ),
                3 => (
                    format!(
                        "pub mod {host} {{ pub mod {provider} {{ pub(in crate::{host}) fn {target}() -> Int {{ 1 }} }} pub mod {client} {{ use crate::{host}::{provider}::{target}{use_tail}; }} }}"
                    ),
                    vec![host.as_str(), client.as_str()],
                    vec![host.as_str(), provider.as_str()],
                    Visibility::Restricted {
                        path: format!("crate::{host}").into(),
                    },
                    true,
                ),
                4 => (
                    format!(
                        "pub mod {host} {{ pub mod {provider} {{ fn {target}() -> Int {{ 1 }} use crate::{host}::{provider}::{target}{use_tail}; }} }}"
                    ),
                    vec![host.as_str(), provider.as_str()],
                    vec![host.as_str(), provider.as_str()],
                    Visibility::Inherited,
                    false,
                ),
                _ => (
                    format!(
                        "pub mod {host} {{ pub mod {provider} {{ pub(self) fn {target}() -> Int {{ 1 }} use crate::{host}::{provider}::{target}{use_tail}; }} }}"
                    ),
                    vec![host.as_str(), provider.as_str()],
                    vec![host.as_str(), provider.as_str()],
                    Visibility::Self_,
                    false,
                ),
            };
        let (root_key, graph) = parsed_graph(&source, "generated-scoped-simple-import");
        let importer_refs = importer_segments.as_slice();
        let importing_module = module_key(&root_key, importer_refs);
        let target_module = module_key(&root_key, target_segments.as_slice());
        let scope_snapshot = scopes(&graph);
        let plan = resolve(&graph, &scope_snapshot)
            .expect("each generated fixture uses a canonical importer inside its visibility region");
        let bound = bind_scoped_simple_ordinary_function_imports(&graph, &scope_snapshot)
            .expect("the dedicated binder must project every generated scoped-simple plan");
        let plan_binding = plan
            .binding(&importing_module, local_name)
            .expect("the resolver retains the generated natural or explicit local name");
        let bound_binding = bound
            .binding(&importing_module, local_name)
            .expect("the binder retains the generated natural or explicit local name");

        prop_assert_eq!(bound_binding, plan_binding);
        prop_assert_eq!(plan_binding.defining_identity().module_key(), &target_module);
        prop_assert_eq!(plan_binding.defining_identity().name(), target.as_str());
        prop_assert_eq!(plan_binding.visibility(), &expected_visibility);
        prop_assert_eq!(plan.import_edges().len(), usize::from(cross_module));
        if cross_module {
            prop_assert_eq!(plan.import_edges()[0].local_name(), local_name);
        }
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
fn scoped_simple_import_route_is_the_only_new_binding_authority_and_leaves_the_generic_binder_unchanged()
 {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dedicated_path = manifest_dir.join("src/canonical_structural_module_binder.rs");
    let dedicated_source = fs::read_to_string(&dedicated_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated scoped-simple binder at {}: {error}",
            dedicated_path.display()
        )
    });
    let generic_path = manifest_dir.join("src/canonical_module_binder.rs");
    let generic_source = fs::read_to_string(&generic_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 retains the generic compatibility binder at {}: {error}",
            generic_path.display()
        )
    });
    let lib_path = manifest_dir.join("src/lib.rs");
    let lib_source = fs::read_to_string(&lib_path).unwrap_or_else(|error| {
        panic!(
            "read type-checker public exports at {}: {error}",
            lib_path.display()
        )
    });

    assert!(
        dedicated_source.contains("resolve_scoped_simple_ordinary_function_imports_with_scopes"),
        "only the dedicated scoped binder consumes the new scoped-simple resolver",
    );
    assert!(
        dedicated_source.contains("bind_scoped_simple_ordinary_function_imports"),
        "the private structural binder owns the named scoped-simple projection",
    );
    assert!(
        dedicated_source.contains(".map(|plan| plan.into_bound_set())"),
        "the dedicated binder must only project a successful resolver plan",
    );
    assert!(
        lib_source.contains("bind_scoped_simple_ordinary_function_imports"),
        "lib.rs alone re-exports the dedicated scoped-simple binding API",
    );

    for forbidden_identifier in [
        "CanonicalStructuralImportError",
        "CanonicalProvisionalModuleScopes",
        "resolve_simple_parsed_imports_with_scopes",
        "resolve_scoped_simple_ordinary_function_imports_with_scopes",
        "bind_scoped_simple_ordinary_function_imports",
    ] {
        assert!(
            !contains_exact_identifier(&generic_source, forbidden_identifier),
            "the generic binder must remain generic-only and omit {forbidden_identifier}",
        );
    }

    let planner_path = manifest_dir.join("src/canonical_simple_import_planner.rs");
    let planner_source = fs::read_to_string(&planner_path).unwrap_or_else(|error| {
        panic!(
            "read scoped-simple planner authority boundary at {}: {error}",
            planner_path.display()
        )
    });
    for forbidden_identifier in [
        "PublicModuleInterface",
        "FinalizedModuleInterface",
        "InterfaceImportResolver",
        "CheckedInterfaceStore",
        "TypeEnvModuleInterfaceCollection",
        "RawCoreProgram",
        "CoreExpr",
        "CpsProgram",
        "Engine",
        "eval",
        "evaluate",
        "execute",
        "std::fs",
        "read_to_string",
        "parse_surface",
        "resolve_root",
        "from_legacy",
        "into_legacy",
    ] {
        assert!(
            !contains_exact_identifier(&planner_source, forbidden_identifier),
            "the scoped-simple planner must not gain wider authority: {forbidden_identifier}",
        );
    }
}
