//! TASK-2068 RED contracts for scoped grouped ordinary-function imports.
//!
//! This target reserves the parser-first `use crate::path::{name, name as
//! local}` route. It is a Type-layer binding plan only: grouped imports must
//! retain per-member parser anchors, provisional-scope visibility, atomic
//! preflight, and the dedicated binding projection without widening generic
//! imports or authorizing later layers.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::surface::{Definition, FnDef, Visibility};
use ash_parser::use_tree::UsePath;
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver, Span};
use ash_typeck::{
    CanonicalProvisionalModuleScopes, CanonicalStructuralImportError,
    bind_scoped_grouped_ordinary_function_imports,
    resolve_scoped_grouped_ordinary_function_imports_with_scopes,
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
            "ash-task-2068-scoped-grouped-imports-{label}-{}-{serial}",
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

fn group_member_spans(graph: &CanonicalModuleGraph, module: &ModuleKey) -> Vec<Span> {
    let use_declaration = graph
        .module_unit(module)
        .expect("fixture importing module has an acquired canonical unit")
        .body()
        .uses()
        .first()
        .expect("fixture importing module contains one grouped parsed use declaration");
    let UsePath::Nested(_, members) = &use_declaration.path else {
        panic!("fixture must retain parser-owned grouped use members");
    };
    members.iter().map(|member| member.span).collect()
}

fn scopes(graph: &CanonicalModuleGraph) -> CanonicalProvisionalModuleScopes {
    CanonicalProvisionalModuleScopes::from_graph(graph)
        .expect("a structurally complete parser graph derives immutable provisional scopes")
}

fn resolve(
    graph: &CanonicalModuleGraph,
    scopes: &CanonicalProvisionalModuleScopes,
) -> Result<ash_typeck::CanonicalResolvedSimpleImports, CanonicalStructuralImportError> {
    resolve_scoped_grouped_ordinary_function_imports_with_scopes(graph, scopes)
}

#[test]
fn grouped_imports_project_root_and_deep_natural_and_aliased_members() {
    let (root_key, root_graph) = parsed_graph(
        r#"
            pub fn root_first() -> Int { 1 }
            pub fn root_second() -> Int { 2 }
            pub mod client { use crate::{root_first, root_second as local_second}; }
        "#,
        "root-positive",
    );
    let root_client = module_key(&root_key, &["client"]);
    let root_scopes = scopes(&root_graph);
    let root_plan = resolve(&root_graph, &root_scopes)
        .expect("a root grouped ordinary-function import is scope-resolvable");
    let root_bound = bind_scoped_grouped_ordinary_function_imports(&root_graph, &root_scopes)
        .expect("the dedicated grouped binder projects the root grouped plan");

    for (local_name, defining_name) in [
        ("root_first", "root_first"),
        ("local_second", "root_second"),
    ] {
        let binding = root_plan
            .binding(&root_client, local_name)
            .expect("every grouped root member is staged under its requested local spelling");
        assert_eq!(root_bound.binding(&root_client, local_name), Some(binding));
        assert_eq!(binding.defining_identity().module_key(), &root_key);
        assert_eq!(binding.defining_identity().name(), defining_name);
    }
    assert_eq!(root_plan.import_edges().len(), 2);

    let (deep_root, deep_graph) = parsed_graph(
        r#"
            pub mod api {
                pub mod text {
                    pub fn first() -> Int { 1 }
                    pub fn second() -> Int { 2 }
                }
            }
            use crate::api::text::{first, second as local_second};
        "#,
        "deep-positive",
    );
    let text_key = module_key(&deep_root, &["api", "text"]);
    let deep_scopes = scopes(&deep_graph);
    let deep_plan = resolve(&deep_graph, &deep_scopes)
        .expect("a deep grouped ordinary-function import is scope-resolvable");
    let deep_bound = bind_scoped_grouped_ordinary_function_imports(&deep_graph, &deep_scopes)
        .expect("the dedicated grouped binder projects the deep grouped plan");

    for (local_name, defining_name) in [("first", "first"), ("local_second", "second")] {
        let binding = deep_plan
            .binding(&deep_root, local_name)
            .expect("every grouped deep member is staged under its requested local spelling");
        assert_eq!(deep_bound.binding(&deep_root, local_name), Some(binding));
        assert_eq!(binding.defining_identity().module_key(), &text_key);
        assert_eq!(binding.defining_identity().name(), defining_name);
    }
    assert_eq!(deep_plan.import_edges().len(), 2);
}

#[test]
fn grouped_members_preserve_defining_identity_and_individual_parser_provenance() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub fn first() -> Int { 1 }
                pub fn second() -> Int { 2 }
            }
            pub mod client { use crate::api::{first, second as local_second}; }
        "#,
        "member-identity",
    );
    let api_key = module_key(&root_key, &["api"]);
    let client_key = module_key(&root_key, &["client"]);
    let target_origin = graph
        .module_unit(&api_key)
        .expect("target module is graph-owned")
        .artifact()
        .origin()
        .clone();
    let member_spans = group_member_spans(&graph, &client_key);
    let scope_snapshot = scopes(&graph);
    let plan = resolve(&graph, &scope_snapshot)
        .expect("the grouped resolver retains parser facts for each selected member");
    let bound = bind_scoped_grouped_ordinary_function_imports(&graph, &scope_snapshot)
        .expect("the dedicated grouped binder retains every successful member projection");

    for (index, local_name, defining_name) in [(0, "first", "first"), (1, "local_second", "second")]
    {
        let target = function(&graph, &api_key, defining_name);
        let binding = plan
            .binding(&client_key, local_name)
            .expect("the plan retains every selected grouped member");
        let edge = plan
            .import_edges()
            .iter()
            .find(|edge| edge.local_name() == local_name)
            .expect("every cross-module grouped member retains its own edge");

        assert_eq!(bound.binding(&client_key, local_name), Some(binding));
        assert_eq!(binding.defining_identity().module_key(), &api_key);
        assert_eq!(binding.defining_identity().name(), defining_name);
        assert_eq!(binding.declaration_span(), target.span);
        assert_eq!(binding.origin(), &target_origin);
        assert_eq!(binding.visibility(), &Visibility::Public);
        assert_eq!(edge.defining_identity(), binding.defining_identity());
        assert_eq!(edge.declaration_span(), target.span);
        assert_eq!(edge.origin(), &target_origin);
        assert_eq!(edge.visibility(), &Visibility::Public);
        assert_eq!(edge.use_span(), member_spans[index]);
    }
    assert_ne!(member_spans[0], member_spans[1]);
}

