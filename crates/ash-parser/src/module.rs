//! Module AST types for the Ash parser.
//!
//! This module defines the AST types for module declarations, supporting
//! both file-based modules (`mod foo;`) and inline modules (`mod foo { ... }`).

use std::ops::Deref;

use ash_core::module_graph::ModuleArtifact;

use crate::parse_utils::CommentTable;
use crate::surface::Definition;
use crate::surface::Visibility;
use crate::token::Span;
use crate::use_tree::Use;

/// One source-ordered syntactic item in a module body.
///
/// This is deliberately syntax-only. In particular, a [`Use`] here has not
/// been bound, checked, or turned into a runtime import.
#[allow(
    clippy::large_enum_variant,
    reason = "the public ordered-item API deliberately exposes direct typed payloads"
)]
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleItem {
    /// An existing `use` declaration.
    Use(Use),
    /// An existing top-level definition.
    Definition(Definition),
    /// A nested module declaration.
    ModuleDecl(ModuleDecl),
}

/// Parsed module contents shared by file-backed and inline module units.
///
/// The typed collections are the canonical parser-owned storage. A private
/// source-order index rebuilds [`Self::items`] as a read-only projection, so
/// compatibility transforms never turn the ordered view into a second mutable
/// authority. This carrier is syntax-only: it does not bind imports, create
/// interfaces, lower definitions, or grant Engine access.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModuleBody {
    uses: Vec<Use>,
    definitions: Vec<Definition>,
    module_decls: Vec<ModuleDecl>,
    order: Vec<ModuleItemIndex>,
    items: Vec<ModuleItem>,
    span: Span,
}

/// An internal source-order entry into one canonical typed collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModuleItemIndex {
    Use(usize),
    Definition(usize),
    ModuleDecl(usize),
}

impl ModuleBody {
    /// Creates an empty body spanning `span`.
    #[must_use]
    pub fn empty(span: Span) -> Self {
        Self {
            span,
            ..Self::default()
        }
    }

    /// Creates a body containing definition items in their supplied order.
    #[must_use]
    pub fn from_definitions(definitions: Vec<Definition>, span: Span) -> Self {
        let mut body = Self::empty(span);
        for definition in definitions {
            body.push_definition(definition);
        }
        body
    }

    /// Creates a body from source-ordered parsed items.
    ///
    /// The supplied order is retained as parser provenance while the canonical
    /// typed collections remain the sole mutable storage. This constructor is
    /// syntax-only: it does not bind, check, or authorize any item.
    #[must_use]
    pub fn from_items(items: Vec<ModuleItem>, span: Span) -> Self {
        let mut body = Self::empty(span);
        body.order = Vec::with_capacity(items.len());

        for item in items {
            match item {
                ModuleItem::Use(use_declaration) => {
                    body.uses.push(use_declaration);
                    body.order.push(ModuleItemIndex::Use(body.uses.len() - 1));
                }
                ModuleItem::Definition(definition) => {
                    body.definitions.push(definition);
                    body.order
                        .push(ModuleItemIndex::Definition(body.definitions.len() - 1));
                }
                ModuleItem::ModuleDecl(declaration) => {
                    body.module_decls.push(declaration);
                    body.order
                        .push(ModuleItemIndex::ModuleDecl(body.module_decls.len() - 1));
                }
            }
        }

        body.rebuild_item_snapshot();
        body
    }

    /// Returns parsed `use` declarations in their typed compatibility view.
    #[must_use]
    pub fn uses(&self) -> &[Use] {
        &self.uses
    }

