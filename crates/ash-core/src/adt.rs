//! Algebraic Data Type (ADT) support for Ash
//!
//! Provides `AdtName` for fully qualified ADT names that preserve module paths.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

/// Fully qualified name for an Algebraic Data Type (ADT).
///
/// `AdtName` preserves the full module path to allow disambiguating types
/// with the same root name defined in different modules, as required by
/// SPEC-003 Section 3.3.
///
/// # Examples
///
/// ```
/// use ash_core::adt::AdtName;
///
/// // Create from a fully qualified string
/// let name = AdtName::new("std::option::Option");
/// assert_eq!(name.qualified, "std::option::Option");
/// assert_eq!(name.module_path(), "std::option");
/// assert_eq!(name.root, "Option");
///
/// // Root-level type (no module)
/// let root = AdtName::new("MyType");
/// assert_eq!(root.module_path(), "");
/// assert_eq!(root.root, "MyType");
/// ```
#[derive(Clone, Debug, Eq, Serialize, Deserialize)]
pub struct AdtName {
    /// Fully qualified name: "std::option::Option"
    pub qualified: String,
    /// Module path components: ["std", "option"]
    pub module: Vec<String>,
    /// Root name without module: "Option"
    pub root: String,
}

impl Hash for AdtName {
    /// Hash only the qualified name for consistency with PartialEq.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.qualified.hash(state);
    }
}

impl AdtName {
    /// Create an `AdtName` from a fully qualified string.
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_core::adt::AdtName;
    ///
    /// let name = AdtName::new("a::b::MyType");
    /// assert_eq!(name.qualified, "a::b::MyType");
    /// assert_eq!(name.module, vec!["a", "b"]);
    /// assert_eq!(name.root, "MyType");
    /// ```
    pub fn new(qualified: impl Into<String>) -> Self {
        let qualified = qualified.into();
        let parts: Vec<_> = qualified.split("::").collect();
        let root = parts.last().unwrap_or(&"").to_string();
        let module = parts[..parts.len().saturating_sub(1)]
            .iter()
            .map(|s| s.to_string())
            .collect();

        Self {
            qualified,
            module,
            root,
        }
    }

    /// Create an `AdtName` from module path components and a root name.
    ///
    /// # Examples
    ///
    /// ```
    /// use ash_core::adt::AdtName;
    ///
    /// let name = AdtName::from_parts(&["std".to_string(), "option".to_string()], "Option");
    /// assert_eq!(name.qualified, "std::option::Option");
    /// assert_eq!(name.module_path(), "std::option");
    /// ```
    pub fn from_parts(module: &[String], root: &str) -> Self {
        let mut qualified = module.join("::");
        if !qualified.is_empty() {
            qualified.push_str("::");
        }
        qualified.push_str(root);

        Self {
            qualified,
            module: module.to_vec(),
            root: root.to_string(),
        }
    }

    /// Get the module path as a string (e.g., "std::option").
    ///
    /// Returns an empty string for root-level types.
    pub fn module_path(&self) -> String {
        self.module.join("::")
    }

    /// Check if this is a root-level type (no module path).
    pub fn is_root(&self) -> bool {
        self.module.is_empty()
    }

    /// Get the display representation (same as qualified).
    pub fn display(&self) -> &str {
        &self.qualified
    }
}

impl PartialEq for AdtName {
    /// Equality compares the fully qualified name.
    ///
    /// This ensures that `a::T` and `b::T` are considered different types,
    /// as required by SPEC-003 Section 3.3.
    fn eq(&self, other: &Self) -> bool {
        self.qualified == other.qualified
    }
}

impl fmt::Display for AdtName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.qualified)
    }
}

impl From<&str> for AdtName {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for AdtName {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// Reference to an ADT that may or may not be fully resolved.
///
/// Used during name resolution to track whether a type reference
/// has been fully qualified with its module path.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AdtRef {
    /// Unresolved reference - just the name as written in source
    Unresolved(String),
    /// Fully resolved reference with qualified name
    Resolved(AdtName),
}

impl AdtRef {
    /// Get the qualified name if resolved, or the raw name if unresolved.
    pub fn as_str(&self) -> &str {
        match self {
            AdtRef::Unresolved(name) => name,
            AdtRef::Resolved(adt_name) => &adt_name.qualified,
        }
    }

