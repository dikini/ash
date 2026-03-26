//! Visibility checking for the Ash type checker
//!
//! This module provides visibility checking during type checking,
//! ensuring that items are only accessed from modules where they are visible.

use ash_parser::surface::Visibility;
use thiserror::Error;

/// A module path represented as a sequence of segments.
///
/// This provides proper hierarchical module path handling with methods
/// for parent lookup, ancestor checks, and descendant checks.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct ModulePath {
    segments: Vec<String>,
}

impl ModulePath {
    /// Create a new ModulePath from a vector of segment strings.
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    /// Create a root module path (empty segments).
    pub fn root() -> Self {
        Self { segments: vec![] }
    }

    /// Get the parent module path.
    ///
    /// Returns the root path if already at root.
    pub fn parent(&self) -> Self {
        if self.segments.is_empty() {
            return Self::root();
        }
        Self {
            segments: self.segments[..self.segments.len() - 1].to_vec(),
        }
    }

    /// Create a child module path by appending a segment.
    pub fn child(&self, name: impl Into<String>) -> Self {
        let mut segments = self.segments.clone();
        segments.push(name.into());
        Self { segments }
    }

    /// Check if this path starts with the given prefix.
    ///
    /// The root path is a prefix of all paths.
    pub fn starts_with(&self, prefix: &Self) -> bool {
        if prefix.segments.is_empty() {
            return true;
        }
        if prefix.segments.len() > self.segments.len() {
            return false;
        }
        self.segments[..prefix.segments.len()] == prefix.segments
    }

    /// Check if this path is a parent of (or equal to) the given path.
    pub fn is_ancestor_of(&self, other: &Self) -> bool {
        other.starts_with(self)
    }

    /// Check if this path is a strict parent of (but not equal to) the given path.
    pub fn is_strict_ancestor_of(&self, other: &Self) -> bool {
        other.starts_with(self) && other.segments.len() > self.segments.len()
    }

    /// Check if this path is empty (root).
    pub fn is_root(&self) -> bool {
        self.segments.is_empty()
    }

    /// Get the segments of this path.
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// Parse a module path from a string using `::` as delimiter.
    pub fn parse(path: &str) -> Self {
        if path.is_empty() || path == "crate" {
            return Self::root();
        }
        let segments: Vec<String> = path.split("::").map(|s| s.to_string()).collect();
        // Filter out empty segments that might result from leading/trailing delimiters
        let segments: Vec<String> = segments.into_iter().filter(|s| !s.is_empty()).collect();
        Self { segments }
    }
}

impl std::fmt::Display for ModulePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.segments.is_empty() {
            write!(f, "crate")
        } else {
            write!(f, "{}", self.segments.join("::"))
        }
    }
}

/// Extension trait for Visibility to work with ModulePath.
pub trait VisibilityExt {
    /// Check if an item with this visibility in `item_module`
    /// is visible from `from_module`.
    fn is_visible_path(&self, item_module: &ModulePath, from_module: &ModulePath) -> bool;
}

impl VisibilityExt for Visibility {
    fn is_visible_path(&self, item_module: &ModulePath, from_module: &ModulePath) -> bool {
        match self {
            Visibility::Public => true,

            Visibility::Inherited => from_module == item_module,

            Visibility::Crate => {
                // For now, we assume same crate if both paths start with the same root
                // In a real implementation, this would check crate roots
                true
            }

            Visibility::Super => {
                let parent = item_module.parent();
                if parent.is_root() {
                    // At root, pub(super) = pub(crate)
                    return true;
                }
                // Visible if from_module is parent or descendant of parent
                from_module == &parent || from_module.starts_with(&parent)
            }

            Visibility::Self_ => from_module == item_module,

            Visibility::Restricted { path } => {
                let restricted_path = ModulePath::parse(path);
                from_module == &restricted_path || from_module.starts_with(&restricted_path)
            }
        }
    }
}

/// Error type for visibility violations
#[derive(Debug, Clone, Error, PartialEq)]
pub enum VisibilityError {
    /// Item is not visible from the calling module
    #[error("item `{item}` is private")]
    PrivateItem {
        /// Name of the item being accessed
        item: String,
        /// Module where the item is defined
        owner_module: String,
        /// Module attempting to access the item
        current_module: String,
    },
    /// Missing context information needed for visibility check
    #[error("missing visibility context")]
    MissingContext {
        /// Description of what context is missing
        reason: String,
    },
}

