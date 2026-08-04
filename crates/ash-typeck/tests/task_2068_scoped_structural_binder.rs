//! TASK-2068 RED contracts for the dedicated scoped structural binder.
//!
//! This target admits only the delivered scope-backed structural-alias route.
//! It proves that binding is a projection of the scoped resolver, without
//! widening the generic binder or authorizing interfaces, lowering, admission,
//! runtime, or client execution.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::surface::{Definition, FnDef, Visibility};
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver, Span};
use ash_typeck::{
    CanonicalProvisionalModuleScopes, CanonicalStructuralImportError,
    bind_scoped_structural_parsed_uses, resolve_simple_parsed_imports_with_scopes,
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
            "ash-task-2068-scoped-structural-binder-{label}-{}-{serial}",
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
    resolve_simple_parsed_imports_with_scopes(graph, scopes)
}

#[test]
fn scoped_structural_binder_preserves_deep_defining_identity_and_parser_facts() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub mod text {
                    pub fn normalize(value: Int) -> Int { value }
                }
            }
            use crate::api::text::normalize as normalize_text;
        "#,
        "deep-positive",
    );
    let text_key = module_key(&root_key, &["api", "text"]);
    let target = function(&graph, &text_key, "normalize");
    let target_origin = graph
        .module_unit(&text_key)
        .expect("target module is graph-owned")
        .artifact()
        .origin()
        .clone();
    let scope_snapshot = scopes(&graph);

    let bound = bind_scoped_structural_parsed_uses(&graph, &scope_snapshot)
        .expect("the dedicated binder projects a permitted deep structural alias");
    let binding = bound
        .binding(&root_key, "normalize_text")
        .expect("the dedicated binder retains the local alias");

    assert_eq!(binding.defining_identity().module_key(), &text_key);
    assert_eq!(binding.defining_identity().name(), "normalize");
    assert_eq!(binding.declaration_span(), target.span);
    assert_eq!(binding.origin(), &target_origin);
    assert_eq!(binding.visibility(), &Visibility::Public);
}

#[test]
fn scoped_structural_binder_is_exactly_the_scoped_plan_binding_projection() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub fn first() -> Int { 1 }
                pub fn second() -> Int { 2 }
            }
            use crate::api::first as alpha;
            use crate::api::second as beta;
        "#,
        "resolver-delegation",
    );
    let scope_snapshot = scopes(&graph);
    let plan = resolve(&graph, &scope_snapshot)
        .expect("the delivered scoped resolver accepts both structural aliases");
    let bound = bind_scoped_structural_parsed_uses(&graph, &scope_snapshot)
        .expect("the dedicated binder must project the successful scoped plan");

    for alias in ["alpha", "beta"] {
        assert_eq!(
            bound.binding(&root_key, alias),
            plan.binding(&root_key, alias),
            "the dedicated binder may expose only the resolver's binding projection for {alias}",
        );
    }
}

#[test]
fn scoped_structural_binder_propagates_anchored_inaccessible_errors_unchanged() {
    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api {
                pub mod hidden {
                    fn normalize(value: Int) -> Int { value }
                }
            }
            pub mod client {
                use crate::api::hidden::normalize as normalize_text;
            }
        "#,
        "inaccessible-propagation",
    );
    let hidden_key = module_key(&root_key, &["api", "hidden"]);
    let client_key = module_key(&root_key, &["client"]);
    let target = function(&graph, &hidden_key, "normalize");
    let use_span = first_use_span(&graph, &client_key);
    let scope_snapshot = scopes(&graph);

    let resolver_error = resolve(&graph, &scope_snapshot)
        .expect_err("the scoped resolver rejects an inaccessible target");
    let binder_error = bind_scoped_structural_parsed_uses(&graph, &scope_snapshot)
        .expect_err("the binder must not replace the scoped resolver diagnostic");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::Inaccessible {
            declaration_span,
            use_span: rejected_use_span,
            defining_module,
            attempted_path,
            violated_visibility,
        } => {
            assert_eq!(declaration_span, target.span);
            assert_eq!(rejected_use_span, use_span);
            assert_eq!(defining_module, hidden_key);
            assert_eq!(
                attempted_path,
                vec![
                    "crate".into(),
                    "api".into(),
                    "hidden".into(),
                    "normalize".into(),
                ]
            );
            assert_eq!(violated_visibility, Visibility::Inherited);
        }
        other => panic!("expected anchored structural visibility rejection, got {other:?}"),
    }
}