    /// Check if this reference has been resolved.
    pub fn is_resolved(&self) -> bool {
        matches!(self, AdtRef::Resolved(_))
    }

    /// Get the resolved name if available.
    pub fn resolved(&self) -> Option<&AdtName> {
        match self {
            AdtRef::Resolved(name) => Some(name),
            AdtRef::Unresolved(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // AdtName Tests
    // ============================================================

    #[test]
    fn adt_name_from_qualified_string() {
        let name = AdtName::new("std::option::Option");
        assert_eq!(name.qualified, "std::option::Option");
        assert_eq!(name.module, vec!["std", "option"]);
        assert_eq!(name.root, "Option");
    }

    #[test]
    fn adt_name_root_level() {
        let name = AdtName::new("MyType");
        assert_eq!(name.qualified, "MyType");
        assert!(name.module.is_empty());
        assert_eq!(name.root, "MyType");
        assert!(name.is_root());
    }

    #[test]
    fn adt_name_from_parts() {
        let module = vec!["a".to_string(), "b".to_string()];
        let name = AdtName::from_parts(&module, "T");
        assert_eq!(name.qualified, "a::b::T");
        assert_eq!(name.module_path(), "a::b");
        assert_eq!(name.root, "T");
    }

    #[test]
    fn adt_name_from_empty_parts() {
        let name = AdtName::from_parts(&[], "T");
        assert_eq!(name.qualified, "T");
        assert_eq!(name.module_path(), "");
        assert_eq!(name.root, "T");
    }

    #[test]
    fn adt_name_equality_uses_qualified() {
        let a = AdtName::new("a::T");
        let b = AdtName::new("b::T");
        let a2 = AdtName::new("a::T");

        // Same qualified name should be equal
        assert_eq!(a, a2);

        // Different qualified names should not be equal
        assert_ne!(a, b);
        assert_ne!(a2, b);
    }

    #[test]
    fn adt_name_same_root_different_modules() {
        // This is the key test for SPEC-003 compliance
        let option_std = AdtName::new("std::option::Option");
        let option_custom = AdtName::new("myapp::option::Option");

        // Same root name, different modules
        assert_eq!(option_std.root, option_custom.root);
        assert_ne!(option_std.module, option_custom.module);

        // Should NOT be equal
        assert_ne!(option_std, option_custom);
    }

    #[test]
    fn adt_name_display() {
        let name = AdtName::new("a::b::C");
        assert_eq!(name.to_string(), "a::b::C");
        assert_eq!(name.display(), "a::b::C");
    }

    #[test]
    fn adt_name_from_str() {
        let name: AdtName = "x::y::Z".into();
        assert_eq!(name.qualified, "x::y::Z");
        assert_eq!(name.root, "Z");
    }

    #[test]
    fn adt_name_from_string() {
        let name: AdtName = "x::y::Z".to_string().into();
        assert_eq!(name.qualified, "x::y::Z");
        assert_eq!(name.root, "Z");
    }

    // ============================================================
    // AdtRef Tests
    // ============================================================

    #[test]
    fn adt_ref_unresolved() {
        let r = AdtRef::Unresolved("MyType".to_string());
        assert!(!r.is_resolved());
        assert_eq!(r.as_str(), "MyType");
        assert!(r.resolved().is_none());
    }

    #[test]
    fn adt_ref_resolved() {
        let name = AdtName::new("a::b::T");
        let r = AdtRef::Resolved(name);
        assert!(r.is_resolved());
        assert_eq!(r.as_str(), "a::b::T");
        assert!(r.resolved().is_some());
    }

    // ============================================================
    // Edge Cases
    // ============================================================

    #[test]
    fn adt_name_deeply_nested() {
        let name = AdtName::new("a::b::c::d::e::Type");
        assert_eq!(name.module, vec!["a", "b", "c", "d", "e"]);
        assert_eq!(name.root, "Type");
    }

    #[test]
    fn adt_name_single_module() {
        let name = AdtName::new("module::Type");
        assert_eq!(name.module, vec!["module"]);
        assert_eq!(name.root, "Type");
    }
}