/// Checks visibility of items during type checking
///
/// The visibility checker uses the module graph and visibility annotations
/// to determine if an item can be accessed from a given module.
#[derive(Debug, Clone)]
pub struct VisibilityChecker;

impl VisibilityChecker {
    /// Create a new visibility checker
    pub fn new() -> Self {
        Self
    }

    /// Check if an item is accessible from the current module
    ///
    /// # Arguments
    /// * `item_visibility` - The visibility annotation on the item
    /// * `owner_module` - The module path where the item is defined (as string)
    /// * `current_module` - The module path where the access is occurring (as string)
    /// * `item_name` - The name of the item being accessed
    ///
    /// # Returns
    /// * `Ok(())` if the item is accessible
    /// * `Err(VisibilityError::PrivateItem)` if the item is not visible
    pub fn check_access(
        &self,
        item_visibility: &Visibility,
        owner_module: &str,
        current_module: &str,
        item_name: &str,
    ) -> Result<(), VisibilityError> {
        let owner = ModulePath::parse(owner_module);
        let current = ModulePath::parse(current_module);

        if item_visibility.is_visible_path(&owner, &current) {
            Ok(())
        } else {
            Err(VisibilityError::PrivateItem {
                item: item_name.to_string(),
                owner_module: owner_module.to_string(),
                current_module: current_module.to_string(),
            })
        }
    }
}

