//! Tests for visibility checking
//!
//! These tests verify the correct behavior of visibility rules including:
//! - pub(super): visible in parent module and its descendants
//! - pub(self): visible only in current module
//! - pub(in path): visible only in specified module and its descendants
//! - pub(crate): visible throughout the crate
//! - pub: visible everywhere

use ash_parser::surface::Visibility;
use ash_typeck::visibility::{ModulePath, VisibilityExt};

/// Helper to create a ModulePath from string segments
fn path(segments: &[&str]) -> ModulePath {
    ModulePath::new(segments.iter().map(|s| s.to_string()).collect())
}

// ============================================================
// pub(super) visibility tests
// ============================================================

#[test]
fn test_pub_super_in_parent() {
    let item_module = path(&["crate", "a", "b"]);
    let from_module = path(&["crate", "a"]); // parent of b

    assert!(Visibility::Super.is_visible_path(&item_module, &from_module));
}

#[test]
fn test_pub_super_in_descendant_of_parent() {
    let item_module = path(&["crate", "a", "b"]);
    let from_module = path(&["crate", "a", "c", "d"]); // descendant of a (parent)

    assert!(Visibility::Super.is_visible_path(&item_module, &from_module));
}

#[test]
fn test_pub_super_not_in_sibling() {
    // This is a subtle point - siblings under the same parent ARE allowed
    // because they share the same parent and are descendants of that parent
    let item_module = path(&["crate", "a", "b"]);
    let from_module = path(&["crate", "a", "c"]); // sibling of b under same parent a

    // c IS a descendant of a, so it should be visible
    assert!(Visibility::Super.is_visible_path(&item_module, &from_module));
}

#[test]
fn test_pub_super_not_in_grandparent() {
    let item_module = path(&["crate", "a", "b"]);
    let from_module = path(&["crate"]); // grandparent, not parent

    assert!(!Visibility::Super.is_visible_path(&item_module, &from_module));
}

#[test]
fn test_pub_super_not_in_unrelated_branch() {
    let item_module = path(&["crate", "a", "b"]);
    let from_module = path(&["crate", "x", "y"]); // unrelated branch

    assert!(!Visibility::Super.is_visible_path(&item_module, &from_module));
}

#[test]
fn test_pub_super_at_root_is_crate() {
    let root = path(&["crate"]);
    let from_anywhere = path(&["crate", "a", "b"]);

    // pub(super) at root = pub(crate)
    assert!(Visibility::Super.is_visible_path(&root, &from_anywhere));
}

// ============================================================
// pub(self) visibility tests
// ============================================================

#[test]
fn test_pub_self_only_in_same_module() {
    let module = path(&["crate", "a", "b"]);

    assert!(Visibility::Self_.is_visible_path(&module, &module));

    let other = path(&["crate", "a", "c"]);
    assert!(!Visibility::Self_.is_visible_path(&module, &other));

    let child = path(&["crate", "a", "b", "c"]);
    assert!(!Visibility::Self_.is_visible_path(&module, &child));
}

// ============================================================
// pub(in path) visibility tests
// ============================================================

#[test]
fn test_pub_restricted_in_target_module() {
    let item_module = path(&["crate", "a", "b"]);
    let restricted_to = "crate::x::y".to_string();
    let from_allowed = path(&["crate", "x", "y"]);
    let from_descendant = path(&["crate", "x", "y", "z"]);
    let from_other = path(&["crate", "a"]);

    let vis = Visibility::Restricted {
        path: restricted_to.into(),
    };

    assert!(vis.is_visible_path(&item_module, &from_allowed));
    assert!(vis.is_visible_path(&item_module, &from_descendant));
    assert!(!vis.is_visible_path(&item_module, &from_other));
}

#[test]
fn test_pub_restricted_not_in_parent_of_target() {
    let item_module = path(&["crate", "a", "b"]);
    let restricted_to = "crate::x::y::z".to_string();
    let from_parent = path(&["crate", "x", "y"]); // parent of restricted

    let vis = Visibility::Restricted {
        path: restricted_to.into(),
    };

    assert!(!vis.is_visible_path(&item_module, &from_parent));
}

