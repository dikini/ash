//! Multi-crate visibility regression tests (TASK-341)
//!
//! These tests verify that visibility rules are correctly enforced across crate boundaries
//! using the import_resolver with properly constructed multi-crate graphs.

use ash_core::module_graph::{ModuleGraph, ModuleNode, ModuleSource};
use ash_parser::import_resolver::{ImportError, ImportResolver};
use ash_parser::surface::Visibility;
use ash_parser::token::Span;
use ash_parser::use_tree::{SimplePath, Use, UseItem, UsePath};

// Helper to create a SimplePath from segments
fn simple_path(segments: &[&str]) -> SimplePath {
    SimplePath {
        segments: segments.iter().map(|s| (*s).into()).collect(),
    }
}

// Helper to create a Use statement
fn use_stmt(path: UsePath) -> Use {
    Use {
        visibility: Visibility::Inherited,
        path,
        alias: None,
        span: Span::new(0, 0, 1, 1),
    }
}

// Helper to create UseItem
fn use_item(name: &str) -> UseItem {
    UseItem {
        name: name.into(),
        alias: None,
    }
}

/// Create a multi-crate graph for testing cross-crate visibility.
///
/// Structure:
/// - main_crate (CrateId 0): main_mod -> internal_mod
/// - util_crate (CrateId 1): util_mod -> helpers_mod
///   (declared as "util" dependency of main_crate)
///
/// Both crates are in the same graph, simulating what ModuleResolver produces.
fn create_multi_crate_graph() -> (
    ModuleGraph,
    ash_core::module_graph::CrateId,
    ash_core::module_graph::CrateId,
) {
    use ash_core::module_graph::CrateId;

    let mut graph = ModuleGraph::new();

    // Create main_crate (CrateId 0)
    let main_crate_id = CrateId(0);
    let main_mod = graph.add_node(ModuleNode::new(
        "main".to_string(),
        ModuleSource::File("main/main.ash".to_string()),
    ));
    let main_internal = graph.add_node(ModuleNode::new(
        "internal".to_string(),
        ModuleSource::File("main/internal.ash".to_string()),
    ));
    graph.add_edge(main_mod, main_internal);
    graph.assign_module_to_crate(main_mod, main_crate_id);
    graph.assign_module_to_crate(main_internal, main_crate_id);

    // Create util_crate (CrateId 1)
    let util_crate_id = CrateId(1);
    let util_mod = graph.add_node(ModuleNode::new(
        "util".to_string(),
        ModuleSource::File("util/util.ash".to_string()),
    ));
    let util_helpers = graph.add_node(ModuleNode::new(
        "helpers".to_string(),
        ModuleSource::File("util/helpers.ash".to_string()),
    ));
    graph.add_edge(util_mod, util_helpers);
    graph.assign_module_to_crate(util_mod, util_crate_id);
    graph.assign_module_to_crate(util_helpers, util_crate_id);

    // Register crates in the graph
    graph.add_crate(
        "main_crate".to_string(),
        "main/main.ash".to_string(),
        main_mod,
    );
    graph.add_crate(
        "util_crate".to_string(),
        "util/util.ash".to_string(),
        util_mod,
    );

    // Add dependency: main_crate depends on util_crate as "util"
    graph.add_dependency(main_crate_id, "util".to_string(), util_crate_id);

    (graph, main_crate_id, util_crate_id)
}

// ========================================================================
// Cross-crate visibility tests using ImportResolver
// ========================================================================

#[test]
fn test_external_pub_import_allowed() {
    // external::util::public_item should succeed when item is pub
    let (graph, _main_crate, _util_crate) = create_multi_crate_graph();

    let mut resolver = ImportResolver::new(&graph);

    // Get module IDs
    let main_mod = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "main")
        .map(|(id, _)| *id)
        .unwrap();
    let util_mod = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "util")
        .map(|(id, _)| *id)
        .unwrap();

    // Add pub export to util module
    resolver.add_module_exports(
        util_mod,
        vec![("public_item".to_string(), Visibility::Public)],
    );

    // Add use statement from main module importing from external crate
    let use_path = UsePath::Simple(simple_path(&["external", "util", "public_item"]));
    resolver.add_module_uses(main_mod, vec![use_stmt(use_path)]);

    // Import should succeed - pub item is visible across crates
    let result = resolver.resolve_all();
    assert!(
        result.is_ok(),
        "Public item should be importable from external crate"
    );
}

