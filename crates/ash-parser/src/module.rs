//! Module AST types for the Ash parser.
//!
//! This module defines the AST types for module declarations, supporting
//! both file-based modules (`mod foo;`) and inline modules (`mod foo { ... }`).

use crate::surface::Definition;
use crate::surface::Visibility;
use crate::token::Span;

/// Module source tracking - distinguishes between file-based and inline modules.
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleSource {
    /// File-based module: `mod foo;`
    /// The module's content is loaded from an external file.
    File,
    /// Inline module: `mod foo { ... }`
    /// The module's content is defined inline with the given definitions.
    Inline(Vec<Definition>),
}

/// A module declaration in the AST.
///
/// Represents either a file-based module (`mod foo;`) or an inline module
/// (`mod foo { ... }`) with optional visibility modifiers.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDecl {
    /// Name of the module
    pub name: Box<str>,
    /// Visibility of the module (e.g., `pub`, `pub(crate)`, inherited)
    pub visibility: Visibility,
    /// Module source - either file-based or inline with definitions
    pub source: ModuleSource,
    /// Source span for error reporting
    pub span: Span,
}

impl ModuleDecl {
    /// Create a new file-based module declaration.
    ///
    /// # Arguments
    /// * `name` - The name of the module
    /// * `visibility` - The visibility modifier for the module
    /// * `span` - The source span for error reporting
    ///
    /// # Examples
    /// ```
    /// use ash_parser::module::ModuleDecl;
    /// use ash_parser::surface::Visibility;
    /// use ash_parser::token::Span;
    ///
    /// let decl = ModuleDecl::file("utils".into(), Visibility::Public, Span::new(0, 10, 1, 1));
    /// ```
    pub fn file(name: Box<str>, visibility: Visibility, span: Span) -> Self {
        Self {
            name,
            visibility,
            source: ModuleSource::File,
            span,
        }
    }

    /// Create a new inline module declaration.
    ///
    /// # Arguments
    /// * `name` - The name of the module
    /// * `visibility` - The visibility modifier for the module
    /// * `definitions` - The definitions contained within the inline module
    /// * `span` - The source span for error reporting
    ///
    /// # Examples
    /// ```
    /// use ash_parser::module::ModuleDecl;
    /// use ash_parser::surface::Visibility;
    /// use ash_parser::token::Span;
    ///
    /// let decl = ModuleDecl::inline("utils".into(), Visibility::Inherited, vec![], Span::new(0, 20, 1, 1));
    /// ```
    pub fn inline(
        name: Box<str>,
        visibility: Visibility,
        definitions: Vec<Definition>,
        span: Span,
    ) -> Self {
        Self {
            name,
            visibility,
            source: ModuleSource::Inline(definitions),
            span,
        }
    }

    /// Check if this is a file-based module.
    pub fn is_file_based(&self) -> bool {
        matches!(self.source, ModuleSource::File)
    }

    /// Check if this is an inline module.
    pub fn is_inline(&self) -> bool {
        matches!(self.source, ModuleSource::Inline(_))
    }

