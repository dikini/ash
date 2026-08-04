//! TASK-2072 activation contract and RED test inventory.
//!
//! The complete parsed-import resolver is intentionally not implemented yet.
//! This file therefore has one executable activation contract and a set of
//! ignored RED placeholders.  The placeholders use only parser-owned fixtures
//! today; they must be unignored and changed to call the eventual
//! name-view-only resolver once its public API is available.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ash_core::module_graph::ModuleKey;
use ash_parser::{CanonicalModuleGraph, CanonicalModuleGraphResolver};

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

fn parsed_graph(source: &str, label: &str) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write("src/main.ash", source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("fixture source resolves through the canonical parser graph");
    (root_key, graph)
}

fn file_inline_graph(
    file_source: &str,
    inline_source: &str,
    label: &str,
) -> (ModuleKey, CanonicalModuleGraph) {
    let tree = TempTree::new(label);
    let root_path = tree.write(
        "src/main.ash",
        format!("mod file; mod inline {{ {inline_source} }}").as_str(),
    );
    tree.write("src/file.ash", file_source);
    let root_key = ModuleKey::root("app").expect("fixture crate key is canonical");
    let graph = CanonicalModuleGraphResolver::new()
        .resolve_root(root_key.clone(), root_path)
        .expect("file/inline fixture resolves through the canonical parser graph");
    (root_key, graph)
}

#[test]
fn task_2072_activation_contract_is_recorded_before_semantic_implementation() {
    let task_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/plan/tasks/TASK-2072-parsed-import-resolution-and-atomic-binding.md");
    let task = fs::read_to_string(&task_path).expect("TASK-2072 task file exists");

    assert!(task.contains("**Status:** In progress"));
    assert!(task.contains("**Owned rule:** MOD-REAL-004"));
    assert!(task.contains("CanonicalProvisionalNameView"));
    assert!(task.contains("**Implementation:** partial"));
    assert!(task.contains("**Evidence:** none"));
    assert!(task.contains("**Parity:** below_spec"));
    assert!(task.contains("## TDD Steps"));
    assert!(task.contains("staged `pub use`"));
    assert!(task.contains("atomic"));
}

/// RED inventory: every admitted parsed use shape must preserve identity,
/// namespace, visibility, alias, source ordering, declaration span, use span,
/// and origin while resolving through the provisional name view.
#[test]
#[ignore = "RED: unignore after TASK-2072 name-view-only resolver API is implemented"]
fn red_all_parsed_grammar_forms_preserve_binding_provenance() {
    let (root, graph) = parsed_graph(
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

    let root_unit = graph
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
    panic!("RED TASK-2072: resolve all parsed use forms against CanonicalProvisionalNameView");
}

/// RED inventory: source `pub use` must stage a non-authorizing re-export
/// fact, retaining the original target identity and visibility for TASK-2073.
#[test]
#[ignore = "RED: staged-public-use result type is not implemented yet"]
fn red_pub_use_stages_identity_without_publishing_final_export_closure() {
    let (_root, graph) = parsed_graph(
        r#"
            pub mod provider {
                pub fn exported() -> Int { 7 }
            }
            pub use crate::provider::exported as public_name;
        "#,
        "staged-pub-use",
    );
    assert_eq!(
        graph
            .module_unit(graph.root_key())
            .expect("root parser unit is retained")
            .body()
            .uses()
            .len(),
        1
    );
    panic!("RED TASK-2072: stage pub-use facts without finalizing export closure");
}

/// RED inventory: same-module declarations shadow explicit imports, which
/// shadow globs; equal-ranked candidates must report ambiguity deterministically.
#[test]
#[ignore = "RED: complete precedence/ambiguity API is not implemented yet"]
fn red_local_explicit_glob_precedence_and_ambiguity_are_deterministic() {
    let (_root, graph) = parsed_graph(
        r#"
            pub mod first { pub fn picked() -> Int { 1 } }
            pub mod second { pub fn picked() -> Int { 2 } }
            pub fn picked() -> Int { 3 }
            use crate::first::picked;
            use crate::second::*;
        "#,
        "precedence-and-ambiguity",
    );
    assert_eq!(
        graph
            .module_unit(graph.root_key())
            .expect("root parser unit is retained")
            .body()
            .uses()
            .len(),
        2
    );
    panic!(
        "RED TASK-2072: local > explicit > glob precedence and equal-rank ambiguity diagnostics"
    );
}

/// RED inventory: duplicate local bindings and duplicate aliases must reject
/// the whole graph, including otherwise valid sibling imports.
#[test]
#[ignore = "RED: atomic duplicate-binding diagnostics are not implemented yet"]
fn red_duplicate_bindings_reject_all_siblings_atomically() {
    let (_root, graph) = parsed_graph(
        r#"
            pub mod api { pub fn value() -> Int { 1 } }
            pub fn sibling() -> Int { 2 }
            use crate::api::value as same;
            use crate::api::value as same;
            use crate::sibling as surviving_sibling;
        "#,
        "duplicate-atomicity",
    );
    assert!(
        graph
            .module_unit(graph.root_key())
            .expect("root parser unit is retained")
            .body()
            .uses()
            .len()
            >= 3
    );
    panic!("RED TASK-2072: duplicate binding must publish no partial result");
}

/// RED inventory: cycle discovery must run after complete dependency preflight
/// and publish neither bindings nor edge sets after any cycle.
#[test]
#[ignore = "RED: complete import-cycle result is not implemented yet"]
fn red_complete_cross_module_cycles_fail_before_publication() {
    let (_root, graph) = parsed_graph(
        r#"
            pub mod a {
                pub use crate::b::value as from_b;
                pub fn value() -> Int { 1 }
            }
            pub mod b {
                pub use crate::a::value as from_a;
                pub fn value() -> Int { 2 }
            }
            use crate::a::from_b as root_a;
        "#,
        "complete-cycle-atomicity",
    );
    assert_eq!(graph.module_units().count(), 3);
    panic!("RED TASK-2072: complete dependency-cycle failure must be atomic");
}

/// RED inventory: file-backed and inline modules must yield the same normalized
/// binding projection (excluding layout-specific acquisition provenance).
#[test]
#[ignore = "RED: normalized binding projection API is not implemented yet"]
fn red_file_and_inline_bindings_have_equal_normalized_projection() {
    let (file_root, file_graph) = file_inline_graph(
        "pub fn value() -> Int { 1 }",
        "pub fn value() -> Int { 1 }",
        "file-inline-normalized",
    );
    assert!(file_graph.module_unit(&file_root).is_some());
    panic!("RED TASK-2072: compare normalized file/inline binding projections");
}

/// RED inventory: generated grammar and visibility cases must retain their
/// source-owned use/declaration anchors and reject inaccessible targets.
#[test]
#[ignore = "RED: generated grammar/visibility property is not implemented yet"]
fn red_generated_grammar_visibility_property_preserves_anchors() {
    for visibility in ["pub", "pub(crate)", "pub(super)", "pub(self)"] {
        let source = format!(
            "pub mod api {{ {visibility} fn value() -> Int {{ 1 }} }} use crate::api::value;"
        );
        let (_root, graph) = parsed_graph(&source, "generated-visibility");
        assert_eq!(graph.module_units().count(), 2);
    }
    panic!("RED TASK-2072: generated visibility property must classify every outcome");
}

/// RED inventory: resolver implementation must consume only name-view entries,
/// never checker-internal snapshots, raw definitions, or source rediscovery.
#[test]
#[ignore = "RED: authority-fence assertions become active with the resolver implementation"]
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
