//! TASK-2072 parsed-import resolver evidence.
//!
//! The tests exercise the name-view-only Type-layer resolver and its atomic
//! staging boundary.  They intentionally stop before checked finalization,
//! Core/CPS, admission, runtime, or client parity.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::{CanonicalExpandedModuleGraph, CanonicalModuleGraphResolver};
use ash_typeck::canonical_module_collection::{
    CanonicalDeclarationKind, CanonicalNamespace, collect_canonical_expanded_module_graph,
};
use ash_typeck::{CanonicalParsedImportError, resolve_parsed_imports_from_collection};
use proptest::prelude::*;

static NEXT_TEMP_TREE: AtomicUsize = AtomicUsize::new(0);

struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP_TREE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ash-task-2072-parsed-imports-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create TASK-2072 parser fixture tree");
        Self { root }
    }

    fn write(&self, relative: impl AsRef<Path>, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture path has a parent"))
            .expect("create TASK-2072 fixture parent directory");
        fs::write(&path, source).expect("write TASK-2072 parser fixture");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn expanded_graph(source: &str, label: &str) -> (ModuleKey, CanonicalExpandedModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("fixture source resolves through the canonical parser graph");
    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("fixture source expands through the canonical expanded graph");
    (root_key, expanded)
}

fn provider_graph(
    provider_source: &str,
    inline: bool,
    label: &str,
) -> (ModuleKey, CanonicalExpandedModuleGraph) {
    let tree = TempTree::new(label);
    let root_source = if inline {
        format!("mod provider {{ {provider_source} }} use crate::provider::value as imported;")
    } else {
        tree.write("src/provider.ash", provider_source);
        "mod provider; use crate::provider::value as imported;".to_owned()
    };
    let root_path = tree.write("src/main.ash", &root_source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let parsed = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("provider fixture resolves through the canonical parser graph");
    let expanded = CanonicalExpandedModuleGraph::try_expand(parsed)
        .expect("provider fixture expands through the canonical expanded graph");
    (root_key, expanded)
}

#[test]
fn task_2072_activation_contract_keeps_evidence_axes_explicit() {
    let task_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/plan/tasks/TASK-2072-parsed-import-resolution-and-atomic-binding.md");
    let task = fs::read_to_string(&task_path).expect("TASK-2072 task file exists");

    assert!(task.contains("**Status:** Complete"));
    assert!(task.contains("**Owned rule:** MOD-REAL-004"));
    assert!(task.contains("CanonicalProvisionalNameView"));
    assert!(task.contains("**Implementation:** partial"));
    assert!(task.contains("**Evidence:** tested"));
    assert!(task.contains("**Parity:** below_spec"));
    assert!(task.contains("## TDD Steps"));
    assert!(task.contains("staged `pub use`"));
    assert!(task.contains("atomic"));
}

/// RED inventory: every admitted parsed use shape must preserve identity,
/// namespace, visibility, alias, source ordering, declaration span, use span,
/// and origin while resolving through the provisional name view.
#[test]
fn red_all_parsed_grammar_forms_preserve_binding_provenance() {
    let (root, expanded) = expanded_graph(
        r#"
            pub fn root_fn() -> Int { 1 }
            pub mod api {
                pub fn alpha() -> Int { 2 }
                pub fn beta() -> Int { 3 }
                pub type Token = Int;
                pub mod nested {
                    pub fn deep() -> Int { 4 }
                }
            }
            pub mod child {
                pub fn child_fn() -> Int { 5 }
                use super::root_fn as inherited_root;
                use crate::api::nested::deep as deep_alias;
            }
            use crate::api::alpha;
            use crate::api::beta as renamed_beta;
            use crate::api::{alpha as grouped_alpha, beta};
            use crate::api::*;
            use self::root_fn as self_alias;
        "#,
        "all-grammar-provenance",
    );

    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("the expanded graph supplies paired provisional views");
    let resolved = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("all admitted parsed import forms resolve atomically");
    assert!(
        resolved
            .bindings()
            .any(|(module, name, _)| module == &root && name == "renamed_beta")
    );
    assert!(resolved.binding(&root, "renamed_beta").is_some());
    assert!(resolved.binding(&root, "grouped_alpha").is_some());
    assert!(resolved.binding(&root, "self_alias").is_some());
    let api = root.child("api").expect("api key is canonical");
    let beta = collection
        .provisional_name_view(&api)
        .expect("api view is collected")
        .entries()
        .find(|entry| entry.lookup_name() == "beta")
        .expect("beta entry is retained");
    assert_eq!(
        resolved
            .binding(&root, "renamed_beta")
            .expect("renamed beta binding")
            .source_ordinal(),
        beta.source_ordinal()
    );
    assert!(resolved.import_edges().len() >= 8);
    let root_unit = expanded
        .parsed_graph()
        .module_unit(&root)
        .expect("root parser unit is retained");
    assert!(root_unit.body().uses().len() >= 5);
    assert!(
        root_unit
            .body()
            .module_decls()
            .iter()
            .any(|module| { module.name.as_ref() == "api" || module.name.as_ref() == "child" })
    );
}

#[test]
fn parsed_module_alias_preserves_child_module_identity_and_dependency_edge() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod api { pub fn value() -> Int { 1 } }
            use crate::api as api_alias;
        "#,
        "module-alias-identity",
    );
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("the module alias fixture collects atomically");
    let resolved = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("the module alias resolves atomically");
    let child = root.child("api").expect("fixture child key is canonical");
    let binding = resolved
        .binding(&root, "api_alias")
        .expect("module alias enters the importing scope");
    assert_eq!(binding.defining_identity().module_key(), &child);
    assert_eq!(
        binding.lookup_key().namespace(),
        CanonicalNamespace::StructuralModule
    );
    let edge = resolved
        .import_edges()
        .iter()
        .find(|edge| edge.binding().local_name() == "api_alias")
        .expect("module alias retains a dependency edge");
    assert_eq!(edge.defining_module(), &child);
}