#[test]
fn grouped_imports_anchor_visibility_diagnostics_at_the_rejected_member() {
    let permitted_cases = [
        (
            "public",
            r#"
                pub mod host {
                    pub mod provider { pub fn target() -> Int { 1 } }
                    pub mod client { use crate::host::provider::{target as local}; }
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
                    pub mod client { use crate::host::provider::{target as local}; }
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
                    pub mod client { use crate::host::provider::{target as local}; }
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
                    pub mod client { use crate::host::provider::{target as local}; }
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
                        use crate::host::provider::{target as local};
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
                        use crate::host::provider::{target as local};
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
            .expect("the selected grouped importer lies within its target visibility region");
        let bound = bind_scoped_grouped_ordinary_function_imports(&graph, &scope_snapshot)
            .expect("the grouped binder projects every resolver-permitted visibility case");
        let binding = plan
            .binding(&importing_module, "local")
            .expect("the permitted group member is staged by the resolver");

        assert_eq!(bound.binding(&importing_module, "local"), Some(binding));
        assert_eq!(binding.defining_identity().module_key(), &target_module);
        assert_eq!(binding.visibility(), &expected_visibility);
    }

    let rejected_cases = [
        (
            "public-behind-private-child",
            r#"
                mod hidden { pub fn target() -> Int { 1 } }
                pub mod client { use crate::hidden::{target as local}; }
            "#,
            &["client"][..],
            &["hidden"][..],
            Visibility::Inherited,
            true,
        ),
        (
            "super-at-root",
            r#"
                pub mod host { pub mod provider { pub(super) fn target() -> Int { 1 } } }
                use crate::host::provider::{target as local};
            "#,
            &[][..],
            &["host", "provider"][..],
            Visibility::Super { levels: 1 },
            false,
        ),
        (
            "restricted-at-root",
            r#"
                pub mod host {
                    pub mod provider { pub(in crate::host) fn target() -> Int { 1 } }
                }
                use crate::host::provider::{target as local};
            "#,
            &[][..],
            &["host", "provider"][..],
            Visibility::Restricted {
                path: "crate::host".into(),
            },
            false,
        ),
        (
            "inherited-at-sibling",
            r#"
                pub mod host {
                    pub mod provider { fn target() -> Int { 1 } }
                    pub mod client { use crate::host::provider::{target as local}; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            Visibility::Inherited,
            false,
        ),
        (
            "self-at-sibling",
            r#"
                pub mod host {
                    pub mod provider { pub(self) fn target() -> Int { 1 } }
                    pub mod client { use crate::host::provider::{target as local}; }
                }
            "#,
            &["host", "client"][..],
            &["host", "provider"][..],
            Visibility::Self_,
            false,
        ),
    ];

    for (label, source, importer_segments, rejected_segments, expected_visibility, child_failure) in
        rejected_cases
    {
        let (root_key, graph) = parsed_graph(source, label);
        let importing_module = module_key(&root_key, importer_segments);
        let rejected_module = module_key(&root_key, rejected_segments);
        let rejected_span = if child_failure {
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
        let rejected_member_span = group_member_spans(&graph, &importing_module)[0];
        let scope_snapshot = scopes(&graph);
        let resolver_error = resolve(&graph, &scope_snapshot)
            .expect_err("an inaccessible grouped member must fail before any binding publishes");
        let binder_error = bind_scoped_grouped_ordinary_function_imports(&graph, &scope_snapshot)
            .expect_err("the grouped binder preserves the resolver visibility diagnostic");

        assert_eq!(binder_error, resolver_error);
        match binder_error {
            CanonicalStructuralImportError::Inaccessible {
                declaration_span,
                use_span,
                defining_module,
                violated_visibility,
                ..
            } => {
                assert_eq!(declaration_span, rejected_span, "{label}");
                assert_eq!(use_span, rejected_member_span, "{label}");
                assert_eq!(defining_module, rejected_module, "{label}");
                assert_eq!(violated_visibility, expected_visibility, "{label}");
            }
            other => panic!("expected anchored {label} visibility rejection, got {other:?}"),
        }
    }

    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api { pub(crate) fn target() -> Int { 1 } }
            pub mod client { use crate::api::{target as local}; }
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
        "the grouped route must not manufacture a cross-crate alias for a crate-visible target",
    );
}

#[test]
fn grouped_import_rejects_a_later_member_that_collides_with_a_local_function_atomically() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub fn first() -> Int { 1 }
                pub fn second() -> Int { 2 }
            }
            fn local() -> Int { 3 }
            use crate::api::{first, second as local};
        "#,
        "local-collision",
    );
    let local = function(&graph, &root_key, "local");
    let rejected_member_span = group_member_spans(&graph, &root_key)[1];
    let scope_snapshot = scopes(&graph);
    let resolver_error = resolve(&graph, &scope_snapshot)
        .expect_err("a later grouped member may not overwrite an ordinary local function");
    let binder_error = bind_scoped_grouped_ordinary_function_imports(&graph, &scope_snapshot)
        .expect_err("the group publishes no binding projection when collision preflight fails");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::LocalDeclarationCollision {
            importing_module,
            name,
            declaration_span,
            use_span,
        } => {
            assert_eq!(importing_module, root_key);
            assert_eq!(name, "local");
            assert_eq!(declaration_span, local.span);
            assert_eq!(use_span, rejected_member_span);
        }
        other => panic!("expected anchored grouped local collision, got {other:?}"),
    }
}

