//! Checked public module-interface carriers.
//!
//! The carrier in this module is the core-owned public boundary for a checked
//! module. It deliberately retains no parser AST, binding environment, Engine
//! cache, callable body, or runtime authority. Those facts remain owned by the
//! parser, type checker, and Engine respectively.
//!
//! Existing [`ModuleSemanticSummary`] values are accepted only as a validated
//! compatibility payload. Their version family is intentionally independent
//! from this module-interface schema.

use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize};

use crate::ast::{Name, Visibility};
use crate::module_graph::{ModuleArtifact, ModuleArtifactOrigin, ModuleKey};
use crate::semantic_summary::{ModuleSemanticSummary, ModuleSemanticSummaryValidationError};

/// The durable wire schema implemented by [`PublicModuleInterface`].
pub const PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION: u32 = 1;

/// Versions understood by the checked public module-interface carrier.
///
/// This is deliberately distinct from [`crate::semantic_summary::SummaryVersion`]:
/// semantic summaries retain their own established compatibility contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum PublicModuleInterfaceVersion {
    /// The initial checked module-interface schema.
    V1 = PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
}

impl PublicModuleInterfaceVersion {
    /// Returns the durable numeric schema tag.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }
}

impl TryFrom<u32> for PublicModuleInterfaceVersion {
    type Error = ModuleInterfaceError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION => Ok(Self::V1),
            version => Err(ModuleInterfaceError::UnsupportedSchemaVersion { version }),
        }
    }
}

/// A declaration namespace represented by a public module binding.
///
/// The syntax namespaces are metadata only. In particular, neither
/// [`Self::SyntaxMacro`] nor [`Self::SyntaxNotation`] can represent an Engine
/// callable, an effect-row provider, or runtime authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleInterfaceBindingKind {
    /// A public structural child-module identity.
    ChildModule,
    /// A public value that is not a runtime callable.
    Value,
    /// A public callable declaration summary.
    Callable,
    /// A public ordinary type declaration.
    Type,
    /// A public constructor or variant declaration.
    Constructor,
    /// A public interface declaration.
    Interface,
    /// A public implementation declaration.
    Implementation,
    /// A public effect-row metadata declaration.
    EffectRow,
    /// A public syntax-phase macro declaration.
    SyntaxMacro,
    /// A public syntax-phase notation declaration.
    SyntaxNotation,
}

impl ModuleInterfaceBindingKind {
    /// Whether this namespace may describe a runtime callable.
    ///
    /// Syntax declarations are intentionally false even when their expansion
    /// target is an ordinary callable.
    #[must_use]
    pub const fn is_runtime_callable(self) -> bool {
        matches!(self, Self::Callable)
    }
}

/// Stable defining identity for one public interface binding.
///
/// A visible import or `pub use` alias never mutates this identity. Existing
/// typed identities remain in [`ModuleSemanticSummary`]; this generic carrier
/// covers the module-wide binding index without recreating those identities.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(
    rename_all = "snake_case",
    tag = "kind",
    content = "identity",
    deny_unknown_fields
)]
pub enum ModuleInterfaceDefiningIdentity {
    /// A canonical structural child-module identity.
    ChildModule(ModuleKey),
    /// A canonical declaration identity in one non-child namespace.
    Declaration(ModuleInterfaceDeclarationIdentity),
}

/// Canonical identity for a declaration namespace not otherwise represented by
/// an existing typed semantic-summary identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleInterfaceDeclarationIdentity {
    /// Defining module, never an import alias or filesystem path.
    pub module: ModuleKey,
    /// Defining declaration spelling, before any public aliasing.
    pub name: Name,
    /// Declaration namespace.
    pub kind: ModuleInterfaceBindingKind,
}

impl ModuleInterfaceDeclarationIdentity {
    /// Creates one stable declaration identity.
    #[must_use]
    pub fn new(module: ModuleKey, name: impl Into<Name>, kind: ModuleInterfaceBindingKind) -> Self {
        Self {
            module,
            name: name.into(),
            kind,
        }
    }
}

/// One public name bound by a checked module interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleInterfaceBinding {
    visible_name: Name,
    defining_identity: ModuleInterfaceDefiningIdentity,
    visibility: Visibility,
    origin: ModuleArtifactOrigin,
}

