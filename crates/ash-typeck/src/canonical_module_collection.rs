//! Private carriers for canonical two-tier module collection.
//!
//! TASK-2075 Task 4 defines the closed declaration domain and the read-only
//! carrier boundary. Graph-wide collection remains deliberately fail-closed
//! until Task 5 implements namespace classification and collision handling.

use std::collections::BTreeMap;

use ash_core::module_graph::ModuleKey;
use ash_parser::surface::{
    Definition, ExpandedSurfaceOrigin, ExpansionId, Expr, IdentifierHygieneMetadata, Visibility,
};
use ash_parser::{CanonicalExpandedModuleGraph, Span};
use thiserror::Error;

/// Closed set of declarations consumed by canonical module collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalDeclarationKind {
    /// Source notation declaration.
    Notation,
    /// Expression macro declaration.
    Macro,
    /// Removed capability syntax, which is rejected atomically.
    Capability,
    /// Resource type declaration.
    ResourceType,
    /// Ordinary type declaration.
    Type,
    /// Nominal newtype declaration.
    Newtype,
    /// Effect-row alias declaration.
    EffectAlias,
    /// Named effect-row group declaration.
    EffectGroup,
    /// Promoted data-kind declaration.
    DataKind,
    /// Type-level function declaration.
    TypeFn,
    /// Named proposition predicate declaration.
    PropositionPredicate,
    /// Policy declaration.
    Policy,
    /// Role declaration.
    Role,
    /// Interface declaration.
    Interface,
    /// Interface implementation declaration.
    Impl,
    /// Ordinary function declaration.
    Function,
    /// Handler declaration.
    Handler,
    /// Runtime-provided builtin function declaration.
    BuiltinFn,
    /// Sealed type-level domain declaration.
    SealedDomain,
    /// Law declaration.
    Law,
    /// Proof declaration.
    Proof,
    /// Structural module declaration.
    ModuleDecl,
}

impl CanonicalDeclarationKind {
    /// Every declaration kind in the closed TASK-2075 collection domain.
    pub const ALL: [Self; 22] = [
        Self::Notation,
        Self::Macro,
        Self::Capability,
        Self::ResourceType,
        Self::Type,
        Self::Newtype,
        Self::EffectAlias,
        Self::EffectGroup,
        Self::DataKind,
        Self::TypeFn,
        Self::PropositionPredicate,
        Self::Policy,
        Self::Role,
        Self::Interface,
        Self::Impl,
        Self::Function,
        Self::Handler,
        Self::BuiltinFn,
        Self::SealedDomain,
        Self::Law,
        Self::Proof,
        Self::ModuleDecl,
    ];

    /// Returns the fixed collection treatment for this declaration kind.
    #[must_use]
    pub const fn collection_disposition(self) -> CanonicalCollectionDisposition {
        use CanonicalCollectionDisposition::{Collect, RejectAtomically};
        use CanonicalNamespace::{
            Evidence, ImplementationRegistry, Interface, Macro, Notation, Policy, PromotedKind,
            Proposition, Role, RowName, StructuralModule, TypeComputation, TypeDomain,
            ValueCallable,
        };

        match self {
            Self::Notation => Collect {
                namespace: Notation,
                publish_in_name_view: true,
            },
            Self::Macro => Collect {
                namespace: Macro,
                publish_in_name_view: true,
            },
            Self::Capability => RejectAtomically,
            Self::ResourceType | Self::Type | Self::Newtype | Self::SealedDomain => Collect {
                namespace: TypeDomain,
                publish_in_name_view: true,
            },
            Self::EffectAlias | Self::EffectGroup => Collect {
                namespace: RowName,
                publish_in_name_view: true,
            },
            Self::DataKind => Collect {
                namespace: PromotedKind,
                publish_in_name_view: true,
            },
            Self::TypeFn => Collect {
                namespace: TypeComputation,
                publish_in_name_view: true,
            },
            Self::PropositionPredicate => Collect {
                namespace: Proposition,
                publish_in_name_view: true,
            },
            Self::Policy => Collect {
                namespace: Policy,
                publish_in_name_view: true,
            },
            Self::Role => Collect {
                namespace: Role,
                publish_in_name_view: true,
            },
            Self::Interface => Collect {
                namespace: Interface,
                publish_in_name_view: true,
            },
            Self::Impl => Collect {
                namespace: ImplementationRegistry,
                publish_in_name_view: false,
            },
            Self::Function | Self::Handler | Self::BuiltinFn => Collect {
                namespace: ValueCallable,
                publish_in_name_view: true,
            },
            Self::Law | Self::Proof => Collect {
                namespace: Evidence,
                publish_in_name_view: true,
            },
            Self::ModuleDecl => Collect {
                namespace: StructuralModule,
                publish_in_name_view: true,
            },
        }
    }
}