// ============================================================
// pub(crate) visibility tests
// ============================================================

#[test]
fn test_pub_crate_anywhere() {
    let item_module = path(&["crate", "a"]);
    let from_anywhere = path(&["crate", "b", "c", "d"]);

    assert!(Visibility::Crate.is_visible_path(&item_module, &from_anywhere));
}

// ============================================================
// pub visibility tests
// ============================================================

#[test]
fn test_pub_everywhere() {
    let item_module = path(&["crate", "a"]);
    let from_anywhere = path(&["some", "other", "crate"]);

    assert!(Visibility::Public.is_visible_path(&item_module, &from_anywhere));
}

// ============================================================
// Inherited (private) visibility tests
// ============================================================

#[test]
fn test_inherited_only_same_module() {
    let item_module = path(&["crate", "a", "b"]);
    let same_module = path(&["crate", "a", "b"]);
    let child = path(&["crate", "a", "b", "c"]);
    let parent = path(&["crate", "a"]);
    let sibling = path(&["crate", "a", "c"]);

    assert!(Visibility::Inherited.is_visible_path(&item_module, &same_module));
    assert!(!Visibility::Inherited.is_visible_path(&item_module, &child));
    assert!(!Visibility::Inherited.is_visible_path(&item_module, &parent));
    assert!(!Visibility::Inherited.is_visible_path(&item_module, &sibling));
}

// ============================================================
// ModulePath utility tests
// ============================================================

#[test]
fn test_module_path_parent() {
    let path = ModulePath::new(vec!["crate".to_string(), "a".to_string(), "b".to_string()]);
    let parent = path.parent();
    assert_eq!(parent.segments(), &["crate", "a"]);
}

#[test]
fn test_module_path_starts_with() {
    let path = ModulePath::new(vec![
        "crate".to_string(),
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
    ]);
    let prefix = ModulePath::new(vec!["crate".to_string(), "a".to_string()]);

    assert!(path.starts_with(&prefix));
    assert!(path.starts_with(&ModulePath::root()));
    assert!(!prefix.starts_with(&path));
}

#[test]
fn test_module_path_is_ancestor() {
    let ancestor = ModulePath::new(vec!["crate".to_string(), "a".to_string()]);
    let descendant = ModulePath::new(vec![
        "crate".to_string(),
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
    ]);

    assert!(ancestor.is_ancestor_of(&descendant));
    assert!(ancestor.is_ancestor_of(&ancestor));
    assert!(!descendant.is_ancestor_of(&ancestor));
    assert!(ancestor.is_strict_ancestor_of(&descendant));
    assert!(!ancestor.is_strict_ancestor_of(&ancestor));
}

#[test]
fn test_module_path_parse() {
    let path = ModulePath::parse("crate::a::b::c");
    assert_eq!(path.segments(), &["crate", "a", "b", "c"]);

    let root = ModulePath::parse("crate");
    assert!(root.is_root());

    let empty = ModulePath::parse("");
    assert!(empty.is_root());
}

#[test]
fn test_module_path_display() {
    let path = ModulePath::new(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(format!("{}", path), "a::b");

    let root = ModulePath::root();
    assert_eq!(format!("{}", root), "crate");
}

#[test]
fn test_module_path_child() {
    let parent = ModulePath::new(vec!["crate".to_string(), "a".to_string()]);
    let child = parent.child("b");
    assert_eq!(child.segments(), &["crate", "a", "b"]);
}

// ============================================================
// Edge cases
// ============================================================

#[test]
fn test_pub_super_single_segment() {
    // Test with paths that have single segment (just crate)
    let item = path(&["a"]);
    let from = path(&["a", "b"]);

    // a's parent is root, so pub(super) should be visible everywhere
    assert!(Visibility::Super.is_visible_path(&item, &from));
}

#[test]
fn test_restricted_at_root() {
    let restricted_to = "crate".to_string();
    let from_anywhere = path(&["crate", "a", "b"]);

    let vis = Visibility::Restricted {
        path: restricted_to.into(),
    };
    assert!(vis.is_visible_path(&ModulePath::root(), &from_anywhere));
}