#[test]
fn test_external_pub_crate_import_rejected() {
    // external::util::internal_item should fail when item is pub(crate)
    let (graph, _main_crate, _util_crate) = create_multi_crate_graph();

    let mut resolver = ImportResolver::new(&graph);

    // Get module IDs
    let main_mod = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "main")
        .map(|(id, _)| *id)
        .unwrap();
    let util_mod = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "util")
        .map(|(id, _)| *id)
        .unwrap();

    // Add pub(crate) export to util module
    resolver.add_module_exports(
        util_mod,
        vec![("internal_item".to_string(), Visibility::Crate)],
    );

    // Add use statement from main module importing from external crate
    let use_path = UsePath::Simple(simple_path(&["external", "util", "internal_item"]));
    resolver.add_module_uses(main_mod, vec![use_stmt(use_path)]);

    // Import should fail - pub(crate) item is not visible from external crate
    let result = resolver.resolve_all();
    assert!(
        result.is_err(),
        "pub(crate) item should NOT be importable from external crate"
    );

    let err = result.unwrap_err();
    assert!(
        matches!(err, ImportError::PrivateItem { .. }),
        "Expected PrivateItem error, got {:?}",
        err
    );
}

#[test]
fn test_external_pub_super_import_rejected() {
    // external::util::parent_item should fail when item is pub(super)
    let (graph, _main_crate, _util_crate) = create_multi_crate_graph();

    let mut resolver = ImportResolver::new(&graph);

    // Get module IDs
    let main_mod = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "main")
        .map(|(id, _)| *id)
        .unwrap();
    let util_helpers = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "helpers")
        .map(|(id, _)| *id)
        .unwrap();

    // Add pub(super) export to helpers module (child of util)
    resolver.add_module_exports(
        util_helpers,
        vec![("parent_item".to_string(), Visibility::Super { levels: 1 })],
    );

    // Add use statement from main module importing from external crate
    let use_path = UsePath::Simple(simple_path(&[
        "external",
        "util",
        "helpers",
        "parent_item",
    ]));
    resolver.add_module_uses(main_mod, vec![use_stmt(use_path)]);

    // Import should fail - pub(super) item is not visible from external crate
    let result = resolver.resolve_all();
    assert!(
        result.is_err(),
        "pub(super) item should NOT be importable from external crate"
    );
}

#[test]
fn test_external_pub_in_path_import_rejected() {
    // external::util::restricted_item should fail when item is pub(in crate::util)
    let (graph, _main_crate, _util_crate) = create_multi_crate_graph();

    let mut resolver = ImportResolver::new(&graph);

    // Get module IDs
    let main_mod = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "main")
        .map(|(id, _)| *id)
        .unwrap();
    let util_helpers = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "helpers")
        .map(|(id, _)| *id)
        .unwrap();

    // Add pub(in path) export to helpers module
    resolver.add_module_exports(
        util_helpers,
        vec![(
            "restricted_item".to_string(),
            Visibility::Restricted {
                path: "crate::util".into(),
            },
        )],
    );

    // Add use statement from main module importing from external crate
    let use_path = UsePath::Simple(simple_path(&[
        "external",
        "util",
        "helpers",
        "restricted_item",
    ]));
    resolver.add_module_uses(main_mod, vec![use_stmt(use_path)]);

    // Import should fail - pub(in path) item is not visible from external crate
    let result = resolver.resolve_all();
    assert!(
        result.is_err(),
        "pub(in path) item should NOT be importable from external crate"
    );
}

#[test]
fn test_external_glob_import_only_gets_pub() {
    // external::util::* should only get pub items, not pub(crate) items
    let (graph, _main_crate, _util_crate) = create_multi_crate_graph();

    let mut resolver = ImportResolver::new(&graph);

    // Get module IDs
    let main_mod = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "main")
        .map(|(id, _)| *id)
        .unwrap();
    let util_mod = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "util")
        .map(|(id, _)| *id)
        .unwrap();

    // Add both pub and pub(crate) exports to util module
    resolver.add_module_exports(
        util_mod,
        vec![
            ("public_item".to_string(), Visibility::Public),
            ("internal_item".to_string(), Visibility::Crate),
        ],
    );

    // Add glob use statement from main module
    let use_path = UsePath::Glob(simple_path(&["external", "util"]));
    resolver.add_module_uses(main_mod, vec![use_stmt(use_path)]);

    // Glob import should succeed but only import public_item
    let result = resolver.resolve_all();
    assert!(
        result.is_ok(),
        "Glob import should succeed (skipping non-visible items)"
    );

    let bindings = result.unwrap();
    let main_bindings = bindings.get(&main_mod).unwrap();

    // Should have public_item
    assert!(
        main_bindings.contains_key("public_item"),
        "Glob import should include pub items"
    );

    // Should NOT have internal_item
    assert!(
        !main_bindings.contains_key("internal_item"),
        "Glob import should NOT include pub(crate) items from external crate"
    );
}