/// Namespace buckets used by canonical declaration lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalNamespace {
    /// Notation patterns.
    Notation,
    /// Syntax-phase macros.
    Macro,
    /// Type and domain names.
    TypeDomain,
    /// Named effect rows.
    RowName,
    /// Promoted data kinds.
    PromotedKind,
    /// Type-level computations.
    TypeComputation,
    /// Type-level propositions.
    Proposition,
    /// Policies.
    Policy,
    /// Roles.
    Role,
    /// Interfaces.
    Interface,
    /// Internal-only implementation registry entries.
    ImplementationRegistry,
    /// Value-level callables.
    ValueCallable,
    /// Laws and proofs.
    Evidence,
    /// Structural child modules.
    StructuralModule,
}

/// Whether a declaration is collected and whether its name may be provisionally published.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalCollectionDisposition {
    /// Collect the declaration in the indicated namespace.
    Collect {
        /// Namespace used for lookup and collision checks.
        namespace: CanonicalNamespace,
        /// Whether the declaration may enter the provisional name-only view.
        publish_in_name_view: bool,
    },
    /// Reject the complete graph without publishing either view.
    RejectAtomically,
}

/// Stable typed origin of one collected declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonicalDeclarationOriginKey {
    /// A declaration written directly in source.
    Source {
        /// Layout-stable source-order position within the canonical module.
        source_ordinal: usize,
    },
    /// A declaration produced by surface expansion.
    Expanded {
        /// Stable expansion identity assigned by the parser.
        expansion_id: ExpansionId,
        /// Layout-stable source-order position within the canonical module.
        source_ordinal: usize,
    },
}

/// Stable identity retained for one collected declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonicalDeclarationIdentity {
    module_key: ModuleKey,
    kind: CanonicalDeclarationKind,
    canonical_parent: Option<Box<CanonicalDeclarationIdentity>>,
    origin_key: CanonicalDeclarationOriginKey,
}

impl CanonicalDeclarationIdentity {
    /// Returns the canonical module containing the declaration.
    #[must_use]
    pub fn module_key(&self) -> &ModuleKey {
        &self.module_key
    }

    /// Returns the declaration's closed kind.
    #[must_use]
    pub const fn kind(&self) -> CanonicalDeclarationKind {
        self.kind
    }

    /// Returns the canonical parent identity for a nested declaration, when present.
    #[must_use]
    pub fn canonical_parent(&self) -> Option<&CanonicalDeclarationIdentity> {
        self.canonical_parent.as_deref()
    }

    /// Returns the stable source or expansion origin key.
    #[must_use]
    pub const fn origin_key(&self) -> &CanonicalDeclarationOriginKey {
        &self.origin_key
    }
}

/// Namespace-qualified local lookup key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalLookupKey {
    namespace: CanonicalNamespace,
    visible_local_key: Box<str>,
}

impl CanonicalLookupKey {
    /// Returns the namespace bucket.
    #[must_use]
    pub const fn namespace(&self) -> CanonicalNamespace {
        self.namespace
    }

    /// Returns the local key visible to source lookup.
    #[must_use]
    pub fn visible_local_key(&self) -> &str {
        &self.visible_local_key
    }
}

/// Checker-internal raw facts for one collected declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalCollectedEntry {
    identity: CanonicalDeclarationIdentity,
    lookup_key: CanonicalLookupKey,
    declared_name: Option<Box<str>>,
    raw_definition: Option<Definition>,
}