#[test]
fn grouped_import_rejects_all_duplicate_local_spelling_shapes_at_the_second_member() {
    let duplicate_cases = [
        (
            "natural-natural",
            r#"
                pub mod api { pub fn first() -> Int { 1 } }
                use crate::api::{first, first};
            "#,
            "first",
        ),
        (
            "natural-alias",
            r#"
                pub mod api {
                    pub fn first() -> Int { 1 }
                    pub fn second() -> Int { 2 }
                }
                use crate::api::{first, second as first};
            "#,
            "first",
        ),
        (
            "alias-alias",
            r#"
                pub mod api {
                    pub fn first() -> Int { 1 }
                    pub fn second() -> Int { 2 }
                }
                use crate::api::{first as shared, second as shared};
            "#,
            "shared",
        ),
    ];

    for (label, source, duplicate_name) in duplicate_cases {
        let (root_key, graph) = parsed_graph(source, label);
        let rejected_member_span = group_member_spans(&graph, &root_key)[1];
        let scope_snapshot = scopes(&graph);
        let resolver_error = resolve(&graph, &scope_snapshot)
            .expect_err("a duplicate local spelling in one group must reject atomically");
        let binder_error = bind_scoped_grouped_ordinary_function_imports(&graph, &scope_snapshot)
            .expect_err("the dedicated binder must not project a partially staged group");

        assert_eq!(binder_error, resolver_error);
        match binder_error {
            CanonicalStructuralImportError::DuplicateBinding {
                importing_module,
                name,
                use_span,
            } => {
                assert_eq!(importing_module, root_key, "{label}");
                assert_eq!(name.as_ref(), duplicate_name, "{label}");
                assert_eq!(use_span, rejected_member_span, "{label}");
            }
            other => panic!("expected anchored {label} duplicate rejection, got {other:?}"),
        }
    }
}

