//! Visibility checking for the Ash type checker
//!
//! This module provides visibility checking during type checking,
//! ensuring that items are only accessed from modules where they are visible.

use ash_parser::surface::Visibility;
use thiserror::Error;

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
    /// * `owner_module` - The module path where the item is defined
    /// * `current_module` - The module path where the access is occurring
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
        if item_visibility.is_visible_in_module(current_module, owner_module) {
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

    #[test]
    fn test_pub_crate_not_accessible_from_external() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Crate;

        // Should NOT be accessible from external crates
        let result = checker.check_access(&visibility, "crate::foo", "external::crate", "item");
        assert!(result.is_err());
    }

    // ============================================================
    // pub(super) visibility tests
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
                .check_access(&visibility, "crate::foo", "crate::foo::sibling", "item")
                .is_ok()
        );
    }

    #[test]
    fn test_pub_super_not_accessible_from_unrelated() {
        let checker = VisibilityChecker::new();
        let visibility = Visibility::Super;

        // Should NOT be accessible from unrelated modules
        let result = checker.check_access(&visibility, "crate::foo", "crate::bar", "item");
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