impl CanonicalCollectedEntry {
    /// Returns the stable declaration identity.
    #[must_use]
    pub fn identity(&self) -> &CanonicalDeclarationIdentity {
        &self.identity
    }

    /// Returns the namespace-qualified lookup key.
    #[must_use]
    pub fn lookup_key(&self) -> &CanonicalLookupKey {
        &self.lookup_key
    }

    /// Returns the declared source name, when the declaration has one.
    #[must_use]
    pub fn declared_name(&self) -> Option<&str> {
        self.declared_name.as_deref()
    }

    /// Returns the declaration's closed kind.
    #[must_use]
    pub const fn kind(&self) -> CanonicalDeclarationKind {
        self.identity.kind
    }

    /// Returns the declaration's namespace bucket.
    #[must_use]
    pub const fn namespace(&self) -> CanonicalNamespace {
        self.lookup_key.namespace
    }

    /// Returns the untyped expanded-surface definition, when applicable.
    #[must_use]
    pub fn raw_definition(&self) -> Option<&Definition> {
        self.raw_definition.as_ref()
    }

    /// Derives the raw callable body from the single retained definition.
    #[must_use]
    pub fn callable_body(&self) -> Option<&Expr> {
        match self.raw_definition.as_ref()? {
            Definition::Function(definition) => Some(&definition.body),
            Definition::Handler(definition) => Some(&definition.body),
            Definition::Notation(_)
            | Definition::Macro(_)
            | Definition::Capability(_)
            | Definition::ResourceType(_)
            | Definition::Type(_)
            | Definition::Newtype(_)
            | Definition::EffectAlias(_)
            | Definition::EffectGroup(_)
            | Definition::DataKind(_)
            | Definition::TypeFn(_)
            | Definition::PropositionPredicate(_)
            | Definition::Policy(_)
            | Definition::Role(_)
            | Definition::Interface(_)
            | Definition::Impl(_)
            | Definition::BuiltinFn(_)
            | Definition::SealedDomain(_)
            | Definition::Law(_)
            | Definition::Proof(_) => None,
        }
    }
}

/// Import-facing name-only facts for one declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalProvisionalNameEntry {
    identity: CanonicalDeclarationIdentity,
    lookup_name: Box<str>,
    lookup_key: CanonicalLookupKey,
    namespace: CanonicalNamespace,
    visibility: Visibility,
    exportable: bool,
    origin_anchor: Span,
    source_ordinal: usize,
}

impl CanonicalProvisionalNameEntry {
    /// Returns the stable declaration identity.
    #[must_use]
    pub fn identity(&self) -> &CanonicalDeclarationIdentity {
        &self.identity
    }

    /// Returns the source-visible lookup spelling.
    #[must_use]
    pub fn lookup_name(&self) -> &str {
        &self.lookup_name
    }

    /// Returns the namespace-qualified lookup key.
    #[must_use]
    pub fn lookup_key(&self) -> &CanonicalLookupKey {
        &self.lookup_key
    }

    /// Returns the namespace bucket.
    #[must_use]
    pub fn namespace(&self) -> CanonicalNamespace {
        self.namespace
    }

    /// Returns the declaration's retained source visibility.
    #[must_use]
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    /// Reports whether collection marked this name as provisionally exportable.
    #[must_use]
    pub fn is_exportable(&self) -> bool {
        self.exportable
    }

    /// Returns the declaration's source or generated origin anchor.
    #[must_use]
    pub fn origin_anchor(&self) -> Span {
        self.origin_anchor
    }

    /// Returns the declaration's source-order ordinal within its module.
    #[must_use]
    pub fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }
}

/// Checker-internal snapshot for one canonical module.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalCollectedModuleSnapshot {
    entries: Box<[CanonicalCollectedEntry]>,
    expansion_origins: Box<[ExpandedSurfaceOrigin]>,
    hygiene: Box<[IdentifierHygieneMetadata]>,
}

impl CanonicalCollectedModuleSnapshot {
    /// Iterates over internal entries in retained source order.
    pub fn entries(&self) -> impl Iterator<Item = &CanonicalCollectedEntry> {
        self.entries.iter()
    }

