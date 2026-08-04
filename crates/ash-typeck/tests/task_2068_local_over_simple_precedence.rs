//! TASK-2068 RED contract for local-over-explicit scoped import precedence.
//!
//! This target specifies one dedicated, Type-only route for a same-module
//! ordinary function to shadow the natural binding of one inherited simple
//! `crate::` ordinary-function import. It does not widen the existing
//! M-SIMPLE route.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use ash_parser::surface::{Definition, FnDef, Visibility};
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver};
use ash_typeck::{
    CanonicalProvisionalModuleScopes, CanonicalStructuralImportError,
    bind_scoped_simple_local_precedence_imports, bind_scoped_simple_ordinary_function_imports,
    resolve_scoped_simple_local_precedence_imports_with_scopes,
    resolve_scoped_simple_ordinary_function_imports_with_scopes,
};
use proptest::prelude::*;
use sha2::{Digest, Sha256};

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "ash-task-2068-local-over-simple-{label}-{}-{}",
            std::process::id(),
            NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(&root).expect("temporary fixture directory should be created");
        Self { root }
    }

    fn write(&self, relative_path: impl AsRef<Path>, contents: &str) -> PathBuf {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("fixture parent directory should be created");
        }
        fs::write(&path, contents).expect("fixture source should be written");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn resolve_fixture(tree: &TempTree, root_source: &str) -> (ModuleKey, CanonicalModuleGraph) {
    let root_path = tree.write("main.ash", root_source);
    let root_module = ModuleKey::root("app").expect("fixture crate key should be canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_module.clone(), root_path)
        .expect("fixture source should resolve through the canonical parser graph");
    (root_module, graph)
}

fn function<'graph>(
    graph: &'graph CanonicalModuleGraph,
    module: &ModuleKey,
    name: &str,
) -> &'graph FnDef {
    graph
        .module_unit(module)
        .expect("fixture module should remain graph-owned")
        .body()
        .definitions()
        .iter()
        .find_map(|definition| match definition {
            Definition::Function(function) if function.name.as_ref() == name => Some(function),
            _ => None,
        })
        .expect("fixture should retain the requested ordinary function")
}

fn module(root: &ModuleKey, segment: &str) -> ModuleKey {
    root.child(segment)
        .expect("fixture module segment should remain canonical")
}

fn nested_module(root: &ModuleKey, segments: &[String]) -> ModuleKey {
    segments.iter().fold(root.clone(), |module_key, segment| {
        module(&module_key, segment)
    })
}

// TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-WINS-NONCOLLIDING
#[test]
fn local_wins_and_non_colliding_simple_import_binds() {
    let shadowed_fixture = TempTree::new("wins-shadowed");
    let (shadowed_root, shadowed_graph) = resolve_fixture(
        &shadowed_fixture,
        r#"
            pub mod api {
                pub fn shadowed(value: Int) -> Int { value }
                pub fn imported(value: Int) -> Int { value }
            }
            pub mod client {
                fn shadowed(value: Int) -> Int { value }
                use crate::api::shadowed;
            }
        "#,
    );
    let shadowed_client = module(&shadowed_root, "client");
    let shadowed_scopes = CanonicalProvisionalModuleScopes::from_graph(&shadowed_graph)
        .expect("shadowed fixture graph should derive provisional scopes");
    let shadowed_resolved = resolve_scoped_simple_local_precedence_imports_with_scopes(
        &shadowed_graph,
        &shadowed_scopes,
    )
    .expect("local-over-simple resolution should allow a same-name local declaration");
    let shadowed_bound =
        bind_scoped_simple_local_precedence_imports(&shadowed_graph, &shadowed_scopes)
            .expect("local-over-simple binding should allow a same-name local declaration");

    assert!(
        shadowed_resolved
            .binding(&shadowed_client, "shadowed")
            .is_none(),
        "a shadowed candidate is not observable through public resolver bindings"
    );
    assert!(
        shadowed_bound
            .binding(&shadowed_client, "shadowed")
            .is_none(),
        "the same-module ordinary function wins over its natural-name import"
    );

    let imported_fixture = TempTree::new("wins-imported");
    let (imported_root, imported_graph) = resolve_fixture(
        &imported_fixture,
        r#"
            pub mod api {
                pub fn shadowed(value: Int) -> Int { value }
                pub fn imported(value: Int) -> Int { value }
            }
            pub mod client {
                fn shadowed(value: Int) -> Int { value }
                use crate::api::imported;
            }
        "#,
    );
    let imported_client = module(&imported_root, "client");
    let imported_scopes = CanonicalProvisionalModuleScopes::from_graph(&imported_graph)
        .expect("non-colliding fixture graph should derive provisional scopes");
    let imported_resolved = resolve_scoped_simple_local_precedence_imports_with_scopes(
        &imported_graph,
        &imported_scopes,
    )
    .expect("a non-colliding natural-name import should resolve");
    let imported_bound =
        bind_scoped_simple_local_precedence_imports(&imported_graph, &imported_scopes)
            .expect("a non-colliding natural-name import should bind");

    assert!(
        imported_resolved
            .binding(&imported_client, "imported")
            .is_some(),
        "a distinct imported natural name remains publicly bound"
    );
    assert_eq!(
        imported_bound.binding(&imported_client, "imported"),
        imported_resolved.binding(&imported_client, "imported"),
        "the binder projects the non-colliding resolver binding"
    );

    let same_module_fixture = TempTree::new("same-module-no-edge");
    let (same_module_root, same_module_graph) = resolve_fixture(
        &same_module_fixture,
        r#"
            pub mod client {
                pub fn local(value: Int) -> Int { value }
                use crate::client::local;
            }
        "#,
    );
    let same_module_client = module(&same_module_root, "client");
    let same_module_scopes = CanonicalProvisionalModuleScopes::from_graph(&same_module_graph)
        .expect("same-module fixture graph should derive provisional scopes");
    let same_module_resolved = resolve_scoped_simple_local_precedence_imports_with_scopes(
        &same_module_graph,
        &same_module_scopes,
    )
    .expect("a same-module natural-name import should not form a dependency cycle");
    let same_module_bound =
        bind_scoped_simple_local_precedence_imports(&same_module_graph, &same_module_scopes)
            .expect("a same-module natural-name import should project without a binding");

    assert!(
        same_module_resolved
            .binding(&same_module_client, "local")
            .is_none(),
        "the local ordinary function shadows a same-module natural-name import"
    );
    assert!(
        same_module_bound
            .binding(&same_module_client, "local")
            .is_none(),
        "the binding-only projection must retain the local-precedence result"
    );
    assert!(
        same_module_resolved.import_edges().is_empty(),
        "a same-module import must not add a self edge to canonical cycle detection"
    );
}

// TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-IDENTITY-EDGE
#[test]
fn local_over_simple_precedence_preserves_shadowed_candidate_identity_origin_spans_visibility_and_edges()
 {
    let fixture = TempTree::new("identity-edge");
    let (root, graph) = resolve_fixture(
        &fixture,
        r#"
            pub mod api {
                pub fn shadowed(value: Int) -> Int { value }
            }
            pub mod client {
                fn shadowed(value: Int) -> Int { value }
                use crate::api::shadowed;
            }
        "#,
    );
    let api = module(&root, "api");
    let client = module(&root, "client");
    let use_span = graph
        .module_unit(&client)
        .expect("client should remain graph-owned")
        .body()
        .uses()[0]
        .span;
    let target = function(&graph, &api, "shadowed");
    let target_origin = graph
        .module_unit(&api)
        .expect("api should remain graph-owned")
        .artifact()
        .origin()
        .clone();
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("fixture graph should derive provisional scopes");
    let resolved = resolve_scoped_simple_local_precedence_imports_with_scopes(&graph, &scopes)
        .expect("the resolver should retain the shadowed candidate edge");
    let bound = bind_scoped_simple_local_precedence_imports(&graph, &scopes)
        .expect("the binder should project no shadowed candidate binding");

    assert_eq!(
        resolved.import_edges().len(),
        1,
        "the selected cross-module target retains one edge"
    );
    let edge = &resolved.import_edges()[0];
    assert_eq!(edge.importing_module(), &client);
    assert_eq!(edge.defining_module(), &api);
    assert_eq!(edge.defining_identity().module_key(), &api);
    assert_eq!(edge.defining_identity().name(), "shadowed");
    assert_eq!(edge.local_name(), "shadowed");
    assert_eq!(edge.use_span(), use_span);
    assert_eq!(edge.declaration_span(), target.span);
    assert_eq!(edge.origin(), &target_origin);
    assert_eq!(edge.visibility(), &Visibility::Public);
    assert!(resolved.binding(&client, "shadowed").is_none());
    assert!(bound.binding(&client, "shadowed").is_none());
}

// TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-ALL-SHADOWED-EMPTY-BINDING
#[test]
fn local_over_simple_precedence_all_shadowed_imports_leave_empty_binding_projection_with_retained_edges()
 {
    let fixture = TempTree::new("all-shadowed");
    let (root, graph) = resolve_fixture(
        &fixture,
        r#"
            pub mod api {
                pub fn alpha(value: Int) -> Int { value }
            }
            pub mod client {
                fn alpha(value: Int) -> Int { value }
                use crate::api::alpha;
            }
        "#,
    );
    let api = module(&root, "api");
    let client = module(&root, "client");
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("fixture graph should derive provisional scopes");
    let resolved = resolve_scoped_simple_local_precedence_imports_with_scopes(&graph, &scopes)
        .expect("all shadowed candidates should resolve to an empty public binding projection");
    let bound = bind_scoped_simple_local_precedence_imports(&graph, &scopes)
        .expect("all shadowed candidates should bind to an empty projection");

    assert!(resolved.binding(&client, "alpha").is_none());
    assert!(bound.binding(&client, "alpha").is_none());
    assert_eq!(resolved.import_edges().len(), 1);
    let edge = &resolved.import_edges()[0];
    assert_eq!(edge.importing_module(), &client);
    assert_eq!(edge.defining_module(), &api);
    assert_eq!(edge.local_name(), "alpha");
}

// TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-CYCLE-ATOMICITY
#[test]
fn local_over_simple_precedence_hidden_two_module_cycle_rejects_atomically() {
    let fixture = TempTree::new("hidden-cycle");
    let (root, graph) = resolve_fixture(
        &fixture,
        r#"
            pub mod a {
                pub fn supplied_by_a(value: Int) -> Int { value }
                fn supplied_by_b(value: Int) -> Int { value }
                use crate::b::supplied_by_b;
            }
            pub mod b {
                pub fn supplied_by_b(value: Int) -> Int { value }
                fn supplied_by_a(value: Int) -> Int { value }
                use crate::a::supplied_by_a;
            }
        "#,
    );
    let a = module(&root, "a");
    let b = module(&root, "b");
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("fixture graph should derive provisional scopes");
    let resolver_error =
        resolve_scoped_simple_local_precedence_imports_with_scopes(&graph, &scopes)
            .expect_err("a real hidden two-module cycle must publish no resolver plan");
    let binder_error = bind_scoped_simple_local_precedence_imports(&graph, &scopes)
        .expect_err("a real hidden two-module cycle must publish no binding projection");

    assert_eq!(binder_error, resolver_error);
    match resolver_error {
        CanonicalStructuralImportError::ImportCycle { edges } => {
            assert_eq!(edges.len(), 2, "the reported cycle has two directed edges");
            assert!(
                edges
                    .iter()
                    .any(|edge| edge.importing_module() == &a && edge.defining_module() == &b)
            );
            assert!(
                edges
                    .iter()
                    .any(|edge| edge.importing_module() == &b && edge.defining_module() == &a)
            );
            assert!(edges.iter().all(|edge| {
                graph
                    .module_unit(edge.importing_module())
                    .expect("cycle importer should remain graph-owned")
                    .body()
                    .definitions()
                    .iter()
                    .any(|definition| {
                        matches!(definition, Definition::Function(function) if function.name.as_ref() == edge.local_name())
                    })
            }));
        }
        other => panic!("expected an atomic canonical import-cycle diagnostic, got {other:?}"),
    }
}

// TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-VISIBILITY-SHAPE
#[test]
fn local_over_simple_precedence_retains_visibility_and_shape_failures() {
    let cases = [
        (
            "private-target",
            r#"
                pub mod api { fn hidden(value: Int) -> Int { value } }
                pub mod client {
                    fn hidden(value: Int) -> Int { value }
                    use crate::api::hidden;
                }
            "#,
            &["client"][..],
        ),
        (
            "public-use",
            r#"
                pub mod api { pub fn visible(value: Int) -> Int { value } }
                pub mod client {
                    fn visible(value: Int) -> Int { value }
                    pub use crate::api::visible;
                }
            "#,
            &["client"][..],
        ),
        (
            "alias",
            r#"
                pub mod api { pub fn visible(value: Int) -> Int { value } }
                pub mod client {
                    fn visible(value: Int) -> Int { value }
                    use crate::api::visible as local_visible;
                }
            "#,
            &["client"][..],
        ),
        (
            "root-direct-function",
            r#"
                pub fn visible(value: Int) -> Int { value }
                pub mod client {
                    fn visible(value: Int) -> Int { value }
                    use crate::visible;
                }
            "#,
            &["client"][..],
        ),
        (
            "group",
            r#"
                pub mod api { pub fn visible(value: Int) -> Int { value } }
                pub mod client {
                    fn visible(value: Int) -> Int { value }
                    use crate::api::{visible};
                }
            "#,
            &["client"][..],
        ),
        (
            "glob",
            r#"
                pub mod api { pub fn visible(value: Int) -> Int { value } }
                pub mod client {
                    fn visible(value: Int) -> Int { value }
                    use crate::api::*;
                }
            "#,
            &["client"][..],
        ),
        (
            "self",
            r#"
                pub mod client {
                    pub fn visible(value: Int) -> Int { value }
                    use self::visible;
                }
            "#,
            &["client"][..],
        ),
        (
            "super",
            r#"
                pub mod parent {
                    pub fn visible(value: Int) -> Int { value }
                    pub mod client {
                        fn visible(value: Int) -> Int { value }
                        use super::visible;
                    }
                }
            "#,
            &["parent", "client"][..],
        ),
        (
            "multiple-uses",
            r#"
                pub mod api {
                    pub fn one(value: Int) -> Int { value }
                    pub fn two(value: Int) -> Int { value }
                }
                pub mod client {
                    use crate::api::one;
                    use crate::api::two;
                }
            "#,
            &["client"][..],
        ),
        (
            "private-structural-path",
            r#"
                mod hidden { pub fn visible(value: Int) -> Int { value } }
                pub mod client {
                    fn visible(value: Int) -> Int { value }
                    use crate::hidden::visible;
                }
            "#,
            &["client"][..],
        ),
        (
            "nonfunction-target",
            r#"
                pub mod api { pub type Alias = Int; }
                pub mod client { use crate::api::Alias; }
            "#,
            &["client"][..],
        ),
    ];

    for (label, source, importing_segments) in cases {
        let fixture = TempTree::new(label);
        let (root, graph) = resolve_fixture(&fixture, source);
        let client = importing_segments
            .iter()
            .fold(root.clone(), |module_key, segment| {
                module(&module_key, segment)
            });
        let uses = graph
            .module_unit(&client)
            .expect("client should remain graph-owned")
            .body()
            .uses();
        let use_span = uses[0].span;
        let expected_rejected_span = if label == "multiple-uses" {
            uses[1].span
        } else {
            use_span
        };
        let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
            .expect("fixture graph should derive provisional scopes");
        let resolver_error =
            resolve_scoped_simple_local_precedence_imports_with_scopes(&graph, &scopes)
                .expect_err("visibility and use-shape failures must publish no resolver plan");
        let binder_error = bind_scoped_simple_local_precedence_imports(&graph, &scopes)
            .expect_err("visibility and use-shape failures must publish no binding projection");

        assert_eq!(binder_error, resolver_error, "{label}");
        match (label, resolver_error) {
            (
                "private-target",
                CanonicalStructuralImportError::Inaccessible {
                    declaration_span,
                    use_span: rejected_use_span,
                    defining_module,
                    violated_visibility,
                    ..
                },
            ) => {
                let api = module(&root, "api");
                assert_eq!(declaration_span, function(&graph, &api, "hidden").span);
                assert_eq!(rejected_use_span, use_span);
                assert_eq!(defining_module, api);
                assert_eq!(violated_visibility, Visibility::Inherited);
            }
            (
                "private-structural-path",
                CanonicalStructuralImportError::Inaccessible {
                    use_span: rejected_use_span,
                    defining_module,
                    violated_visibility,
                    ..
                },
            ) => {
                assert_eq!(rejected_use_span, use_span);
                assert_eq!(defining_module, module(&root, "hidden"));
                assert_eq!(violated_visibility, Visibility::Inherited);
            }
            (
                "public-use"
                | "alias"
                | "root-direct-function"
                | "group"
                | "glob"
                | "self"
                | "super"
                | "multiple-uses"
                | "nonfunction-target",
                CanonicalStructuralImportError::Unsupported { span, .. },
            ) => {
                assert_eq!(span, expected_rejected_span);
            }
            (_, other) => panic!("expected the retained {label} diagnostic, got {other:?}"),
        }
    }
}

// TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-FILE-INLINE-PARITY
#[test]
fn local_over_simple_precedence_file_and_inline_normalized_scope_results_match() {
    let client_source = r#"
        pub mod client {
            fn shadowed(value: Int) -> Int { value }
            use crate::api::shadowed;
        }
    "#;
    let api_source = "pub fn shadowed(value: Int) -> Int { value }";
    let inline_fixture = TempTree::new("inline-parity");
    let file_fixture = TempTree::new("file-parity");
    let inline_source = format!("pub mod api {{ {api_source} }}\n{client_source}");
    let file_source = format!("pub mod api;\n{client_source}");
    let (inline_root, inline_graph) = resolve_fixture(&inline_fixture, &inline_source);
    file_fixture.write("api.ash", api_source);
    let (file_root, file_graph) = resolve_fixture(&file_fixture, &file_source);
    let inline_api = module(&inline_root, "api");
    let inline_client = module(&inline_root, "client");
    let file_api = module(&file_root, "api");
    let file_client = module(&file_root, "client");
    let inline_scopes = CanonicalProvisionalModuleScopes::from_graph(&inline_graph)
        .expect("inline fixture graph should derive provisional scopes");
    let file_scopes = CanonicalProvisionalModuleScopes::from_graph(&file_graph)
        .expect("file-backed fixture graph should derive provisional scopes");

    assert_eq!(
        inline_scopes.normalized_scope_projection(),
        file_scopes.normalized_scope_projection(),
        "layout is not part of the normalized provisional scope result"
    );

    let inline_resolved =
        resolve_scoped_simple_local_precedence_imports_with_scopes(&inline_graph, &inline_scopes)
            .expect("inline fixture should resolve through the precedence route");
    let file_resolved =
        resolve_scoped_simple_local_precedence_imports_with_scopes(&file_graph, &file_scopes)
            .expect("file-backed fixture should resolve through the precedence route");
    let inline_bound = bind_scoped_simple_local_precedence_imports(&inline_graph, &inline_scopes)
        .expect("inline fixture should bind through the precedence route");
    let file_bound = bind_scoped_simple_local_precedence_imports(&file_graph, &file_scopes)
        .expect("file-backed fixture should bind through the precedence route");

    assert_eq!(inline_resolved.import_edges().len(), 1);
    assert_eq!(file_resolved.import_edges().len(), 1);
    let inline_edge = &inline_resolved.import_edges()[0];
    let file_edge = &file_resolved.import_edges()[0];
    assert_eq!(inline_edge.importing_module(), file_edge.importing_module());
    assert_eq!(inline_edge.defining_module(), file_edge.defining_module());
    assert_eq!(
        inline_edge.defining_identity(),
        file_edge.defining_identity()
    );
    assert_eq!(inline_edge.local_name(), file_edge.local_name());
    assert_eq!(inline_edge.visibility(), file_edge.visibility());
    assert_eq!(inline_edge.defining_module(), &inline_api);
    assert_eq!(file_edge.defining_module(), &file_api);
    assert!(matches!(
        inline_edge.origin(),
        ModuleArtifactOrigin::Inline { parent, .. } if parent == &inline_root
    ));
    assert!(matches!(file_edge.origin(), ModuleArtifactOrigin::File(_)));
    assert!(
        inline_resolved
            .binding(&inline_client, "shadowed")
            .is_none()
    );
    assert!(file_resolved.binding(&file_client, "shadowed").is_none());
    assert!(inline_bound.binding(&inline_client, "shadowed").is_none());
    assert!(file_bound.binding(&file_client, "shadowed").is_none());
}

// TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-PROPERTY
proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn local_over_simple_precedence_generated_names_collision_subsets_and_source_forms(
        name_seed in 0_u8..16,
        collision_mask in 0_u8..2,
        structural_depth in 1_usize..4,
        file_backed in any::<bool>(),
    ) {
        let fixture = TempTree::new(&format!(
            "property-name-{name_seed}-mask-{collision_mask}-depth-{structural_depth}-file-{file_backed}"
        ));
        let name = format!("candidate_{name_seed}");
        let segments: Vec<_> = (0..structural_depth)
            .map(|index| format!("scope_{index}"))
            .collect();
        let structural_path = segments.join("::");
        let target_source = (1..structural_depth).rev().fold(
            format!("pub fn {name}(value: Int) -> Int {{ value }}"),
            |source, index| format!("pub mod {} {{ {source} }}", segments[index]),
        );
        let local_source = if collision_mask == 1 {
            format!("fn {name}(value: Int) -> Int {{ value }}")
        } else {
            String::new()
        };
        let client_source = format!(
            "pub mod client {{ {local_source} use crate::{structural_path}::{name}; }}"
        );
        let root_source = if file_backed {
            fixture.write(format!("{}.ash", segments[0]), &target_source);
            format!("pub mod {}; {client_source}", segments[0])
        } else {
            format!("pub mod {} {{ {target_source} }} {client_source}", segments[0])
        };
        let (root, graph) = resolve_fixture(&fixture, &root_source);
        let client = module(&root, "client");
        let target_module = nested_module(&root, &segments);
        let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
            .expect("generated fixture graph should derive provisional scopes");
        let resolved = resolve_scoped_simple_local_precedence_imports_with_scopes(&graph, &scopes)
            .expect("generated local collisions should not reject candidate selection");
        let bound = bind_scoped_simple_local_precedence_imports(&graph, &scopes)
            .expect("generated local collisions should not reject binding projection");

        prop_assert_eq!(resolved.import_edges().len(), 1);
        let edge = &resolved.import_edges()[0];
        prop_assert_eq!(edge.importing_module(), &client);
        prop_assert_eq!(edge.defining_module(), &target_module);
        prop_assert_eq!(edge.defining_identity().module_key(), &target_module);
        prop_assert_eq!(edge.defining_identity().name(), name.as_str());
        prop_assert_eq!(edge.local_name(), name.as_str());
        if collision_mask == 1 {
            prop_assert!(resolved.binding(&client, &name).is_none());
            prop_assert!(bound.binding(&client, &name).is_none());
        } else {
            let candidate = resolved
                .binding(&client, &name)
                .expect("every non-shadowed generated public candidate should bind");
            prop_assert_eq!(candidate.defining_identity(), edge.defining_identity());
            prop_assert_eq!(bound.binding(&client, &name), Some(candidate));
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

fn source_tree_contains(path: &Path, identifier: &str) -> std::io::Result<bool> {
    if path.is_file() {
        return Ok(path.extension().is_some_and(|extension| extension == "rs")
            && fs::read_to_string(path)?.contains(identifier));
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if source_tree_contains(&entry.path(), identifier)? {
            return Ok(true);
        }
    }
    Ok(false)
}

// TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-AUTHORITY-FENCE
#[test]
fn local_over_simple_precedence_has_dedicated_authority_only() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dedicated_path = manifest_dir.join("src/canonical_structural_module_binder.rs");
    let dedicated_source = fs::read_to_string(&dedicated_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated local-over-simple binder at {}: {error}",
            dedicated_path.display()
        )
    });
    let planner_path = manifest_dir.join("src/canonical_simple_import_planner.rs");
    let planner_source = fs::read_to_string(&planner_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the local-over-simple resolver in {}: {error}",
            planner_path.display()
        )
    });
    let generic_path = manifest_dir.join("src/canonical_module_binder.rs");
    let generic_source = fs::read_to_string(&generic_path).unwrap_or_else(|error| {
        panic!(
            "read generic compatibility binder at {}: {error}",
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

    for required_identifier in [
        "bind_scoped_simple_local_precedence_imports",
        "resolve_scoped_simple_local_precedence_imports_with_scopes",
    ] {
        assert!(
            contains_exact_identifier(&dedicated_source, required_identifier),
            "the dedicated binder must own {required_identifier}",
        );
        assert!(
            contains_exact_identifier(&lib_source, required_identifier),
            "lib.rs must expose the bounded Type-layer API {required_identifier}",
        );
    }
    assert!(
        contains_exact_identifier(
            &planner_source,
            "resolve_scoped_simple_local_precedence_imports_with_scopes"
        ),
        "only the scoped-simple planner may resolve local-over-explicit candidates",
    );

    let generic_binder_bytes = fs::read(&generic_path).unwrap_or_else(|error| {
        panic!(
            "read generic compatibility binder under an authority fence at {}: {error}",
            generic_path.display()
        )
    });
    assert_eq!(
        format!("{:x}", Sha256::digest(generic_binder_bytes)),
        "aea47f6aae83b4b7e3bfaa9ee9a7561d76b12a5f82b50c42d3fd2d98e23ed8b6",
        "the generic compatibility binder must remain byte-for-byte generic-only",
    );
    for forbidden_identifier in [
        "bind_scoped_simple_local_precedence_imports",
        "resolve_scoped_simple_local_precedence_imports_with_scopes",
    ] {
        assert!(
            !contains_exact_identifier(&generic_source, forbidden_identifier),
            "the generic compatibility binder must omit {forbidden_identifier}",
        );
    }

    for forbidden_identifier in [
        "CanonicalCheckedFunction",
        "PublicModuleInterface",
        "FinalizedModuleInterface",
        "InterfaceImportResolver",
        "RawCoreProgram",
        "CoreExpr",
        "CpsProgram",
        "Engine",
        "Admission",
        "Runtime",
    ] {
        assert!(
            !contains_exact_identifier(&dedicated_source, forbidden_identifier),
            "the dedicated binder must not gain wider authority: {forbidden_identifier}",
        );
        assert!(
            !contains_exact_identifier(&planner_source, forbidden_identifier),
            "the scoped-simple planner must not gain wider authority: {forbidden_identifier}",
        );
    }

    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("ash-typeck should live below the workspace root");
    for later_layer_source in [
        manifest_dir.join("src/module_interface_finalization.rs"),
        manifest_dir.join("src/module_core_cps_lowering.rs"),
        workspace_root.join("crates/ash-engine/src"),
        workspace_root.join("crates/ash-runtime/src"),
        workspace_root.join("crates/ash-cli/src"),
    ] {
        for route_identifier in [
            "bind_scoped_simple_local_precedence_imports",
            "resolve_scoped_simple_local_precedence_imports_with_scopes",
        ] {
            assert!(
                !source_tree_contains(&later_layer_source, route_identifier).unwrap_or_else(
                    |error| {
                        panic!(
                            "read later-layer authority fence at {}: {error}",
                            later_layer_source.display()
                        )
                    },
                ),
                "later-layer source {} must not consume {route_identifier}",
                later_layer_source.display(),
            );
        }
    }
}

// TEST-MOD-REAL-004-SCOPED-SIMPLE-LOCAL-PRECEDENCE-LEGACY-M-SIMPLE-REGRESSION
#[test]
fn local_over_simple_precedence_keeps_legacy_simple_local_collision_rejection() {
    let fixture = TempTree::new("legacy-local-collision");
    let (root, graph) = resolve_fixture(
        &fixture,
        r#"
            pub mod api {
                pub fn visible(value: Int) -> Int { value }
            }
            pub mod client {
                fn visible(value: Int) -> Int { value }
                use crate::api::visible;
            }
        "#,
    );
    let client = module(&root, "client");
    let local = function(&graph, &client, "visible");
    let use_span = graph
        .module_unit(&client)
        .expect("client should remain graph-owned")
        .body()
        .uses()[0]
        .span;
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("fixture graph should derive provisional scopes");
    let resolver_error =
        resolve_scoped_simple_ordinary_function_imports_with_scopes(&graph, &scopes)
            .expect_err("the existing M-SIMPLE route must continue rejecting local collisions");
    let binder_error = bind_scoped_simple_ordinary_function_imports(&graph, &scopes)
        .expect_err("the existing M-SIMPLE binder must continue rejecting local collisions");

    assert_eq!(binder_error, resolver_error);
    match resolver_error {
        CanonicalStructuralImportError::LocalDeclarationCollision {
            importing_module,
            name,
            declaration_span,
            use_span: rejected_use_span,
        } => {
            assert_eq!(importing_module, client);
            assert_eq!(name, "visible");
            assert_eq!(declaration_span, local.span);
            assert_eq!(rejected_use_span, use_span);
        }
        other => panic!("expected the legacy local-collision diagnostic, got {other:?}"),
    }
}