impl Default for VisibilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // ModulePath tests
    // ============================================================

    #[test]
    fn test_module_path_parse() {
        let path = ModulePath::parse("crate::a::b::c");
        assert_eq!(path.segments(), &["crate", "a", "b", "c"]);
    }

    #[test]
    fn test_module_path_parent() {
        let path = ModulePath::parse("crate::a::b::c");
        let parent = path.parent();
        assert_eq!(parent.segments(), &["crate", "a", "b"]);

        let grandparent = parent.parent();
        assert_eq!(grandparent.segments(), &["crate", "a"]);
    }

    #[test]
    fn test_module_path_parent_at_root() {
        let root = ModulePath::root();
        let parent = root.parent();
        assert!(parent.is_root());
    }

    #[test]
    fn test_module_path_starts_with() {
        let path = ModulePath::parse("crate::a::b::c");
        let prefix = ModulePath::parse("crate::a");
        assert!(path.starts_with(&prefix));
        assert!(path.starts_with(&ModulePath::root()));
        assert!(!prefix.starts_with(&path));
    }

    #[test]
    fn test_module_path_is_ancestor() {
        let ancestor = ModulePath::parse("crate::a");
        let descendant = ModulePath::parse("crate::a::b::c");
        assert!(ancestor.is_ancestor_of(&descendant));
        assert!(ancestor.is_ancestor_of(&ancestor)); // Equal paths are ancestors
        assert!(!descendant.is_ancestor_of(&ancestor));
    }

    // ============================================================
    // VisibilityExt tests - pub(super)
    // ============================================================

    #[test]
    fn test_pub_super_in_parent() {
        let item_module = ModulePath::parse("crate::a::b");
        let from_module = ModulePath::parse("crate::a"); // parent of b

        assert!(Visibility::Super.is_visible_path(&item_module, &from_module));
    }

    #[test]
    fn test_pub_super_in_descendant_of_parent() {
        let item_module = ModulePath::parse("crate::a::b");
        let from_module = ModulePath::parse("crate::a::c::d"); // descendant of a

        assert!(Visibility::Super.is_visible_path(&item_module, &from_module));
    }

    #[test]
    fn test_pub_super_not_in_grandparent() {
        let item_module = ModulePath::parse("crate::a::b");
        let from_module = ModulePath::parse("crate"); // grandparent

        assert!(!Visibility::Super.is_visible_path(&item_module, &from_module));
    }

    #[test]
    fn test_pub_super_not_in_unrelated() {
        let item_module = ModulePath::parse("crate::a::b");
        let from_module = ModulePath::parse("crate::x::y"); // unrelated branch

        assert!(!Visibility::Super.is_visible_path(&item_module, &from_module));
    }

    #[test]
    fn test_pub_super_at_root_is_crate() {
        let root = ModulePath::root();
        let from_anywhere = ModulePath::parse("crate::a::b");

        // pub(super) at root = pub(crate)
        assert!(Visibility::Super.is_visible_path(&root, &from_anywhere));
    }

    // ============================================================
    // VisibilityExt tests - pub(self)
    // ============================================================

    #[test]
    fn test_pub_self_only_in_same_module() {
        let module = ModulePath::parse("crate::a::b");

        assert!(Visibility::Self_.is_visible_path(&module, &module));

        let other = ModulePath::parse("crate::a::c");
        assert!(!Visibility::Self_.is_visible_path(&module, &other));
    }

    // ============================================================
    // VisibilityExt tests - pub(in path)
    // ============================================================

    #[test]
    fn test_pub_restricted_in_target_module() {
        let from_allowed = ModulePath::parse("crate::x::y");
        let from_descendant = ModulePath::parse("crate::x::y::z");
        let from_other = ModulePath::parse("crate::a");

        let vis = Visibility::Restricted {
            path: "crate::x::y".into(),
        };

        assert!(vis.is_visible_path(&ModulePath::parse("crate::any"), &from_allowed));
        assert!(vis.is_visible_path(&ModulePath::parse("crate::any"), &from_descendant));
        assert!(!vis.is_visible_path(&ModulePath::parse("crate::any"), &from_other));
    }

    #[test]
    fn test_pub_restricted_deeply_nested() {
        let vis = Visibility::Restricted {
            path: "crate::a::b::c".into(),
        };

        assert!(vis.is_visible_path(
            &ModulePath::parse("crate::x"),
            &ModulePath::parse("crate::a::b::c")
        ));
        assert!(vis.is_visible_path(
            &ModulePath::parse("crate::x"),
            &ModulePath::parse("crate::a::b::c::sub")
        ));

        // Should NOT be accessible from parent of restricted path
        assert!(!vis.is_visible_path(
            &ModulePath::parse("crate::x"),
            &ModulePath::parse("crate::a::b")
        ));
    }

    // ============================================================
    // Public visibility tests
    // ============================================================

    #[test]
    fn test_public_item_accessible_everywhere() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Public;

        // Should be accessible from any module
        assert!(
            checker
                .check_access(&visibility, "crate::foo", "crate::bar", "item")
                .is_ok()
        );
        assert!(
            checker
                .check_access(&visibility, "crate::foo", "crate::foo::sub", "item")
                .is_ok()
        );
        assert!(
            checker
                .check_access(&visibility, "crate::foo", "external::crate", "item")
                .is_ok()
        );
    }

    // ============================================================
    // Private (Inherited) visibility tests
    // ============================================================

    #[test]
    fn test_private_item_accessible_in_same_module() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Inherited;

        // Should be accessible in the same module
        assert!(
            checker
                .check_access(&visibility, "crate::foo", "crate::foo", "item")
                .is_ok()
        );
    }

    #[test]
    fn test_private_item_not_accessible_from_other_module() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Inherited;

        // Should NOT be accessible from a different module
        let result = checker.check_access(&visibility, "crate::foo", "crate::bar", "item");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(
            err,
            VisibilityError::PrivateItem {
                item: "item".to_string(),
                owner_module: "crate::foo".to_string(),
                current_module: "crate::bar".to_string(),
            }
        );
    }

    #[test]
    fn test_private_item_not_accessible_from_submodule() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Inherited;

        // Should NOT be accessible from a submodule
        let result = checker.check_access(&visibility, "crate::foo", "crate::foo::sub", "item");
        assert!(result.is_err());
    }

    // ============================================================
    // pub(crate) visibility tests
    // ============================================================

    #[test]
    fn test_pub_crate_accessible_in_same_crate() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Crate;

        // Should be accessible anywhere in the same crate
        assert!(
            checker
                .check_access(&visibility, "crate::foo", "crate::bar", "item")
                .is_ok()
        );
        assert!(
            checker
                .check_access(&visibility, "crate::foo", "crate::foo::sub", "item")
                .is_ok()
        );
        assert!(
            checker
                .check_access(&visibility, "crate::deep::nested", "crate::other", "item")
                .is_ok()
        );
    }

    // ============================================================
    // pub(super) visibility tests via checker
    // ============================================================

    #[test]
    fn test_pub_super_accessible_from_parent() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Super;

        // Should be accessible from parent module
        assert!(
            checker
                .check_access(&visibility, "crate::foo", "crate::foo::sub", "item")
                .is_ok()
        );
        assert!(
            checker
                .check_access(&visibility, "crate::a::b", "crate::a::b::c::d", "item")
                .is_ok()
        );
    }

    #[test]
    fn test_pub_super_accessible_from_sibling() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Super;

        // Should be accessible from sibling modules (same parent)
        assert!(
            checker
                .check_access(
                    &visibility,
                    "crate::foo::bar",
                    "crate::foo::sibling",
                    "item"
                )
                .is_ok()
        );
    }

    #[test]
    fn test_pub_super_not_accessible_from_unrelated() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Super;

        // Should NOT be accessible from unrelated modules
        // Using crate::a::foo so parent is crate::a, not crate (root)
        let result = checker.check_access(&visibility, "crate::a::foo", "crate::bar", "item");
        assert!(result.is_err());
    }

    #[test]
    fn test_pub_super_not_accessible_from_grandparent() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Super;

        // Should NOT be accessible from grandparent
        let result = checker.check_access(&visibility, "crate::a::b", "crate", "item");
        assert!(result.is_err());
    }

    // ============================================================
    // pub(self) visibility tests
    // ============================================================

    #[test]
    fn test_pub_self_accessible_in_same_module() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Self_;

        // Should be accessible only in the same module
        assert!(
            checker
                .check_access(&visibility, "crate::foo", "crate::foo", "item")
                .is_ok()
        );
    }

    #[test]
    fn test_pub_self_not_accessible_from_other() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Self_;

        // Should NOT be accessible from any other module
        let result = checker.check_access(&visibility, "crate::foo", "crate::bar", "item");
        assert!(result.is_err());

        let result = checker.check_access(&visibility, "crate::foo", "crate::foo::sub", "item");
        assert!(result.is_err());
    }

    // ============================================================
    // Restricted path visibility tests
    // ============================================================

    #[test]
    fn test_restricted_visible_in_specified_path() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Restricted {
            path: "crate::foo".into(),
        };

        // Should be accessible from the specified path
        assert!(
            checker
                .check_access(&visibility, "crate::foo::bar", "crate::foo", "item")
                .is_ok()
        );
    }

    #[test]
    fn test_restricted_visible_in_submodules_of_path() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Restricted {
            path: "crate::foo".into(),
        };

        // Should be accessible from submodules of the specified path
        assert!(
            checker
                .check_access(&visibility, "crate::foo::bar", "crate::foo::sub", "item")
                .is_ok()
        );
        assert!(
            checker
                .check_access(
                    &visibility,
                    "crate::foo::bar",
                    "crate::foo::deep::nested",
                    "item"
                )
                .is_ok()
        );
    }

    #[test]
    fn test_restricted_not_accessible_outside_path() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Restricted {
            path: "crate::foo".into(),
        };

        // Should NOT be accessible from outside the specified path
        let result = checker.check_access(&visibility, "crate::foo::bar", "crate::other", "item");
        assert!(result.is_err());

        let result = checker.check_access(&visibility, "crate::foo::bar", "crate::bar", "item");
        assert!(result.is_err());
    }

    #[test]
    fn test_restricted_deeply_nested_path() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Restricted {
            path: "crate::a::b::c".into(),
        };

        // Should work with deeply nested paths
        assert!(
            checker
                .check_access(
                    &visibility,
                    "crate::a::b::c::item",
                    "crate::a::b::c",
                    "item"
                )
                .is_ok()
        );
        assert!(
            checker
                .check_access(
                    &visibility,
                    "crate::a::b::c::item",
                    "crate::a::b::c::sub",
                    "item"
                )
                .is_ok()
        );

        // Should NOT be accessible from parent of restricted path
        let result =
            checker.check_access(&visibility, "crate::a::b::c::item", "crate::a::b", "item");
        assert!(result.is_err());
    }

    // ============================================================
    // Error message tests
    // ============================================================

    #[test]
    fn test_private_item_error_display() {
        let err = VisibilityError::PrivateItem {
            item: "MyStruct".to_string(),
            owner_module: "crate::internal".to_string(),
            current_module: "crate::external".to_string(),
        };

        let display = format!("{err}");
        assert!(display.contains("MyStruct"));
        assert!(display.contains("private"));
    }

    #[test]
    fn test_missing_context_error_display() {
        let err = VisibilityError::MissingContext {
            reason: "module graph not available".to_string(),
        };

        let display = format!("{err}");
        assert!(display.contains("missing visibility context"));
    }
}