impl ModuleInterfaceBinding {
    /// Creates a declaration binding with a stable defining identity.
    #[must_use]
    pub fn declaration(
        visible_name: impl Into<Name>,
        defining_module: ModuleKey,
        defining_name: impl Into<Name>,
        kind: ModuleInterfaceBindingKind,
        visibility: Visibility,
        origin: ModuleArtifactOrigin,
    ) -> Self {
        Self {
            visible_name: visible_name.into(),
            defining_identity: ModuleInterfaceDefiningIdentity::Declaration(
                ModuleInterfaceDeclarationIdentity::new(defining_module, defining_name, kind),
            ),
            visibility,
            origin,
        }
    }

    /// Creates a public binding for an already-declared structural child.
    #[must_use]
    pub fn child(
        visible_name: impl Into<Name>,
        child: ModuleKey,
        visibility: Visibility,
        origin: ModuleArtifactOrigin,
    ) -> Self {
        Self {
            visible_name: visible_name.into(),
            defining_identity: ModuleInterfaceDefiningIdentity::ChildModule(child),
            visibility,
            origin,
        }
    }

    /// Re-exports this binding under `visible_name` without changing its
    /// defining identity, visibility, or source origin.
    #[must_use]
    pub fn reexport_as(&self, visible_name: impl Into<Name>) -> Self {
        Self {
            visible_name: visible_name.into(),
            defining_identity: self.defining_identity.clone(),
            visibility: self.visibility,
            origin: self.origin.clone(),
        }
    }

    /// Returns the name visible to an importing module.
    #[must_use]
    pub fn visible_name(&self) -> &str {
        &self.visible_name
    }

    /// Returns the stable identity of the defining declaration or child.
    #[must_use]
    pub fn defining_identity(&self) -> &ModuleInterfaceDefiningIdentity {
        &self.defining_identity
    }

    /// Returns this binding's declaration namespace.
    #[must_use]
    pub fn kind(&self) -> ModuleInterfaceBindingKind {
        match &self.defining_identity {
            ModuleInterfaceDefiningIdentity::ChildModule(_) => {
                ModuleInterfaceBindingKind::ChildModule
            }
            ModuleInterfaceDefiningIdentity::Declaration(identity) => identity.kind,
        }
    }

    /// Returns the checked visibility retained for this binding.
    #[must_use]
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    /// Returns the source origin retained for diagnostics.
    #[must_use]
    pub fn origin(&self) -> &ModuleArtifactOrigin {
        &self.origin
    }

    /// Returns whether this binding names a runtime callable.
    #[must_use]
    pub fn is_runtime_callable(&self) -> bool {
        self.kind().is_runtime_callable()
    }
}

/// Reference to one checked public interface dependency.
///
/// This carrier contains no binding authority. It is retained so an interface
/// cache can invalidate from canonical module identities rather than paths.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleInterfaceDependency {
    /// Canonical dependency module identity.
    pub module: ModuleKey,
    /// Schema version understood when this dependency was checked.
    pub schema_version: u32,
}

impl ModuleInterfaceDependency {
    /// Creates a dependency reference for a supported public-interface schema.
    #[must_use]
    pub const fn new(module: ModuleKey, schema_version: u32) -> Self {
        Self {
            module,
            schema_version,
        }
    }
}

/// Checked public projection of one module.
///
/// Constructing this type validates the entire supplied projection before a
/// value exists, so callers cannot publish a partial interface after a private
/// binding, duplicate name, invalid child, unsupported version, or malformed
/// compatibility summary.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PublicModuleInterface {
    schema_version: u32,
    artifact: ModuleArtifact,
    bindings: Vec<ModuleInterfaceBinding>,
    dependencies: Vec<ModuleInterfaceDependency>,
    semantic_summary: Option<ModuleSemanticSummary>,
}

impl PublicModuleInterface {
    /// Creates an export-closed interface without a legacy summary payload.
    ///
    /// # Errors
    ///
    /// Returns [`ModuleInterfaceError`] when a binding is private, duplicates a
    /// visible name, or publishes a child that is absent from `artifact`.
    pub fn new(
        artifact: ModuleArtifact,
        bindings: Vec<ModuleInterfaceBinding>,
    ) -> Result<Self, ModuleInterfaceError> {
        Self::from_parts(
            PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
            artifact,
            bindings,
            Vec::new(),
            None,
        )
    }

    /// Creates an interface with a compatibility payload from the existing
    /// semantic-summary substrate.
    ///
    /// The nested summary is validated only through its existing version
    /// contract. This API neither migrates [`crate::semantic_summary::ModuleIdentity`]
    /// nor changes the summary's V1--V8 version family.
    ///
    /// # Errors
    ///
    /// Returns [`ModuleInterfaceError`] when either interface or compatibility
    /// payload validation fails.
    pub fn with_compatibility_summary(
        artifact: ModuleArtifact,
        bindings: Vec<ModuleInterfaceBinding>,
        semantic_summary: ModuleSemanticSummary,
    ) -> Result<Self, ModuleInterfaceError> {
        Self::from_parts(
            PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
            artifact,
            bindings,
            Vec::new(),
            Some(semantic_summary),
        )
    }

