//! TASK-2068 RED contracts for scoped `super` ordinary-function imports.
//!
//! This target reserves one Type-layer route only: a non-root module may use
//! exactly one leading `super` followed by structural children and one
//! ordinary function. It retains the parser-owned whole-use span, provisional
//! scope visibility, atomic preflight, and the dedicated binding projection.
//! It neither widens the generic binder nor authorizes a later layer.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::surface::{Definition, FnDef, Visibility};
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver, Span};
use ash_typeck::{
    CanonicalProvisionalModuleScopes, CanonicalStructuralImportError,
    bind_scoped_super_ordinary_function_imports,
    resolve_scoped_super_ordinary_function_imports_with_scopes,
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
            "ash-task-2068-scoped-super-imports-{label}-{}-{serial}",
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

fn use_spans(graph: &CanonicalModuleGraph, module: &ModuleKey) -> Vec<Span> {
    graph
        .module_unit(module)
        .expect("fixture importing module has an acquired canonical unit")
        .body()
        .uses()
        .iter()
        .map(|use_declaration| use_declaration.span)
        .collect()
}

fn first_use_span(graph: &CanonicalModuleGraph, module: &ModuleKey) -> Span {
    *use_spans(graph, module)
        .first()
        .expect("fixture importing module contains a parsed use declaration")
}

fn scopes(graph: &CanonicalModuleGraph) -> CanonicalProvisionalModuleScopes {
    CanonicalProvisionalModuleScopes::from_graph(graph)
        .expect("a structurally complete parser graph derives immutable provisional scopes")
}

fn resolve(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<ash_typeck::CanonicalResolvedSimpleImports, CanonicalStructuralImportError> {
    resolve_scoped_super_ordinary_function_imports_with_scopes(graph, scopes)
}

#[test]
fn scoped_super_imports_project_parent_and_sibling_targets_with_natural_and_explicit_local_names() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod host {
                pub fn parent_target(value: Int) -> Int { value }
                pub mod sibling {
                    pub fn sibling_target(value: Int) -> Int { value }
                }
                pub mod child {
                    use super::parent_target;
                    use super::sibling::sibling_target as sibling_local;
                }
            }
        "#,
        "parent-and-sibling-positive",
    );
    let host = module_key(&root_key, &["host"]);
    let child = module_key(&root_key, &["host", "child"]);
    let sibling = module_key(&root_key, &["host", "sibling"]);
    let scope_snapshot = scopes(&graph);
    let plan = resolve(&graph, &scope_snapshot)
        .expect("the dedicated route accepts parent and sibling super imports");
    let bound = bind_scoped_super_ordinary_function_imports(&graph, &scope_snapshot)
        .expect("the dedicated binder only projects the successful super plan");
    let spans = use_spans(&graph, &child);

    for (local_name, defining_module, defining_name, use_span) in [
        ("parent_target", &host, "parent_target", spans[0]),
        ("sibling_local", &sibling, "sibling_target", spans[1]),
    ] {
        let plan_binding = plan
            .binding(&child, local_name)
            .expect("the resolver stages each requested local spelling");
        let bound_binding = bound
            .binding(&child, local_name)
            .expect("the dedicated binder retains each requested local spelling");
        let edge = plan
            .import_edges()
            .iter()
            .find(|edge| edge.local_name() == local_name)
            .expect("each cross-module super import retains one edge");

        assert_eq!(bound_binding, plan_binding);
        assert_eq!(
            plan_binding.defining_identity().module_key(),
            defining_module
        );
        assert_eq!(plan_binding.defining_identity().name(), defining_name);
        assert_eq!(edge.importing_module(), &child);
        assert_eq!(edge.defining_module(), defining_module);
        assert_eq!(edge.local_name(), local_name);
        assert_eq!(edge.use_span(), use_span);
    }
}

