//! Surface `visibility_tests` module.

use super::*;

#[test]
fn test_visibility_private() {
    let vis = Visibility::Inherited;
    assert!(!vis.is_pub());
}

#[test]
fn test_visibility_public() {
    let vis = Visibility::Public;
    assert!(vis.is_pub());
}

#[test]
fn test_visibility_crate() {
    let vis = Visibility::Crate;
    assert!(vis.is_pub());
    assert!(vis.is_visible_in_module("crate::foo", "crate::bar"));
}

#[test]
fn test_visibility_super() {
    let vis = Visibility::Super { levels: 1 };
    assert!(vis.is_pub());
    assert!(vis.is_visible_in_module("crate::foo::bar", "crate::foo"));
}

#[test]
fn test_visibility_self() {
    let vis = Visibility::Self_;
    assert!(vis.is_pub());
    assert!(vis.is_visible_in_module("crate::foo", "crate::foo"));
    assert!(!vis.is_visible_in_module("crate::foo", "crate::bar"));
}

#[test]
fn test_visibility_restricted() {
    let vis = Visibility::Restricted {
        path: "crate::internal".into(),
    };
    assert!(vis.is_pub());
    assert!(vis.is_visible_in_module("crate::internal::sub", "crate::other"));
    assert!(!vis.is_visible_in_module("crate::public", "crate::other"));
}