    /// Adds canonical interface dependency references before validating the
    /// completed public projection.
    ///
    /// # Errors
    ///
    /// Returns [`ModuleInterfaceError`] when an unsupported dependency schema
    /// is supplied or the public projection is otherwise invalid.
    pub fn with_dependencies(
        artifact: ModuleArtifact,
        bindings: Vec<ModuleInterfaceBinding>,
        dependencies: Vec<ModuleInterfaceDependency>,
        semantic_summary: Option<ModuleSemanticSummary>,
    ) -> Result<Self, ModuleInterfaceError> {
        Self::from_parts(
            PUBLIC_MODULE_INTERFACE_SCHEMA_VERSION,
            artifact,
            bindings,
            dependencies,
            semantic_summary,
        )
    }

    fn from_parts(
        schema_version: u32,
        artifact: ModuleArtifact,
        mut bindings: Vec<ModuleInterfaceBinding>,
        mut dependencies: Vec<ModuleInterfaceDependency>,
        semantic_summary: Option<ModuleSemanticSummary>,
    ) -> Result<Self, ModuleInterfaceError> {
        PublicModuleInterfaceVersion::try_from(schema_version)?;

        if let Some(summary) = semantic_summary.as_ref() {
            summary
                .validate_summary_version_contract()
                .map_err(ModuleInterfaceError::InvalidCompatibilitySummary)?;
        }

        let mut visible_names = BTreeSet::new();
        for binding in &bindings {
            if binding.visibility != Visibility::Public {
                return Err(ModuleInterfaceError::NonPublicBinding {
                    name: binding.visible_name.clone(),
                    visibility: binding.visibility,
                });
            }
            if !visible_names.insert(binding.visible_name.clone()) {
                return Err(ModuleInterfaceError::DuplicateVisibleBinding {
                    name: binding.visible_name.clone(),
                });
            }
            match &binding.defining_identity {
                ModuleInterfaceDefiningIdentity::ChildModule(child) => {
                    if !artifact.child_keys().contains(child) {
                        return Err(ModuleInterfaceError::MissingStructuralChild {
                            name: binding.visible_name.clone(),
                            child: child.clone(),
                            module: artifact.key().clone(),
                        });
                    }
                    if let ModuleArtifactOrigin::Inline { parent, .. } = &binding.origin
                        && parent != artifact.key()
                    {
                        return Err(ModuleInterfaceError::InlineChildOriginParentMismatch {
                            name: binding.visible_name.clone(),
                            expected_parent: artifact.key().clone(),
                            actual_parent: parent.clone(),
                        });
                    }
                }
                ModuleInterfaceDefiningIdentity::Declaration(identity) => {
                    validate_generic_declaration_kind(identity.kind)?;
                }
            }
        }

        bindings.sort_by(|left, right| {
            left.visible_name
                .cmp(&right.visible_name)
                .then_with(|| left.kind().cmp(&right.kind()))
                .then_with(|| left.defining_identity.cmp(&right.defining_identity))
        });

        let mut dependency_modules = BTreeSet::new();
        for dependency in &dependencies {
            PublicModuleInterfaceVersion::try_from(dependency.schema_version)?;
            if !dependency_modules.insert(dependency.module.clone()) {
                return Err(ModuleInterfaceError::DuplicateDependency {
                    module: dependency.module.clone(),
                });
            }
        }
        dependencies.sort_unstable();

        Ok(Self {
            schema_version,
            artifact,
            bindings,
            dependencies,
            semantic_summary,
        })
    }

    /// Returns this interface's supported durable schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Returns the canonical module identity and source origin.
    #[must_use]
    pub const fn artifact(&self) -> &ModuleArtifact {
        &self.artifact
    }

    /// Returns public bindings in deterministic normalized order.
    #[must_use]
    pub fn bindings(&self) -> &[ModuleInterfaceBinding] {
        &self.bindings
    }

    /// Returns canonical dependency references in deterministic order.
    #[must_use]
    pub fn dependencies(&self) -> &[ModuleInterfaceDependency] {
        &self.dependencies
    }