#[test]
fn scoped_super_import_plan_and_binder_preserve_definition_identity_and_full_use_provenance() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod host {
                pub mod provider {
                    pub fn normalize(value: Int) -> Int { value }
                }
                pub mod client {
                    use super::provider::normalize as local_normalize;
                }
            }
        "#,
        "identity-and-provenance",
    );
    let client = module_key(&root_key, &["host", "client"]);
    let provider = module_key(&root_key, &["host", "provider"]);
    let target = function(&graph, &provider, "normalize");
    let target_origin = graph
        .module_unit(&provider)
        .expect("provider module is graph-owned")
        .artifact()
        .origin()
        .clone();
    let use_span = first_use_span(&graph, &client);
    let scope_snapshot = scopes(&graph);
    let plan = resolve(&graph, &scope_snapshot)
        .expect("a public sibling ordinary function is scope-resolvable through super");
    let bound = bind_scoped_super_ordinary_function_imports(&graph, &scope_snapshot)
        .expect("the binder projects the scoped-super plan without changing it");
    let plan_binding = plan
        .binding(&client, "local_normalize")
        .expect("the alias is staged under its exact requested spelling");
    let bound_binding = bound
        .binding(&client, "local_normalize")
        .expect("the binder retains the resolver binding unchanged");
    let edge = plan
        .import_edges()
        .first()
        .expect("the sibling import retains a full-use import edge");

    assert_eq!(bound_binding, plan_binding);
    assert_eq!(plan_binding.defining_identity().module_key(), &provider);
    assert_eq!(plan_binding.defining_identity().name(), "normalize");
    assert_eq!(plan_binding.declaration_span(), target.span);
    assert_eq!(plan_binding.origin(), &target_origin);
    assert_eq!(plan_binding.visibility(), &Visibility::Public);
    assert_eq!(edge.defining_identity(), plan_binding.defining_identity());
    assert_eq!(edge.declaration_span(), target.span);
    assert_eq!(edge.origin(), &target_origin);
    assert_eq!(edge.use_span(), use_span);
}

