//! Private carriers for canonical two-tier module collection.
//!
//! TASK-2075 defines the closed declaration domain and read-only carrier
//! boundary, then collects each expanded graph atomically into paired internal
//! and provisional name-only views. Import binding, checked finalization, and
//! the later authority fence remain owned by downstream tasks.

use std::collections::{BTreeMap, BTreeSet};

use ash_core::module_graph::ModuleKey;
use ash_parser::surface::{
    Definition, ExpandedSurfaceOrigin, ExpansionId, Expr, IdentifierHygieneMetadata,
    NormalizedNotationPatternKey, NotationFixity, Type, TypeBody, Visibility,
    normalized_notation_pattern_key, render_normalized_notation_pattern_key,
};
use ash_parser::{CanonicalExpandedModuleGraph, Span, module::ModuleItem};
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
    notation: Option<CanonicalNotationLookupKey>,
}

/// Typed notation lookup identity retained without reparsing display strings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalNotationLookupKey {
    pattern: NormalizedNotationPatternKey,
    fixity: CanonicalNotationFixity,
}

impl CanonicalNotationLookupKey {
    /// Returns the normalized, span-free notation pattern.
    #[must_use]
    pub const fn pattern(&self) -> &NormalizedNotationPatternKey {
        &self.pattern
    }

    /// Returns the typed fixity identity.
    #[must_use]
    pub const fn fixity(&self) -> CanonicalNotationFixity {
        self.fixity
    }
}

/// Typed fixity component of a canonical notation lookup key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalNotationFixity {
    /// Prefix notation with optional precedence.
    Prefix { precedence: Option<u16> },
    /// Infix notation with typed associativity and required precedence.
    Infix {
        associativity: ash_parser::surface::NotationAssociativity,
        precedence: u16,
    },
    /// Suffix notation with optional precedence.
    Suffix { precedence: Option<u16> },
    /// Mixfix notation.
    Mixfix,
}

impl From<&NotationFixity> for CanonicalNotationFixity {
    fn from(fixity: &NotationFixity) -> Self {
        match fixity {
            NotationFixity::Prefix { precedence } => Self::Prefix {
                precedence: *precedence,
            },
            NotationFixity::Infix {
                associativity,
                precedence,
            } => Self::Infix {
                associativity: *associativity,
                precedence: *precedence,
            },
            NotationFixity::Suffix { precedence } => Self::Suffix {
                precedence: *precedence,
            },
            NotationFixity::Mixfix => Self::Mixfix,
        }
    }
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

    /// Returns the typed notation identity for notation keys.
    #[must_use]
    pub const fn notation_key(&self) -> Option<&CanonicalNotationLookupKey> {
        self.notation.as_ref()
    }
}

#[cfg(test)]
pub(crate) fn clone_lookup_key_with_namespace(
    key: &CanonicalLookupKey,
    namespace: CanonicalNamespace,
) -> CanonicalLookupKey {
    CanonicalLookupKey {
        namespace,
        visible_local_key: key.visible_local_key.clone(),
        notation: key.notation.clone(),
    }
}

/// Checker-internal raw facts for one collected declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct CanonicalCollectedEntry {
    identity: CanonicalDeclarationIdentity,
    lookup_key: CanonicalLookupKey,
    declared_name: Option<Box<str>>,
    raw_definition: Option<Definition>,
    source_anchor: Span,
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

    /// Returns the expanded-surface span that anchors this declaration or member.
    #[must_use]
    pub const fn source_anchor(&self) -> Span {
        self.source_anchor
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

    /// Revalidates this opaque collection against an independently rebuilt
    /// expanded graph.
    ///
    /// This check is non-authorizing: it only compares the collected facts
    /// against a fresh collection of the candidate graph. It does not refresh
    /// either view or bind/import any declaration.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalModuleCollectionErrorKind::SourceDrift`] when the
    /// rebuilt graph has a different module key set, declaration fact,
    /// expansion-origin sidecar, or hygiene sidecar.
    pub fn revalidate_against(
        &self,
        graph: &CanonicalExpandedModuleGraph,
    ) -> Result<(), CanonicalModuleCollectionError> {
        let candidate = collect_canonical_expanded_module_graph_once(graph)?;
        validate_collected_modules(&self.modules, &candidate.modules)
    }
}

/// Stable category for a canonical module collection failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CanonicalModuleCollectionErrorKind {
    /// Removed capability syntax was encountered.
    RemovedCapabilitySyntax,
    /// Two declarations collide in the same namespace and canonical parent.
    DuplicateLookupKey,
    /// Two implementations have the same full interface application.
    OverlappingImplementation,
    /// An impl interface cannot be assigned a defining canonical module.
    InterfaceIdentityUnavailable,
    /// Retained for compatibility with the Task 4 fail-closed checkpoint.
    CollectorNotImplemented,
    /// The expanded graph no longer matches facts staged for collection.
    SourceDrift,
}

/// Canonical collection rule violated by a namespace/coherence failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CanonicalCollectionRule {
    /// A lookup key was declared twice in one namespace and canonical parent.
    DuplicateLookupKey,
    /// A full interface application overlaps an earlier implementation.
    ImplOverlap,
    /// An impl names no interface in its lexical canonical-module ancestry.
    InterfaceIdentityUnavailable,
    /// A rebuilt expanded graph differs from staged source facts.
    SourceDrift,
}

/// Anchored failure produced before either collection view is published.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("canonical module collection failed for `{module_key}` at {declaration_span:?}: {kind:?}")]
pub struct CanonicalModuleCollectionError {
    kind: CanonicalModuleCollectionErrorKind,
    rule: Option<CanonicalCollectionRule>,
    namespace: Option<CanonicalNamespace>,
    canonical_parent: Option<Box<CanonicalDeclarationIdentity>>,
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

    /// Returns the violated collision/coherence rule, when applicable.
    #[must_use]
    pub const fn rule(&self) -> Option<CanonicalCollectionRule> {
        self.rule
    }

    /// Returns the namespace in which collection failed, when applicable.
    #[must_use]
    pub const fn namespace(&self) -> Option<CanonicalNamespace> {
        self.namespace
    }