#[test]
fn parent_scoped_members_require_qualified_parent_paths() {
    let (root, qualified) = expanded_graph(
        r#"
            pub type Choice = Pick | Skip;
            use crate::Choice::Pick as selected;
            use crate::Choice::{Skip as grouped};
        "#,
        "parent-scoped-qualified",
    );
    let collection = collect_canonical_expanded_module_graph(&qualified)
        .expect("the parent-scoped fixture collects atomically");
    let resolved = resolve_parsed_imports_from_collection(qualified.parsed_graph(), &collection)
        .expect("a qualified parent member resolves atomically");
    let binding = resolved
        .binding(&root, "selected")
        .expect("qualified member enters the importing scope");
    assert!(binding.defining_identity().canonical_parent().is_some());
    assert!(resolved.binding(&root, "grouped").is_some());

    let (_root, unqualified) = expanded_graph(
        r#"
            pub type Choice = Pick | Skip;
            use crate::Pick as selected;
        "#,
        "parent-scoped-unqualified",
    );
    let collection = collect_canonical_expanded_module_graph(&unqualified)
        .expect("the unqualified fixture collects before import resolution");
    assert!(matches!(
        resolve_parsed_imports_from_collection(unqualified.parsed_graph(), &collection),
        Err(CanonicalParsedImportError::Unresolved { .. })
    ));
}

#[test]
fn notation_imports_preserve_all_public_fixity_keys_without_bindings() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub prefix 9 <*> = leading
                pub infixl 6 <*> = combine
            }
            use crate::provider::(<*>);
        "#,
        "notation-imports",
    );
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("the notation providers collect atomically");
    let resolved = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("public notation imports resolve atomically");
    assert!(resolved.binding(&root, "<*>").is_none());
    assert_eq!(resolved.notation_imports().len(), 2);
    assert!(
        resolved
            .notation_imports()
            .iter()
            .all(|import| import.lookup_key().notation_key().is_some())
    );
    assert!(resolved.notation_imports().iter().all(|import| matches!(
        import.declaration_visibility(),
        ash_parser::Visibility::Public
    )));
}

#[test]
fn parsed_use_visibility_carriers_are_checked_without_final_export_authority() {
    let (_root, invalid) = expanded_graph(
        r#"
            pub mod api { pub fn value() -> Int { 1 } }
            pub(super) use crate::api::value as invalid;
        "#,
        "invalid-root-use-visibility",
    );
    let collection = collect_canonical_expanded_module_graph(&invalid)
        .expect("invalid use visibility still collects before binding");
    assert!(matches!(
        resolve_parsed_imports_from_collection(invalid.parsed_graph(), &collection),
        Err(CanonicalParsedImportError::Unsupported { .. })
    ));

    let (root, private_reexport) = expanded_graph(
        r#"
            fn private() -> Int { 1 }
            pub(self) use self::private as local;
        "#,
        "narrow-private-reexport",
    );
    let collection = collect_canonical_expanded_module_graph(&private_reexport)
        .expect("a narrow re-export collects before binding");
    let resolved =
        resolve_parsed_imports_from_collection(private_reexport.parsed_graph(), &collection)
            .expect("a narrow re-export remains a non-authorizing staged binding");
    assert!(resolved.binding(&root, "local").is_some());
    assert_eq!(resolved.public_uses().len(), 1);
}