    /// Get the definitions if this is an inline module.
    ///
    /// Returns `Some(&Vec<Definition>)` for inline modules, `None` for file-based modules.
    pub fn definitions(&self) -> Option<&Vec<Definition>> {
        match &self.source {
            ModuleSource::Inline(defs) => Some(defs),
            ModuleSource::File => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Construction Tests
    // =========================================================================

    #[test]
    fn test_module_decl_creation() {
        // Test creating a basic ModuleDecl with file-based source
        let decl = ModuleDecl {
            name: "my_module".into(),
            visibility: Visibility::Inherited,
            source: ModuleSource::File,
            span: Span::new(0, 20, 1, 1),
        };

        assert_eq!(decl.name, "my_module".into());
        assert_eq!(decl.visibility, Visibility::Inherited);
        assert!(decl.is_file_based());
        assert!(!decl.is_inline());
        assert!(decl.definitions().is_none());
    }

    #[test]
    fn test_file_based_module() {
        // Test file-based module using the constructor
        let decl = ModuleDecl::file("utils".into(), Visibility::Public, Span::new(0, 15, 1, 1));

        assert_eq!(decl.name, "utils".into());
        assert_eq!(decl.visibility, Visibility::Public);
        assert!(decl.is_file_based());
        assert!(!decl.is_inline());
        assert!(matches!(decl.source, ModuleSource::File));
        assert!(decl.definitions().is_none());
    }

    #[test]
    fn test_inline_module_empty() {
        // Test inline module with no definitions
        let decl = ModuleDecl::inline(
            "internal".into(),
            Visibility::Crate,
            vec![],
            Span::new(0, 25, 1, 1),
        );

        assert_eq!(decl.name, "internal".into());
        assert_eq!(decl.visibility, Visibility::Crate);
        assert!(!decl.is_file_based());
        assert!(decl.is_inline());
        assert!(decl.definitions().is_some());
        assert!(decl.definitions().unwrap().is_empty());
    }

    #[test]
    fn test_inline_module_with_definitions() {
        // Test inline module with actual definitions
        let capability_def = Definition::Capability(crate::surface::CapabilityDef {
            name: "read_file".into(),
            effect: crate::surface::EffectType::Read,
            params: vec![],
            return_type: None,
            constraints: vec![],
            span: Span::new(10, 30, 1, 1),
        });

        let policy_def = Definition::Policy(crate::surface::PolicyDef {
            name: "RateLimit".into(),
            type_params: vec![],
            fields: vec![],
            where_clause: None,
            span: Span::new(35, 55, 1, 1),
        });

        let definitions = vec![capability_def, policy_def];

        let decl = ModuleDecl::inline(
            "submodule".into(),
            Visibility::Restricted {
                path: "parent::child".into(),
            },
            definitions,
            Span::new(0, 100, 1, 1),
        );

        assert_eq!(decl.name, "submodule".into());
        assert!(matches!(decl.visibility, Visibility::Restricted { .. }));
        assert!(!decl.is_file_based());
        assert!(decl.is_inline());
        assert_eq!(decl.definitions().unwrap().len(), 2);
    }

    // =========================================================================
    // Visibility Tests
    // =========================================================================

    #[test]
    fn test_module_visibility_variants() {
        // Test all visibility variants
        let visibilities = [
            Visibility::Inherited,
            Visibility::Public,
            Visibility::Crate,
            Visibility::Super,
            Visibility::Self_,
            Visibility::Restricted {
                path: "some::path".into(),
            },
        ];

        for (i, visibility) in visibilities.iter().enumerate() {
            let decl = ModuleDecl {
                name: format!("mod_{}", i).into(),
                visibility: visibility.clone(),
                source: ModuleSource::File,
                span: Span::new(0, 10, 1, 1),
            };

            assert_eq!(decl.visibility, *visibility);
        }
    }

    #[test]
    fn test_file_based_with_public_visibility() {
        // `pub mod foo;` - public file-based module
        let decl = ModuleDecl::file(
            "public_mod".into(),
            Visibility::Public,
            Span::new(0, 20, 1, 1),
        );

        assert!(decl.visibility.is_pub());
        assert!(decl.is_file_based());
    }

    // =========================================================================
    // Edge Case Tests
    // =========================================================================

    #[test]
    fn test_module_decl_clone() {
        let decl = ModuleDecl::inline(
            "test".into(),
            Visibility::Inherited,
            vec![],
            Span::new(0, 10, 1, 1),
        );

        let cloned = decl.clone();

        assert_eq!(cloned.name, decl.name);
        assert_eq!(cloned.visibility, decl.visibility);
        assert_eq!(cloned.source, decl.source);
    }

    #[test]
    fn test_module_source_equality() {
        let file1 = ModuleSource::File;
        let file2 = ModuleSource::File;
        let inline1 = ModuleSource::Inline(vec![]);
        let inline2 = ModuleSource::Inline(vec![]);

        assert_eq!(file1, file2);
        assert_eq!(inline1, inline2);
        assert_ne!(file1, inline1);
    }
}