#[test]
fn grouped_import_rejects_a_structural_child_member_as_an_anchored_unsupported_target() {
    let (root_key, graph) = parsed_graph(
        r#"
            mod api { fn greet() -> Int { 1 } }
            use crate::{api};
        "#,
        "direct-child-member",
    );
    let member_span = group_member_spans(&graph, &root_key)[0];
    let scope_snapshot = scopes(&graph);
    let resolver_error = resolve(&graph, &scope_snapshot)
        .expect_err("a grouped member may select an ordinary function, not a structural child");
    let binder_error = bind_scoped_grouped_ordinary_function_imports(&graph, &scope_snapshot)
        .expect_err("the grouped binder must preserve the unsupported-member diagnostic");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::Unsupported { span, reason } => {
            assert_eq!(span, member_span);
            assert_eq!(reason, "only ordinary function targets are accepted");
        }
        other => panic!("expected anchored nonfunction rejection, got {other:?}"),
    }
}

#[test]
fn grouped_imports_reject_late_cross_module_cycles_without_publishing_earlier_members() {
    let (root_key, graph) = file_inline_tail_cycle_graph(
        r#"
            use crate::b::{b_fn};
            pub fn a_fn() -> Int { 1 }
        "#,
        r#"
            use crate::c::{c_fn};
            pub fn b_fn() -> Int { 2 }
        "#,
        r#"
            use crate::b::{b_fn};
            pub fn c_fn() -> Int { 3 }
        "#,
        "late-cycle-atomicity",
    );
    let a_key = module_key(&root_key, &["a"]);
    let b_key = module_key(&root_key, &["b"]);
    let c_key = module_key(&root_key, &["c"]);
    let b_member_span = group_member_spans(&graph, &b_key)[0];
    let c_member_span = group_member_spans(&graph, &c_key)[0];
    let scope_snapshot = scopes(&graph);
    let resolver_error = resolve(&graph, &scope_snapshot)
        .expect_err("the later b-to-c-to-b grouped members close a canonical cycle");
    let binder_error = bind_scoped_grouped_ordinary_function_imports(&graph, &scope_snapshot)
        .expect_err("the binder must not publish the earlier a-to-b group after cycle rejection");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::ImportCycle { edges } => {
            assert_eq!(edges.edges().len(), 2);
            assert!(
                edges
                    .edges()
                    .iter()
                    .all(|edge| edge.importing_module() != &a_key),
                "the earlier valid grouped member remains unavailable after a later cycle failure",
            );
            assert_eq!(edges.edges()[0].importing_module(), &b_key);
            assert_eq!(edges.edges()[0].defining_module(), &c_key);
            assert_eq!(edges.edges()[0].use_span(), b_member_span);
            assert_eq!(edges.edges()[1].importing_module(), &c_key);
            assert_eq!(edges.edges()[1].defining_module(), &b_key);
            assert_eq!(edges.edges()[1].use_span(), c_member_span);
        }
        other => panic!("expected grouped canonical import cycle, got {other:?}"),
    }
}