#[test]
fn bare_module_paths_climb_lexical_parents_and_super_prefixes_repeat() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod sibling { pub fn value() -> Int { 1 } }
            pub mod parent {
                pub fn root_value() -> Int { 2 }
                pub mod child {
                    pub mod nested {
                        use sibling::value as lifted;
                        use super::super::root_value as repeated;
                    }
                }
            }
        "#,
        "lexical-parent-and-super",
    );
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("the lexical-path fixture collects atomically");
    let resolved = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("bare and repeated-super paths resolve atomically");
    let nested = root
        .child("parent")
        .and_then(|key| key.child("child"))
        .and_then(|key| key.child("nested"))
        .expect("nested fixture key is canonical");
    assert!(resolved.binding(&nested, "lifted").is_some());
    assert!(resolved.binding(&nested, "repeated").is_some());
}

#[test]
fn module_and_parent_namespace_interpretations_are_ambiguous() {
    let (_root, expanded) = expanded_graph(
        r#"
            pub mod X { pub fn Y() -> Int { 1 } }
            pub type X = Y | Z;
            use crate::X::Y as selected;
        "#,
        "module-parent-ambiguity",
    );
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("cross-namespace spellings collect atomically");
    assert!(matches!(
        resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection),
        Err(CanonicalParsedImportError::Ambiguous { .. })
    ));
}

#[test]
fn nearest_inaccessible_lexical_module_shadows_outer_fallbacks() {
    let (_root, expanded) = expanded_graph(
        r#"
            pub mod outer {
                mod hidden { pub fn value() -> Int { 1 } }
                pub mod child { use hidden::value as selected; }
            }
            pub mod hidden { pub fn value() -> Int { 2 } }
        "#,
        "nearest-private-lexical-module",
    );
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("the nearest lexical fixture collects before binding");
    assert!(matches!(
        resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection),
        Err(CanonicalParsedImportError::Inaccessible { .. })
    ));
}

#[test]
fn mutated_graph_key_set_rejects_without_publishing_bindings() {
    let (_root, original) = expanded_graph(
        "pub mod api { pub fn value() -> Int { 1 } } use crate::api::value;",
        "mutated-graph-original",
    );
    let (_root, mutated) = expanded_graph(
        "pub fn value() -> Int { 1 } use crate::value;",
        "mutated-graph-key-set",
    );
    let collection = collect_canonical_expanded_module_graph(&original)
        .expect("the original graph collects atomically");
    assert!(matches!(
        resolve_parsed_imports_from_collection(mutated.parsed_graph(), &collection),
        Err(CanonicalParsedImportError::GraphMismatch)
    ));
}

#[test]
fn empty_nested_groups_reject_without_publishing_bindings() {
    let (_root, expanded) = expanded_graph(
        "pub mod api { pub fn value() -> Int { 1 } } use crate::api::{};",
        "empty-nested-group",
    );
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("the empty-group source collects atomically");
    let result = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection);
    assert!(matches!(
        result,
        Err(CanonicalParsedImportError::Unsupported { reason, .. })
            if reason.contains("at least one member")
    ));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn generated_visibility_property_preserves_import_outcomes(
        visibility in prop::sample::select(vec![
            "pub",
            "pub(crate)",
            "pub(super)",
            "pub(self)",
        ]),
    ) {
        let source = format!(
            "pub mod api {{ {visibility} fn value() -> Int {{ 1 }} }} use crate::api::value;"
        );
        let (root, expanded) = expanded_graph(&source, "generated-property-visibility");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("generated visibility source collects atomically");
        let result = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection);
        if visibility == "pub(self)" {
            let inaccessible = matches!(
                result,
                Err(CanonicalParsedImportError::Inaccessible { .. })
            );
            prop_assert!(inaccessible);
        } else {
            let resolved = result.expect("the generated visibility form is reachable");
            prop_assert!(resolved.binding(&root, "value").is_some());
        }
    }
}