    /// Returns parsed definitions in their typed compatibility view.
    #[must_use]
    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }

    /// Returns parsed nested module declarations in their typed compatibility view.
    #[must_use]
    pub fn module_decls(&self) -> &[ModuleDecl] {
        &self.module_decls
    }

    /// Returns all module items in their original source order.
    #[must_use]
    pub fn items(&self) -> &[ModuleItem] {
        &self.items
    }

    /// Returns the source span covering this module body.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Adds one parsed use declaration to both body views.
    pub(crate) fn push_use(&mut self, use_declaration: Use) {
        self.uses.push(use_declaration);
        self.order.push(ModuleItemIndex::Use(self.uses.len() - 1));
        self.rebuild_item_snapshot();
    }

    /// Adds one parsed definition to both body views.
    pub(crate) fn push_definition(&mut self, definition: Definition) {
        self.definitions.push(definition);
        self.order
            .push(ModuleItemIndex::Definition(self.definitions.len() - 1));
        self.rebuild_item_snapshot();
    }

    /// Adds one parsed child declaration to both body views.
    pub(crate) fn push_module_decl(&mut self, declaration: ModuleDecl) {
        self.module_decls.push(declaration);
        self.order
            .push(ModuleItemIndex::ModuleDecl(self.module_decls.len() - 1));
        self.rebuild_item_snapshot();
    }

    /// Returns mutable definitions for parser-owned compatibility transforms.
    pub(crate) fn definitions_mut(&mut self) -> &mut [Definition] {
        &mut self.definitions
    }

    /// Returns mutable child declarations for parser-owned compatibility transforms.
    pub(crate) fn module_decls_mut(&mut self) -> &mut [ModuleDecl] {
        &mut self.module_decls
    }

    /// Sets the complete source span after parsing the enclosing delimiter.
    pub(crate) fn set_span(&mut self, span: Span) {
        self.span = span;
    }

    /// Rebuilds the read-only ordered view after parser-owned transforms.
    ///
    /// The order is immutable parser provenance; only its canonical payloads
    /// may be rewritten by syntax-phase compatibility transforms.
    pub(crate) fn rebuild_item_snapshot(&mut self) {
        self.items = self
            .order
            .iter()
            .map(|index| match index {
                ModuleItemIndex::Use(index) => ModuleItem::Use(self.uses[*index].clone()),
                ModuleItemIndex::Definition(index) => {
                    ModuleItem::Definition(self.definitions[*index].clone())
                }
                ModuleItemIndex::ModuleDecl(index) => {
                    ModuleItem::ModuleDecl(self.module_decls[*index].clone())
                }
            })
            .collect();
    }
}

/// Preserve historical inline-definition slice ergonomics while the canonical
/// inline payload is a complete [`ModuleBody`].
impl Deref for ModuleBody {
    type Target = [Definition];

    fn deref(&self) -> &Self::Target {
        self.definitions()
    }
}

/// A source-kind-independent parser handoff for later module realization.
///
/// This carrier records source acquisition facts only. It does not bind uses,
/// create checked interfaces, lower definitions, or grant Engine admission.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleUnit {
    artifact: ModuleArtifact,
    body: ModuleBody,
    source_path: Option<Box<str>>,
    comments: CommentTable,
}

impl ModuleUnit {
    /// Constructs a fully acquired module unit.
    #[must_use]
    pub fn new(
        artifact: ModuleArtifact,
        mut body: ModuleBody,
        source_path: Option<Box<str>>,
        comments: CommentTable,
    ) -> Self {
        body.rebuild_item_snapshot();
        Self {
            artifact,
            body,
            source_path,
            comments,
        }
    }

    /// Returns the validated identity and source origin of this unit.
    #[must_use]
    pub fn artifact(&self) -> &ModuleArtifact {
        &self.artifact
    }

    /// Returns the source-kind-independent parsed body.
    #[must_use]
    pub fn body(&self) -> &ModuleBody {
        &self.body
    }

    /// Returns the enclosing-source diagnostic anchor.
    ///
    /// File units return their selected source path. Inline units return the
    /// enclosing parent source path, because their declaration span is anchored
    /// in that source rather than in a standalone file.
    #[must_use]
    pub fn source_path(&self) -> Option<&str> {
        self.source_path.as_deref()
    }

    /// Returns parser comment trivia retained for diagnostics and later syntax work.
    #[must_use]
    pub fn comments(&self) -> &CommentTable {
        &self.comments
    }
}