#[test]
fn grouped_imports_keep_file_and_inline_projection_parity_including_member_spans() {
    let (inline_root, inline_graph) = parsed_graph(
        r#"
            pub mod api {
                pub fn normalize(value: Int) -> Int { value }
                pub fn sanitize(value: Int) -> Int { value }
            }
            use crate::api::{normalize, sanitize as clean};
        "#,
        "inline-projection-parity",
    );
    let (file_root, file_graph) = file_backed_graph(
        r#"
            pub mod api;
            use crate::api::{normalize, sanitize as clean};
        "#,
        r#"
            pub fn normalize(value: Int) -> Int { value }
            pub fn sanitize(value: Int) -> Int { value }
        "#,
        "file-projection-parity",
    );
    let inline_api = module_key(&inline_root, &["api"]);
    let file_api = module_key(&file_root, &["api"]);
    let inline_member_spans = group_member_spans(&inline_graph, &inline_root);
    let file_member_spans = group_member_spans(&file_graph, &file_root);
    let inline_scopes = scopes(&inline_graph);
    let file_scopes = scopes(&file_graph);
    let inline_plan = resolve(&inline_graph, &inline_scopes)
        .expect("the inline grouped graph is scope-resolvable");
    let file_plan = resolve(&file_graph, &file_scopes)
        .expect("the file-backed grouped graph is scope-resolvable");
    let inline_bound = bind_scoped_grouped_ordinary_function_imports(&inline_graph, &inline_scopes)
        .expect("the inline grouped plan projects to bindings");
    let file_bound = bind_scoped_grouped_ordinary_function_imports(&file_graph, &file_scopes)
        .expect("the file-backed grouped plan projects to bindings");

    for (index, local_name, defining_name) in
        [(0, "normalize", "normalize"), (1, "clean", "sanitize")]
    {
        let inline_binding = inline_plan
            .binding(&inline_root, local_name)
            .expect("the inline graph retains each grouped local name");
        let file_binding = file_plan
            .binding(&file_root, local_name)
            .expect("the file graph retains each grouped local name");
        let inline_edge = inline_plan
            .import_edges()
            .iter()
            .find(|edge| edge.local_name() == local_name)
            .expect("the inline graph retains the member edge");
        let file_edge = file_plan
            .import_edges()
            .iter()
            .find(|edge| edge.local_name() == local_name)
            .expect("the file graph retains the member edge");

        assert_eq!(
            inline_bound.binding(&inline_root, local_name),
            Some(inline_binding)
        );
        assert_eq!(
            file_bound.binding(&file_root, local_name),
            Some(file_binding)
        );
        assert_eq!(inline_binding.defining_identity().module_key(), &inline_api);
        assert_eq!(file_binding.defining_identity().module_key(), &file_api);
        assert_eq!(inline_binding.defining_identity().name(), defining_name);
        assert_eq!(file_binding.defining_identity().name(), defining_name);
        assert_eq!(inline_binding.visibility(), file_binding.visibility());
        assert_eq!(inline_edge.use_span(), inline_member_spans[index]);
        assert_eq!(file_edge.use_span(), file_member_spans[index]);
    }
}