#[test]
fn transitive_public_reexports_preserve_the_original_defining_identity() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider { pub fn value() -> Int { 1 } }
            pub mod facade { pub use crate::provider::value as exposed; }
            use crate::facade::exposed as imported;
        "#,
        "transitive-public-reexport",
    );
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("the transitive re-export fixture collects atomically");
    let resolved = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("transitive public re-export resolves atomically");
    let provider = root.child("provider").expect("provider key is canonical");
    let binding = resolved
        .binding(&root, "imported")
        .expect("the transitive binding enters the root scope");
    assert_eq!(binding.defining_identity().module_key(), &provider);
    assert_eq!(
        binding.defining_identity().kind(),
        CanonicalDeclarationKind::Function
    );
}

#[test]
fn cyclic_public_reexports_fail_before_any_binding_is_published() {
    let (_root, expanded) = expanded_graph(
        r#"
            pub mod a { pub use crate::b::from_a as from_b; }
            pub mod b { pub use crate::a::from_b as from_a; }
            use crate::a::from_b as imported;
        "#,
        "cyclic-public-reexports",
    );
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("the re-export cycle collects before binding");
    assert!(matches!(
        resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection),
        Err(CanonicalParsedImportError::ImportCycle { .. })
    ));
}

/// RED inventory: source `pub use` must stage a non-authorizing re-export
/// fact, retaining the original target identity and visibility for TASK-2073.
#[test]
fn red_pub_use_stages_identity_without_publishing_final_export_closure() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod provider {
                pub fn exported() -> Int { 7 }
                pub fn second() -> Int { 8 }
            }
            pub use crate::provider::exported as zulu;
            pub use crate::provider::second as alpha;
        "#,
        "staged-pub-use",
    );
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("the staged public-use fixture collects atomically");
    let resolved = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("the staged public-use resolves atomically");
    let binding = resolved
        .binding(&root, "zulu")
        .expect("public use introduces its local binding");
    assert!(binding.is_reexport());
    assert!(matches!(
        binding.import_visibility(),
        ash_parser::Visibility::Public
    ));
    assert_eq!(resolved.public_uses().len(), 2);
    assert_eq!(resolved.public_uses()[0].binding().local_name(), "zulu");
    assert_eq!(resolved.public_uses()[1].binding().local_name(), "alpha");
    assert_eq!(
        resolved.public_uses()[0].binding().defining_identity(),
        binding.defining_identity()
    );
    assert_eq!(resolved.public_uses()[0].importing_module(), &root);
}

/// RED inventory: same-module declarations shadow explicit imports, which
/// shadow globs; equal-ranked candidates must report ambiguity deterministically.
#[test]
fn red_local_explicit_glob_precedence_and_ambiguity_are_deterministic() {
    let (root, expanded) = expanded_graph(
        r#"
            pub mod first { pub fn picked() -> Int { 1 } }
            pub mod second { pub fn picked() -> Int { 2 } }
            pub fn picked() -> Int { 3 }
            use crate::first::picked;
            use crate::second::*;
        "#,
        "precedence-and-ambiguity",
    );
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("the precedence fixture collects atomically");
    let resolved = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect("a local declaration suppresses imported candidates");
    assert!(resolved.binding(&root, "picked").is_none());
    assert_eq!(resolved.import_edges().len(), 2);

    let (_root, ambiguous) = expanded_graph(
        r#"
            pub mod first { pub fn picked() -> Int { 1 } }
            pub mod second { pub fn picked() -> Int { 2 } }
            use crate::first::picked;
            use crate::second::picked;
        "#,
        "explicit-ambiguity",
    );
    let collection = collect_canonical_expanded_module_graph(&ambiguous)
        .expect("the ambiguity fixture collects before import resolution");
    let error = resolve_parsed_imports_from_collection(ambiguous.parsed_graph(), &collection)
        .expect_err("equal-ranked explicit imports must reject");
    assert!(matches!(
        error,
        CanonicalParsedImportError::Ambiguous { .. }
    ));
}

/// RED inventory: duplicate local bindings and duplicate aliases must reject
/// the whole graph, including otherwise valid sibling imports.
#[test]
fn red_duplicate_bindings_reject_all_siblings_atomically() {
    let (_root, expanded) = expanded_graph(
        r#"
            pub mod api { pub fn value() -> Int { 1 } }
            pub fn sibling() -> Int { 2 }
            use crate::api::value as same;
            use crate::api::value as same;
            use crate::sibling as surviving_sibling;
        "#,
        "duplicate-atomicity",
    );
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("the duplicate fixture collects before import resolution");
    let error = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect_err("duplicate binding must reject the complete result");
    assert!(matches!(
        error,
        CanonicalParsedImportError::DuplicateBinding { .. }
    ));
}