/// Module source tracking - distinguishes between file-based and inline modules.
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleSource {
    /// File-based module: `mod foo;`
    /// The module's content is loaded from an external file.
    File,
    /// Inline module: `mod foo { ... }`
    /// The module's content is defined inline with its complete parsed body.
    Inline(Box<ModuleBody>),
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
    /// Module source - either file-based or inline with a complete body
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
            source: ModuleSource::Inline(Box::new(ModuleBody::from_definitions(definitions, span))),
            span,
        }
    }

    /// Create a new inline module declaration with a complete parsed body.
    #[must_use]
    pub fn inline_body(
        name: Box<str>,
        visibility: Visibility,
        body: ModuleBody,
        span: Span,
    ) -> Self {
        Self {
            name,
            visibility,
            source: ModuleSource::Inline(Box::new(body)),
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
    /// Returns `Some(&[Definition])` for inline modules, `None` for file-based modules.
    pub fn definitions(&self) -> Option<&[Definition]> {
        match &self.source {
            ModuleSource::Inline(body) => Some(body.definitions()),
            ModuleSource::File => None,
        }
    }

    /// Returns the complete parsed inline body, if this declaration is inline.
    #[must_use]
    pub fn body(&self) -> Option<&ModuleBody> {
        match &self.source {
            ModuleSource::Inline(body) => Some(body),
            ModuleSource::File => None,
        }
    }

    /// Iterate over parsed inline-module role definitions.
    #[cfg(test)]
    pub(crate) fn role_definitions(&self) -> impl Iterator<Item = &crate::surface::RoleDef> {
        self.definitions()
            .into_iter()
            .flatten()
            .filter_map(|definition| match definition {
                Definition::Role(role) => Some(role),
                _ => None,
            })
    }

    /// Lower parsed inline-module role definitions into core role metadata.
    #[cfg(test)]
    pub(crate) fn lower_role_definitions(
        &self,
    ) -> Result<Vec<ash_core::Role>, crate::lower::RoleLoweringError> {
        crate::lower::lower_module_role_definitions(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_tree::{SimplePath, UsePath};
    use ash_core::RoleObligationRef;

    // =========================================================================
    // Construction Tests
    // =========================================================================

    #[test]
    fn module_body_from_items_preserves_source_order_and_typed_views() {
        let span = Span::new(0, 40, 1, 1);
        let use_declaration = Use {
            visibility: Visibility::Inherited,
            path: UsePath::Simple(SimplePath {
                segments: vec!["crate".into(), "support".into()],
            }),
            alias: None,
            span,
        };
        let child = ModuleDecl::file("child".into(), Visibility::Public, span);
        let definition = Definition::Policy(crate::surface::PolicyDef {
            name: "Policy".into(),
            type_params: Vec::new(),
            fields: Vec::new(),
            where_clause: None,
            span,
        });

        let body = ModuleBody::from_items(
            vec![
                ModuleItem::Use(use_declaration.clone()),
                ModuleItem::ModuleDecl(child.clone()),
                ModuleItem::Definition(definition.clone()),
            ],
            span,
        );

        assert_eq!(body.uses(), std::slice::from_ref(&use_declaration));
        assert_eq!(body.module_decls(), std::slice::from_ref(&child));
        assert_eq!(body.definitions(), std::slice::from_ref(&definition));
        assert_eq!(
            body.items(),
            [
                ModuleItem::Use(use_declaration),
                ModuleItem::ModuleDecl(child),
                ModuleItem::Definition(definition),
            ]
        );
    }

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
            visibility: crate::surface::Visibility::Inherited,
            name: "read_file".into(),
            effect: crate::surface::EffectType::Read,
            params: vec![],
            return_type: None,
            constraints: vec![],
            target_provider: None,
            target_action: None,
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

    #[test]
    fn test_inline_module_role_definitions_exposes_only_roles() {
        let role_def = Definition::Role(crate::surface::RoleDef {
            name: "reviewer".into(),
            capabilities: vec![crate::surface::CapabilityDecl {
                capability: "approve".into(),
                constraints: None,
                span: Span::new(10, 40, 1, 1),
            }],
            obligations: vec!["check_tests".into()],
            span: Span::new(10, 40, 1, 1),
        });

        let capability_def = Definition::Capability(crate::surface::CapabilityDef {
            visibility: crate::surface::Visibility::Inherited,
            name: "read_file".into(),
            effect: crate::surface::EffectType::Read,
            params: vec![],
            return_type: None,
            constraints: vec![],
            target_provider: None,
            target_action: None,
            span: Span::new(45, 75, 1, 1),
        });

        let decl = ModuleDecl::inline(
            "governance".into(),
            Visibility::Inherited,
            vec![capability_def, role_def],
            Span::new(0, 90, 1, 1),
        );

        let roles = decl.role_definitions().collect::<Vec<_>>();

        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name.as_ref(), "reviewer");
        assert_eq!(roles[0].capabilities.len(), 1);
        assert_eq!(roles[0].capabilities[0].capability.as_ref(), "approve");
        assert_eq!(roles[0].obligations, vec!["check_tests".into()]);
    }

    #[test]
    fn test_inline_module_lower_role_definitions_uses_core_role_carrier() {
        let decl = ModuleDecl::inline(
            "governance".into(),
            Visibility::Inherited,
            vec![
                Definition::Capability(crate::surface::CapabilityDef {
                    visibility: crate::surface::Visibility::Inherited,
                    name: "approve".into(),
                    effect: crate::surface::EffectType::Decide,
                    params: vec![],
                    return_type: None,
                    constraints: vec![],
                    target_provider: None,
                    target_action: None,
                    span: Span::new(10, 30, 1, 1),
                }),
                Definition::Capability(crate::surface::CapabilityDef {
                    visibility: crate::surface::Visibility::Inherited,
                    name: "review".into(),
                    effect: crate::surface::EffectType::Analyze,
                    params: vec![],
                    return_type: None,
                    constraints: vec![],
                    target_provider: None,
                    target_action: None,
                    span: Span::new(31, 50, 1, 1),
                }),
                Definition::Role(crate::surface::RoleDef {
                    name: "reviewer".into(),
                    capabilities: vec![
                        crate::surface::CapabilityDecl {
                            capability: "approve".into(),
                            constraints: None,
                            span: Span::new(51, 70, 1, 1),
                        },
                        crate::surface::CapabilityDecl {
                            capability: "review".into(),
                            constraints: None,
                            span: Span::new(71, 90, 1, 1),
                        },
                    ],
                    obligations: vec!["check_tests".into(), "audit_log".into()],
                    span: Span::new(51, 100, 1, 1),
                }),
            ],
            Span::new(0, 80, 1, 1),
        );

        let roles = decl
            .lower_role_definitions()
            .expect("matching capability definitions should lower authority metadata");

        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "reviewer");
        assert_eq!(roles[0].authority.len(), 2);
        assert!(matches!(
            &roles[0].obligations[..],
            [
                RoleObligationRef { name: first },
                RoleObligationRef { name: second }
            ] if first == "check_tests" && second == "audit_log"
        ));
    }

    #[test]
    fn test_file_module_role_helpers_are_empty() {
        let decl = ModuleDecl::file(
            "governance".into(),
            Visibility::Inherited,
            Span::new(0, 15, 1, 1),
        );

        assert_eq!(decl.role_definitions().count(), 0);
        assert!(
            decl.lower_role_definitions()
                .expect("file modules should have no lowered roles")
                .is_empty()
        );
    }

    #[test]
    fn test_inline_module_lower_role_definitions_rejects_unknown_capability_name() {
        let decl = ModuleDecl::inline(
            "governance".into(),
            Visibility::Inherited,
            vec![Definition::Role(crate::surface::RoleDef {
                name: "reviewer".into(),
                capabilities: vec![crate::surface::CapabilityDecl {
                    capability: "approve".into(),
                    constraints: None,
                    span: Span::new(10, 70, 1, 1),
                }],
                obligations: vec!["check_tests".into()],
                span: Span::new(10, 70, 1, 1),
            })],
            Span::new(0, 80, 1, 1),
        );

        let error = decl
            .lower_role_definitions()
            .expect_err("unknown capability names should be rejected");

        assert_eq!(error.role, "reviewer");
        assert_eq!(error.authority, "approve");
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
            Visibility::Super { levels: 1 },
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
        let inline1 = ModuleSource::Inline(Box::default());
        let inline2 = ModuleSource::Inline(Box::default());

        assert_eq!(file1, file2);
        assert_eq!(inline1, inline2);
        assert_ne!(file1, inline1);
    }
}