    /// Returns the canonical parent collision scope, when applicable.
    #[must_use]
    pub fn canonical_parent(&self) -> Option<&CanonicalDeclarationIdentity> {
        self.canonical_parent.as_deref()
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

/// Collects an expanded graph into paired internal and provisional name-only
/// module views.
///
/// Collection stages every module and graph-wide implementation-coherence
/// fact before returning the paired map. Any validation, collision, or
/// coherence failure therefore publishes no partial collection.
///
/// # Errors
///
/// Returns a typed error when the graph contains removed syntax, an
/// intra-namespace collision, an unresolved interface identity without an
/// import handoff, or an overlapping implementation head. An implementation
/// whose interface is named through a module import is retained in the
/// internal snapshot and deferred to checked finalization, where parsed
/// binding identities are available; it never enters the provisional name
/// view as an interface authority.
pub fn collect_canonical_expanded_module_graph(
    graph: &CanonicalExpandedModuleGraph,
) -> Result<CanonicalModuleCollection, CanonicalModuleCollectionError> {
    let staged = collect_canonical_expanded_module_graph_once(graph)?;
    let revalidated = collect_canonical_expanded_module_graph_once(graph)?;
    validate_collected_modules(&staged.modules, &revalidated.modules)?;
    Ok(staged)
}

fn collect_canonical_expanded_module_graph_once(
    graph: &CanonicalExpandedModuleGraph,
) -> Result<CanonicalModuleCollection, CanonicalModuleCollectionError> {
    let mut modules = BTreeMap::new();
    let mut impl_heads = Vec::new();
    let interface_definitions = graph
        .modules()
        .flat_map(|module| {
            module
                .body()
                .definitions()
                .iter()
                .filter_map(|definition| match definition {
                    Definition::Interface(interface) => Some((
                        module.key().clone(),
                        Box::<str>::from(interface.name.as_ref()),
                    )),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>();
    for module in graph.modules() {
        validate_definition_batch(module.key(), module.body().definitions())?;
        let collected = collect_module(
            module.key(),
            module.body().items(),
            module.origins(),
            module.hygiene(),
            &mut impl_heads,
            &interface_definitions,
        )?;
        modules.insert(module.key().clone(), collected);
    }
    Ok(CanonicalModuleCollection { modules })
}

fn validate_collected_modules(
    expected: &BTreeMap<ModuleKey, CanonicalCollectedModule>,
    actual: &BTreeMap<ModuleKey, CanonicalCollectedModule>,
) -> Result<(), CanonicalModuleCollectionError> {
    let keys = expected
        .keys()
        .chain(actual.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for module_key in keys {
        let Some(expected_module) = expected.get(&module_key) else {
            let actual_module = actual
                .get(&module_key)
                .expect("actual key came from the union");
            let (declaration_name, declaration_span) =
                collected_module_drift_anchor(None, Some(actual_module));
            return Err(source_drift_error(
                &module_key,
                declaration_name,
                declaration_span,
            ));
        };
        let Some(actual_module) = actual.get(&module_key) else {
            let (declaration_name, declaration_span) =
                collected_module_drift_anchor(Some(expected_module), None);
            return Err(source_drift_error(
                &module_key,
                declaration_name,
                declaration_span,
            ));
        };
        if expected_module == actual_module {
            continue;
        }
        let (declaration_name, declaration_span) =
            collected_module_drift_anchor(Some(expected_module), Some(actual_module));
        return Err(source_drift_error(
            &module_key,
            declaration_name,
            declaration_span,
        ));
    }
    Ok(())
}

fn collected_module_drift_anchor(
    expected: Option<&CanonicalCollectedModule>,
    actual: Option<&CanonicalCollectedModule>,
) -> (Option<Box<str>>, Span) {
    let expected_snapshot = expected.map(|module| &module.internal_snapshot);
    let actual_snapshot = actual.map(|module| &module.internal_snapshot);
    let expected_entries = expected_snapshot.map_or(&[][..], |snapshot| &snapshot.entries);
    let actual_entries = actual_snapshot.map_or(&[][..], |snapshot| &snapshot.entries);
    let entry_count = expected_entries.len().max(actual_entries.len());
    for index in 0..entry_count {
        if expected_entries.get(index) != actual_entries.get(index)
            && let Some(entry) = actual_entries
                .get(index)
                .or_else(|| expected_entries.get(index))
        {
            return (entry.declared_name.clone(), entry.source_anchor);
        }
    }

    let expected_names = expected.map_or(&[][..], |module| &module.provisional_name_view.entries);
    let actual_names = actual.map_or(&[][..], |module| &module.provisional_name_view.entries);
    let name_count = expected_names.len().max(actual_names.len());
    for index in 0..name_count {
        if expected_names.get(index) != actual_names.get(index)
            && let Some(entry) = actual_names
                .get(index)
                .or_else(|| expected_names.get(index))
        {
            return (Some(entry.lookup_name.clone()), entry.origin_anchor);
        }
    }

    let expected_origins =
        expected_snapshot.map_or(&[][..], |snapshot| &snapshot.expansion_origins);
    let actual_origins = actual_snapshot.map_or(&[][..], |snapshot| &snapshot.expansion_origins);
    let origin_count = expected_origins.len().max(actual_origins.len());
    for index in 0..origin_count {
        if expected_origins.get(index) != actual_origins.get(index)
            && let Some(origin) = actual_origins
                .get(index)
                .or_else(|| expected_origins.get(index))
        {
            return (None, origin.generated_span);
        }
    }

    let expected_hygiene = expected_snapshot.map_or(&[][..], |snapshot| &snapshot.hygiene);
    let actual_hygiene = actual_snapshot.map_or(&[][..], |snapshot| &snapshot.hygiene);
    let hygiene_count = expected_hygiene.len().max(actual_hygiene.len());
    for index in 0..hygiene_count {
        if expected_hygiene.get(index) != actual_hygiene.get(index)
            && let Some(metadata) = actual_hygiene
                .get(index)
                .or_else(|| expected_hygiene.get(index))
        {
            return (Some(metadata.name.as_ref().into()), metadata.span);
        }
    }

    expected_entries
        .first()
        .or_else(|| actual_entries.first())
        .map_or((None, Span::default()), |entry| {
            (entry.declared_name.clone(), entry.source_anchor)
        })
}

fn collect_module(
    module_key: &ModuleKey,
    items: &[ModuleItem],
    origins: &[ExpandedSurfaceOrigin],
    hygiene: &[IdentifierHygieneMetadata],
    impl_heads: &mut Vec<CanonicalImplHead>,
    interface_definitions: &BTreeSet<(ModuleKey, Box<str>)>,
) -> Result<CanonicalCollectedModule, CanonicalModuleCollectionError> {
    let mut entries = Vec::new();
    let mut names = Vec::new();
    let mut collision_keys =
        Vec::<(CanonicalLookupKey, Option<CanonicalDeclarationIdentity>)>::new();
    let definitions = items
        .iter()
        .filter_map(|item| match item {
            ModuleItem::Definition(definition) => Some(definition),
            ModuleItem::Use(_) | ModuleItem::ModuleDecl(_) => None,
        })
        .collect::<Vec<_>>();
    let has_imports = items.iter().any(|item| matches!(item, ModuleItem::Use(_)));

    for (source_ordinal, item) in items.iter().enumerate() {
        match item {
            ModuleItem::Use(_) => {}
            ModuleItem::ModuleDecl(declaration) => {
                push_entry(
                    module_key,
                    CanonicalDeclarationKind::ModuleDecl,
                    declaration.name.as_ref(),
                    &declaration.visibility,
                    declaration.span,
                    source_ordinal,
                    origins,
                    None,
                    None,
                    &mut entries,
                    &mut names,
                    &mut collision_keys,
                )?;
            }
            ModuleItem::Definition(definition) => {
                collect_definition(
                    module_key,
                    definition,
                    &definitions,
                    source_ordinal,
                    origins,
                    impl_heads,
                    interface_definitions,
                    has_imports,
                    &mut entries,
                    &mut names,
                    &mut collision_keys,
                )?;
            }
        }
    }

    Ok(CanonicalCollectedModule {
        internal_snapshot: CanonicalCollectedModuleSnapshot {
            entries: entries.into_boxed_slice(),
            expansion_origins: origins.into(),
            hygiene: hygiene.into(),
        },
        provisional_name_view: CanonicalProvisionalNameView {
            entries: names.into_boxed_slice(),
        },
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalInterfaceIdentity {
    module_key: ModuleKey,
    local_name: Box<str>,
}

#[derive(Debug, Clone)]
struct CanonicalImplHead {
    interface: CanonicalInterfaceIdentity,
    arguments: Box<[ImplType<usize>]>,
}

impl CanonicalImplHead {
    fn from_definition(
        module_key: &ModuleKey,
        definition: &ash_parser::surface::ImplDef,
        interface_definitions: &BTreeSet<(ModuleKey, Box<str>)>,
    ) -> Option<Self> {
        let variables = definition
            .type_params
            .iter()
            .map(|parameter| parameter.name.as_ref())
            .collect::<Vec<_>>();
        let mut defining_module = module_key.clone();
        loop {
            if interface_definitions.contains(&(
                defining_module.clone(),
                definition.interface.as_ref().into(),
            )) {
                break;
            }
            let parent = defining_module.parent()?;
            defining_module = parent;
        }
        Some(Self {
            interface: CanonicalInterfaceIdentity {
                module_key: defining_module,
                local_name: definition.interface.as_ref().into(),
            },
            arguments: definition
                .type_args
                .iter()
                .map(|argument| canonical_impl_type(argument, &variables))
                .collect(),
        })
    }

    fn overlaps(&self, other: &Self) -> bool {
        if self.interface != other.interface || self.arguments.len() != other.arguments.len() {
            return false;
        }
        let left = self
            .arguments
            .iter()
            .map(|argument| instantiate_impl_type(argument, 0))
            .collect::<Vec<_>>();
        let right = other
            .arguments
            .iter()
            .map(|argument| instantiate_impl_type(argument, 1))
            .collect::<Vec<_>>();
        let mut substitutions = BTreeMap::new();
        left.iter()
            .zip(&right)
            .all(|(left, right)| unify_impl_types(left, right, &mut substitutions))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ImplType<V> {
    Variable(V),
    Name(Box<str>),
    Hole,
    List(Box<Self>),
    Tuple(Box<[Self]>),
    Record(Box<[(Box<str>, Self)]>),
    Capability(Box<str>),
    Constructor {
        name: Box<str>,
        args: Box<[Self]>,
    },
    Associated {
        base: Box<Self>,
        name: Box<str>,
    },
    AssociatedFamilyProjection {
        interface: Box<str>,
        args: Box<[Self]>,
        member: Box<str>,
    },
    Function {
        params: Box<[Self]>,
        row: Option<ImplRow<V>>,
        result: Box<Self>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ImplRow<V> {
    items: Box<[ImplRowItem<V>]>,
    open: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ImplRowItem<V> {
    Operation {
        path: Box<[Box<str>]>,
        separator: Option<u8>,
    },
    Resource {
        path: Box<[Box<str>]>,
        mode: Option<Box<str>>,
    },
    Role {
        path: Box<[Box<str>]>,
    },
    Policy {
        path: Box<[Box<str>]>,
    },
    Channel {
        mode: Option<Box<str>>,
        path: Box<[Box<str>]>,
        payload: Option<ImplType<V>>,
    },
    Process {
        keyword: Box<str>,
        operation: Option<Box<str>>,
    },
    Fail {
        path: Option<Box<[Box<str>]>>,
    },
    Evidence {
        path: Box<[Box<str>]>,
    },
    Group {
        path: Box<[Box<str>]>,
    },
}

fn canonical_impl_type(ty: &Type, variables: &[&str]) -> ImplType<usize> {
    match ty {
        Type::Name(name) => variables
            .iter()
            .position(|variable| *variable == name.as_ref())
            .map_or_else(|| ImplType::Name(name.as_ref().into()), ImplType::Variable),
        Type::Hole { .. } => ImplType::Hole,
        Type::List(element) => ImplType::List(Box::new(canonical_impl_type(element, variables))),
        Type::Tuple(elements) => ImplType::Tuple(
            elements
                .iter()
                .map(|element| canonical_impl_type(element, variables))
                .collect(),
        ),
        Type::Record(fields) => {
            let mut fields: Vec<(Box<str>, ImplType<usize>)> = fields
                .iter()
                .map(|(name, ty)| (name.as_ref().into(), canonical_impl_type(ty, variables)))
                .collect::<Vec<_>>();
            fields.sort_by(|left, right| left.0.cmp(&right.0));
            ImplType::Record(fields.into_boxed_slice())
        }
        Type::Capability(name) => ImplType::Capability(name.as_ref().into()),
        Type::Constructor { name, args } => ImplType::Constructor {
            name: name.as_ref().into(),
            args: args
                .iter()
                .map(|argument| canonical_impl_type(argument, variables))
                .collect(),
        },
        Type::Associated { base, name } => ImplType::Associated {
            base: Box::new(canonical_impl_type(base, variables)),
            name: name.as_ref().into(),
        },
        Type::AssociatedFamilyProjection {
            interface,
            args,
            member,
            ..
        } => ImplType::AssociatedFamilyProjection {
            interface: interface.as_ref().into(),
            args: args
                .iter()
                .map(|argument| canonical_impl_type(argument, variables))
                .collect(),
            member: member.as_ref().into(),
        },
        Type::Fn(params, row, result) => ImplType::Function {
            params: params
                .iter()
                .map(|parameter| canonical_impl_type(parameter, variables))
                .collect(),
            row: row.as_ref().map(|row| canonical_impl_row(row, variables)),
            result: Box::new(canonical_impl_type(result, variables)),
        },
    }
}

fn canonical_impl_row(
    row: &ash_parser::surface::ComputationRow,
    variables: &[&str],
) -> ImplRow<usize> {
    use ash_parser::surface::ComputationRowItem;

    let open = row.items.iter().any(|item| {
        matches!(
            item,
            ComputationRowItem::WholeRow { .. } | ComputationRowItem::Tail { .. }
        )
    });
    let mut items = row
        .items
        .iter()
        .filter_map(|item| match item {
            ComputationRowItem::Operation {
                path, separator, ..
            } => Some(ImplRowItem::Operation {
                path: canonical_name_path(path),
                separator: separator.map(|separator| match separator {
                    ash_parser::surface::RowPathSeparator::Dot => 0,
                    ash_parser::surface::RowPathSeparator::DoubleColon => 1,
                }),
            }),
            ComputationRowItem::WholeRow { .. } | ComputationRowItem::Tail { .. } => None,
            ComputationRowItem::Resource { path, mode, .. } => Some(ImplRowItem::Resource {
                path: canonical_name_path(path),
                mode: mode.as_ref().map(|name| name.as_ref().into()),
            }),
            ComputationRowItem::Role { path, .. } => Some(ImplRowItem::Role {
                path: canonical_name_path(path),
            }),
            ComputationRowItem::Policy { path, .. } => Some(ImplRowItem::Policy {
                path: canonical_name_path(path),
            }),
            ComputationRowItem::Channel {
                mode,
                path,
                payload,
                ..
            } => Some(ImplRowItem::Channel {
                mode: mode.as_ref().map(|name| name.as_ref().into()),
                path: canonical_name_path(path),
                payload: payload
                    .as_ref()
                    .map(|payload| canonical_impl_type(payload, variables)),
            }),
            ComputationRowItem::Process {
                keyword, operation, ..
            } => Some(ImplRowItem::Process {
                keyword: keyword.as_ref().into(),
                operation: operation.as_ref().map(|name| name.as_ref().into()),
            }),
            ComputationRowItem::Fail { path, .. } => Some(ImplRowItem::Fail {
                path: path.as_ref().map(|path| canonical_name_path(path)),
            }),
            ComputationRowItem::Evidence { path, .. } => Some(ImplRowItem::Evidence {
                path: canonical_name_path(path),
            }),
            ComputationRowItem::Group { path, .. } => Some(ImplRowItem::Group {
                path: canonical_name_path(path),
            }),
        })
        .collect::<Vec<_>>();
    items.sort_unstable();
    items.dedup();
    ImplRow {
        items: items.into_boxed_slice(),
        open,
    }
}

fn canonical_name_path(path: &[ash_parser::surface::Name]) -> Box<[Box<str>]> {
    path.iter().map(|name| name.as_ref().into()).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ImplVariable {
    side: u8,
    index: usize,
}

fn instantiate_impl_type(ty: &ImplType<usize>, side: u8) -> ImplType<ImplVariable> {
    map_impl_type_variables(ty, &|index| ImplVariable { side, index })
}

fn map_impl_type_variables<A: Copy, B: Copy>(
    ty: &ImplType<A>,
    map: &impl Fn(A) -> B,
) -> ImplType<B> {
    match ty {
        ImplType::Variable(variable) => ImplType::Variable(map(*variable)),
        ImplType::Name(name) => ImplType::Name(name.clone()),
        ImplType::Hole => ImplType::Hole,
        ImplType::List(element) => ImplType::List(Box::new(map_impl_type_variables(element, map))),
        ImplType::Tuple(elements) => ImplType::Tuple(
            elements
                .iter()
                .map(|element| map_impl_type_variables(element, map))
                .collect(),
        ),
        ImplType::Record(fields) => ImplType::Record(
            fields
                .iter()
                .map(|(name, ty)| (name.clone(), map_impl_type_variables(ty, map)))
                .collect(),
        ),
        ImplType::Capability(name) => ImplType::Capability(name.clone()),
        ImplType::Constructor { name, args } => ImplType::Constructor {
            name: name.clone(),
            args: args
                .iter()
                .map(|argument| map_impl_type_variables(argument, map))
                .collect(),
        },
        ImplType::Associated { base, name } => ImplType::Associated {
            base: Box::new(map_impl_type_variables(base, map)),
            name: name.clone(),
        },
        ImplType::AssociatedFamilyProjection {
            interface,
            args,
            member,
        } => ImplType::AssociatedFamilyProjection {
            interface: interface.clone(),
            args: args
                .iter()
                .map(|argument| map_impl_type_variables(argument, map))
                .collect(),
            member: member.clone(),
        },
        ImplType::Function {
            params,
            row,
            result,
        } => ImplType::Function {
            params: params
                .iter()
                .map(|parameter| map_impl_type_variables(parameter, map))
                .collect(),
            row: row.as_ref().map(|row| ImplRow {
                items: row
                    .items
                    .iter()
                    .map(|item| map_impl_row_item_variables(item, map))
                    .collect(),
                open: row.open,
            }),
            result: Box::new(map_impl_type_variables(result, map)),
        },
    }
}

fn map_impl_row_item_variables<A: Copy, B: Copy>(
    item: &ImplRowItem<A>,
    map: &impl Fn(A) -> B,
) -> ImplRowItem<B> {
    match item {
        ImplRowItem::Operation { path, separator } => ImplRowItem::Operation {
            path: path.clone(),
            separator: *separator,
        },
        ImplRowItem::Resource { path, mode } => ImplRowItem::Resource {
            path: path.clone(),
            mode: mode.clone(),
        },
        ImplRowItem::Role { path } => ImplRowItem::Role { path: path.clone() },
        ImplRowItem::Policy { path } => ImplRowItem::Policy { path: path.clone() },
        ImplRowItem::Channel {
            mode,
            path,
            payload,
        } => ImplRowItem::Channel {
            mode: mode.clone(),
            path: path.clone(),
            payload: payload
                .as_ref()
                .map(|payload| map_impl_type_variables(payload, map)),
        },
        ImplRowItem::Process { keyword, operation } => ImplRowItem::Process {
            keyword: keyword.clone(),
            operation: operation.clone(),
        },
        ImplRowItem::Fail { path } => ImplRowItem::Fail { path: path.clone() },
        ImplRowItem::Evidence { path } => ImplRowItem::Evidence { path: path.clone() },
        ImplRowItem::Group { path } => ImplRowItem::Group { path: path.clone() },
    }
}

fn unify_impl_types(
    left: &ImplType<ImplVariable>,
    right: &ImplType<ImplVariable>,
    substitutions: &mut BTreeMap<ImplVariable, ImplType<ImplVariable>>,
) -> bool {
    if let ImplType::Variable(variable) = left {
        return bind_impl_variable(*variable, right, substitutions);
    }
    if let ImplType::Variable(variable) = right {
        return bind_impl_variable(*variable, left, substitutions);
    }
    match (left, right) {
        (ImplType::Name(left), ImplType::Name(right))
        | (ImplType::Capability(left), ImplType::Capability(right)) => left == right,
        (ImplType::Hole, ImplType::Hole) => true,
        (ImplType::List(left), ImplType::List(right)) => {
            unify_impl_types(left, right, substitutions)
        }
        (ImplType::Tuple(left), ImplType::Tuple(right)) => {
            unify_impl_type_slices(left, right, substitutions)
        }
        (ImplType::Record(left), ImplType::Record(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|((left_name, left), (right_name, right))| {
                        left_name == right_name && unify_impl_types(left, right, substitutions)
                    })
        }
        (
            ImplType::Constructor {
                name: left_name,
                args: left,
            },
            ImplType::Constructor {
                name: right_name,
                args: right,
            },
        ) => left_name == right_name && unify_impl_type_slices(left, right, substitutions),
        (
            ImplType::Associated {
                base: left,
                name: left_name,
            },
            ImplType::Associated {
                base: right,
                name: right_name,
            },
        ) => left_name == right_name && unify_impl_types(left, right, substitutions),
        (
            ImplType::AssociatedFamilyProjection {
                interface: left_interface,
                args: left,
                member: left_member,
            },
            ImplType::AssociatedFamilyProjection {
                interface: right_interface,
                args: right,
                member: right_member,
            },
        ) => {
            left_interface == right_interface
                && left_member == right_member
                && unify_impl_type_slices(left, right, substitutions)
        }
        (
            ImplType::Function {
                params: left_params,
                row: left_row,
                result: left_result,
            },
            ImplType::Function {
                params: right_params,
                row: right_row,
                result: right_result,
            },
        ) => {
            unify_impl_type_slices(left_params, right_params, substitutions)
                && unify_impl_rows(left_row.as_ref(), right_row.as_ref(), substitutions)
                && unify_impl_types(left_result, right_result, substitutions)
        }
        _ => false,
    }
}

fn unify_impl_type_slices(
    left: &[ImplType<ImplVariable>],
    right: &[ImplType<ImplVariable>],
    substitutions: &mut BTreeMap<ImplVariable, ImplType<ImplVariable>>,
) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| unify_impl_types(left, right, substitutions))
}

fn unify_impl_rows(
    left: Option<&ImplRow<ImplVariable>>,
    right: Option<&ImplRow<ImplVariable>>,
    substitutions: &mut BTreeMap<ImplVariable, ImplType<ImplVariable>>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (None, Some(right)) => right.open && right.items.is_empty(),
        (Some(left), None) => left.open && left.items.is_empty(),
        (Some(left), Some(right)) => match (left.open, right.open) {
            (false, false) => {
                row_subset_overlaps(&left.items, &right.items, substitutions)
                    && row_subset_overlaps(&right.items, &left.items, substitutions)
            }
            (true, false) => row_subset_overlaps(&left.items, &right.items, substitutions),
            (false, true) => row_subset_overlaps(&right.items, &left.items, substitutions),
            (true, true) => open_rows_are_compatible(&left.items, &right.items, substitutions),
        },
    }
}

fn row_subset_overlaps(
    required: &[ImplRowItem<ImplVariable>],
    available: &[ImplRowItem<ImplVariable>],
    substitutions: &mut BTreeMap<ImplVariable, ImplType<ImplVariable>>,
) -> bool {
    required.iter().all(|required| {
        available.iter().any(|available| {
            let mut trial = substitutions.clone();
            if unify_impl_row_items(required, available, &mut trial) {
                *substitutions = trial;
                true
            } else {
                false
            }
        })
    })
}

fn open_rows_are_compatible(
    left: &[ImplRowItem<ImplVariable>],
    right: &[ImplRowItem<ImplVariable>],
    substitutions: &mut BTreeMap<ImplVariable, ImplType<ImplVariable>>,
) -> bool {
    for left in left {
        for right in right {
            if same_impl_row_slot(left, right) {
                let mut trial = substitutions.clone();
                if !unify_impl_row_items(left, right, &mut trial) {
                    return false;
                }
                *substitutions = trial;
            }
        }
    }
    true
}

fn same_impl_row_slot(left: &ImplRowItem<ImplVariable>, right: &ImplRowItem<ImplVariable>) -> bool {
    match (left, right) {
        (
            ImplRowItem::Channel {
                mode: left_mode,
                path: left_path,
                ..
            },
            ImplRowItem::Channel {
                mode: right_mode,
                path: right_path,
                ..
            },
        ) => left_mode == right_mode && left_path == right_path,
        _ => left == right,
    }
}

fn unify_impl_row_items(
    left: &ImplRowItem<ImplVariable>,
    right: &ImplRowItem<ImplVariable>,
    substitutions: &mut BTreeMap<ImplVariable, ImplType<ImplVariable>>,
) -> bool {
    match (left, right) {
        (
            ImplRowItem::Channel {
                mode: left_mode,
                path: left_path,
                payload: left_payload,
            },
            ImplRowItem::Channel {
                mode: right_mode,
                path: right_path,
                payload: right_payload,
            },
        ) => {
            left_mode == right_mode
                && left_path == right_path
                && match (left_payload, right_payload) {
                    (None, None) => true,
                    (Some(left), Some(right)) => unify_impl_types(left, right, substitutions),
                    _ => false,
                }
        }
        _ => left == right,
    }
}

fn bind_impl_variable(
    variable: ImplVariable,
    term: &ImplType<ImplVariable>,
    substitutions: &mut BTreeMap<ImplVariable, ImplType<ImplVariable>>,
) -> bool {
    if let Some(bound) = substitutions.get(&variable).cloned() {
        return unify_impl_types(&bound, term, substitutions);
    }
    if term == &ImplType::Variable(variable) {
        return true;
    }
    if impl_variable_occurs(variable, term, substitutions) {
        return false;
    }
    substitutions.insert(variable, term.clone());
    true
}

fn impl_variable_occurs(
    variable: ImplVariable,
    term: &ImplType<ImplVariable>,
    substitutions: &BTreeMap<ImplVariable, ImplType<ImplVariable>>,
) -> bool {
    match term {
        ImplType::Variable(candidate) => {
            *candidate == variable
                || substitutions
                    .get(candidate)
                    .is_some_and(|bound| impl_variable_occurs(variable, bound, substitutions))
        }
        ImplType::List(element) => impl_variable_occurs(variable, element, substitutions),
        ImplType::Tuple(elements) => elements
            .iter()
            .any(|element| impl_variable_occurs(variable, element, substitutions)),
        ImplType::Record(fields) => fields
            .iter()
            .any(|(_, ty)| impl_variable_occurs(variable, ty, substitutions)),
        ImplType::Constructor { args, .. } | ImplType::AssociatedFamilyProjection { args, .. } => {
            args.iter()
                .any(|argument| impl_variable_occurs(variable, argument, substitutions))
        }
        ImplType::Associated { base, .. } => impl_variable_occurs(variable, base, substitutions),
        ImplType::Function {
            params,
            row,
            result,
        } => {
            params
                .iter()
                .any(|parameter| impl_variable_occurs(variable, parameter, substitutions))
                || row.as_ref().is_some_and(|row| {
                    row.items.iter().any(|item| match item {
                        ImplRowItem::Channel {
                            payload: Some(payload),
                            ..
                        } => impl_variable_occurs(variable, payload, substitutions),
                        _ => false,
                    })
                })
                || impl_variable_occurs(variable, result, substitutions)
        }
        ImplType::Name(_) | ImplType::Hole | ImplType::Capability(_) => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_definition(
    module_key: &ModuleKey,
    definition: &Definition,
    module_definitions: &[&Definition],
    source_ordinal: usize,
    origins: &[ExpandedSurfaceOrigin],
    impl_heads: &mut Vec<CanonicalImplHead>,
    interface_definitions: &BTreeSet<(ModuleKey, Box<str>)>,
    has_imports: bool,
    entries: &mut Vec<CanonicalCollectedEntry>,
    names: &mut Vec<CanonicalProvisionalNameEntry>,
    collision_keys: &mut Vec<(CanonicalLookupKey, Option<CanonicalDeclarationIdentity>)>,
) -> Result<(), CanonicalModuleCollectionError> {
    let kind = classify_definition(module_key, definition)?;
    let (_, visibility, span) = definition_header(definition);

    if let Definition::Impl(implementation) = definition {
        // Collection owns graph-wide coherence only for lexical interface
        // identities. Imported interfaces are resolved by TASK-2072 and
        // validated by TASK-2073; retain the implementation but do not guess
        // an identity or publish a provisional authority entry here.
        if let Some(head) =
            CanonicalImplHead::from_definition(module_key, implementation, interface_definitions)
        {
            if impl_heads.iter().any(|prior| prior.overlaps(&head)) {
                return Err(collection_rule_error(
                    CanonicalModuleCollectionErrorKind::OverlappingImplementation,
                    CanonicalCollectionRule::ImplOverlap,
                    CanonicalNamespace::ImplementationRegistry,
                    module_key,
                    None,
                    implementation.interface.as_ref(),
                    implementation.span,
                ));
            }
            impl_heads.push(head);
        } else if !has_imports {
            return Err(collection_rule_error(
                CanonicalModuleCollectionErrorKind::InterfaceIdentityUnavailable,
                CanonicalCollectionRule::InterfaceIdentityUnavailable,
                CanonicalNamespace::ImplementationRegistry,
                module_key,
                None,
                implementation.interface.as_ref(),
                implementation.span,
            ));
        }
    }

    let lookup_name = definition_lookup_name(definition);
    let parent = push_entry(
        module_key,
        kind,
        &lookup_name,
        visibility,
        span,
        source_ordinal,
        origins,
        None,
        Some(definition.clone()),
        entries,
        names,
        collision_keys,
    )?;

    match definition {
        Definition::Type(type_definition) => match &type_definition.body {
            TypeBody::Enum(variants) => {
                for (member_ordinal, variant) in variants.iter().enumerate() {
                    push_entry_with_namespace(
                        module_key,
                        CanonicalDeclarationKind::Function,
                        CanonicalNamespace::ValueCallable,
                        variant.name.as_ref(),
                        visibility,
                        variant.span,
                        member_ordinal,
                        origins,
                        Some(parent.clone()),
                        Some(definition.clone()),
                        true,
                        entries,
                        names,
                        collision_keys,
                    )?;
                }
            }
            TypeBody::Struct(_) | TypeBody::Alias(_) => {}
        },
        Definition::Newtype(newtype) => {
            push_entry_with_namespace(
                module_key,
                CanonicalDeclarationKind::Function,
                CanonicalNamespace::ValueCallable,
                newtype.constructor.as_ref(),
                visibility,
                newtype.span,
                0,
                origins,
                Some(parent),
                Some(definition.clone()),
                true,
                entries,
                names,
                collision_keys,
            )?;
        }
        Definition::DataKind(data_kind) => {
            if let Some(source_type) =
                module_definitions
                    .iter()
                    .find_map(|candidate| match candidate {
                        Definition::Type(source) if source.name == data_kind.source_adt => {
                            Some(source)
                        }
                        _ => None,
                    })
                && let TypeBody::Enum(variants) = &source_type.body
            {
                for (member_ordinal, variant) in variants.iter().enumerate() {
                    push_entry_with_namespace(
                        module_key,
                        kind,
                        CanonicalNamespace::PromotedKind,
                        variant.name.as_ref(),
                        visibility,
                        variant.span,
                        member_ordinal,
                        origins,
                        Some(parent.clone()),
                        Some(definition.clone()),
                        true,
                        entries,
                        names,
                        collision_keys,
                    )?;
                }
            }
        }
        Definition::SealedDomain(domain) => {
            for (member_ordinal, constructor) in domain.constructors.iter().enumerate() {
                push_entry_with_namespace(
                    module_key,
                    kind,
                    CanonicalNamespace::TypeDomain,
                    constructor.name.as_ref(),
                    visibility,
                    constructor.span,
                    member_ordinal,
                    origins,
                    Some(parent.clone()),
                    Some(definition.clone()),
                    true,
                    entries,
                    names,
                    collision_keys,
                )?;
            }
        }
        Definition::Interface(interface) => {
            let inherited = Visibility::Inherited;
            for (member_ordinal, associated_type) in interface.associated_types.iter().enumerate() {
                push_entry_with_namespace(
                    module_key,
                    CanonicalDeclarationKind::Type,
                    CanonicalNamespace::TypeDomain,
                    associated_type.name.as_ref(),
                    &inherited,
                    associated_type.span,
                    member_ordinal,
                    origins,
                    Some(parent.clone()),
                    Some(definition.clone()),
                    true,
                    entries,
                    names,
                    collision_keys,
                )?;
            }
            for (member_ordinal, method) in interface.methods.iter().enumerate() {
                push_entry_with_namespace(
                    module_key,
                    CanonicalDeclarationKind::Function,
                    CanonicalNamespace::ValueCallable,
                    method.name.as_ref(),
                    &inherited,
                    method.span,
                    member_ordinal,
                    origins,
                    Some(parent.clone()),
                    Some(definition.clone()),
                    true,
                    entries,
                    names,
                    collision_keys,
                )?;
            }
            for (member_ordinal, law) in interface.laws.iter().enumerate() {
                push_entry_with_namespace(
                    module_key,
                    CanonicalDeclarationKind::Law,
                    CanonicalNamespace::Evidence,
                    law.name.as_ref(),
                    &law.visibility,
                    law.span,
                    member_ordinal,
                    origins,
                    Some(parent.clone()),
                    Some(definition.clone()),
                    true,
                    entries,
                    names,
                    collision_keys,
                )?;
            }
        }
        Definition::Impl(implementation) => {
            let inherited = Visibility::Inherited;
            for (member_ordinal, associated_type) in
                implementation.associated_type_bindings.iter().enumerate()
            {
                push_entry_with_namespace(
                    module_key,
                    CanonicalDeclarationKind::Type,
                    CanonicalNamespace::TypeDomain,
                    associated_type.name.as_ref(),
                    &inherited,
                    associated_type.span,
                    member_ordinal,
                    origins,
                    Some(parent.clone()),
                    Some(definition.clone()),
                    false,
                    entries,
                    names,
                    collision_keys,
                )?;
            }
            for (member_ordinal, method) in implementation.methods.iter().enumerate() {
                push_entry_with_namespace(
                    module_key,
                    CanonicalDeclarationKind::Function,
                    CanonicalNamespace::ValueCallable,
                    method.name.as_ref(),
                    &inherited,
                    method.span,
                    member_ordinal,
                    origins,
                    Some(parent.clone()),
                    Some(definition.clone()),
                    false,
                    entries,
                    names,
                    collision_keys,
                )?;
            }
            for (member_ordinal, handler) in implementation.handlers.iter().enumerate() {
                push_entry_with_namespace(
                    module_key,
                    CanonicalDeclarationKind::Handler,
                    CanonicalNamespace::ValueCallable,
                    handler.name.as_ref(),
                    &inherited,
                    handler.span,
                    member_ordinal,
                    origins,
                    Some(parent.clone()),
                    Some(definition.clone()),
                    false,
                    entries,
                    names,
                    collision_keys,
                )?;
            }
            for (member_ordinal, proof) in implementation.proofs.iter().enumerate() {
                push_entry_with_namespace(
                    module_key,
                    CanonicalDeclarationKind::Proof,
                    CanonicalNamespace::Evidence,
                    proof.name.as_ref(),
                    &proof.visibility,
                    proof.span,
                    member_ordinal,
                    origins,
                    Some(parent.clone()),
                    Some(definition.clone()),
                    false,
                    entries,
                    names,
                    collision_keys,
                )?;
            }
        }
        Definition::Notation(_)
        | Definition::Macro(_)
        | Definition::Capability(_)
        | Definition::ResourceType(_)
        | Definition::EffectAlias(_)
        | Definition::EffectGroup(_)
        | Definition::TypeFn(_)
        | Definition::PropositionPredicate(_)
        | Definition::Policy(_)
        | Definition::Role(_)
        | Definition::Function(_)
        | Definition::Handler(_)
        | Definition::BuiltinFn(_)
        | Definition::Law(_)
        | Definition::Proof(_) => {}
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn push_entry(
    module_key: &ModuleKey,
    kind: CanonicalDeclarationKind,
    lookup_name: &str,
    visibility: &Visibility,
    span: Span,
    source_ordinal: usize,
    origins: &[ExpandedSurfaceOrigin],
    canonical_parent: Option<CanonicalDeclarationIdentity>,
    raw_definition: Option<Definition>,
    entries: &mut Vec<CanonicalCollectedEntry>,
    names: &mut Vec<CanonicalProvisionalNameEntry>,
    collision_keys: &mut Vec<(CanonicalLookupKey, Option<CanonicalDeclarationIdentity>)>,
) -> Result<CanonicalDeclarationIdentity, CanonicalModuleCollectionError> {
    let CanonicalCollectionDisposition::Collect {
        namespace,
        publish_in_name_view,
    } = kind.collection_disposition()
    else {
        return Err(collection_error(
            CanonicalModuleCollectionErrorKind::RemovedCapabilitySyntax,
            module_key,
            Some(lookup_name),
            span,
        ));
    };
    push_entry_with_namespace(
        module_key,
        kind,
        namespace,
        lookup_name,
        visibility,
        span,
        source_ordinal,
        origins,
        canonical_parent,
        raw_definition,
        publish_in_name_view,
        entries,
        names,
        collision_keys,
    )
}

#[allow(clippy::too_many_arguments)]
fn push_entry_with_namespace(
    module_key: &ModuleKey,
    kind: CanonicalDeclarationKind,
    namespace: CanonicalNamespace,
    lookup_name: &str,
    visibility: &Visibility,
    span: Span,
    source_ordinal: usize,
    origins: &[ExpandedSurfaceOrigin],
    canonical_parent: Option<CanonicalDeclarationIdentity>,
    raw_definition: Option<Definition>,
    publish_in_name_view: bool,
    entries: &mut Vec<CanonicalCollectedEntry>,
    names: &mut Vec<CanonicalProvisionalNameEntry>,
    collision_keys: &mut Vec<(CanonicalLookupKey, Option<CanonicalDeclarationIdentity>)>,
) -> Result<CanonicalDeclarationIdentity, CanonicalModuleCollectionError> {
    let lookup_key = CanonicalLookupKey {
        namespace,
        visible_local_key: lookup_name.into(),
        notation: raw_definition.as_ref().and_then(|definition| {
            let Definition::Notation(declaration) = definition else {
                return None;
            };
            Some(CanonicalNotationLookupKey {
                pattern: normalized_notation_pattern_key(&declaration.pattern.parts),
                fixity: (&declaration.fixity).into(),
            })
        }),
    };
    if namespace != CanonicalNamespace::ImplementationRegistry {
        if collision_keys.iter().any(|(prior_key, prior_parent)| {
            prior_key == &lookup_key && prior_parent.as_ref() == canonical_parent.as_ref()
        }) {
            return Err(collection_rule_error(
                CanonicalModuleCollectionErrorKind::DuplicateLookupKey,
                CanonicalCollectionRule::DuplicateLookupKey,
                namespace,
                module_key,
                canonical_parent.as_ref(),
                lookup_name,
                span,
            ));
        }
        collision_keys.push((lookup_key.clone(), canonical_parent.clone()));
    }
    let origin_key = origins
        .iter()
        .find(|origin| origin.generated_span == span)
        .map_or(
            CanonicalDeclarationOriginKey::Source { source_ordinal },
            |origin| CanonicalDeclarationOriginKey::Expanded {
                expansion_id: origin.expansion_id,
                source_ordinal,
            },
        );
    let identity_module_key = if kind == CanonicalDeclarationKind::ModuleDecl {
        module_key.child(lookup_name).map_err(|_| {
            collection_error(
                CanonicalModuleCollectionErrorKind::CollectorNotImplemented,
                module_key,
                Some(lookup_name),
                span,
            )
        })?
    } else {
        module_key.clone()
    };
    let identity = CanonicalDeclarationIdentity {
        module_key: identity_module_key,
        kind,
        canonical_parent: canonical_parent.map(Box::new),
        origin_key,
    };
    entries.push(CanonicalCollectedEntry {
        identity: identity.clone(),
        lookup_key: lookup_key.clone(),
        declared_name: Some(lookup_name.into()),
        raw_definition,
        source_anchor: span,
    });
    if publish_in_name_view {
        names.push(CanonicalProvisionalNameEntry {
            identity: identity.clone(),
            lookup_name: lookup_name.into(),
            lookup_key,
            namespace,
            visibility: visibility.clone(),
            exportable: matches!(visibility, Visibility::Public),
            origin_anchor: span,
            source_ordinal,
        });
    }
    Ok(identity)
}

fn collection_error(
    kind: CanonicalModuleCollectionErrorKind,
    module_key: &ModuleKey,
    declaration_name: Option<&str>,
    declaration_span: Span,
) -> CanonicalModuleCollectionError {
    CanonicalModuleCollectionError {
        kind,
        rule: None,
        namespace: None,
        canonical_parent: None,
        module_key: module_key.clone(),
        declaration_name: declaration_name.map(Into::into),
        declaration_span,
    }
}

fn source_drift_error(
    module_key: &ModuleKey,
    declaration_name: Option<Box<str>>,
    declaration_span: Span,
) -> CanonicalModuleCollectionError {
    CanonicalModuleCollectionError {
        kind: CanonicalModuleCollectionErrorKind::SourceDrift,
        rule: Some(CanonicalCollectionRule::SourceDrift),
        namespace: None,
        canonical_parent: None,
        module_key: module_key.clone(),
        declaration_name,
        declaration_span,
    }
}

#[allow(clippy::too_many_arguments)]
fn collection_rule_error(
    kind: CanonicalModuleCollectionErrorKind,
    rule: CanonicalCollectionRule,
    namespace: CanonicalNamespace,
    module_key: &ModuleKey,
    canonical_parent: Option<&CanonicalDeclarationIdentity>,
    declaration_name: &str,
    declaration_span: Span,
) -> CanonicalModuleCollectionError {
    CanonicalModuleCollectionError {
        kind,
        rule: Some(rule),
        namespace: Some(namespace),
        canonical_parent: canonical_parent.cloned().map(Box::new),
        module_key: module_key.clone(),
        declaration_name: Some(declaration_name.into()),
        declaration_span,
    }
}

fn definition_lookup_name(definition: &Definition) -> Box<str> {
    match definition {
        Definition::Notation(declaration) => {
            let pattern = normalized_notation_pattern_key(&declaration.pattern.parts);
            render_notation_lookup_name(
                &render_normalized_notation_pattern_key(&pattern),
                &declaration.fixity,
            )
        }
        Definition::Impl(definition) => definition.interface.as_ref().into(),
        _ => definition_header(definition).0.into(),
    }
}

fn render_notation_lookup_name(pattern: &str, fixity: &NotationFixity) -> Box<str> {
    match fixity {
        NotationFixity::Prefix { precedence } => format!(
            "prefix:precedence: {}:{pattern}",
            precedence.map_or_else(|| "none".to_owned(), |value| value.to_string())
        )
        .into_boxed_str(),
        NotationFixity::Infix {
            associativity,
            precedence,
        } => {
            let associativity = match associativity {
                ash_parser::surface::NotationAssociativity::Left => "left",
                ash_parser::surface::NotationAssociativity::Right => "right",
                ash_parser::surface::NotationAssociativity::Nonassoc => "nonassoc",
            };
            format!("infix:{associativity}:precedence: {precedence}:{pattern}").into_boxed_str()
        }
        NotationFixity::Suffix { precedence } => format!(
            "suffix:precedence: {}:{pattern}",
            precedence.map_or_else(|| "none".to_owned(), |value| value.to_string())
        )
        .into_boxed_str(),
        NotationFixity::Mixfix => format!("mixfix:{pattern}").into_boxed_str(),
    }
}

fn definition_header(definition: &Definition) -> (&str, &Visibility, Span) {
    match definition {
        Definition::Notation(d) => (d.pattern.raw.as_ref(), &d.visibility, d.span),
        Definition::Macro(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::Capability(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::ResourceType(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::Type(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::Newtype(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::EffectAlias(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::EffectGroup(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::DataKind(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::TypeFn(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::PropositionPredicate(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::Policy(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::Role(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::Interface(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::Impl(d) => (d.interface.as_ref(), &d.visibility, d.span),
        Definition::Function(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::Handler(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::BuiltinFn(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::SealedDomain(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::Law(d) => (d.name.as_ref(), &d.visibility, d.span),
        Definition::Proof(d) => (d.name.as_ref(), &d.visibility, d.span),
    }
}

fn classify_definition(
    module_key: &ModuleKey,
    definition: &Definition,
) -> Result<CanonicalDeclarationKind, CanonicalModuleCollectionError> {
    let kind = match definition {
        Definition::Notation(_) => CanonicalDeclarationKind::Notation,
        Definition::Macro(_) => CanonicalDeclarationKind::Macro,
        Definition::Capability(definition) => {
            return Err(collection_error(
                CanonicalModuleCollectionErrorKind::RemovedCapabilitySyntax,
                module_key,
                Some(definition.name.as_ref()),
                definition.span,
            ));
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