/// RED inventory: cycle discovery must run after complete dependency preflight
/// and publish neither bindings nor edge sets after any cycle.
#[test]
fn red_complete_cross_module_cycles_fail_before_publication() {
    let (_root, expanded) = expanded_graph(
        r#"
            pub mod a {
                pub use crate::b::value as from_b;
                pub fn value() -> Int { 1 }
            }
            pub mod b {
                pub use crate::a::value as from_a;
                pub fn value() -> Int { 2 }
            }
            use crate::a::value as root_a;
        "#,
        "complete-cycle-atomicity",
    );
    let collection = collect_canonical_expanded_module_graph(&expanded)
        .expect("the cycle fixture collects before import resolution");
    let error = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection)
        .expect_err("a complete dependency cycle must reject atomically");
    match error {
        CanonicalParsedImportError::ImportCycle { cycle } => {
            assert!(cycle.edges().len() >= 2 || cycle.modules().len() >= 3);
        }
        other => panic!("expected import-cycle rejection, got {other:?}"),
    }
}

/// RED inventory: file-backed and inline modules must yield the same normalized
/// binding projection (excluding layout-specific acquisition provenance).
#[test]
fn red_file_and_inline_bindings_have_equal_normalized_projection() {
    let (file_root, file_expanded) = provider_graph(
        "pub fn value() -> Int { 1 }",
        false,
        "file-inline-normalized",
    );
    let (inline_root, inline_expanded) =
        provider_graph("pub fn value() -> Int { 1 }", true, "inline-normalized");
    let file_collection = collect_canonical_expanded_module_graph(&file_expanded)
        .expect("the file-backed provider collects atomically");
    let inline_collection = collect_canonical_expanded_module_graph(&inline_expanded)
        .expect("the inline provider collects atomically");
    let file_resolved =
        resolve_parsed_imports_from_collection(file_expanded.parsed_graph(), &file_collection)
            .expect("the file-backed provider imports atomically");
    let inline_resolved =
        resolve_parsed_imports_from_collection(inline_expanded.parsed_graph(), &inline_collection)
            .expect("the inline provider imports atomically");
    assert_eq!(file_root, inline_root);
    assert_eq!(
        file_resolved.normalized_projection(),
        inline_resolved.normalized_projection()
    );
}

/// RED inventory: generated grammar and visibility cases must retain their
/// source-owned use/declaration anchors and reject inaccessible targets.
#[test]
fn red_generated_grammar_visibility_property_preserves_anchors() {
    for visibility in ["pub", "pub(crate)", "pub(super)", "pub(self)"] {
        let source = format!(
            "pub mod api {{ {visibility} fn value() -> Int {{ 1 }} }} use crate::api::value;"
        );
        let (root, expanded) = expanded_graph(&source, "generated-visibility");
        let collection = collect_canonical_expanded_module_graph(&expanded)
            .expect("generated visibility source collects atomically");
        let result = resolve_parsed_imports_from_collection(expanded.parsed_graph(), &collection);
        if visibility == "pub(self)" {
            assert!(matches!(
                result,
                Err(CanonicalParsedImportError::Inaccessible { .. })
            ));
            continue;
        }
        let resolved = result.expect("the visibility form is reachable from the crate root");
        let binding = resolved
            .binding(&root, "value")
            .expect("the visible declaration enters the importing scope");
        let child = root.child("api").expect("fixture child key is canonical");
        let entry = collection
            .provisional_name_view(&child)
            .expect("child view is collected")
            .entries()
            .find(|entry| entry.lookup_name() == "value")
            .expect("value entry is retained");
        assert_eq!(binding.declaration_span(), entry.origin_anchor());
        assert!(binding.use_span().start < binding.use_span().end);
        assert!(binding.declaration_span().start < binding.declaration_span().end);
    }
}

/// RED inventory: resolver implementation must consume only name-view entries,
/// never checker-internal snapshots, raw definitions, or source rediscovery.
#[test]
fn red_import_resolver_has_name_view_only_authority() {
    let resolver =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/canonical_parsed_import_resolver.rs");
    let source = fs::read_to_string(&resolver)
        .expect("TASK-2072 implementation should create the dedicated resolver module");
    assert!(source.contains("CanonicalProvisionalNameView"));
    for forbidden in [
        "internal_snapshot",
        "CanonicalCollectedModuleSnapshot",
        "raw_definition",
        "read_to_string",
        "CanonicalModuleGraphResolver",
    ] {
        assert!(
            !source.contains(forbidden),
            "name-view-only resolver must not gain {forbidden} authority"
        );
    }
}