    /// Returns the validated legacy summary compatibility payload, if supplied.
    #[must_use]
    pub const fn compatibility_summary(&self) -> Option<&ModuleSemanticSummary> {
        self.semantic_summary.as_ref()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SerializedPublicModuleInterface {
    schema_version: u32,
    artifact: ModuleArtifact,
    bindings: Vec<ModuleInterfaceBinding>,
    dependencies: Vec<ModuleInterfaceDependency>,
    semantic_summary: Option<ModuleSemanticSummary>,
}

impl<'de> Deserialize<'de> for PublicModuleInterface {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serialized = SerializedPublicModuleInterface::deserialize(deserializer)?;
        Self::from_parts(
            serialized.schema_version,
            serialized.artifact,
            serialized.bindings,
            serialized.dependencies,
            serialized.semantic_summary,
        )
        .map_err(serde::de::Error::custom)
    }
}

/// A public-interface construction or validation failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModuleInterfaceError {
    /// The durable interface schema is newer than this core crate supports.
    #[error("unsupported public module-interface schema version {version}")]
    UnsupportedSchemaVersion {
        /// Unsupported numeric schema version.
        version: u32,
    },
    /// The public projection attempted to retain a non-public binding.
    #[error(
        "public module interface cannot publish non-public binding {name:?} with {visibility:?} visibility"
    )]
    NonPublicBinding {
        /// Visible spelling rejected from the public projection.
        name: Name,
        /// Visibility that prevented publication.
        visibility: Visibility,
    },
    /// Two bindings would publish the same visible name.
    #[error("duplicate public module-interface binding {name:?}")]
    DuplicateVisibleBinding {
        /// Duplicated visible spelling.
        name: Name,
    },
    /// A public child binding was not present in the canonical artifact.
    #[error(
        "public child binding {name:?} targets {child}, which is not a structural child of {module}"
    )]
    MissingStructuralChild {
        /// Visible child binding spelling.
        name: Name,
        /// Canonical child identity missing from the artifact.
        child: ModuleKey,
        /// Interface module that attempted publication.
        module: ModuleKey,
    },
    /// An inline child binding named an enclosing module other than the
    /// interface artifact that publishes it.
    #[error(
        "inline public child binding {name:?} names enclosing module {actual_parent}, expected {expected_parent}"
    )]
    InlineChildOriginParentMismatch {
        /// Visible child binding spelling.
        name: Name,
        /// The enclosing interface's canonical module key.
        expected_parent: ModuleKey,
        /// Parent retained by the inline diagnostic origin.
        actual_parent: ModuleKey,
    },
    /// A generic declaration attempted to occupy the structural child namespace.
    #[error("generic declaration identity cannot use the child-module namespace")]
    GenericDeclarationUsesChildModuleKind,
    /// A declaration namespace already has a typed semantic-summary identity
    /// carrier and therefore cannot use the generic binding identity.
    #[error("generic declaration identity cannot use typed summary namespace {kind:?}")]
    GenericDeclarationRequiresTypedSummary {
        /// Typed namespace that must use its existing summary identity carrier.
        kind: ModuleInterfaceBindingKind,
    },
    /// Implementation publication remains deferred until it has a dedicated
    /// checked identity and closure contract.
    #[error("generic implementation declarations are unsupported in public module interfaces")]
    ImplementationBindingDeferred,
    /// The dependency list named one module more than once.
    #[error("duplicate public module-interface dependency {module}")]
    DuplicateDependency {
        /// Canonical duplicated dependency identity.
        module: ModuleKey,
    },
    /// The retained legacy semantic summary failed its existing validation contract.
    #[error("invalid module semantic-summary compatibility payload: {0:?}")]
    InvalidCompatibilitySummary(ModuleSemanticSummaryValidationError),
}

fn validate_generic_declaration_kind(
    kind: ModuleInterfaceBindingKind,
) -> Result<(), ModuleInterfaceError> {
    match kind {
        ModuleInterfaceBindingKind::ChildModule => {
            Err(ModuleInterfaceError::GenericDeclarationUsesChildModuleKind)
        }
        ModuleInterfaceBindingKind::Type
        | ModuleInterfaceBindingKind::Constructor
        | ModuleInterfaceBindingKind::Interface
        | ModuleInterfaceBindingKind::EffectRow => {
            Err(ModuleInterfaceError::GenericDeclarationRequiresTypedSummary { kind })
        }
        ModuleInterfaceBindingKind::Implementation => {
            Err(ModuleInterfaceError::ImplementationBindingDeferred)
        }
        ModuleInterfaceBindingKind::Value
        | ModuleInterfaceBindingKind::Callable
        | ModuleInterfaceBindingKind::SyntaxMacro
        | ModuleInterfaceBindingKind::SyntaxNotation => Ok(()),
    }
}