#[test]
fn scoped_super_imports_enforce_each_visibility_region_before_binding() {
    let permitted_cases = [
        (
            "public-sibling",
            r#"
                pub mod host {
                    pub mod provider { pub fn target() -> Int { 1 } }
                    pub mod client { use super::provider::target as local; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            Visibility::Public,
            1,
        ),
        (
            "crate-sibling",
            r#"
                pub mod host {
                    pub mod provider { pub(crate) fn target() -> Int { 1 } }
                    pub mod client { use super::provider::target as local; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            Visibility::Crate,
            1,
        ),
        (
            "super-sibling",
            r#"
                pub mod host {
                    pub mod provider { pub(super) fn target() -> Int { 1 } }
                    pub mod client { use super::provider::target as local; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            Visibility::Super { levels: 1 },
            1,
        ),
        (
            "restricted-sibling",
            r#"
                pub mod host {
                    pub mod provider {
                        pub(in crate::host) fn target() -> Int { 1 }
                    }
                    pub mod client { use super::provider::target as local; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            Visibility::Restricted {
                path: "crate::host".into(),
            },
            1,
        ),
        (
            "inherited-same-module",
            r#"
                pub mod host {
                    pub mod client {
                        fn target() -> Int { 1 }
                        use super::client::target as local;
                    }
                }
            "#,
            &["host", "client"][..],
            &["host", "client"][..],
            Visibility::Inherited,
            0,
        ),
        (
            "self-same-module",
            r#"
                pub mod host {
                    pub mod client {
                        pub(self) fn target() -> Int { 1 }
                        use super::client::target as local;
                    }
                }
            "#,
            &["host", "client"][..],
            &["host", "client"][..],
            Visibility::Self_,
            0,
        ),
    ];

    for (label, source, importer_segments, target_segments, expected_visibility, edge_count) in
        permitted_cases
    {
        let (root_key, graph) = parsed_graph(source, label);
        let importer = module_key(&root_key, importer_segments);
        let target = module_key(&root_key, target_segments);
        let scope_snapshot = scopes(&graph);
        let plan = resolve(&graph, &scope_snapshot)
            .expect("the selected super importer lies within its visibility region");
        let bound = bind_scoped_super_ordinary_function_imports(&graph, &scope_snapshot)
            .expect("the binder must project every permitted scoped-super visibility case");
        let binding = plan
            .binding(&importer, "local")
            .expect("the permitted import retains its requested local spelling");

        assert_eq!(bound.binding(&importer, "local"), Some(binding), "{label}");
        assert_eq!(binding.defining_identity().module_key(), &target, "{label}");
        assert_eq!(binding.visibility(), &expected_visibility, "{label}");
        assert_eq!(plan.import_edges().len(), edge_count, "{label}");
    }

    let rejected_cases = [
        (
            "private-structural-child",
            r#"
                pub mod host {
                    mod hidden { pub fn target() -> Int { 1 } }
                    pub mod client { use super::hidden::target as local; }
                }
            "#,
            &["host", "client"][..],
            &["host", "hidden"][..],
            Visibility::Inherited,
            false,
        ),
        (
            "super-outside-parent-region",
            r#"
                pub mod host {
                    pub mod provider { pub(super) fn target() -> Int { 1 } }
                }
                pub mod client { use super::host::provider::target as local; }
            "#,
            &["client"][..],
            &["host", "provider"][..],
            Visibility::Super { levels: 1 },
            true,
        ),
        (
            "restricted-outside-region",
            r#"
                pub mod host {
                    pub mod provider {
                        pub(in crate::host) fn target() -> Int { 1 }
                    }
                }
                pub mod client { use super::host::provider::target as local; }
            "#,
            &["client"][..],
            &["host", "provider"][..],
            Visibility::Restricted {
                path: "crate::host".into(),
            },
            true,
        ),
        (
            "inherited-from-child",
            r#"
                pub mod host {
                    fn target() -> Int { 1 }
                    pub mod client { use super::target as local; }
                }
            "#,
            &["host", "client"][..],
            &["host"][..],
            Visibility::Inherited,
            true,
        ),
        (
            "self-from-child",
            r#"
                pub mod host {
                    pub(self) fn target() -> Int { 1 }
                    pub mod client { use super::target as local; }
                }
            "#,
            &["host", "client"][..],
            &["host"][..],
            Visibility::Self_,
            true,
        ),
    ];

    for (label, source, importer_segments, defining_segments, expected_visibility, is_function) in
        rejected_cases
    {
        let (root_key, graph) = parsed_graph(source, label);
        let importer = module_key(&root_key, importer_segments);
        let defining_module = module_key(&root_key, defining_segments);
        let declaration_span = if is_function {
            function(&graph, &defining_module, "target").span
        } else {
            graph
                .module_unit(&module_key(&root_key, &["host"]))
                .expect("host module is graph-owned")
                .body()
                .module_decls()
                .first()
                .expect("fixture retains the private structural child")
                .span
        };
        let use_span = first_use_span(&graph, &importer);
        let scope_snapshot = scopes(&graph);
        let resolver_error = resolve(&graph, &scope_snapshot)
            .expect_err("an out-of-region super import must reject before binding");
        let binder_error = bind_scoped_super_ordinary_function_imports(&graph, &scope_snapshot)
            .expect_err("the binder must preserve the anchored visibility diagnostic");

        assert_eq!(binder_error, resolver_error, "{label}");
        match binder_error {
            CanonicalStructuralImportError::Inaccessible {
                declaration_span: rejected_declaration_span,
                use_span: rejected_use_span,
                defining_module: rejected_defining_module,
                violated_visibility,
                ..
            } => {
                assert_eq!(rejected_declaration_span, declaration_span, "{label}");
                assert_eq!(rejected_use_span, use_span, "{label}");
                assert_eq!(rejected_defining_module, defining_module, "{label}");
                assert_eq!(violated_visibility, expected_visibility, "{label}");
            }
            other => panic!("expected a {label} accessibility diagnostic, got {other:?}"),
        }
    }

    let (root_key, graph) = parsed_graph(
        r#"
            pub mod provider { pub(crate) fn target() -> Int { 1 } }
        "#,
        "crate-cross-root-predicate",
    );
    let provider = module_key(&root_key, &["provider"]);
    let external_importer = ModuleKey::root("outside")
        .expect("cross-crate fixture key is canonical")
        .child("client")
        .expect("cross-crate fixture key is canonical");
    assert!(
        !scopes(&graph)
            .is_visible_from(&Visibility::Crate, &provider, &external_importer)
            .expect("the canonical predicate decides the cross-crate pub(crate) boundary"),
        "the scoped-super route must not manufacture a cross-crate crate-visible binding",
    );
}

#[test]
fn scoped_super_imports_reject_root_and_repeated_super_with_the_full_use_span() {
    let cases = [
        (
            "root-super",
            r#"
                pub fn target() -> Int { 1 }
                use super::target;
            "#,
            &[][..],
        ),
        (
            "repeated-super",
            r#"
                pub fn target() -> Int { 1 }
                pub mod host {
                    pub mod child { use super::super::target; }
                }
            "#,
            &["host", "child"][..],
        ),
    ];

    for (label, source, importer_segments) in cases {
        let (root_key, graph) = parsed_graph(source, label);
        let importer = module_key(&root_key, importer_segments);
        let use_span = first_use_span(&graph, &importer);
        let scope_snapshot = scopes(&graph);
        let resolver_error = resolve(&graph, &scope_snapshot)
            .expect_err("the dedicated route accepts neither root nor repeated super paths");
        let binder_error = bind_scoped_super_ordinary_function_imports(&graph, &scope_snapshot)
            .expect_err("an unsupported super path publishes no binding projection");

        assert_eq!(binder_error, resolver_error, "{label}");
        match binder_error {
            CanonicalStructuralImportError::Unsupported { span, .. } => {
                assert_eq!(span, use_span, "{label}");
            }
            other => panic!("expected an unsupported {label} diagnostic, got {other:?}"),
        }
    }
}

#[test]
fn scoped_super_import_rejects_a_final_function_segment_named_super() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod host {
                fn super() -> Int { 1 }
                pub mod child { use super::super; }
            }
        "#,
        "final-super-segment",
    );
    let child = module_key(&root_key, &["host", "child"]);
    let use_span = first_use_span(&graph, &child);
    let scope_snapshot = scopes(&graph);
    let resolver_error = resolve(&graph, &scope_snapshot).expect_err(
        "a final function segment named super is a second super prefix, not a scoped-super target",
    );
    let binder_error = bind_scoped_super_ordinary_function_imports(&graph, &scope_snapshot)
        .expect_err("the binder must not project a final-super binding");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::Unsupported { span, .. } => {
            assert_eq!(span, use_span);
        }
        other => panic!("expected final-super unsupported diagnostic, got {other:?}"),
    }
}

#[test]
fn scoped_super_import_rejects_a_local_function_collision_without_a_binding_set() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod host {
                pub fn target() -> Int { 1 }
                pub mod child {
                    fn local() -> Int { 2 }
                    use super::target as local;
                }
            }
        "#,
        "local-collision",
    );
    let child = module_key(&root_key, &["host", "child"]);
    let local = function(&graph, &child, "local");
    let use_span = first_use_span(&graph, &child);
    let scope_snapshot = scopes(&graph);
    let resolver_error = resolve(&graph, &scope_snapshot)
        .expect_err("a super alias may not overwrite an ordinary local function");
    let binder_error = bind_scoped_super_ordinary_function_imports(&graph, &scope_snapshot)
        .expect_err("the binding projection is absent when local collision preflight fails");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::LocalDeclarationCollision {
            importing_module,
            name,
            declaration_span,
            use_span: rejected_use_span,
        } => {
            assert_eq!(importing_module, child);
            assert_eq!(name, "local");
            assert_eq!(declaration_span, local.span);
            assert_eq!(rejected_use_span, use_span);
        }
        other => panic!("expected local ordinary-function collision, got {other:?}"),
    }
}

#[test]
fn scoped_super_imports_reject_all_duplicate_local_spelling_shapes_atomically() {
    let duplicate_cases = [
        (
            "natural-natural",
            r#"
                pub mod host {
                    pub fn target() -> Int { 1 }
                    pub mod child {
                        use super::target;
                        use super::target;
                    }
                }
            "#,
            "target",
        ),
        (
            "natural-alias",
            r#"
                pub mod host {
                    pub fn first() -> Int { 1 }
                    pub fn second() -> Int { 2 }
                    pub mod child {
                        use super::first;
                        use super::second as first;
                    }
                }
            "#,
            "first",
        ),
        (
            "alias-alias",
            r#"
                pub mod host {
                    pub fn first() -> Int { 1 }
                    pub fn second() -> Int { 2 }
                    pub mod child {
                        use super::first as shared;
                        use super::second as shared;
                    }
                }
            "#,
            "shared",
        ),
    ];

    for (label, source, duplicate_name) in duplicate_cases {
        let (root_key, graph) = parsed_graph(source, label);
        let child = module_key(&root_key, &["host", "child"]);
        let spans = use_spans(&graph, &child);
        let scope_snapshot = scopes(&graph);
        let resolver_error = resolve(&graph, &scope_snapshot)
            .expect_err("a duplicate local spelling must reject the complete staged import set");
        let binder_error = bind_scoped_super_ordinary_function_imports(&graph, &scope_snapshot)
            .expect_err("the binder must not project a partial duplicate binding set");

        assert_eq!(binder_error, resolver_error, "{label}");
        match binder_error {
            CanonicalStructuralImportError::DuplicateBinding {
                importing_module,
                name,
                use_span,
            } => {
                assert_eq!(importing_module, child, "{label}");
                assert_eq!(name.as_ref(), duplicate_name, "{label}");
                assert_eq!(use_span, spans[1], "{label}");
            }
            other => panic!("expected a {label} duplicate binding rejection, got {other:?}"),
        }
    }
}

#[test]
fn scoped_super_imports_reject_a_late_parent_and_sibling_cycle_before_any_projection() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod host {
                pub fn host_fn() -> Int { 0 }
                pub mod a {
                    use super::host_fn;
                    use super::b::b_fn;
                    pub fn a_fn() -> Int { 1 }
                }
                pub mod b {
                    use super::c::c_fn;
                    pub fn b_fn() -> Int { 2 }
                }
                pub mod c {
                    use super::b::b_fn;
                    pub fn c_fn() -> Int { 3 }
                }
            }
        "#,
        "late-parent-sibling-cycle",
    );
    let a = module_key(&root_key, &["host", "a"]);
    let b = module_key(&root_key, &["host", "b"]);
    let c = module_key(&root_key, &["host", "c"]);
    let b_use_span = first_use_span(&graph, &b);
    let c_use_span = first_use_span(&graph, &c);
    let scope_snapshot = scopes(&graph);
    let resolver_error = resolve(&graph, &scope_snapshot)
        .expect_err("the b-to-c-to-b super edges close after earlier parent and sibling imports");
    let binder_error = bind_scoped_super_ordinary_function_imports(&graph, &scope_snapshot)
        .expect_err("the binder must not publish earlier super bindings after a late cycle");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::ImportCycle { edges } => {
            assert_eq!(edges.edges().len(), 2);
            assert!(
                edges
                    .edges()
                    .iter()
                    .all(|edge| edge.importing_module() != &a),
                "the earlier valid parent and sibling bindings remain unavailable after cycle rejection",
            );
            assert_eq!(edges.edges()[0].importing_module(), &b);
            assert_eq!(edges.edges()[0].defining_module(), &c);
            assert_eq!(edges.edges()[0].use_span(), b_use_span);
            assert_eq!(edges.edges()[1].importing_module(), &c);
            assert_eq!(edges.edges()[1].defining_module(), &b);
            assert_eq!(edges.edges()[1].use_span(), c_use_span);
        }
        other => panic!("expected a scoped-super import cycle, got {other:?}"),
    }
}