fn rendered_member(name: &str, alias: Option<&str>) -> String {
    alias.map_or_else(|| name.to_owned(), |alias| format!("{name} as {alias}"))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn grouped_imports_match_resolver_projection_for_generated_names_positions_aliases_orders_and_visibility_regions(
        function_suffix in "[a-z][a-z0-9_]{0,8}",
        alias_suffix in "[a-z][a-z0-9_]{0,8}",
        visibility_case in 0_u8..6,
        target_at_root in any::<bool>(),
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
        let local_first = if first_has_alias || matches!(visibility_case, 4 | 5) {
            first_alias.as_str()
        } else {
            first.as_str()
        };
        let local_second = if second_has_alias || matches!(visibility_case, 4 | 5) {
            second_alias.as_str()
        } else {
            second.as_str()
        };
        let first_member = rendered_member(
            &first,
            (first_has_alias || matches!(visibility_case, 4 | 5)).then_some(first_alias.as_str()),
        );
        let second_member = rendered_member(
            &second,
            (second_has_alias || matches!(visibility_case, 4 | 5)).then_some(second_alias.as_str()),
        );
        let members = if reversed {
            format!("{second_member}, {first_member}")
        } else {
            format!("{first_member}, {second_member}")
        };

        let (source, importer_segments, target_segments, expected_visibility, cross_module) =
            match visibility_case {
                0 | 1 if target_at_root => {
                    let visibility = if visibility_case == 0 { "pub" } else { "pub(crate)" };
                    (
                        format!(
                            "{visibility} fn {first}() -> Int {{ 1 }} {visibility} fn {second}() -> Int {{ 2 }} pub mod {client} {{ use crate::{{{members}}}; }}"
                        ),
                        vec![client.as_str()],
                        Vec::new(),
                        if visibility_case == 0 { Visibility::Public } else { Visibility::Crate },
                        true,
                    )
                }
                0 | 1 => {
                    let visibility = if visibility_case == 0 { "pub" } else { "pub(crate)" };
                    (
                        format!(
                            "pub mod {host} {{ pub mod {provider} {{ {visibility} fn {first}() -> Int {{ 1 }} {visibility} fn {second}() -> Int {{ 2 }} }} pub mod {client} {{ use crate::{host}::{provider}::{{{members}}}; }} }}"
                        ),
                        vec![host.as_str(), client.as_str()],
                        vec![host.as_str(), provider.as_str()],
                        if visibility_case == 0 { Visibility::Public } else { Visibility::Crate },
                        true,
                    )
                }
                2 => (
                    format!(
                        "pub mod {host} {{ pub mod {provider} {{ pub(super) fn {first}() -> Int {{ 1 }} pub(super) fn {second}() -> Int {{ 2 }} }} pub mod {client} {{ use crate::{host}::{provider}::{{{members}}}; }} }}"
                    ),
                    vec![host.as_str(), client.as_str()],
                    vec![host.as_str(), provider.as_str()],
                    Visibility::Super { levels: 1 },
                    true,
                ),
                3 => (
                    format!(
                        "pub mod {host} {{ pub mod {provider} {{ pub(in crate::{host}) fn {first}() -> Int {{ 1 }} pub(in crate::{host}) fn {second}() -> Int {{ 2 }} }} pub mod {client} {{ use crate::{host}::{provider}::{{{members}}}; }} }}"
                    ),
                    vec![host.as_str(), client.as_str()],
                    vec![host.as_str(), provider.as_str()],
                    Visibility::Restricted { path: format!("crate::{host}").into() },
                    true,
                ),
                4 => (
                    format!(
                        "pub mod {host} {{ pub mod {provider} {{ fn {first}() -> Int {{ 1 }} fn {second}() -> Int {{ 2 }} use crate::{host}::{provider}::{{{members}}}; }} }}"
                    ),
                    vec![host.as_str(), provider.as_str()],
                    vec![host.as_str(), provider.as_str()],
                    Visibility::Inherited,
                    false,
                ),
                _ => (
                    format!(
                        "pub mod {host} {{ pub mod {provider} {{ pub(self) fn {first}() -> Int {{ 1 }} pub(self) fn {second}() -> Int {{ 2 }} use crate::{host}::{provider}::{{{members}}}; }} }}"
                    ),
                    vec![host.as_str(), provider.as_str()],
                    vec![host.as_str(), provider.as_str()],
                    Visibility::Self_,
                    false,
                ),
            };
        let (root_key, graph) = parsed_graph(&source, "generated-scoped-grouped-import");
        let importing_module = module_key(&root_key, importer_segments.as_slice());
        let target_module = module_key(&root_key, target_segments.as_slice());
        let scope_snapshot = scopes(&graph);
        let plan = resolve(&graph, &scope_snapshot)
            .expect("each generated fixture stays inside its selected canonical visibility region");
        let bound = bind_scoped_grouped_ordinary_function_imports(&graph, &scope_snapshot)
            .expect("the dedicated grouped binder projects every generated plan");

        for (local_name, defining_name) in [(local_first, first.as_str()), (local_second, second.as_str())] {
            let plan_binding = plan
                .binding(&importing_module, local_name)
                .expect("the resolver retains each generated grouped local name");
            let bound_binding = bound
                .binding(&importing_module, local_name)
                .expect("the binder retains each generated grouped local name");

            prop_assert_eq!(bound_binding, plan_binding);
            prop_assert_eq!(plan_binding.defining_identity().module_key(), &target_module);
            prop_assert_eq!(plan_binding.defining_identity().name(), defining_name);
            prop_assert_eq!(plan_binding.visibility(), &expected_visibility);
        }
        prop_assert_eq!(plan.import_edges().len(), if cross_module { 2 } else { 0 });
        if cross_module {
            prop_assert!(plan.import_edges().iter().any(|edge| edge.local_name() == local_first));
            prop_assert!(plan.import_edges().iter().any(|edge| edge.local_name() == local_second));
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
fn grouped_import_route_has_only_dedicated_binding_authority_and_no_later_layer_path() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dedicated_path = manifest_dir.join("src/canonical_structural_module_binder.rs");
    let dedicated_source = fs::read_to_string(&dedicated_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated grouped binder at {}: {error}",
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
            "read grouped import planner authority boundary at {}: {error}",
            planner_path.display()
        )
    });

    assert!(
        dedicated_source.contains("resolve_scoped_grouped_ordinary_function_imports_with_scopes"),
        "only the dedicated grouped binder consumes the grouped resolver",
    );
    assert!(
        dedicated_source.contains("bind_scoped_grouped_ordinary_function_imports"),
        "the private structural binder owns the named grouped projection",
    );
    assert!(
        dedicated_source.contains(".map(|plan| plan.into_bound_set())"),
        "the grouped binder must only project a successful resolver plan",
    );
    assert!(
        lib_source.contains("bind_scoped_grouped_ordinary_function_imports"),
        "lib.rs alone re-exports the dedicated grouped binding API",
    );

    for scoped_term in [
        "CanonicalProvisionalModuleScopes",
        "resolve_scoped_grouped_ordinary_function_imports_with_scopes",
        "CanonicalStructuralImportError",
    ] {
        assert!(
            !generic_source.contains(scoped_term),
            "the generic binder must remain generic-only and omit {scoped_term}",
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
            "the dedicated grouped binder must not gain wider authority: {forbidden_identifier}",
        );
        assert!(
            !contains_exact_identifier(&planner_source, forbidden_identifier),
            "the grouped planner must not gain wider authority: {forbidden_identifier}",
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
            "the dedicated grouped binder must not bypass the grouped resolver: {forbidden_bypass}",
        );
        assert!(
            !planner_source.contains(forbidden_bypass),
            "the grouped planner must not bypass parser-owned graph facts: {forbidden_bypass}",
        );
    }
}