#[test]
fn test_external_nested_import_with_alias() {
    // use external::util::{foo as bar, baz}; should work for pub items
    let (graph, _main_crate, _util_crate) = create_multi_crate_graph();

    let mut resolver = ImportResolver::new(&graph);

    // Get module IDs
    let main_mod = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "main")
        .map(|(id, _)| *id)
        .unwrap();
    let util_mod = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "util")
        .map(|(id, _)| *id)
        .unwrap();

    // Add pub exports to util module
    resolver.add_module_exports(
        util_mod,
        vec![
            ("foo".to_string(), Visibility::Public),
            ("baz".to_string(), Visibility::Public),
        ],
    );

    // Add nested use statement with alias
    let use_path = UsePath::Nested(
        simple_path(&["external", "util"]),
        vec![
            UseItem {
                name: "foo".into(),
                alias: Some("bar".into()),
            },
            use_item("baz"),
        ],
    );
    resolver.add_module_uses(main_mod, vec![use_stmt(use_path)]);

    // Import should succeed
    let result = resolver.resolve_all();
    assert!(result.is_ok(), "Nested import with alias should succeed");

    let bindings = result.unwrap();
    let main_bindings = bindings.get(&main_mod).unwrap();

    // foo should be aliased as bar
    assert!(
        main_bindings.contains_key("bar"),
        "Aliased import should create 'bar' binding"
    );
    assert!(
        !main_bindings.contains_key("foo"),
        "Original name 'foo' should not be bound when aliased"
    );

    // baz should be bound directly
    assert!(
        main_bindings.contains_key("baz"),
        "Direct import should create 'baz' binding"
    );
}

#[test]
fn test_undeclared_external_dependency_rejected() {
    // external::unknown::item should fail when "unknown" is not declared as dependency
    let (graph, _main_crate, _util_crate) = create_multi_crate_graph();

    let mut resolver = ImportResolver::new(&graph);

    // Get module IDs
    let main_mod = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "main")
        .map(|(id, _)| *id)
        .unwrap();

    // Add use statement to undeclared dependency
    let use_path = UsePath::Simple(simple_path(&["external", "unknown", "item"]));
    resolver.add_module_uses(main_mod, vec![use_stmt(use_path)]);

    // Import should fail - "unknown" is not a declared dependency
    let result = resolver.resolve_all();
    assert!(
        result.is_err(),
        "Import from undeclared external dependency should fail"
    );

    let err = result.unwrap_err();
    assert!(
        matches!(err, ImportError::ModuleNotFound { .. }),
        "Expected ModuleNotFound error for undeclared dependency, got {:?}",
        err
    );
}

#[test]
fn test_same_crate_pub_crate_allowed() {
    // crate::internal::item should succeed when item is pub(crate) from same crate
    let (graph, _main_crate, _util_crate) = create_multi_crate_graph();

    let mut resolver = ImportResolver::new(&graph);

    // Get module IDs
    let main_mod = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "main")
        .map(|(id, _)| *id)
        .unwrap();
    let main_internal = graph
        .nodes
        .iter()
        .find(|(_, n)| n.name == "internal")
        .map(|(id, _)| *id)
        .unwrap();

    // Add pub(crate) export to internal module (same crate as main_mod)
    resolver.add_module_exports(
        main_internal,
        vec![("internal_item".to_string(), Visibility::Crate)],
    );

    // Add use statement from main module to internal module (same crate)
    let use_path = UsePath::Simple(simple_path(&["crate", "internal", "internal_item"]));
    resolver.add_module_uses(main_mod, vec![use_stmt(use_path)]);

    // Import should succeed - pub(crate) is visible within the same crate
    let result = resolver.resolve_all();
    assert!(
        result.is_ok(),
        "pub(crate) item should be importable from same crate"
    );
}
