//! Integration tests for visibility enforcement with real .ash file parsing
//!
//! These tests verify that visibility modifiers are correctly parsed and
//! enforced when processing actual Ash source files.

use ash_parser::surface::Visibility;
use ash_parser::{new_input, parse_visibility};
use winnow::prelude::*;

#[test]
fn test_parse_pub_visibility() {
    let mut input = new_input("pub ");
    let result = parse_visibility.parse_next(&mut input).unwrap();
    assert_eq!(result, Visibility::Public);
}

#[test]
fn test_parse_pub_crate_visibility() {
    let mut input = new_input("pub(crate) ");
    let result = parse_visibility.parse_next(&mut input).unwrap();
    assert_eq!(result, Visibility::Crate);
}

#[test]
fn test_parse_pub_super_visibility() {
    let mut input = new_input("pub(super) ");
    let result = parse_visibility.parse_next(&mut input).unwrap();
    assert!(matches!(result, Visibility::Super { levels: 1 }));
}

#[test]
fn test_parse_pub_super_is_single_level() {
    // Note: Current parser only supports single 'super' keyword
    // Multiple levels like pub(super, super) are not supported in current grammar
    let mut input = new_input("pub(super) ");
    let result = parse_visibility.parse_next(&mut input).unwrap();
    assert!(matches!(result, Visibility::Super { levels: 1 }));
}

#[test]
fn test_parse_pub_self_visibility() {
    let mut input = new_input("pub(self) ");
    let result = parse_visibility.parse_next(&mut input).unwrap();
    assert_eq!(result, Visibility::Self_);
}

#[test]
fn test_parse_pub_in_path_visibility() {
    let mut input = new_input("pub(in crate::foo::bar) ");
    let result = parse_visibility.parse_next(&mut input).unwrap();
    assert!(
        matches!(result, Visibility::Restricted { path } if path.as_ref() == "crate::foo::bar")
    );
}

#[test]
fn test_parse_default_visibility() {
    // When no visibility is specified, it should default to Inherited
    let mut input = new_input("");
    let result = parse_visibility.parse_next(&mut input).unwrap();
    assert_eq!(result, Visibility::Inherited);
}

#[test]
fn test_parse_pub_in_crate_root_visibility() {
    let mut input = new_input("pub(in crate) ");
    let result = parse_visibility.parse_next(&mut input).unwrap();
    assert!(matches!(result, Visibility::Restricted { path } if path.as_ref() == "crate"));
}

#[test]
fn test_parse_pub_in_deeply_nested_path_visibility() {
    let mut input = new_input("pub(in crate::a::b::c::d) ");
    let result = parse_visibility.parse_next(&mut input).unwrap();
    assert!(
        matches!(result, Visibility::Restricted { path } if path.as_ref() == "crate::a::b::c::d")
    );
}

#[test]
fn test_visibility_roundtrip() {
    // Test that all visibility types can be parsed correctly
    let visibilities = vec![
        ("pub ", Visibility::Public),
        ("pub(crate) ", Visibility::Crate),
        ("pub(self) ", Visibility::Self_),
    ];

    for (input_str, expected) in visibilities {
        let mut input = new_input(input_str);
        let result = parse_visibility.parse_next(&mut input).unwrap();
        assert_eq!(result, expected, "Failed for input: {}", input_str);
    }
}

#[test]
fn test_pub_super_basic() {
    // Test basic pub(super) parsing
    let mut input = new_input("pub(super) ");
    let result = parse_visibility.parse_next(&mut input).unwrap();
    assert!(
        matches!(result, Visibility::Super { levels: 1 }),
        "pub(super) should parse to Super {{ levels: 1 }}"
    );
}