    /// Returns every module-level surface expansion origin sidecar.
    #[must_use]
    pub fn expansion_origins(&self) -> &[ExpandedSurfaceOrigin] {
        &self.expansion_origins
    }

    /// Returns every module-level identifier hygiene sidecar.
    #[must_use]
    pub fn hygiene(&self) -> &[IdentifierHygieneMetadata] {
        &self.hygiene
    }
}

/// Import-facing provisional name-only view for one canonical module.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalProvisionalNameView {
    entries: Box<[CanonicalProvisionalNameEntry]>,
}

impl CanonicalProvisionalNameView {
    /// Iterates over provisional names in retained source order.
    pub fn entries(&self) -> impl Iterator<Item = &CanonicalProvisionalNameEntry> {
        self.entries.iter()
    }
}

/// Read-only paired view of one collected canonical module.
#[derive(Debug, Clone, Copy)]
pub struct CanonicalCollectedModuleRef<'a> {
    module_key: &'a ModuleKey,
    module: &'a CanonicalCollectedModule,
}

impl<'a> CanonicalCollectedModuleRef<'a> {
    /// Returns the canonical module identity.
    #[must_use]
    pub fn module_key(&self) -> &ModuleKey {
        self.module_key
    }

    /// Returns the checker-internal snapshot.
    #[must_use]
    pub const fn internal_snapshot(&self) -> &'a CanonicalCollectedModuleSnapshot {
        &self.module.internal_snapshot
    }

    /// Returns the import-facing name-only view.
    #[must_use]
    pub const fn provisional_name_view(&self) -> &'a CanonicalProvisionalNameView {
        &self.module.provisional_name_view
    }
}

#[derive(Debug, Clone, PartialEq)]
struct CanonicalCollectedModule {
    internal_snapshot: CanonicalCollectedModuleSnapshot,
    provisional_name_view: CanonicalProvisionalNameView,
}

/// Opaque atomic result containing paired internal and provisional views.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalModuleCollection {
    modules: BTreeMap<ModuleKey, CanonicalCollectedModule>,
}

impl CanonicalModuleCollection {
    /// Iterates over every collected module in canonical-key order.
    pub fn modules(&self) -> impl Iterator<Item = CanonicalCollectedModuleRef<'_>> {
        self.modules
            .iter()
            .map(|(module_key, module)| CanonicalCollectedModuleRef { module_key, module })
    }

    /// Returns one paired module view by canonical identity.
    #[must_use]
    pub fn module(&self, module_key: &ModuleKey) -> Option<CanonicalCollectedModuleRef<'_>> {
        self.modules
            .get_key_value(module_key)
            .map(|(module_key, module)| CanonicalCollectedModuleRef { module_key, module })
    }

    /// Returns one checker-internal module snapshot.
    #[must_use]
    pub fn internal_snapshot(
        &self,
        module_key: &ModuleKey,
    ) -> Option<&CanonicalCollectedModuleSnapshot> {
        self.modules
            .get(module_key)
            .map(|module| &module.internal_snapshot)
    }

    /// Returns one import-facing provisional name view.
    #[must_use]
    pub fn provisional_name_view(
        &self,
        module_key: &ModuleKey,
    ) -> Option<&CanonicalProvisionalNameView> {
        self.modules
            .get(module_key)
            .map(|module| &module.provisional_name_view)
    }
}

/// Stable category for a canonical module collection failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CanonicalModuleCollectionErrorKind {
    /// Removed capability syntax was encountered.
    RemovedCapabilitySyntax,
    /// Temporary fail-closed Task 4 boundary before Task 5 collection exists.
    CollectorNotImplemented,
}

/// Anchored failure produced before either collection view is published.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("canonical module collection failed for `{module_key}` at {declaration_span:?}: {kind:?}")]
pub struct CanonicalModuleCollectionError {
    kind: CanonicalModuleCollectionErrorKind,
    module_key: ModuleKey,
    declaration_name: Option<Box<str>>,
    declaration_span: Span,
}

impl CanonicalModuleCollectionError {
    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(&self) -> CanonicalModuleCollectionErrorKind {
        self.kind
    }

    /// Returns the canonical module in which collection failed.
    #[must_use]
    pub fn module_key(&self) -> &ModuleKey {
        &self.module_key
    }