#[test]
fn scoped_super_imports_keep_file_and_inline_projection_parity_with_full_use_spans() {
    let (inline_root, inline_graph) = parsed_graph(
        r#"
            pub mod api { pub fn normalize(value: Int) -> Int { value } }
            pub mod client { use super::api::normalize as local_normalize; }
        "#,
        "inline-projection-parity",
    );
    let (file_root, file_graph) = file_backed_graph(
        r#"
            pub mod api;
            pub mod client { use super::api::normalize as local_normalize; }
        "#,
        r#"
            pub fn normalize(value: Int) -> Int { value }
        "#,
        "file-projection-parity",
    );
    let inline_client = module_key(&inline_root, &["client"]);
    let file_client = module_key(&file_root, &["client"]);
    let inline_api = module_key(&inline_root, &["api"]);
    let file_api = module_key(&file_root, &["api"]);
    let inline_use_span = first_use_span(&inline_graph, &inline_client);
    let file_use_span = first_use_span(&file_graph, &file_client);
    let inline_scopes = scopes(&inline_graph);
    let file_scopes = scopes(&file_graph);
    let inline_plan =
        resolve(&inline_graph, &inline_scopes).expect("the inline super graph is scope-resolvable");
    let file_plan = resolve(&file_graph, &file_scopes)
        .expect("the file-backed super graph is scope-resolvable");
    let inline_bound = bind_scoped_super_ordinary_function_imports(&inline_graph, &inline_scopes)
        .expect("the inline plan projects to binding-only facts");
    let file_bound = bind_scoped_super_ordinary_function_imports(&file_graph, &file_scopes)
        .expect("the file-backed plan projects to binding-only facts");
    let inline_binding = inline_plan
        .binding(&inline_client, "local_normalize")
        .expect("the inline graph retains the alias");
    let file_binding = file_plan
        .binding(&file_client, "local_normalize")
        .expect("the file graph retains the alias");
    let inline_edge = inline_plan
        .import_edges()
        .first()
        .expect("the inline graph retains one cross-module import edge");
    let file_edge = file_plan
        .import_edges()
        .first()
        .expect("the file graph retains one cross-module import edge");

    assert_eq!(
        inline_bound.binding(&inline_client, "local_normalize"),
        Some(inline_binding)
    );
    assert_eq!(
        file_bound.binding(&file_client, "local_normalize"),
        Some(file_binding)
    );
    assert_eq!(inline_binding.defining_identity().module_key(), &inline_api);
    assert_eq!(file_binding.defining_identity().module_key(), &file_api);
    assert_eq!(inline_binding.defining_identity().name(), "normalize");
    assert_eq!(file_binding.defining_identity().name(), "normalize");
    assert_eq!(inline_binding.visibility(), file_binding.visibility());
    assert_eq!(inline_edge.use_span(), inline_use_span);
    assert_eq!(file_edge.use_span(), file_use_span);
}