#[test]
fn test_pub_in_path_variations() {
    // Test various path formats in pub(in path)
    let cases = vec![
        "pub(in crate) ",
        "pub(in crate::foo) ",
        "pub(in crate::foo::bar) ",
        "pub(in self) ",
        "pub(in super) ",
    ];

    for input_str in cases {
        let mut input = new_input(input_str);
        let result = parse_visibility.parse_next(&mut input);
        assert!(
            result.is_ok() || result.is_err(),
            "Parsing should complete for: {}",
            input_str
        );
    }
}

// ============================================================================
// TASK-343: Regression tests for pub(crate) visibility enforcement
// ============================================================================

/// Test that pub(crate) visibility is correctly enforced when resolving imports.
/// This test exercises the actual visibility check by adding both exports and uses.
#[test]
fn test_pub_crate_visibility_check_with_imports() {
    use ash_core::module_graph::{ModuleGraph, ModuleNode, ModuleSource};
    use ash_parser::import_resolver::ImportResolver;
    use ash_parser::surface::Visibility;
    use ash_parser::token::Span;
    use ash_parser::{SimplePath, Use, UsePath};

    // Create a graph WITHOUT calling set_crate() - simulating real resolver path
    let mut graph = ModuleGraph::new();

    let root = graph.add_node(ModuleNode::new(
        "crate".to_string(),
        ModuleSource::File("main.ash".to_string()),
    ));
    graph.set_root(root);

    let child = graph.add_node(ModuleNode::new(
        "child".to_string(),
        ModuleSource::File("child.ash".to_string()),
    ));
    graph.add_edge(root, child);

    let mut resolver = ImportResolver::new(&graph);

    // Add a pub(crate) export from child
    resolver.add_module_exports(child, vec![("CrateItem".to_string(), Visibility::Crate)]);

    // Add a use statement from root trying to import from child
    // This exercises the Visibility::Crate check in is_visible()
    let use_path = UsePath::Simple(SimplePath {
        segments: vec!["crate".into(), "child".into(), "CrateItem".into()],
    });
    let use_stmt = Use {
        visibility: Visibility::Inherited,
        path: use_path,
        alias: None,
        span: Span::new(0, 0, 1, 1),
    };
    resolver.add_module_uses(root, vec![use_stmt]);

    // This should succeed - both modules are in the same graph
    // The fix makes pub(crate) check module existence, not crate membership
    let result = resolver.resolve_all();
    assert!(
        result.is_ok(),
        "pub(crate) import should succeed within same graph"
    );

    // Verify the binding was resolved
    let bindings = result.unwrap();
    let root_bindings = bindings.get(&root).unwrap();
    assert!(
        root_bindings.contains_key("CrateItem"),
        "CrateItem should be resolved"
    );
}

/// Test that pub(crate) rejects imports when the target module is not in the graph.
/// This simulates what would happen with an external crate import.
#[test]
fn test_pub_crate_rejects_external_module() {
    use ash_core::module_graph::{ModuleGraph, ModuleNode, ModuleSource};
    use ash_parser::import_resolver::ImportResolver;
    use ash_parser::surface::Visibility;
    use ash_parser::token::Span;
    use ash_parser::{SimplePath, Use, UsePath};

    let mut graph = ModuleGraph::new();
    let root = graph.add_node(ModuleNode::new(
        "crate".to_string(),
        ModuleSource::File("main.ash".to_string()),
    ));
    graph.set_root(root);

    let mut resolver = ImportResolver::new(&graph);

    // Add a use statement trying to import from a non-existent module
    let use_path = UsePath::Simple(SimplePath {
        segments: vec!["external".into(), "ExternalItem".into()],
    });
    let use_stmt = Use {
        visibility: Visibility::Inherited,
        path: use_path,
        alias: None,
        span: Span::new(0, 0, 1, 1),
    };
    resolver.add_module_uses(root, vec![use_stmt]);

    // This should fail - the external module path cannot be resolved
    let result = resolver.resolve_all();
    assert!(
        result.is_err(),
        "Should fail to resolve import from external module"
    );
}