    /// Returns the rejected declaration name, when applicable.
    #[must_use]
    pub fn declaration_name(&self) -> Option<&str> {
        self.declaration_name.as_deref()
    }

    /// Returns the exact source anchor for the failure.
    #[must_use]
    pub const fn declaration_span(&self) -> Span {
        self.declaration_span
    }
}

/// Validates the closed definition domain without exposing collection carriers.
///
/// # Errors
///
/// Returns a removed-capability error when any definition in the batch uses
/// removed `Capability` syntax. Validation completes before returning success,
/// so callers cannot observe partially staged output.
fn validate_definition_batch(
    module_key: &ModuleKey,
    definitions: &[Definition],
) -> Result<(), CanonicalModuleCollectionError> {
    for definition in definitions {
        classify_definition(module_key, definition)?;
    }
    Ok(())
}

/// Validates an expanded graph at the Task 4 carrier boundary.
///
/// Task 4 deliberately publishes no collection. Task 5 will replace the
/// fail-closed result after namespace and collision semantics are implemented.
///
/// # Errors
///
/// Returns a removed-capability error when the graph contains removed syntax;
/// otherwise returns `CollectorNotImplemented` without publishing partial views.
pub fn collect_canonical_expanded_module_graph(
    graph: &CanonicalExpandedModuleGraph,
) -> Result<CanonicalModuleCollection, CanonicalModuleCollectionError> {
    for module in graph.modules() {
        validate_definition_batch(module.key(), module.body().definitions())?;
    }

    let root_key = graph.parsed_graph().root_key().clone();
    let declaration_span = graph
        .module(&root_key)
        .map_or_else(Span::default, |module| module.body().span());
    Err(CanonicalModuleCollectionError {
        kind: CanonicalModuleCollectionErrorKind::CollectorNotImplemented,
        module_key: root_key,
        declaration_name: None,
        declaration_span,
    })
}

fn classify_definition(
    module_key: &ModuleKey,
    definition: &Definition,
) -> Result<CanonicalDeclarationKind, CanonicalModuleCollectionError> {
    let kind = match definition {
        Definition::Notation(_) => CanonicalDeclarationKind::Notation,
        Definition::Macro(_) => CanonicalDeclarationKind::Macro,
        Definition::Capability(definition) => {
            return Err(CanonicalModuleCollectionError {
                kind: CanonicalModuleCollectionErrorKind::RemovedCapabilitySyntax,
                module_key: module_key.clone(),
                declaration_name: Some(definition.name.as_ref().into()),
                declaration_span: definition.span,
            });
        }
        Definition::ResourceType(_) => CanonicalDeclarationKind::ResourceType,
        Definition::Type(_) => CanonicalDeclarationKind::Type,
        Definition::Newtype(_) => CanonicalDeclarationKind::Newtype,
        Definition::EffectAlias(_) => CanonicalDeclarationKind::EffectAlias,
        Definition::EffectGroup(_) => CanonicalDeclarationKind::EffectGroup,
        Definition::DataKind(_) => CanonicalDeclarationKind::DataKind,
        Definition::TypeFn(_) => CanonicalDeclarationKind::TypeFn,
        Definition::PropositionPredicate(_) => CanonicalDeclarationKind::PropositionPredicate,
        Definition::Policy(_) => CanonicalDeclarationKind::Policy,
        Definition::Role(_) => CanonicalDeclarationKind::Role,
        Definition::Interface(_) => CanonicalDeclarationKind::Interface,
        Definition::Impl(_) => CanonicalDeclarationKind::Impl,
        Definition::Function(_) => CanonicalDeclarationKind::Function,
        Definition::Handler(_) => CanonicalDeclarationKind::Handler,
        Definition::BuiltinFn(_) => CanonicalDeclarationKind::BuiltinFn,
        Definition::SealedDomain(_) => CanonicalDeclarationKind::SealedDomain,
        Definition::Law(_) => CanonicalDeclarationKind::Law,
        Definition::Proof(_) => CanonicalDeclarationKind::Proof,
    };
    Ok(kind)
}

#[cfg(test)]
mod tests;
