//! TASK-2068 RED contract for local-over-glob scoped import precedence.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::{ModuleArtifactOrigin, ModuleKey};
use ash_parser::surface::{Definition, FnDef, Visibility};
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver};
use ash_typeck::{
    CanonicalProvisionalModuleScopes, CanonicalStructuralImportError,
    bind_scoped_glob_local_precedence_imports,
    resolve_scoped_glob_local_precedence_imports_with_scopes,
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
            "ash-task-2068-local-over-glob-{label}-{}-{}",
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

// TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-WINS-NONCOLLIDING
#[test]
fn local_wins_and_non_colliding_import_binds() {
    let fixture = TempTree::new("wins-noncolliding");
    let (root, graph) = resolve_fixture(
        &fixture,
        r#"
            pub mod api {
                pub fn shadowed(value: Int) -> Int { value }
                pub fn imported(value: Int) -> Int { value }
            }
            pub mod client {
                fn shadowed(value: Int) -> Int { value }
                use crate::api::*;
            }
        "#,
    );
    let client = module(&root, "client");
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("fixture graph should derive provisional scopes");

    let resolved = resolve_scoped_glob_local_precedence_imports_with_scopes(&graph, &scopes)
        .expect("local-over-glob resolution should succeed");
    let bound = bind_scoped_glob_local_precedence_imports(&graph, &scopes)
        .expect("local-over-glob binding should succeed");

    assert!(
        resolved.binding(&client, "shadowed").is_none(),
        "a shadowed imported declaration is not observable through public resolver bindings"
    );
    assert!(
        bound.binding(&client, "shadowed").is_none(),
        "the local declaration wins over its same-name glob candidate"
    );
    assert_eq!(
        bound.binding(&client, "imported"),
        resolved.binding(&client, "imported"),
        "a non-colliding public function remains bound"
    );
}

// TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-IDENTITY-EDGE
#[test]
fn local_over_glob_precedence_preserves_shadowed_and_bound_candidate_identity_origin_spans_visibility_and_edges()
 {
    let fixture = TempTree::new("identity-edge");
    let (root, graph) = resolve_fixture(
        &fixture,
        r#"
            pub mod api {
                pub fn shadowed(value: Int) -> Int { value }
                pub fn imported(value: Int) -> Int { value }
            }
            pub mod client {
                fn shadowed(value: Int) -> Int { value }
                use crate::api::*;
            }
        "#,
    );
    let api = module(&root, "api");
    let client = module(&root, "client");
    let client_use_span = graph
        .module_unit(&client)
        .expect("client should remain graph-owned")
        .body()
        .uses()[0]
        .span;
    let target_origin = graph
        .module_unit(&api)
        .expect("api should remain graph-owned")
        .artifact()
        .origin()
        .clone();
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("fixture graph should derive provisional scopes");
    let resolved = resolve_scoped_glob_local_precedence_imports_with_scopes(&graph, &scopes)
        .expect("the resolver should retain both selected candidates");
    let bound = bind_scoped_glob_local_precedence_imports(&graph, &scopes)
        .expect("the binder should project only the non-shadowed candidate");

    assert_eq!(
        resolved.import_edges().len(),
        2,
        "every selected function retains one cross-module edge"
    );
    for name in ["shadowed", "imported"] {
        let target = function(&graph, &api, name);
        let edges: Vec<_> = resolved
            .import_edges()
            .iter()
            .filter(|edge| edge.local_name() == name)
            .collect();

        assert_eq!(edges.len(), 1, "exactly one edge is retained for {name}");

        let edge = edges[0];
        assert_eq!(edge.importing_module(), &client);
        assert_eq!(edge.defining_module(), &api);
        assert_eq!(edge.defining_identity().module_key(), &api);
        assert_eq!(edge.defining_identity().name(), name);
        assert_eq!(edge.local_name(), name);
        assert_eq!(edge.use_span(), client_use_span);
        assert_eq!(edge.declaration_span(), target.span);
        assert_eq!(edge.origin(), &target_origin);
        assert_eq!(edge.visibility(), &Visibility::Public);

        if name == "shadowed" {
            assert!(
                resolved.binding(&client, name).is_none(),
                "the public resolver binding projection hides local-shadowed imports"
            );
            assert!(bound.binding(&client, name).is_none());
        } else {
            let candidate = resolved
                .binding(&client, name)
                .expect("the non-shadowed imported name remains publicly bound");

            assert_eq!(candidate.defining_identity(), edge.defining_identity());
            assert_eq!(candidate.declaration_span(), target.span);
            assert_eq!(candidate.origin(), &target_origin);
            assert_eq!(candidate.visibility(), &Visibility::Public);
            assert_eq!(bound.binding(&client, name), Some(candidate));
        }
    }
}

// TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-ALL-SHADOWED-EMPTY-BINDING
#[test]
fn local_over_glob_precedence_all_shadowed_imports_leave_empty_binding_projection_with_retained_edges()
 {
    let fixture = TempTree::new("all-shadowed");
    let (root, graph) = resolve_fixture(
        &fixture,
        r#"
            pub mod api {
                pub fn alpha(value: Int) -> Int { value }
                pub fn beta(value: Int) -> Int { value }
            }
            pub mod client {
                fn alpha(value: Int) -> Int { value }
                fn beta(value: Int) -> Int { value }
                use crate::api::*;
            }
        "#,
    );
    let client = module(&root, "client");
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("fixture graph should derive provisional scopes");
    let resolved = resolve_scoped_glob_local_precedence_imports_with_scopes(&graph, &scopes)
        .expect("local declarations should not reject candidate selection");
    let bound = bind_scoped_glob_local_precedence_imports(&graph, &scopes)
        .expect("all shadowed candidates should produce an empty binding projection");

    assert_eq!(resolved.import_edges().len(), 2);
    for name in ["alpha", "beta"] {
        assert!(resolved.binding(&client, name).is_none());
        assert!(bound.binding(&client, name).is_none());
        assert!(
            resolved
                .import_edges()
                .iter()
                .any(|edge| edge.local_name() == name)
        );
    }
}

// TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-CYCLE-ATOMICITY
#[test]
fn local_over_glob_precedence_hidden_two_module_cycle_rejects_atomically() {
    let fixture = TempTree::new("hidden-cycle");
    let (root, graph) = resolve_fixture(
        &fixture,
        r#"
            pub mod a {
                pub fn supplied_by_a(value: Int) -> Int { value }
                pub fn supplied_by_b(value: Int) -> Int { value }
                use crate::b::*;
            }
            pub mod b {
                pub fn supplied_by_a(value: Int) -> Int { value }
                pub fn supplied_by_b(value: Int) -> Int { value }
                use crate::a::*;
            }
        "#,
    );
    let a = module(&root, "a");
    let b = module(&root, "b");
    let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
        .expect("fixture graph should derive provisional scopes");
    let resolver_error = resolve_scoped_glob_local_precedence_imports_with_scopes(&graph, &scopes)
        .expect_err("an actual two-module glob cycle must publish no resolver plan");
    let binder_error = bind_scoped_glob_local_precedence_imports(&graph, &scopes)
        .expect_err("an actual two-module glob cycle must publish no binding projection");

    assert_eq!(binder_error, resolver_error);
    match resolver_error {
        CanonicalStructuralImportError::ImportCycle { edges } => {
            assert_eq!(
                edges.len(),
                2,
                "the reported cycle contains its two directed closing edges"
            );
            assert!(
                edges
                    .iter()
                    .any(|edge| { edge.importing_module() == &a && edge.defining_module() == &b })
            );
            assert!(
                edges
                    .iter()
                    .any(|edge| { edge.importing_module() == &b && edge.defining_module() == &a })
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

// TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-VISIBILITY-SHAPE
#[test]
fn local_over_glob_precedence_retains_visibility_and_shape_failures() {
    let cases = [
        (
            "private-target",
            r#"
                pub mod api { fn hidden(value: Int) -> Int { value } }
                pub mod client {
                    fn hidden(value: Int) -> Int { value }
                    use crate::api::*;
                }
            "#,
        ),
        (
            "public-use",
            r#"
                pub mod api { pub fn visible(value: Int) -> Int { value } }
                pub mod client {
                    fn visible(value: Int) -> Int { value }
                    pub use crate::api::*;
                }
            "#,
        ),
    ];

    for (label, source) in cases {
        let fixture = TempTree::new(label);
        let (root, graph) = resolve_fixture(&fixture, source);
        let api = module(&root, "api");
        let client = module(&root, "client");
        let use_span = graph
            .module_unit(&client)
            .expect("client should remain graph-owned")
            .body()
            .uses()[0]
            .span;
        let scopes = CanonicalProvisionalModuleScopes::from_graph(&graph)
            .expect("fixture graph should derive provisional scopes");
        let resolver_error =
            resolve_scoped_glob_local_precedence_imports_with_scopes(&graph, &scopes)
                .expect_err("visibility and use-shape failures must publish no resolver plan");
        let binder_error = bind_scoped_glob_local_precedence_imports(&graph, &scopes)
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
                assert_eq!(declaration_span, function(&graph, &api, "hidden").span);
                assert_eq!(rejected_use_span, use_span);
                assert_eq!(defining_module, api);
                assert_eq!(violated_visibility, Visibility::Inherited);
            }
            ("public-use", CanonicalStructuralImportError::Unsupported { span, .. }) => {
                assert_eq!(span, use_span);
            }
            (_, other) => panic!("expected the retained {label} diagnostic, got {other:?}"),
        }
    }
}

// TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-FILE-INLINE-PARITY
#[test]
fn local_over_glob_precedence_file_and_inline_normalized_scope_results_match() {
    let client_source = r#"
        pub mod client {
            fn shadowed(value: Int) -> Int { value }
            use crate::api::*;
        }
    "#;
    let api_functions = r#"
        pub fn shadowed(value: Int) -> Int { value }
        pub fn imported(value: Int) -> Int { value }
    "#;
    let inline_fixture = TempTree::new("inline-parity");
    let file_fixture = TempTree::new("file-parity");
    let inline_source = format!("pub mod api {{ {api_functions} }}\n{client_source}");
    let file_source = format!("pub mod api;\n{client_source}");
    let (inline_root, inline_graph) = resolve_fixture(&inline_fixture, &inline_source);
    file_fixture.write("api.ash", api_functions);
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
        resolve_scoped_glob_local_precedence_imports_with_scopes(&inline_graph, &inline_scopes)
            .expect("inline fixture should resolve through the precedence route");
    let file_resolved =
        resolve_scoped_glob_local_precedence_imports_with_scopes(&file_graph, &file_scopes)
            .expect("file-backed fixture should resolve through the precedence route");
    let inline_bound = bind_scoped_glob_local_precedence_imports(&inline_graph, &inline_scopes)
        .expect("inline fixture should bind through the precedence route");
    let file_bound = bind_scoped_glob_local_precedence_imports(&file_graph, &file_scopes)
        .expect("file-backed fixture should bind through the precedence route");

    assert_eq!(inline_resolved.import_edges().len(), 2);
    assert_eq!(file_resolved.import_edges().len(), 2);
    for name in ["shadowed", "imported"] {
        let inline_edge = inline_resolved
            .import_edges()
            .iter()
            .find(|edge| edge.local_name() == name)
            .expect("inline fixture should retain one candidate edge per name");
        let file_edge = file_resolved
            .import_edges()
            .iter()
            .find(|edge| edge.local_name() == name)
            .expect("file-backed fixture should retain one candidate edge per name");

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

        if name == "shadowed" {
            assert!(inline_resolved.binding(&inline_client, name).is_none());
            assert!(file_resolved.binding(&file_client, name).is_none());
            assert!(inline_bound.binding(&inline_client, name).is_none());
            assert!(file_bound.binding(&file_client, name).is_none());
        } else {
            let inline_candidate = inline_resolved
                .binding(&inline_client, name)
                .expect("inline fixture should retain the non-shadowed binding");
            let file_candidate = file_resolved
                .binding(&file_client, name)
                .expect("file-backed fixture should retain the non-shadowed binding");

            assert_eq!(
                inline_candidate.defining_identity(),
                file_candidate.defining_identity()
            );
            assert_eq!(
                inline_candidate.defining_identity(),
                inline_edge.defining_identity()
            );
            assert_eq!(
                file_candidate.defining_identity(),
                file_edge.defining_identity()
            );
            assert_eq!(inline_candidate.visibility(), &Visibility::Public);
            assert_eq!(file_candidate.visibility(), &Visibility::Public);
            assert_eq!(inline_edge.origin(), inline_candidate.origin());
            assert_eq!(file_edge.origin(), file_candidate.origin());
            assert_eq!(
                inline_bound.binding(&inline_client, name),
                Some(inline_candidate)
            );
            assert_eq!(file_bound.binding(&file_client, name), Some(file_candidate));
        }
    }
}

// TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-PROPERTY
proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn local_over_glob_precedence_generated_names_collision_subsets_and_source_forms(
        name_seed in 0_u8..16,
        collision_mask in 0_u8..4,
        structural_depth in 1_usize..4,
        file_backed in any::<bool>(),
    ) {
        let fixture = TempTree::new(&format!(
            "property-names-{name_seed}-mask-{collision_mask}-depth-{structural_depth}-file-{file_backed}"
        ));
        let names = [
            format!("alpha_{name_seed}"),
            format!("beta_{name_seed}"),
        ];
        let api_functions = names
            .iter()
            .map(|name| format!("pub fn {name}(value: Int) -> Int {{ value }}"))
            .collect::<Vec<_>>()
            .join("\n");
        let local_functions = names
            .iter()
            .enumerate()
            .filter(|(index, _)| collision_mask & (1 << index) != 0)
            .map(|(_, name)| format!("fn {name}(value: Int) -> Int {{ value }}"))
            .collect::<Vec<_>>()
            .join("\n");
        let segments: Vec<_> = (0..structural_depth)
            .map(|index| format!("scope_{index}"))
            .collect();
        let structural_path = segments.join("::");
        let target_source = (1..structural_depth)
            .rev()
            .fold(api_functions, |source, index| {
                format!("pub mod {} {{ {source} }}", segments[index])
            });
        let client_source =
            format!("pub mod client {{ {local_functions} use crate::{structural_path}::*; }}");
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
        let resolved = resolve_scoped_glob_local_precedence_imports_with_scopes(&graph, &scopes)
            .expect("generated local collisions should not reject candidate selection");
        let bound = bind_scoped_glob_local_precedence_imports(&graph, &scopes)
            .expect("generated local collisions should not reject binding projection");

        prop_assert_eq!(resolved.import_edges().len(), names.len());
        for (index, name) in names.iter().enumerate() {
            let edge = resolved
                .import_edges()
                .iter()
                .find(|edge| edge.local_name() == name)
                .expect("every generated public candidate should retain a dependency edge");
            prop_assert_eq!(edge.defining_module(), &target_module);
            prop_assert_eq!(edge.defining_identity().module_key(), &target_module);
            prop_assert_eq!(edge.defining_identity().name(), name);
            if collision_mask & (1 << index) != 0 {
                prop_assert!(resolved.binding(&client, name).is_none());
                prop_assert!(bound.binding(&client, name).is_none());
            } else {
                let candidate = resolved
                    .binding(&client, name)
                    .expect("every non-shadowed generated public candidate should bind");
                prop_assert_eq!(candidate.defining_identity(), edge.defining_identity());
                prop_assert_eq!(bound.binding(&client, name), Some(candidate));
            }
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

// TEST-MOD-REAL-004-SCOPED-GLOB-LOCAL-PRECEDENCE-AUTHORITY-FENCE
#[test]
fn local_over_glob_precedence_has_dedicated_authority_only() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dedicated_path = manifest_dir.join("src/canonical_structural_module_binder.rs");
    let dedicated_source = fs::read_to_string(&dedicated_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated local-over-glob binder at {}: {error}",
            dedicated_path.display()
        )
    });
    let planner_path = manifest_dir.join("src/canonical_simple_import_planner.rs");
    let planner_source = fs::read_to_string(&planner_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the local-over-glob resolver in {}: {error}",
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
        "bind_scoped_glob_local_precedence_imports",
        "resolve_scoped_glob_local_precedence_imports_with_scopes",
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
            "resolve_scoped_glob_local_precedence_imports_with_scopes"
        ),
        "only the scoped-glob planner may resolve local-over-glob candidates",
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
        "bind_scoped_glob_local_precedence_imports",
        "resolve_scoped_glob_local_precedence_imports_with_scopes",
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
            "the scoped-glob planner must not gain wider authority: {forbidden_identifier}",
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
            "bind_scoped_glob_local_precedence_imports",
            "resolve_scoped_glob_local_precedence_imports_with_scopes",
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