fn rendered_import(path: &str, name: &str, alias: Option<&str>) -> String {
    alias.map_or_else(
        || format!("use {path}::{name};"),
        |alias| format!("use {path}::{name} as {alias};"),
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn scoped_super_imports_match_the_plan_projection_for_generated_parent_sibling_alias_order_and_visibility_cases(
        function_suffix in "[a-z][a-z0-9_]{0,8}",
        alias_suffix in "[a-z][a-z0-9_]{0,8}",
        visibility_case in 0_u8..6,
        sibling_target in any::<bool>(),
        first_has_alias in any::<bool>(),
        second_has_alias in any::<bool>(),
        reversed in any::<bool>(),
    ) {
        let host = format!("host_{function_suffix}");
        let provider = format!("provider_{function_suffix}");
        let client = format!("client_{function_suffix}");
        let first = format!("first_{function_suffix}");
        let second = format!("second_{function_suffix}");
        let first_alias = format!("first_alias_{alias_suffix}");
        let second_alias = format!("second_alias_{alias_suffix}");
        let force_alias = matches!(visibility_case, 4 | 5);
        let first_local = if first_has_alias || force_alias { first_alias.as_str() } else { first.as_str() };
        let second_local = if second_has_alias || force_alias { second_alias.as_str() } else { second.as_str() };
        let first_alias = (first_has_alias || force_alias).then_some(first_alias.as_str());
        let second_alias = (second_has_alias || force_alias).then_some(second_alias.as_str());
        let (source, importer_segments, target_segments, expected_visibility, expected_edge_count) = match visibility_case {
            0..=3 => {
                let visibility = match visibility_case {
                    0 => "pub".to_owned(),
                    1 => "pub(crate)".to_owned(),
                    2 => "pub(super)".to_owned(),
                    _ => format!("pub(in crate::{host})"),
                };
                let path = if sibling_target {
                    format!("super::{provider}")
                } else {
                    "super".to_owned()
                };
                let first_import = rendered_import(&path, &first, first_alias);
                let second_import = rendered_import(&path, &second, second_alias);
                let imports = if reversed {
                    format!("{second_import} {first_import}")
                } else {
                    format!("{first_import} {second_import}")
                };
                let target_declarations = format!(
                    "{visibility} fn {first}() -> Int {{ 1 }} {visibility} fn {second}() -> Int {{ 2 }}"
                );
                let source = if sibling_target {
                    format!(
                        "pub mod {host} {{ pub mod {provider} {{ {target_declarations} }} pub mod {client} {{ {imports} }} }}"
                    )
                } else {
                    format!(
                        "pub mod {host} {{ {target_declarations} pub mod {client} {{ {imports} }} }}"
                    )
                };
                let visibility = match visibility_case {
                    0 => Visibility::Public,
                    1 => Visibility::Crate,
                    2 => Visibility::Super { levels: 1 },
                    _ => Visibility::Restricted { path: format!("crate::{host}").into() },
                };
                let target_segments = if sibling_target {
                    vec![host.as_str(), provider.as_str()]
                } else {
                    vec![host.as_str()]
                };
                (source, vec![host.as_str(), client.as_str()], target_segments, visibility, 2)
            }
            4 | 5 => {
                let visibility = if visibility_case == 4 { "" } else { "pub(self) " };
                let path = format!("super::{client}");
                let first_import = rendered_import(&path, &first, first_alias);
                let second_import = rendered_import(&path, &second, second_alias);
                let imports = if reversed {
                    format!("{second_import} {first_import}")
                } else {
                    format!("{first_import} {second_import}")
                };
                (
                    format!(
                        "pub mod {host} {{ pub mod {client} {{ {visibility}fn {first}() -> Int {{ 1 }} {visibility}fn {second}() -> Int {{ 2 }} {imports} }} }}"
                    ),
                    vec![host.as_str(), client.as_str()],
                    vec![host.as_str(), client.as_str()],
                    if visibility_case == 4 { Visibility::Inherited } else { Visibility::Self_ },
                    0,
                )
            }
            _ => unreachable!("proptest visibility range is exhaustive"),
        };
        let (root_key, graph) = parsed_graph(&source, "generated-scoped-super-import");
        let importing_module = module_key(&root_key, importer_segments.as_slice());
        let target_module = module_key(&root_key, target_segments.as_slice());
        let scope_snapshot = scopes(&graph);
        let plan = resolve(&graph, &scope_snapshot)
            .expect("every generated fixture stays inside the selected visibility region");
        let bound = bind_scoped_super_ordinary_function_imports(&graph, &scope_snapshot)
            .expect("the dedicated binder must project every generated scoped-super plan");

        for (local_name, defining_name) in [(first_local, first.as_str()), (second_local, second.as_str())] {
            let plan_binding = plan
                .binding(&importing_module, local_name)
                .expect("the resolver retains each generated local spelling");
            let bound_binding = bound
                .binding(&importing_module, local_name)
                .expect("the binder retains each generated local spelling");

            prop_assert_eq!(bound_binding, plan_binding);
            prop_assert_eq!(plan_binding.defining_identity().module_key(), &target_module);
            prop_assert_eq!(plan_binding.defining_identity().name(), defining_name);
            prop_assert_eq!(plan_binding.visibility(), &expected_visibility);
        }
        prop_assert_eq!(plan.import_edges().len(), expected_edge_count);
    }
}

#[test]
fn scoped_super_imports_reject_every_reserved_path_and_target_form_with_the_full_use_span() {
    let cases = [
        (
            "self-head",
            r#"
                pub mod host {
                    pub fn target() -> Int { 1 }
                    pub mod child { use self::target; }
                }
            "#,
            &["host", "child"][..],
        ),
        (
            "crate-head",
            r#"
                pub mod host {
                    pub fn target() -> Int { 1 }
                    pub mod child { use crate::host::target; }
                }
            "#,
            &["host", "child"][..],
        ),
        (
            "unprefixed-head",
            r#"
                pub mod host {
                    pub fn target() -> Int { 1 }
                    pub mod child { use target; }
                }
            "#,
            &["host", "child"][..],
        ),
        (
            "std-head",
            r#"
                pub mod host { pub mod child { use std::target; } }
            "#,
            &["host", "child"][..],
        ),
        (
            "external-head",
            r#"
                pub mod host { pub mod child { use external::target; } }
            "#,
            &["host", "child"][..],
        ),
        (
            "grouped-path",
            r#"
                pub mod host {
                    pub fn target() -> Int { 1 }
                    pub mod child { use super::{target}; }
                }
            "#,
            &["host", "child"][..],
        ),
        (
            "glob-path",
            r#"
                pub mod host {
                    pub fn target() -> Int { 1 }
                    pub mod child { use super::*; }
                }
            "#,
            &["host", "child"][..],
        ),
        (
            "public-use",
            r#"
                pub mod host {
                    pub fn target() -> Int { 1 }
                    pub mod child { pub use super::target; }
                }
            "#,
            &["host", "child"][..],
        ),
        (
            "restricted-use",
            r#"
                pub mod host {
                    pub fn target() -> Int { 1 }
                    pub mod child { pub(crate) use super::target; }
                }
            "#,
            &["host", "child"][..],
        ),
        (
            "nonfunction-target",
            r#"
                pub mod host {
                    pub type Target = Value { value: Int };
                    pub mod child { use super::Target; }
                }
            "#,
            &["host", "child"][..],
        ),
    ];

    for (label, source, importer_segments) in cases {
        let (root_key, graph) = parsed_graph(source, label);
        let importer = module_key(&root_key, importer_segments);
        let use_span = first_use_span(&graph, &importer);
        let scope_snapshot = scopes(&graph);
        let resolver_error = resolve(&graph, &scope_snapshot)
            .expect_err("reserved forms remain outside the dedicated scoped-super route");
        let binder_error = bind_scoped_super_ordinary_function_imports(&graph, &scope_snapshot)
            .expect_err("the binder must not publish a reserved-form binding");

        assert_eq!(binder_error, resolver_error, "{label}");
        match binder_error {
            CanonicalStructuralImportError::Unsupported { span, .. } => {
                assert_eq!(span, use_span, "{label}");
            }
            other => panic!("expected an unsupported {label} diagnostic, got {other:?}"),
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

fn source_tree_contains(path: &Path, identifier: &str) -> bool {
    if path.is_file() {
        return path.extension().is_some_and(|extension| extension == "rs")
            && fs::read_to_string(path)
                .expect("read Rust source under an authority fence")
                .contains(identifier);
    }

    fs::read_dir(path)
        .expect("read authority-fenced source directory")
        .flatten()
        .any(|entry| source_tree_contains(&entry.path(), identifier))
}

#[test]
fn scoped_super_import_route_has_only_dedicated_binding_authority_and_no_later_layer_path() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dedicated_path = manifest_dir.join("src/canonical_structural_module_binder.rs");
    let dedicated_source = fs::read_to_string(&dedicated_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated scoped-super binder at {}: {error}",
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
    let planner_path = manifest_dir.join("src/canonical_simple_import_planner.rs");
    let planner_source = fs::read_to_string(&planner_path).unwrap_or_else(|error| {
        panic!(
            "read scoped-super planner authority boundary at {}: {error}",
            planner_path.display()
        )
    });

    assert!(
        dedicated_source.contains("resolve_scoped_super_ordinary_function_imports_with_scopes"),
        "only the dedicated scoped binder consumes the scoped-super resolver",
    );
    assert!(
        dedicated_source.contains("bind_scoped_super_ordinary_function_imports"),
        "the private structural binder owns the named scoped-super projection",
    );
    assert!(
        dedicated_source.contains(".map(|plan| plan.into_bound_set())"),
        "the dedicated binder must only project a successful resolver plan",
    );
    assert!(
        lib_source.contains("bind_scoped_super_ordinary_function_imports"),
        "lib.rs alone re-exports the dedicated scoped-super binding API",
    );
    assert!(
        lib_source.contains("resolve_scoped_super_ordinary_function_imports_with_scopes"),
        "lib.rs re-exports the scoped-super resolver as the public Type-layer entry point",
    );

    let checksum = Command::new("sha256sum")
        .arg(&generic_path)
        .output()
        .expect("run sha256sum for the generic binder authority fence");
    assert!(
        checksum.status.success(),
        "sha256sum must succeed for the generic binder"
    );
    let actual_checksum = std::str::from_utf8(&checksum.stdout)
        .expect("sha256sum output is UTF-8")
        .split_whitespace()
        .next()
        .expect("sha256sum emits a digest");
    assert_eq!(
        actual_checksum, "aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6",
        "the generic compatibility binder must remain byte-for-byte generic-only",
    );

    for forbidden_identifier in [
        "CanonicalStructuralImportError",
        "CanonicalProvisionalModuleScopes",
        "resolve_simple_parsed_imports_with_scopes",
        "resolve_scoped_simple_ordinary_function_imports_with_scopes",
        "resolve_scoped_grouped_ordinary_function_imports_with_scopes",
        "resolve_scoped_super_ordinary_function_imports_with_scopes",
        "bind_scoped_super_ordinary_function_imports",
    ] {
        assert!(
            !contains_exact_identifier(&generic_source, forbidden_identifier),
            "the generic binder must remain generic-only and omit {forbidden_identifier}",
        );
    }

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
        "Admission",
        "Runtime",
        "Daemon",
        "Cli",
    ] {
        assert!(
            !contains_exact_identifier(&dedicated_source, forbidden_identifier),
            "the dedicated scoped-super binder must not gain wider authority: {forbidden_identifier}",
        );
        assert!(
            !contains_exact_identifier(&planner_source, forbidden_identifier),
            "the scoped-super planner must not gain wider authority: {forbidden_identifier}",
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
            !dedicated_source.contains(forbidden_bypass),
            "the dedicated scoped-super binder must not bypass the resolver: {forbidden_bypass}",
        );
        assert!(
            !planner_source.contains(forbidden_bypass),
            "the scoped-super planner must not bypass parser-owned graph facts: {forbidden_bypass}",
        );
    }

    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("ash-typeck lives below the workspace root");
    for later_layer_source in [
        manifest_dir.join("src/module_interface_finalization.rs"),
        manifest_dir.join("src/module_core_cps_lowering.rs"),
        workspace_root.join("crates/ash-engine/src"),
        workspace_root.join("crates/ash-cli/src"),
    ] {
        for route_identifier in [
            "resolve_scoped_super_ordinary_function_imports_with_scopes",
            "bind_scoped_super_ordinary_function_imports",
        ] {
            assert!(
                !source_tree_contains(&later_layer_source, route_identifier),
                "later-layer source {} must not consume scoped-super authority {route_identifier}",
                later_layer_source.display(),
            );
        }
    }
}