#[test]
fn scoped_structural_binder_enforces_permitted_and_rejected_restricted_visibility_regions() {
    let permitted_cases = [
        (
            "crate",
            r#"
                pub mod api { pub(crate) fn target() -> Int { 1 } }
                use crate::api::target as local;
            "#,
            &[][..],
            &["api"][..],
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
                    pub mod provider { pub(in crate::host) fn target() -> Int { 1 } }
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
        let resolver = resolve(&graph, &scope_snapshot)
            .expect("the selected importer lies within the target's canonical visibility region");
        let bound = bind_scoped_structural_parsed_uses(&graph, &scope_snapshot)
            .expect("the dedicated binder must retain every resolver-permitted target");
        let binding = bound
            .binding(&importing_module, "local")
            .expect("the permitted alias is published only after scoped resolution succeeds");

        assert_eq!(
            binding,
            resolver
                .binding(&importing_module, "local")
                .expect("the resolver stages the permitted alias"),
            "the binder is only a plan projection for the {label} region",
        );
        assert_eq!(binding.defining_identity().module_key(), &target_module);
        assert_eq!(binding.visibility(), &expected_visibility);
    }

    let rejected_cases = [
        (
            "super-rejected-at-root",
            r#"
                pub mod host { pub mod provider { pub(super) fn target() -> Int { 1 } } }
                use crate::host::provider::target as local;
            "#,
            &[][..],
            &["host", "provider"][..],
            Visibility::Super { levels: 1 },
        ),
        (
            "restricted-rejected-at-root",
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
            "inherited-rejected-at-sibling",
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
            "self-rejected-at-sibling",
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

    for (label, source, importer_segments, target_segments, expected_visibility) in rejected_cases {
        let (root_key, graph) = parsed_graph(source, label);
        let importing_module = module_key(&root_key, importer_segments);
        let target_module = module_key(&root_key, target_segments);
        let target = function(&graph, &target_module, "target");
        let use_span = first_use_span(&graph, &importing_module);
        let scope_snapshot = scopes(&graph);
        let resolver_error = resolve(&graph, &scope_snapshot)
            .expect_err("a canonical importer outside the target region must reject");
        let binder_error = bind_scoped_structural_parsed_uses(&graph, &scope_snapshot)
            .expect_err("the dedicated binder must preserve the rejected route exactly");

        assert_eq!(binder_error, resolver_error);
        match binder_error {
            CanonicalStructuralImportError::Inaccessible {
                declaration_span,
                use_span: rejected_use_span,
                defining_module,
                violated_visibility,
                ..
            } => {
                assert_eq!(declaration_span, target.span, "{label}");
                assert_eq!(rejected_use_span, use_span, "{label}");
                assert_eq!(defining_module, target_module, "{label}");
                assert_eq!(violated_visibility, expected_visibility, "{label}");
            }
            other => panic!("expected a {label} visibility diagnostic, got {other:?}"),
        }
    }

    let (root_key, graph) = parsed_graph(
        r#"
            pub mod api { pub(crate) fn target() -> Int { 1 } }
            use crate::api::target as local;
        "#,
        "crate-cross-root-predicate",
    );
    let api_key = module_key(&root_key, &["api"]);
    let external_importer = ModuleKey::root("outside")
        .expect("cross-crate fixture key is canonical")
        .child("client")
        .expect("cross-crate fixture key is canonical");
    let scope_snapshot = scopes(&graph);
    assert!(
        !scope_snapshot
            .is_visible_from(&Visibility::Crate, &api_key, &external_importer)
            .expect("the canonical predicate decides the cross-crate pub(crate) boundary"),
        "the exact crate-root-only grammar cannot manufacture a cross-crate route, so the scope predicate records its rejected canonical key boundary",
    );
}

#[test]
fn scoped_structural_binder_rejects_a_late_cycle_before_publishing_any_binding_projection() {
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
    let a_key = module_key(&root_key, &["a"]);
    let b_key = module_key(&root_key, &["b"]);
    let c_key = module_key(&root_key, &["c"]);
    let scope_snapshot = scopes(&graph);
    let resolver_error = resolve(&graph, &scope_snapshot)
        .expect_err("the scoped resolver rejects the later b-to-c-to-b cycle");
    let binder_error = bind_scoped_structural_parsed_uses(&graph, &scope_snapshot)
        .expect_err("the binder must not publish the earlier a-to-b alias after a late cycle");

    assert_eq!(binder_error, resolver_error);
    match binder_error {
        CanonicalStructuralImportError::ImportCycle { edges } => {
            assert_eq!(edges.edges().len(), 2);
            assert!(
                edges
                    .edges()
                    .iter()
                    .all(|edge| edge.importing_module() != &a_key),
                "the earlier valid alias remains unavailable after the later cycle failure",
            );
            assert_eq!(edges.edges()[0].importing_module(), &b_key);
            assert_eq!(edges.edges()[0].defining_module(), &c_key);
            assert_eq!(edges.edges()[1].importing_module(), &c_key);
            assert_eq!(edges.edges()[1].defining_module(), &b_key);
        }
        other => panic!("expected outer structural cycle diagnostic, got {other:?}"),
    }
}

#[test]
fn scoped_structural_binder_has_equal_file_and_inline_binding_projections() {
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
    let inline_bound = bind_scoped_structural_parsed_uses(&inline_graph, &inline_scopes)
        .expect("the inline graph is scope-bindable");
    let file_bound = bind_scoped_structural_parsed_uses(&file_graph, &file_scopes)
        .expect("the file-backed graph is scope-bindable");
    let inline_binding = inline_bound
        .binding(&inline_root, "normalize_text")
        .expect("the inline graph retains its local alias");
    let file_binding = file_bound
        .binding(&file_root, "normalize_text")
        .expect("the file-backed graph retains its local alias");
    let inline_target = module_key(&inline_root, &["api", "text"]);
    let file_target = module_key(&file_root, &["api", "text"]);

    assert_eq!(
        inline_binding.defining_identity().module_key(),
        &inline_target
    );
    assert_eq!(file_binding.defining_identity().module_key(), &file_target);
    assert_eq!(inline_binding.defining_identity().name(), "normalize");
    assert_eq!(file_binding.defining_identity().name(), "normalize");
    assert_eq!(inline_binding.visibility(), file_binding.visibility());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn scoped_structural_binder_matches_the_scoped_resolver_for_generated_permitted_regions(
        module_suffix in "[a-z][a-z0-9_]{0,8}",
        alias_suffix in "[a-z][a-z0-9_]{0,8}",
        visibility_case in 0_u8..6,
    ) {
        let host = format!("host_{module_suffix}");
        let provider = format!("provider_{module_suffix}");
        let client = format!("client_{module_suffix}");
        let alias = format!("alias_{alias_suffix}");
        let (source, importer_segments, expected_visibility) = match visibility_case {
            0 => (
                format!(
                    "pub mod {host} {{ pub mod {provider} {{ pub fn target() -> Int {{ 1 }} }} pub mod {client} {{ use crate::{host}::{provider}::target as {alias}; }} }}"
                ),
                vec![host.as_str(), client.as_str()],
                Visibility::Public,
            ),
            1 => (
                format!(
                    "pub mod {host} {{ pub mod {provider} {{ pub(crate) fn target() -> Int {{ 1 }} }} pub mod {client} {{ use crate::{host}::{provider}::target as {alias}; }} }}"
                ),
                vec![host.as_str(), client.as_str()],
                Visibility::Crate,
            ),
            2 => (
                format!(
                    "pub mod {host} {{ pub mod {provider} {{ pub(super) fn target() -> Int {{ 1 }} }} pub mod {client} {{ use crate::{host}::{provider}::target as {alias}; }} }}"
                ),
                vec![host.as_str(), client.as_str()],
                Visibility::Super { levels: 1 },
            ),
            3 => (
                format!(
                    "pub mod {host} {{ pub mod {provider} {{ pub(in crate::{host}) fn target() -> Int {{ 1 }} }} pub mod {client} {{ use crate::{host}::{provider}::target as {alias}; }} }}"
                ),
                vec![host.as_str(), client.as_str()],
                Visibility::Restricted {
                    path: format!("crate::{host}").into(),
                },
            ),
            4 => (
                format!(
                    "pub mod {host} {{ pub mod {provider} {{ fn target() -> Int {{ 1 }} use crate::{host}::{provider}::target as {alias}; }} }}"
                ),
                vec![host.as_str(), provider.as_str()],
                Visibility::Inherited,
            ),
            _ => (
                format!(
                    "pub mod {host} {{ pub mod {provider} {{ pub(self) fn target() -> Int {{ 1 }} use crate::{host}::{provider}::target as {alias}; }} }}"
                ),
                vec![host.as_str(), provider.as_str()],
                Visibility::Self_,
            ),
        };
        let (root_key, graph) = parsed_graph(&source, "generated-permitted-region");
        let importing_module = module_key(&root_key, &importer_segments);
        let provider_key = module_key(&root_key, &[host.as_str(), provider.as_str()]);
        let scope_snapshot = scopes(&graph);
        let plan = resolve(&graph, &scope_snapshot)
            .expect("each generated fixture uses a canonical importer inside its selected region");
        let bound = bind_scoped_structural_parsed_uses(&graph, &scope_snapshot)
            .expect("the dedicated binder must accept every resolver-permitted generated alias");
        let plan_binding = plan.binding(&importing_module, &alias)
            .expect("the resolver retains the generated local alias");
        let bound_binding = bound.binding(&importing_module, &alias)
            .expect("the binder retains the generated local alias");

        prop_assert_eq!(bound_binding, plan_binding);
        prop_assert_eq!(bound_binding.defining_identity().module_key(), &provider_key);
        prop_assert_eq!(bound_binding.defining_identity().name(), "target");
        prop_assert_eq!(bound_binding.visibility(), &expected_visibility);
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
fn scoped_structural_binder_is_the_only_scoped_binding_authority_and_leaves_the_generic_binder_unchanged()
 {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dedicated_path = manifest_dir.join("src/canonical_structural_module_binder.rs");
    let dedicated_source = fs::read_to_string(&dedicated_path).unwrap_or_else(|error| {
        panic!(
            "TASK-2068 requires the dedicated scoped structural binder at {}: {error}",
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
        dedicated_source.contains("resolve_simple_parsed_imports_with_scopes"),
        "the dedicated binder must consume the scoped resolver directly",
    );
    assert!(
        dedicated_source.contains(".map(|plan| plan.into_bound_set())"),
        "the dedicated binder must only project the successful scoped resolver plan",
    );
    assert!(
        dedicated_source.contains("CanonicalStructuralImportError"),
        "the dedicated binder must propagate the scoped error contract unchanged",
    );
    assert!(
        lib_source.contains("mod canonical_structural_module_binder;"),
        "the dedicated binder module remains private behind its named public API",
    );
    assert!(
        lib_source.contains(
            "pub use canonical_structural_module_binder::bind_scoped_structural_parsed_uses;"
        ),
        "only the named dedicated scope-backed binding API is re-exported",
    );
    for forbidden_identifier in [
        "ModuleIdentity",
        "ModuleId",
        "ModuleGraph",
        "LegacyModuleResolver",
        "ModuleResolver",
        "NameBinder",
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
            !contains_exact_identifier(&dedicated_source, forbidden_identifier),
            "the binding-only projection must not gain wider authority: {forbidden_identifier}",
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
            "the binding-only projection must not bypass the scoped resolver: {forbidden_bypass}",
        );
    }
    assert!(
        generic_source.contains("resolve_simple_parsed_imports"),
        "the generic binder retains its original planner delegation",
    );
    for scoped_term in [
        "CanonicalProvisionalModuleScopes",
        "resolve_simple_parsed_imports_with_scopes",
        "CanonicalStructuralImportError",
    ] {
        assert!(
            !generic_source.contains(scoped_term),
            "the generic binder must remain outside the dedicated scoped route: {scoped_term}",
        );
    }
}
