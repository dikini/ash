//! Core-owned module semantic summary carriers for ordinary type metadata.
//!
//! SPEC-057 defines this as an ordinary-type summary substrate. It intentionally
//! does not interpret future type-computation namespaces and does not replace the
//! Phase 108 workflow-summary carriers.
//!
//! SPEC-059 extends this with sealed type-level domain identities, marker
//! constructor identities, field metadata, and domain/constructor summary
//! carriers. Sealed-domain identities are distinct from ordinary `TypeDeclId` /
//! `ConstructorId` because marker constructors are type-level only.

use crate::ast::{Name, Span, TypeBody, TypeExpr, TypeVar, Visibility};
use crate::kind::Kind;
use crate::module_graph::{CrateId, ModuleId};
use crate::type_ir::{
    AssociatedFamilyHeadId, AssociatedFamilyProjection, AssociatedFamilyScheme, CanonicalTypeExpr,
    PropositionOutcome, TypeComputationHeadId, TypeFunctionEquation, TypeFunctionResultConstraint,
    TypeFunctionSourceAnchors, TypeProposition,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The only sanitizer wire schema understood for V7 effect-row closure
/// metadata.  Keep this core-owned so producers and consumers cannot drift.
pub const EFFECT_ROW_SANITIZER_SCHEMA_VERSION: u16 = 1;

/// Canonical identity plus diagnostic metadata for a resolved Ash module.
///
/// The numeric `ModuleId`/`CrateId` pair anchors graph identity. `path` and
/// `source` provide stable transport/debug metadata for summary consumers, but
/// they intentionally do not participate in equality or hashing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleIdentity {
    pub crate_id: Option<CrateId>,
    pub module_id: ModuleId,
    pub path: Vec<String>,
    pub source: ModuleSourceOrigin,
}

impl PartialEq for ModuleIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.crate_id == other.crate_id && self.module_id == other.module_id
    }
}

impl Eq for ModuleIdentity {}

impl std::hash::Hash for ModuleIdentity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.crate_id.hash(state);
        self.module_id.hash(state);
    }
}

impl ModuleIdentity {
    #[must_use]
    pub fn new(
        crate_id: Option<CrateId>,
        module_id: ModuleId,
        path: Vec<String>,
        source: ModuleSourceOrigin,
    ) -> Self {
        Self {
            crate_id,
            module_id,
            path,
            source,
        }
    }
}

/// Source origin for module/type summary diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleSourceOrigin {
    File(String),
    Inline { parent: ModuleId, offset: usize },
    Synthetic { reason: String },
}

/// Source origin for a summary fact. This is diagnostic metadata, not identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceOrigin {
    File(String),
    InlineModule {
        module: ModuleId,
        offset: usize,
    },
    ImportedSummary {
        module: Vec<String>,
        public_anchor: String,
    },
    Synthetic {
        reason: String,
    },
}

/// Diagnostic anchor for source/module context. Spans must not be used as ID inputs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceAnchor {
    pub origin: SourceOrigin,
    pub span: Option<Span>,
    pub label: String,
}

impl SourceAnchor {
    #[must_use]
    pub fn new(origin: SourceOrigin, span: Option<Span>, label: impl Into<String>) -> Self {
        Self {
            origin,
            span,
            label: label.into(),
        }
    }
}

/// Ordinary type declaration item kind participating in canonical identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeDeclItemKind {
    OrdinaryType,
}

/// Canonical ordinary type declaration identity.
///
/// Import aliases and re-export paths should point at this identity; they must
/// not construct new origin identities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeDeclId {
    pub module: ModuleIdentity,
    pub name: Name,
    pub item_kind: TypeDeclItemKind,
}

impl TypeDeclId {
    #[must_use]
    pub fn ordinary(module: ModuleIdentity, name: impl Into<Name>) -> Self {
        Self {
            module,
            name: name.into(),
            item_kind: TypeDeclItemKind::OrdinaryType,
        }
    }
}

/// Constructor/variant payload kind participating in constructor identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructorPayloadKind {
    Unit,
    Record,
    Tuple,
    Struct,
}

/// Canonical constructor/variant identity derived from parent type identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConstructorId {
    pub parent: TypeDeclId,
    pub name: Name,
    pub payload_kind: ConstructorPayloadKind,
}

impl ConstructorId {
    #[must_use]
    pub fn variant(
        parent: TypeDeclId,
        name: impl Into<Name>,
        payload_kind: ConstructorPayloadKind,
    ) -> Self {
        Self {
            parent,
            name: name.into(),
            payload_kind,
        }
    }
}

/// Canonical promoted data-kind identity derived from an explicit promotion declaration.
///
/// This is distinct from `TypeDeclId`: `source_type` remains the ordinary runtime ADT,
/// while this identity names the opt-in type-level data kind exported from a module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PromotedDataKindId {
    pub module: ModuleIdentity,
    pub source_type: TypeDeclId,
    pub name: Name,
}

impl PromotedDataKindId {
    #[must_use]
    pub fn new(module: ModuleIdentity, source_type: TypeDeclId, name: impl Into<Name>) -> Self {
        Self {
            module,
            source_type,
            name: name.into(),
        }
    }
}

/// Canonical promoted data-constructor identity.
///
/// `source_constructor` is runtime ADT provenance only. Promoted constructors are
/// type-level identities and must not be collapsed into runtime `ConstructorId`s
/// or sealed-domain marker constructors.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PromotedConstructorId {
    pub kind: PromotedDataKindId,
    pub source_constructor: ConstructorId,
    pub name: Name,
}

impl PromotedConstructorId {
    #[must_use]
    pub fn new(
        kind: PromotedDataKindId,
        source_constructor: ConstructorId,
        name: impl Into<Name>,
    ) -> Self {
        Self {
            kind,
            source_constructor,
            name: name.into(),
        }
    }
}

/// Opaque identity for an interface declaration in the current metadata model.
///
/// This is a reserved identity carrier only: it is not a projection IR handle and
/// must not imply associated-family computation, normalization, or equality rules.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InterfaceIdentityId {
    pub module: ModuleIdentity,
    pub name: Name,
}

impl InterfaceIdentityId {
    #[must_use]
    pub fn new(module: ModuleIdentity, name: impl Into<Name>) -> Self {
        Self {
            module,
            name: name.into(),
        }
    }
}

/// Opaque current associated-member kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssociatedMemberIdentityKind {
    AssociatedType,
}

/// Opaque identity for an associated member declared by an interface.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedMemberIdentityId {
    pub interface: InterfaceIdentityId,
    pub name: Name,
    pub member_path: Vec<String>,
    pub kind: AssociatedMemberIdentityKind,
}

impl AssociatedMemberIdentityId {
    #[must_use]
    pub fn associated_type(
        interface: InterfaceIdentityId,
        name: impl Into<Name>,
        member_path: Vec<String>,
    ) -> Self {
        Self {
            interface,
            name: name.into(),
            member_path,
            kind: AssociatedMemberIdentityKind::AssociatedType,
        }
    }
}

/// Canonical sealed type-level domain identity.
///
/// This is distinct from `TypeDeclId` because sealed domains are not ordinary
/// type declarations; they define a type-level namespace for marker constructors.
/// Marker constructors are type-level only and must not be confused with runtime
/// constructors represented by `ConstructorId`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SealedDomainId {
    pub module: ModuleIdentity,
    pub name: Name,
}

impl SealedDomainId {
    #[must_use]
    pub fn new(module: ModuleIdentity, name: impl Into<Name>) -> Self {
        Self {
            module,
            name: name.into(),
        }
    }
}

/// Canonical marker-constructor identity within a sealed domain.
///
/// This is distinct from `ConstructorId` because marker constructors are
/// type-level only; they do not participate in runtime value construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DomainConstructorId {
    pub domain: SealedDomainId,
    pub name: Name,
}

impl DomainConstructorId {
    #[must_use]
    pub fn new(domain: SealedDomainId, name: impl Into<Name>) -> Self {
        Self {
            domain,
            name: name.into(),
        }
    }
}

/// Canonical identity for an explicit type-level proposition predicate.
///
/// This identity is distinct from runtime workflow/capability predicates and from
/// ordinary function/type names. It is a typed summary/diagnostic handle, not a
/// solver rule.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropositionPredicateId {
    pub module: ModuleIdentity,
    pub name: Name,
}

impl PropositionPredicateId {
    #[must_use]
    pub fn new(module: ModuleIdentity, name: impl Into<Name>) -> Self {
        Self {
            module,
            name: name.into(),
        }
    }
}

/// Parameter metadata for an exported proposition predicate identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropositionPredicateParamSummary {
    pub name: Name,
    pub ty: CanonicalTypeExpr,
    pub kind: Kind,
    pub source_anchor: SourceAnchor,
}

/// Source-anchored summary for an exported proposition predicate identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropositionPredicateSummary {
    pub id: PropositionPredicateId,
    pub exported_name: Name,
    pub visibility: Visibility,
    pub params: Vec<PropositionPredicateParamSummary>,
    pub source_anchor: SourceAnchor,
}

/// Structural classification of a domain field relative to its enclosing domain.
///
/// Per SPEC-059 §7.4, at most one `StructuralSelfDomain` field is permitted per
/// constructor, and self-recursion is only allowed through a field that names the
/// enclosing domain directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuralFieldStatus {
    /// Field does not structurally reference the enclosing domain.
    NonStructural,
    /// Field references the enclosing domain directly (self-recursive slot).
    StructuralSelfDomain,
}

/// Field metadata for a domain constructor (SPEC-059 §7).
///
/// Each field carries its `Kind`, an optional domain constraint, and a
/// structural status indicating whether it introduces self-recursion into
/// the enclosing domain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainFieldSummary {
    pub name: Name,
    pub kind: Kind,
    pub domain_constraint: Option<SealedDomainId>,
    pub structural_status: StructuralFieldStatus,
}

impl DomainFieldSummary {
    /// Create an unconstrained type-level field (slot class: Type).
    #[must_use]
    pub fn unconstrained(name: impl Into<Name>) -> Self {
        Self {
            name: name.into(),
            kind: Kind::Type,
            domain_constraint: None,
            structural_status: StructuralFieldStatus::NonStructural,
        }
    }

    /// Create a domain-constrained field pointing at the specified domain.
    ///
    /// If `domain` matches the enclosing domain, `structural_status` is set to
    /// `StructuralSelfDomain`; otherwise it is `NonStructural`.
    #[must_use]
    pub fn constrained_to(
        name: impl Into<Name>,
        enclosing: &SealedDomainId,
        domain: SealedDomainId,
    ) -> Self {
        let structural_status = if &domain == enclosing {
            StructuralFieldStatus::StructuralSelfDomain
        } else {
            StructuralFieldStatus::NonStructural
        };
        Self {
            name: name.into(),
            kind: Kind::Type,
            domain_constraint: Some(domain),
            structural_status,
        }
    }
}

/// Summary for one marker constructor within a sealed domain (SPEC-059 §8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainConstructorSummary {
    pub id: DomainConstructorId,
    pub exported_name: Name,
    pub fields: Vec<DomainFieldSummary>,
    pub anchor: SourceAnchor,
}

impl DomainConstructorSummary {
    #[must_use]
    pub fn new(
        id: DomainConstructorId,
        exported_name: impl Into<Name>,
        fields: Vec<DomainFieldSummary>,
        anchor: SourceAnchor,
    ) -> Self {
        Self {
            id,
            exported_name: exported_name.into(),
            fields,
            anchor,
        }
    }
}

/// Summary for one sealed type-level domain exported from a module (SPEC-059 §8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SealedDomainSummary {
    pub id: SealedDomainId,
    pub exported_name: Name,
    pub visibility: Visibility,
    pub constructors: Vec<DomainConstructorSummary>,
    pub anchor: SourceAnchor,
}

impl SealedDomainSummary {
    #[must_use]
    pub fn new(
        id: SealedDomainId,
        exported_name: impl Into<Name>,
        visibility: Visibility,
        anchor: SourceAnchor,
    ) -> Self {
        Self {
            id,
            exported_name: exported_name.into(),
            visibility,
            constructors: Vec::new(),
            anchor,
        }
    }

    #[must_use]
    pub fn with_constructor(mut self, constructor: DomainConstructorSummary) -> Self {
        self.constructors.push(constructor);
        self
    }
}

/// Exported name/path pointing at an origin type identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeExportRef {
    pub exported_name: Name,
    pub origin: TypeDeclId,
    pub anchor: Option<SourceAnchor>,
}

impl TypeExportRef {
    #[must_use]
    pub fn new(exported_name: impl Into<Name>, origin: TypeDeclId) -> Self {
        Self {
            exported_name: exported_name.into(),
            origin,
            anchor: None,
        }
    }

    #[must_use]
    pub fn with_anchor(mut self, anchor: SourceAnchor) -> Self {
        self.anchor = Some(anchor);
        self
    }
}

/// Public re-export path preserving the original type identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReExportSummary {
    pub exported_path: Vec<String>,
    pub origin: TypeDeclId,
    pub anchor: Option<SourceAnchor>,
}

impl ReExportSummary {
    #[must_use]
    pub fn new(exported_path: Vec<String>, origin: TypeDeclId) -> Self {
        Self {
            exported_path,
            origin,
            anchor: None,
        }
    }

    #[must_use]
    pub fn with_anchor(mut self, anchor: SourceAnchor) -> Self {
        self.anchor = Some(anchor);
        self
    }
}

/// Representation visibility carried by ordinary type summaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepresentationExposure {
    Exposed,
    Opaque,
}

/// Public/opaque ordinary type representation metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeRepresentationSummary {
    Exposed(TypeBody),
    Opaque { builtin: bool },
}

impl TypeRepresentationSummary {
    #[must_use]
    pub fn exposed(body: TypeBody) -> Self {
        Self::Exposed(body)
    }

    #[must_use]
    pub fn opaque(builtin: bool) -> Self {
        Self::Opaque { builtin }
    }
}

/// Declaration class carried independently of a type's representation.
///
/// A nominal newtype deliberately uses an alias-shaped representation carrier,
/// so importers must never infer its nominality from that shape or from its
/// constructor summary. The parser/lowerer is the sole source of this fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TypeDeclarationKind {
    /// Ordinary declarations, including transparent aliases and ADTs.
    #[default]
    Ordinary,
    /// A nominal wrapper with a representation carrier and sole constructor.
    NominalNewtype,
}

/// Summary for one ordinary type declaration visible in a module summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDeclSummary {
    pub id: TypeDeclId,
    pub exported_name: Name,
    pub visibility: Visibility,
    pub params: Vec<TypeVar>,
    /// Source declaration class; this is not derivable from representation shape.
    #[serde(default)]
    pub declaration_kind: TypeDeclarationKind,
    pub representation_exposure: RepresentationExposure,
    pub representation: TypeRepresentationSummary,
    pub source_anchor: SourceAnchor,
}

impl TypeDeclSummary {
    #[must_use]
    pub fn new(
        id: TypeDeclId,
        exported_name: impl Into<Name>,
        visibility: Visibility,
        representation_exposure: RepresentationExposure,
        representation: TypeRepresentationSummary,
        source_anchor: SourceAnchor,
    ) -> Self {
        Self {
            id,
            exported_name: exported_name.into(),
            visibility,
            params: Vec::new(),
            declaration_kind: TypeDeclarationKind::Ordinary,
            representation_exposure,
            representation,
            source_anchor,
        }
    }

    #[must_use]
    pub fn with_params(mut self, params: Vec<TypeVar>) -> Self {
        self.params = params;
        self
    }

    /// Mark the source declaration class without changing representation data.
    #[must_use]
    pub fn with_declaration_kind(mut self, declaration_kind: TypeDeclarationKind) -> Self {
        self.declaration_kind = declaration_kind;
        self
    }
}

/// Summary for an exposed ordinary ADT constructor/variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructorSummary {
    pub id: ConstructorId,
    pub parent: TypeDeclId,
    pub exported_name: Name,
    pub payload_kind: ConstructorPayloadKind,
    pub visibility: Visibility,
    pub source_anchor: SourceAnchor,
}

/// Identity for a named effect-row export. This is summary metadata, not an
/// authority/capability identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectRowExportId {
    pub module: ModuleIdentity,
    pub name: Name,
}

impl EffectRowExportId {
    #[must_use]
    pub fn new(module: ModuleIdentity, name: impl Into<Name>) -> Self {
        Self {
            module,
            name: name.into(),
        }
    }
}

/// Immutable identity of the module declaration which provides an effect row.
///
/// This identity is intentionally independent of the name by which a caller or
/// facade exposes the row.  In particular, importing `Audit` as `PublicAudit`
/// does not manufacture a provider owned by the importing module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectRowProviderIdentity {
    pub module: ModuleIdentity,
    pub declaration_name: Name,
}

impl EffectRowProviderIdentity {
    #[must_use]
    pub fn new(module: ModuleIdentity, declaration_name: impl Into<Name>) -> Self {
        Self {
            module,
            declaration_name: declaration_name.into(),
        }
    }
}

/// How an effect-row provider is exposed at one visible binding site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectRowBindingExposure {
    Declaration,
    NamedImport,
    GlobImport,
    PublicReExport,
}

/// A caller- or facade-visible name for an immutable effect-row provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectRowVisibleBinding {
    pub visible_name: Name,
    pub provider: EffectRowProviderIdentity,
    pub exposure: EffectRowBindingExposure,
    pub closure_status: EffectRowClosureStatus,
}

impl EffectRowVisibleBinding {
    #[must_use]
    pub fn new(
        visible_name: impl Into<Name>,
        provider: EffectRowProviderIdentity,
        exposure: EffectRowBindingExposure,
    ) -> Self {
        Self {
            visible_name: visible_name.into(),
            provider,
            exposure,
            closure_status: EffectRowClosureStatus::Complete,
        }
    }

    #[must_use]
    pub fn with_closure_status(mut self, closure_status: EffectRowClosureStatus) -> Self {
        self.closure_status = closure_status;
        self
    }
}

/// A deliberately content-free marker for a dependency that cannot cross a
/// module visibility boundary.
///
/// The marker has no fields so a serialized summary cannot disclose a private
/// name, path, source anchor, row text, signature, or provider identity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OpaqueInaccessibleDependency;

/// Whether the public dependency closure for a visible row binding is usable.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectRowClosureStatus {
    #[default]
    Complete,
    OpaqueInaccessibleDependency(OpaqueInaccessibleDependency),
}

/// The source-language role retained for a named effect-row export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectRowExportClassification {
    TransparentAlias,
    DiagnosticGroup,
}

/// Effect-row exports never grant authority by themselves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectRowAuthority {
    NonGranting,
}

/// Source-order effect-row item retained for later checked expansion.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectRowItemSummary {
    pub text: String,
}

impl EffectRowItemSummary {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// Versioned evidence emitted by the effect-row dependency-closure sanitizer.
///
/// The digest covers only the selected public closure. It is deliberately
/// absent for opaque inaccessible boundaries, whose private source details
/// must not cross the semantic-summary boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectRowClosureMetadata {
    pub sanitizer_schema_version: u16,
    pub public_closure_digest: String,
}

/// Checked module-summary metadata for one named effect row.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectRowExportSummary {
    pub id: EffectRowExportId,
    pub exported_name: Name,
    /// Provider-owned identity retained independently from a visible binding.
    pub provider: EffectRowProviderIdentity,
    /// The visible binding represented by this summary row.
    pub binding: EffectRowVisibleBinding,
    pub visibility: Visibility,
    pub classification: EffectRowExportClassification,
    pub authority: EffectRowAuthority,
    pub row_items: Vec<EffectRowItemSummary>,
    pub source_anchor: SourceAnchor,
    /// Sanitizer evidence required by the provider-binding V7 schema.
    ///
    /// This remains optional in-memory so legacy and malformed wire payloads
    /// can be decoded and rejected deterministically at the version boundary.
    pub closure_metadata: Option<EffectRowClosureMetadata>,
}

impl Serialize for EffectRowExportSummary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct OpaqueBoundary<'a> {
            closure_status: &'a EffectRowClosureStatus,
        }

        #[derive(Serialize)]
        struct CompleteSummary<'a> {
            id: &'a EffectRowExportId,
            exported_name: &'a Name,
            provider: &'a EffectRowProviderIdentity,
            binding: &'a EffectRowVisibleBinding,
            visibility: Visibility,
            classification: EffectRowExportClassification,
            authority: EffectRowAuthority,
            row_items: &'a [EffectRowItemSummary],
            source_anchor: &'a SourceAnchor,
            closure_metadata: &'a Option<EffectRowClosureMetadata>,
        }

        match self.binding.closure_status {
            EffectRowClosureStatus::OpaqueInaccessibleDependency(_) => OpaqueBoundary {
                closure_status: &self.binding.closure_status,
            }
            .serialize(serializer),
            EffectRowClosureStatus::Complete => CompleteSummary {
                id: &self.id,
                exported_name: &self.exported_name,
                provider: &self.provider,
                binding: &self.binding,
                visibility: self.visibility,
                classification: self.classification,
                authority: self.authority,
                row_items: &self.row_items,
                source_anchor: &self.source_anchor,
                closure_metadata: &self.closure_metadata,
            }
            .serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for EffectRowExportSummary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct OpaqueBoundary {
            closure_status: EffectRowClosureStatus,
        }

        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct CompleteSummary {
            id: EffectRowExportId,
            exported_name: Name,
            provider: EffectRowProviderIdentity,
            binding: EffectRowVisibleBinding,
            visibility: Visibility,
            classification: EffectRowExportClassification,
            authority: EffectRowAuthority,
            row_items: Vec<EffectRowItemSummary>,
            source_anchor: SourceAnchor,
            closure_metadata: Option<EffectRowClosureMetadata>,
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum WireSummary {
            Opaque(OpaqueBoundary),
            Complete(Box<CompleteSummary>),
        }

        match WireSummary::deserialize(deserializer)? {
            WireSummary::Opaque(OpaqueBoundary {
                closure_status: EffectRowClosureStatus::OpaqueInaccessibleDependency(_),
            }) => Ok(Self::opaque_inaccessible_boundary()),
            WireSummary::Opaque(OpaqueBoundary {
                closure_status: EffectRowClosureStatus::Complete,
            }) => Err(serde::de::Error::custom(
                "opaque effect-row boundary must carry opaque inaccessible status",
            )),
            WireSummary::Complete(complete) => {
                if matches!(
                    complete.binding.closure_status,
                    EffectRowClosureStatus::OpaqueInaccessibleDependency(_)
                ) {
                    return Err(serde::de::Error::custom(
                        "complete effect-row payload must not carry opaque inaccessible status",
                    ));
                }
                let CompleteSummary {
                    id,
                    exported_name,
                    provider,
                    binding,
                    visibility,
                    classification,
                    authority,
                    row_items,
                    source_anchor,
                    closure_metadata,
                } = *complete;
                Ok(Self {
                    id,
                    exported_name,
                    provider,
                    binding,
                    visibility,
                    classification,
                    authority,
                    row_items,
                    source_anchor,
                    closure_metadata,
                })
            }
        }
    }
}

impl EffectRowExportSummary {
    fn opaque_inaccessible_boundary() -> Self {
        let module = ModuleIdentity::new(
            None,
            ModuleId(usize::MAX),
            Vec::new(),
            ModuleSourceOrigin::Synthetic {
                reason: "opaque inaccessible effect-row boundary".to_string(),
            },
        );
        let provider = EffectRowProviderIdentity::new(module.clone(), "<opaque-effect-row>");
        let binding = EffectRowVisibleBinding::new(
            "<opaque-effect-row>",
            provider.clone(),
            EffectRowBindingExposure::Declaration,
        )
        .with_closure_status(EffectRowClosureStatus::OpaqueInaccessibleDependency(
            OpaqueInaccessibleDependency,
        ));

        Self {
            id: EffectRowExportId::new(module, "<opaque-effect-row>"),
            exported_name: "<opaque-effect-row>".into(),
            provider,
            binding,
            visibility: Visibility::Private,
            classification: EffectRowExportClassification::TransparentAlias,
            authority: EffectRowAuthority::NonGranting,
            row_items: Vec::new(),
            source_anchor: SourceAnchor::new(
                SourceOrigin::Synthetic {
                    reason: "opaque inaccessible effect-row boundary".to_string(),
                },
                None,
                "opaque inaccessible effect-row boundary",
            ),
            closure_metadata: None,
        }
    }

    #[must_use]
    pub fn new(
        id: EffectRowExportId,
        exported_name: impl Into<Name>,
        visibility: Visibility,
        classification: EffectRowExportClassification,
        row_items: Vec<EffectRowItemSummary>,
        source_anchor: SourceAnchor,
    ) -> Self {
        let provider = EffectRowProviderIdentity::new(id.module.clone(), id.name.clone());
        let exported_name = exported_name.into();
        Self {
            id,
            exported_name: exported_name.clone(),
            binding: EffectRowVisibleBinding::new(
                exported_name,
                provider.clone(),
                EffectRowBindingExposure::Declaration,
            ),
            provider,
            visibility,
            classification,
            authority: EffectRowAuthority::NonGranting,
            row_items,
            source_anchor,
            closure_metadata: None,
        }
    }

    /// Rebind this provider under a caller- or facade-visible name.
    ///
    /// This compatibility helper keeps the legacy binding identifier and
    /// `exported_name` mirror in step while all new code can inspect the
    /// explicit provider/binding contract.
    pub fn set_visible_binding(
        &mut self,
        visible_name: impl Into<Name>,
        exposure: EffectRowBindingExposure,
    ) {
        let visible_name = visible_name.into();
        self.exported_name = visible_name.clone();
        self.id.name = visible_name.clone();
        self.binding = EffectRowVisibleBinding::new(visible_name, self.provider.clone(), exposure);
    }

    /// Mark the binding unusable without transporting any inaccessible
    /// dependency details across the summary boundary.
    pub fn mark_opaque_inaccessible_dependency(&mut self) {
        self.binding.closure_status =
            EffectRowClosureStatus::OpaqueInaccessibleDependency(OpaqueInaccessibleDependency);
    }
}

/// Public value-export classification retained independently of syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueExportKind {
    Handler,
}

/// Checked module-summary metadata for a public value declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValueExportSummary {
    pub exported_name: Name,
    pub visibility: Visibility,
    pub kind: ValueExportKind,
    pub source_anchor: SourceAnchor,
}

impl ValueExportSummary {
    #[must_use]
    pub fn new(
        exported_name: impl Into<Name>,
        visibility: Visibility,
        kind: ValueExportKind,
        source_anchor: SourceAnchor,
    ) -> Self {
        Self {
            exported_name: exported_name.into(),
            visibility,
            kind,
            source_anchor,
        }
    }
}

impl ConstructorSummary {
    #[must_use]
    pub fn new(
        id: ConstructorId,
        parent: TypeDeclId,
        exported_name: impl Into<Name>,
        payload_kind: ConstructorPayloadKind,
        visibility: Visibility,
        source_anchor: SourceAnchor,
    ) -> Self {
        Self {
            id,
            parent,
            exported_name: exported_name.into(),
            payload_kind,
            visibility,
            source_anchor,
        }
    }
}

/// Field metadata for a promoted data constructor.
///
/// `data_kind_constraint` records that this field must inhabit a promoted data kind.
/// It is intentionally not a sealed-domain constraint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PromotedConstructorFieldSummary {
    pub name: Name,
    pub kind: Kind,
    pub data_kind_constraint: Option<PromotedDataKindId>,
    pub source_anchor: SourceAnchor,
}

impl PromotedConstructorFieldSummary {
    #[must_use]
    pub fn new(
        name: impl Into<Name>,
        kind: Kind,
        data_kind_constraint: Option<PromotedDataKindId>,
        source_anchor: SourceAnchor,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            data_kind_constraint,
            source_anchor,
        }
    }
}

/// Summary for one promoted constructor within a promoted data kind.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PromotedConstructorSummary {
    pub id: PromotedConstructorId,
    pub exported_name: Name,
    pub source_constructor: ConstructorId,
    pub fields: Vec<PromotedConstructorFieldSummary>,
    pub visibility: Visibility,
    pub source_anchor: SourceAnchor,
}

impl PromotedConstructorSummary {
    #[must_use]
    pub fn new(
        id: PromotedConstructorId,
        exported_name: impl Into<Name>,
        source_constructor: ConstructorId,
        fields: Vec<PromotedConstructorFieldSummary>,
        visibility: Visibility,
        source_anchor: SourceAnchor,
    ) -> Self {
        Self {
            id,
            exported_name: exported_name.into(),
            source_constructor,
            fields,
            visibility,
            source_anchor,
        }
    }
}

/// Summary for one promoted data kind exported from a module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PromotedDataKindSummary {
    pub id: PromotedDataKindId,
    pub exported_name: Name,
    pub visibility: Visibility,
    pub source_type: TypeDeclId,
    pub constructors: Vec<PromotedConstructorSummary>,
    pub source_anchor: SourceAnchor,
}

impl PromotedDataKindSummary {
    #[must_use]
    pub fn new(
        id: PromotedDataKindId,
        exported_name: impl Into<Name>,
        visibility: Visibility,
        source_type: TypeDeclId,
        source_anchor: SourceAnchor,
    ) -> Self {
        Self {
            id,
            exported_name: exported_name.into(),
            visibility,
            source_type,
            constructors: Vec::new(),
            source_anchor,
        }
    }

    #[must_use]
    pub fn with_constructor(mut self, constructor: PromotedConstructorSummary) -> Self {
        self.constructors.push(constructor);
        self
    }
}

/// Reference to a transported summary for import/cache accounting.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleSummaryRef {
    pub module: ModuleIdentity,
    pub version: SummaryVersion,
}

/// Version tag for semantic summaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SummaryVersion(pub u16);

impl SummaryVersion {
    pub const SPEC057_ORDINARY_TYPE_V1: Self = Self(1);
    pub const SPEC059_SEALED_DOMAIN_V2: Self = Self(2);
    pub const SPEC062_TYPE_COMPUTATION_V3: Self = Self(3);
    pub const SPEC063_ASSOCIATED_FAMILY_V4: Self = Self(4);
    pub const SPEC064_PROPOSITIONS_V5: Self = Self(5);
    pub const SPEC064_PROPOSITION_V5: Self = Self(5);
    pub const SPEC065_PROMOTED_DATA_KIND_V6: Self = Self(6);
    pub const SPEC065_PROMOTED_DATA_KINDS_V6: Self = Self(6);
    /// Provider identities, visible bindings, and sanitized effect-row closure
    /// evidence. Older summaries must not be reinterpreted as this schema.
    pub const EFFECT_ROW_PROVIDER_BINDINGS_V7: Self = Self(7);
}

/// Core schema-level validation failures for semantic-summary version contracts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleSemanticSummaryValidationError {
    /// Public type-computation facts are only valid in SPEC-062/V3 or newer summaries.
    TypeFunctionsRequireV3 { version: SummaryVersion },
    /// Public associated-family facts are only valid in SPEC-063/V4 summaries.
    AssociatedFamiliesRequireV4 { version: SummaryVersion },
    /// Public proposition facts are only valid in SPEC-064/V5 summaries.
    PropositionFactsRequireV5 { version: SummaryVersion },
    /// Public promoted data-kind facts are only valid in SPEC-065/V6 summaries.
    PromotedDataKindsRequireV6 { version: SummaryVersion },
    /// Provider/binding effect-row payloads are valid only in V7.
    EffectRowProviderBindingsRequireV7 { version: SummaryVersion },
    /// A V7 provider/binding row omitted required sanitizer closure evidence.
    EffectRowProviderBindingClosureIncomplete { version: SummaryVersion },
    /// A V7 provider/binding row uses a closure-sanitizer schema this core
    /// crate does not know how to interpret.
    UnsupportedEffectRowSanitizerSchemaVersion { version: u16 },
    /// A V7 row disagrees about its immutable provider or visible binding.
    EffectRowProviderBindingIncoherent { version: SummaryVersion },
    /// A V7 row reaches an opaque inaccessible dependency and cannot be used
    /// at a public import boundary.
    EffectRowProviderBindingOpaqueInaccessible { version: SummaryVersion },
    /// The summary version is newer than this core crate knows how to interpret.
    UnsupportedSummaryVersion { version: SummaryVersion },
}

/// Explicit public export/transparency mode for type-function summaries.
///
/// SPEC-062 MVP supports only direct transparent equation export. The enum keeps
/// this decision explicit so future opaque/header-only modes cannot be confused
/// with missing equation data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeFunctionExportMode {
    TransparentEquations,
}

/// Checked public type-function parameter metadata transported in summaries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeFunctionParamSummary {
    pub name: String,
    pub ty: CanonicalTypeExpr,
    pub kind: Kind,
    pub domain_constraint: Option<SealedDomainId>,
    pub source_anchor: SourceAnchor,
}

/// Summary dependency reference plus cache/dedup invalidation metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeFunctionDependencySummaryRef {
    pub summary_ref: ModuleSummaryRef,
    /// Optional content digest for in-memory dedup now and persistent cache keys later.
    pub digest: Option<String>,
    /// Optional algorithm/version dimension for future type-computation cache invalidation.
    pub compiler_algorithm_version: Option<String>,
}

/// Public-closure evidence produced by export validation.
///
/// Import-side revalidation must not trust this blindly, but preserving it gives
/// future TypeEnv import and cache code the dimensions required by SPEC-062 §6/§11.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeFunctionClosureMetadata {
    pub public_closure_checked: bool,
    pub public_ordinary_type_count: usize,
    pub public_sealed_domain_count: usize,
    pub public_type_function_count: usize,
    pub public_projection_count: usize,
}

/// Revalidation metadata sufficient for future TypeEnv import diagnostics/cache keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeFunctionRevalidationMetadata {
    pub spec_version: SummaryVersion,
    pub structural_recursion_checked: bool,
    pub kind_and_domain_checked: bool,
    pub coverage_and_overlap_checked: bool,
    /// Name of the checked decreasing parameter when structural recursion is present.
    #[serde(default)]
    pub decreases_param: Option<String>,
}

/// Core-owned public type-function summary carrier for SPEC-062.
///
/// This is intentionally distinct from engine-private export structures. It
/// preserves the checked public signature, transparent source-order equations,
/// dependency refs/digests, and closure/revalidation metadata needed for a
/// future TypeEnv import path without implementing that import path here.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeFunctionSummary {
    pub exported_name: Name,
    pub head: TypeComputationHeadId,
    pub visibility: Visibility,
    pub params: Vec<TypeFunctionParamSummary>,
    pub return_type: CanonicalTypeExpr,
    pub return_kind: Kind,
    pub result_constraint: TypeFunctionResultConstraint,
    pub export_mode: TypeFunctionExportMode,
    pub source_anchors: TypeFunctionSourceAnchors,
    pub equations: Vec<TypeFunctionEquation>,
    pub dependency_summary_refs: Vec<TypeFunctionDependencySummaryRef>,
    pub closure_metadata: TypeFunctionClosureMetadata,
    pub revalidation_metadata: TypeFunctionRevalidationMetadata,
}

/// Explicit public export/transparency mode for associated-family summaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssociatedFamilyExportMode {
    TransparentEquations,
}

/// Validated structural-decreases metadata for recursive associated families.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValidatedDecreasesSummary {
    pub parameter: String,
    pub parameter_index: usize,
    pub domain: SealedDomainId,
    pub structural_recursion_checked: bool,
    pub source_anchor: SourceAnchor,
}

/// Summary dependency reference plus cache/dedup invalidation metadata for an
/// imported associated-family summary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedFamilyDependencySummaryRef {
    pub summary_ref: ModuleSummaryRef,
    pub family: AssociatedFamilyHeadId,
    /// Optional content digest for in-memory dedup now and persistent cache keys later.
    pub digest: Option<String>,
    /// Optional algorithm/version dimension for future family cache invalidation.
    pub compiler_algorithm_version: Option<String>,
    /// Whether this dependency is source-name visible to importers.
    pub source_visible: bool,
    /// Whether validated equations are available to the normalizer through the summary.
    pub normalizer_available: bool,
}

/// Public-closure evidence produced by associated-family export validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedFamilyClosureMetadata {
    pub public_closure_checked: bool,
    pub public_ordinary_type_count: usize,
    pub public_sealed_domain_count: usize,
    pub public_domain_constructor_count: usize,
    pub public_type_function_count: usize,
    pub public_associated_family_count: usize,
    pub public_projection_count: usize,
    pub helper_family_count: usize,
}

/// Public dependency closure required to revalidate/import an associated family.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedFamilyDependencyClosure {
    pub ordinary_types: Vec<TypeDeclId>,
    pub sealed_domains: Vec<SealedDomainId>,
    pub domain_constructors: Vec<DomainConstructorId>,
    pub type_functions: Vec<TypeComputationHeadId>,
    pub associated_projections: Vec<AssociatedFamilyProjection>,
    pub associated_families: Vec<AssociatedFamilyDependencySummaryRef>,
    #[serde(default)]
    pub type_function_summaries: Vec<TypeFunctionDependencySummaryRef>,
    pub closure_metadata: AssociatedFamilyClosureMetadata,
}

/// Revalidation metadata sufficient for future associated-family import checks
/// and cache keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedFamilyRevalidationMetadata {
    pub spec_version: SummaryVersion,
    pub kind_and_domain_checked: bool,
    pub coverage_and_overlap_checked: bool,
    pub coherence_checked: bool,
    pub recursion_checked: bool,
    pub decreases: Vec<ValidatedDecreasesSummary>,
}

/// Core-owned public associated-family summary carrier for SPEC-063/V4.
///
/// It preserves typed interface/member/family identities, visible names, result
/// kind/domain, transparent checked schemes, dependency closure, source anchors,
/// and revalidation metadata. It does not perform TypeEnv import, normalizer
/// registration, impl selection, or proof/search.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedFamilySummary {
    pub head: AssociatedFamilyHeadId,
    pub interface_identity: InterfaceIdentityId,
    pub member_identity: AssociatedMemberIdentityId,
    pub visible_name: String,
    pub result_domain: CanonicalTypeExpr,
    pub result_kind: Kind,
    pub export_mode: AssociatedFamilyExportMode,
    pub schemes: Vec<AssociatedFamilyScheme>,
    pub dependency_closure: AssociatedFamilyDependencyClosure,
    pub source_anchor: SourceAnchor,
    pub revalidation_metadata: AssociatedFamilyRevalidationMetadata,
}

/// Summary dependency reference plus cache/dedup invalidation metadata for a
/// transported proposition fact.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropositionDependencySummaryRef {
    pub summary_ref: ModuleSummaryRef,
    pub digest: Option<String>,
    pub compiler_algorithm_version: Option<String>,
    pub source_visible: bool,
}

/// Role of a proposition fact exported through a semantic summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropositionFactRole {
    Requirement,
    Assumption,
    Evidence,
}

/// Public proposition fact transported by SPEC-064/V5 summaries.
///
/// The fact is intentionally typed: propositions, predicate dependencies, and
/// optional boundary outcomes are structural carriers, not strings or debug
/// fragments. Consumers still must revalidate imported facts locally.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropositionFactSummary {
    pub proposition: TypeProposition,
    pub role: PropositionFactRole,
    pub source_anchor: SourceAnchor,
    pub predicate_dependencies: Vec<PropositionPredicateId>,
    pub dependency_summary_refs: Vec<PropositionDependencySummaryRef>,
    pub outcome: Option<PropositionOutcome>,
}

/// Reserved future identity namespaces. SPEC-057 leaves these uninterpreted.
///
/// Note: `sealed_domains` is a placeholder string list. As of SPEC-059, typed
/// sealed-domain metadata is carried by `ModuleSemanticSummary::exported_sealed_domains`
/// using `SealedDomainSummary` carriers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ReservedSemanticIdentitySlots {
    pub future_type_functions: Vec<String>,
    /// Placeholder only; superseded by `ModuleSemanticSummary::exported_sealed_domains`.
    pub sealed_domains: Vec<String>,
    pub generalized_projections: Vec<String>,
    pub associated_families: Vec<String>,
    pub extensions: Vec<(String, String)>,
}

impl ReservedSemanticIdentitySlots {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.future_type_functions.is_empty()
            && self.sealed_domains.is_empty()
            && self.generalized_projections.is_empty()
            && self.associated_families.is_empty()
            && self.extensions.is_empty()
    }
}

/// Opaque summary entry for an interface declaration identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceIdentitySummary {
    pub id: InterfaceIdentityId,
    pub name: Name,
    pub path: Vec<String>,
    /// Interface-owned required evidence constraints transported with this identity.
    #[serde(default)]
    pub evidence_constraints: Vec<InterfaceEvidenceConstraintSummary>,
    pub source_anchor: SourceAnchor,
}

impl InterfaceIdentitySummary {
    #[must_use]
    pub fn new(
        id: InterfaceIdentityId,
        name: impl Into<Name>,
        path: Vec<String>,
        source_anchor: SourceAnchor,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            path,
            evidence_constraints: Vec::new(),
            source_anchor,
        }
    }

    #[must_use]
    pub fn with_evidence_constraints(
        mut self,
        constraints: Vec<InterfaceEvidenceConstraintSummary>,
    ) -> Self {
        self.evidence_constraints = constraints;
        self
    }
}

/// Summary payload for one interface-owned required evidence constraint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceEvidenceConstraintSummary {
    pub subject: TypeExpr,
    pub required_evidence: TypeExpr,
}

impl InterfaceEvidenceConstraintSummary {
    #[must_use]
    pub fn new(subject: TypeExpr, required_evidence: TypeExpr) -> Self {
        Self {
            subject,
            required_evidence,
        }
    }
}

/// Opaque summary entry for an associated member declaration identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssociatedMemberIdentitySummary {
    pub id: AssociatedMemberIdentityId,
    pub name: Name,
    pub source_anchor: SourceAnchor,
}

impl AssociatedMemberIdentitySummary {
    #[must_use]
    pub fn new(
        id: AssociatedMemberIdentityId,
        name: impl Into<Name>,
        source_anchor: SourceAnchor,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            source_anchor,
        }
    }
}

/// Core-owned ordinary-type semantic summary for one module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleSemanticSummary {
    pub module: ModuleIdentity,
    pub version: SummaryVersion,
    pub exported_types: Vec<TypeDeclSummary>,
    pub exported_constructors: Vec<ConstructorSummary>,
    /// Named effect rows are non-authority metadata for later checking and diagnostics.
    #[serde(default)]
    pub exported_effect_rows: Vec<EffectRowExportSummary>,
    /// Public value-namespace declarations with their distinct callable markers.
    #[serde(default)]
    pub exported_values: Vec<ValueExportSummary>,
    pub re_exports: Vec<ReExportSummary>,
    pub imported_summary_refs: Vec<ModuleSummaryRef>,
    #[serde(default)]
    pub interface_identities: Vec<InterfaceIdentitySummary>,
    #[serde(default)]
    pub associated_member_identities: Vec<AssociatedMemberIdentitySummary>,
    #[serde(default)]
    pub reserved_identity_slots: ReservedSemanticIdentitySlots,
    #[serde(default)]
    pub diagnostic_anchors: Vec<SourceAnchor>,
    /// Sealed type-level domains exported from this module (SPEC-059 §8).
    ///
    /// `#[serde(default)]` ensures backward compatibility with V1 summaries
    /// that predate sealed-domain support.
    #[serde(default)]
    pub exported_sealed_domains: Vec<SealedDomainSummary>,
    /// Public promoted data-kind summaries exported from this module (SPEC-065 §7).
    ///
    /// `#[serde(default)]` preserves V1-V5 wire compatibility. Non-empty values
    /// are valid only when `version` is SPEC-065/V6 or newer.
    #[serde(default)]
    pub exported_promoted_data_kinds: Vec<PromotedDataKindSummary>,
    /// Public type-function summaries exported from this module (SPEC-062 §6).
    ///
    /// `#[serde(default)]` preserves V1/V2 wire compatibility. Non-empty values
    /// are valid only when `version` is SPEC-062/V3 or newer.
    #[serde(default)]
    pub exported_type_functions: Vec<TypeFunctionSummary>,
    /// Public associated-family summaries exported from this module (SPEC-063 §11).
    ///
    /// `#[serde(default)]` preserves V1/V2/V3 wire compatibility. Non-empty
    /// values are valid only when `version` is SPEC-063/V4.
    #[serde(default)]
    pub exported_associated_families: Vec<AssociatedFamilySummary>,
    /// Public proposition predicate identities exported from this module (SPEC-064 §10).
    ///
    /// `#[serde(default)]` preserves V1-V4 wire compatibility. Non-empty values
    /// are valid only when `version` is SPEC-064/V5.
    #[serde(default)]
    pub exported_proposition_predicates: Vec<PropositionPredicateSummary>,
    /// Public proposition facts exported from this module (SPEC-064 §10).
    ///
    /// `#[serde(default)]` preserves V1-V4 wire compatibility. Non-empty values
    /// are valid only when `version` is SPEC-064/V5.
    #[serde(default)]
    pub exported_proposition_facts: Vec<PropositionFactSummary>,
}

impl ModuleSemanticSummary {
    #[must_use]
    pub fn new(module: ModuleIdentity) -> Self {
        Self {
            module,
            version: SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
            exported_types: Vec::new(),
            exported_constructors: Vec::new(),
            exported_effect_rows: Vec::new(),
            exported_values: Vec::new(),
            re_exports: Vec::new(),
            imported_summary_refs: Vec::new(),
            interface_identities: Vec::new(),
            associated_member_identities: Vec::new(),
            reserved_identity_slots: ReservedSemanticIdentitySlots::default(),
            diagnostic_anchors: Vec::new(),
            exported_sealed_domains: Vec::new(),
            exported_promoted_data_kinds: Vec::new(),
            exported_type_functions: Vec::new(),
            exported_associated_families: Vec::new(),
            exported_proposition_predicates: Vec::new(),
            exported_proposition_facts: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_version(mut self, version: SummaryVersion) -> Self {
        self.version = version;
        self
    }

    #[must_use]
    pub fn with_exported_type(mut self, ty: TypeDeclSummary) -> Self {
        self.exported_types.push(ty);
        self
    }

    #[must_use]
    pub fn with_exported_constructor(mut self, constructor: ConstructorSummary) -> Self {
        self.exported_constructors.push(constructor);
        self
    }

    #[must_use]
    pub fn with_exported_effect_row(mut self, row: EffectRowExportSummary) -> Self {
        self.exported_effect_rows.push(row);
        self
    }

    #[must_use]
    pub fn with_exported_value(mut self, value: ValueExportSummary) -> Self {
        self.exported_values.push(value);
        self
    }

    #[must_use]
    pub fn with_re_export(mut self, re_export: ReExportSummary) -> Self {
        self.re_exports.push(re_export);
        self
    }

    #[must_use]
    pub fn with_imported_summary_ref(mut self, summary_ref: ModuleSummaryRef) -> Self {
        self.imported_summary_refs.push(summary_ref);
        self
    }

    #[must_use]
    pub fn with_interface_identity(mut self, identity: InterfaceIdentitySummary) -> Self {
        self.interface_identities.push(identity);
        self
    }

    #[must_use]
    pub fn with_associated_member_identity(
        mut self,
        identity: AssociatedMemberIdentitySummary,
    ) -> Self {
        self.associated_member_identities.push(identity);
        self
    }

    #[must_use]
    pub fn with_diagnostic_anchor(mut self, anchor: SourceAnchor) -> Self {
        self.diagnostic_anchors.push(anchor);
        self
    }

    /// Add a sealed domain summary to this module summary.
    #[must_use]
    pub fn with_exported_sealed_domain(mut self, domain: SealedDomainSummary) -> Self {
        self.exported_sealed_domains.push(domain);
        self
    }

    /// Add a public promoted data-kind summary to this module summary.
    #[must_use]
    pub fn with_exported_promoted_data_kind(mut self, data_kind: PromotedDataKindSummary) -> Self {
        self.exported_promoted_data_kinds.push(data_kind);
        self
    }

    /// Add a public type-function summary to this module summary.
    #[must_use]
    pub fn with_exported_type_function(mut self, type_function: TypeFunctionSummary) -> Self {
        self.exported_type_functions.push(type_function);
        self
    }

    /// Add a public associated-family summary to this module summary.
    #[must_use]
    pub fn with_exported_associated_family(mut self, family: AssociatedFamilySummary) -> Self {
        self.exported_associated_families.push(family);
        self
    }

    /// Add a public proposition predicate identity to this module summary.
    #[must_use]
    pub fn with_exported_proposition_predicate(
        mut self,
        predicate: PropositionPredicateSummary,
    ) -> Self {
        self.exported_proposition_predicates.push(predicate);
        self
    }

    /// Add a public proposition fact to this module summary.
    #[must_use]
    pub fn with_exported_proposition_fact(mut self, fact: PropositionFactSummary) -> Self {
        self.exported_proposition_facts.push(fact);
        self
    }

    /// Build the semantic content key used by in-memory import dedup/cache boundaries.
    ///
    /// The key is intentionally structural and process-local: it is suitable for
    /// comparing summaries already decoded into this core version, but it is not a
    /// stable persistent-cache digest format. A future persistent cache should feed
    /// at least these inputs into an explicit digest algorithm: summary schema
    /// version, module identity, ordinary exported type/constructor facts,
    /// imported summary refs, sealed-domain summaries, promoted data-kind summaries,
    /// public type-function signatures/equations, type-function dependency refs/digests,
    /// and compiler algorithm version metadata.
    #[must_use]
    pub fn semantic_cache_key(&self) -> Vec<String> {
        if self.exported_effect_rows.iter().any(|row| {
            matches!(
                row.binding.closure_status,
                EffectRowClosureStatus::OpaqueInaccessibleDependency(_)
            )
        }) {
            // An opaque boundary must not indirectly disclose private source
            // context through enclosing module, re-export, import-ref, or
            // diagnostic-anchor key components. All unusable opaque rows share
            // one cache identity for this schema version.
            return vec![
                format!("version::{:?}", self.version),
                "effect_row::opaque_inaccessible_dependency".to_string(),
            ];
        }

        let mut key = Vec::new();
        key.push(format!("version::{:?}", self.version));
        key.push(format!("module::{:?}", self.module));
        key.extend(self.exported_types.iter().map(|ty| {
            format!(
                "type::{}::{:?}::{:?}::{:?}::{:?}::{:?}::{:?}",
                ty.exported_name,
                ty.id,
                ty.visibility,
                ty.params,
                ty.declaration_kind,
                ty.representation_exposure,
                ty.representation
            )
        }));
        key.extend(self.exported_constructors.iter().map(|constructor| {
            format!(
                "ctor::{}::{:?}::{:?}::{:?}::{:?}",
                constructor.exported_name,
                constructor.id,
                constructor.parent,
                constructor.payload_kind,
                constructor.visibility
            )
        }));
        key.extend(self.exported_effect_rows.iter().map(|row| {
            match row.binding.closure_status {
                EffectRowClosureStatus::OpaqueInaccessibleDependency(_) => {
                    "effect_row::opaque_inaccessible_dependency".to_string()
                }
                EffectRowClosureStatus::Complete => {
                    let closure = row.closure_metadata.as_ref().map_or_else(
                        || "missing".to_string(),
                        |metadata| {
                            format!(
                                "schema={}::digest={}",
                                metadata.sanitizer_schema_version, metadata.public_closure_digest
                            )
                        },
                    );
                    format!(
                        "effect_row::provider={:?}::binding={}::exposure={:?}::closure={closure}",
                        row.provider, row.binding.visible_name, row.binding.exposure
                    )
                }
            }
        }));
        key.extend(
            self.re_exports
                .iter()
                .map(|re_export| format!("re_export::{re_export:?}")),
        );
        key.extend(
            self.interface_identities
                .iter()
                .map(|identity| format!("interface::{identity:?}")),
        );
        key.extend(
            self.associated_member_identities
                .iter()
                .map(|identity| format!("associated_member::{identity:?}")),
        );
        key.push(format!(
            "reserved_identity_slots::{:?}",
            self.reserved_identity_slots
        ));
        key.extend(
            self.diagnostic_anchors
                .iter()
                .map(|anchor| format!("diagnostic_anchor::{anchor:?}")),
        );
        key.extend(
            self.imported_summary_refs
                .iter()
                .map(|summary_ref| format!("summary_ref::{summary_ref:?}")),
        );
        key.extend(self.exported_sealed_domains.iter().map(|domain| {
            format!(
                "domain::{}::{:?}::{:?}::{:?}",
                domain.exported_name, domain.id, domain.visibility, domain.constructors
            )
        }));
        key.extend(self.exported_promoted_data_kinds.iter().map(|data_kind| {
            format!(
                "promoted_data_kind::{}::{:?}::{:?}::{:?}::{:?}",
                data_kind.exported_name,
                data_kind.id,
                data_kind.visibility,
                data_kind.source_type,
                data_kind.constructors
            )
        }));
        key.extend(self.exported_type_functions.iter().map(|type_function| {
            format!(
                "typefn::{}::{:?}::{:?}::{:?}::{:?}::{:?}::{:?}::{:?}::{:?}::{:?}::{:?}::{:?}::{:?}",
                type_function.exported_name,
                type_function.head,
                type_function.visibility,
                type_function.params,
                type_function.return_type,
                type_function.return_kind,
                type_function.result_constraint,
                type_function.export_mode,
                type_function.source_anchors,
                type_function.equations,
                type_function.dependency_summary_refs,
                type_function.closure_metadata,
                type_function.revalidation_metadata
            )
        }));
        key.extend(self.exported_associated_families.iter().map(|family| {
            format!(
                "assoc_family::{:?}::{:?}::{:?}::{}::{:?}::{:?}::{:?}::{:?}::{:?}::{:?}::{:?}",
                family.head,
                family.interface_identity,
                family.member_identity,
                family.visible_name,
                family.result_domain,
                family.result_kind,
                family.export_mode,
                family.schemes,
                family.dependency_closure,
                family.source_anchor,
                family.revalidation_metadata
            )
        }));
        key.extend(
            self.exported_proposition_predicates
                .iter()
                .map(|predicate| format!("proposition_predicate::{predicate:?}")),
        );
        key.extend(
            self.exported_proposition_facts
                .iter()
                .map(|fact| format!("proposition_fact::{fact:?}")),
        );
        key.sort_unstable();
        key
    }

    /// Validate only core summary-version/content compatibility.
    ///
    /// This helper deliberately does not perform TypeEnv import revalidation of
    /// type-function signatures/equations. It enforces the SPEC-062 schema rule
    /// that V1/V2 summaries must not carry public computation facts and rejects
    /// unknown future summary versions before any consumer partially registers
    /// their contents.
    pub fn validate_summary_version_contract(
        &self,
    ) -> Result<(), ModuleSemanticSummaryValidationError> {
        if !matches!(
            self.version,
            SummaryVersion::SPEC057_ORDINARY_TYPE_V1
                | SummaryVersion::SPEC059_SEALED_DOMAIN_V2
                | SummaryVersion::SPEC062_TYPE_COMPUTATION_V3
                | SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4
                | SummaryVersion::SPEC064_PROPOSITIONS_V5
                | SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6
                | SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7
        ) {
            return Err(
                ModuleSemanticSummaryValidationError::UnsupportedSummaryVersion {
                    version: self.version,
                },
            );
        }

        if !self.exported_effect_rows.is_empty()
            && self.version != SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7
        {
            return Err(
                ModuleSemanticSummaryValidationError::EffectRowProviderBindingsRequireV7 {
                    version: self.version,
                },
            );
        }

        if self.version == SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7
            && self.exported_effect_rows.iter().any(|row| {
                row.id.module != self.module
                    || row.id.name != row.binding.visible_name
                    || row.binding.provider != row.provider
                    || row.exported_name != row.binding.visible_name
            })
        {
            return Err(
                ModuleSemanticSummaryValidationError::EffectRowProviderBindingIncoherent {
                    version: self.version,
                },
            );
        }

        if self.version == SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7
            && self.exported_effect_rows.iter().any(|row| {
                matches!(
                    row.binding.closure_status,
                    EffectRowClosureStatus::OpaqueInaccessibleDependency(_)
                )
            })
        {
            return Err(
                ModuleSemanticSummaryValidationError::EffectRowProviderBindingOpaqueInaccessible {
                    version: self.version,
                },
            );
        }

        if self.version == SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7
            && self.exported_effect_rows.iter().any(|row| {
                !matches!(
                    &row.closure_metadata,
                    Some(metadata)
                        if metadata.sanitizer_schema_version != 0
                            && !metadata.public_closure_digest.trim().is_empty()
                )
            })
        {
            return Err(
                ModuleSemanticSummaryValidationError::EffectRowProviderBindingClosureIncomplete {
                    version: self.version,
                },
            );
        }

        if self.version == SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7
            && self.exported_effect_rows.iter().any(|row| {
                row.closure_metadata.as_ref().is_some_and(|metadata| {
                    metadata.sanitizer_schema_version != EFFECT_ROW_SANITIZER_SCHEMA_VERSION
                })
            })
        {
            // The preceding completeness check guarantees that this is a
            // non-zero, structurally complete but unknown schema version.
            let version = self
                .exported_effect_rows
                .iter()
                .filter_map(|row| row.closure_metadata.as_ref())
                .find(|metadata| {
                    metadata.sanitizer_schema_version != EFFECT_ROW_SANITIZER_SCHEMA_VERSION
                })
                .map_or(0, |metadata| metadata.sanitizer_schema_version);
            return Err(
                ModuleSemanticSummaryValidationError::UnsupportedEffectRowSanitizerSchemaVersion {
                    version,
                },
            );
        }

        if !matches!(
            self.version,
            SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6
                | SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7
        ) && !self.exported_promoted_data_kinds.is_empty()
        {
            return Err(
                ModuleSemanticSummaryValidationError::PromotedDataKindsRequireV6 {
                    version: self.version,
                },
            );
        }

        match self.version {
            SummaryVersion::SPEC057_ORDINARY_TYPE_V1 | SummaryVersion::SPEC059_SEALED_DOMAIN_V2 => {
                if !self.exported_proposition_facts.is_empty()
                    || !self.exported_proposition_predicates.is_empty()
                {
                    Err(
                        ModuleSemanticSummaryValidationError::PropositionFactsRequireV5 {
                            version: self.version,
                        },
                    )
                } else if !self.exported_associated_families.is_empty() {
                    Err(
                        ModuleSemanticSummaryValidationError::AssociatedFamiliesRequireV4 {
                            version: self.version,
                        },
                    )
                } else if self.exported_type_functions.is_empty() {
                    Ok(())
                } else {
                    Err(
                        ModuleSemanticSummaryValidationError::TypeFunctionsRequireV3 {
                            version: self.version,
                        },
                    )
                }
            }
            SummaryVersion::SPEC062_TYPE_COMPUTATION_V3 => {
                if !self.exported_proposition_facts.is_empty()
                    || !self.exported_proposition_predicates.is_empty()
                {
                    Err(
                        ModuleSemanticSummaryValidationError::PropositionFactsRequireV5 {
                            version: self.version,
                        },
                    )
                } else if self.exported_associated_families.is_empty() {
                    Ok(())
                } else {
                    Err(
                        ModuleSemanticSummaryValidationError::AssociatedFamiliesRequireV4 {
                            version: self.version,
                        },
                    )
                }
            }
            SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4 => {
                if self.exported_proposition_facts.is_empty()
                    && self.exported_proposition_predicates.is_empty()
                {
                    Ok(())
                } else {
                    Err(
                        ModuleSemanticSummaryValidationError::PropositionFactsRequireV5 {
                            version: self.version,
                        },
                    )
                }
            }
            SummaryVersion::SPEC064_PROPOSITIONS_V5
            | SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6
            | SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7 => Ok(()),
            version => {
                Err(ModuleSemanticSummaryValidationError::UnsupportedSummaryVersion { version })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Span, TypeBody, TypeExpr, VariantDef, VariantPayload, Visibility};
    use crate::module_graph::{CrateId, ModuleId};

    fn module_identity() -> ModuleIdentity {
        ModuleIdentity::new(
            Some(CrateId(7)),
            ModuleId(42),
            vec!["crate".into(), "domain".into()],
            ModuleSourceOrigin::File("/repo/src/domain.ash".into()),
        )
    }

    #[test]
    fn type_decl_identity_ignores_module_diagnostic_metadata_spans_and_export_aliases() {
        let module = module_identity();
        let same_module_with_different_diagnostics = ModuleIdentity::new(
            Some(CrateId(7)),
            ModuleId(42),
            vec!["renamed".into(), "domain".into()],
            ModuleSourceOrigin::File("/different/path/domain.ash".into()),
        );
        let origin = TypeDeclId::ordinary(module.clone(), "UserId");
        let alias_import = TypeExportRef::new("Id", origin.clone()).with_anchor(SourceAnchor::new(
            SourceOrigin::File("/repo/src/client.ash".into()),
            Some(Span {
                start: 100,
                end: 102,
            }),
            "import alias",
        ));
        let re_export = ReExportSummary::new(
            vec!["crate".into(), "api".into(), "Id".into()],
            origin.clone(),
        )
        .with_anchor(SourceAnchor::new(
            SourceOrigin::File("/repo/src/api.ash".into()),
            Some(Span {
                start: 200,
                end: 220,
            }),
            "pub use",
        ));

        assert_eq!(module, same_module_with_different_diagnostics);
        assert_eq!(origin, alias_import.origin);
        assert_eq!(origin, re_export.origin);
        assert_eq!(
            origin,
            TypeDeclId::ordinary(same_module_with_different_diagnostics, "UserId")
        );
    }

    #[test]
    fn constructor_identity_derives_from_parent_type_name_and_payload_kind() {
        let parent = TypeDeclId::ordinary(module_identity(), "Status");

        let unit_pending =
            ConstructorId::variant(parent.clone(), "Pending", ConstructorPayloadKind::Unit);
        let record_pending =
            ConstructorId::variant(parent.clone(), "Pending", ConstructorPayloadKind::Record);
        let unit_done =
            ConstructorId::variant(parent.clone(), "Done", ConstructorPayloadKind::Unit);

        assert_eq!(unit_pending.parent, parent);
        assert_ne!(unit_pending, record_pending);
        assert_ne!(unit_pending, unit_done);
    }

    #[test]
    fn interface_and_associated_member_identity_slots_are_opaque_current_metadata() {
        let module = module_identity();
        let interface_id = InterfaceIdentityId::new(module.clone(), "Serializer");
        let associated_id = AssociatedMemberIdentityId::associated_type(
            interface_id.clone(),
            "Ok",
            vec!["Serializer".into(), "Ok".into()],
        );
        let interface_anchor = SourceAnchor::new(
            SourceOrigin::File("/repo/src/domain.ash".into()),
            Some(Span { start: 10, end: 40 }),
            "interface Serializer",
        );
        let associated_anchor = SourceAnchor::new(
            SourceOrigin::File("/repo/src/domain.ash".into()),
            Some(Span { start: 20, end: 22 }),
            "associated type Ok",
        );

        let summary = ModuleSemanticSummary::new(module.clone())
            .with_interface_identity(InterfaceIdentitySummary::new(
                interface_id.clone(),
                "Serializer",
                vec!["Serializer".into()],
                interface_anchor.clone(),
            ))
            .with_associated_member_identity(AssociatedMemberIdentitySummary::new(
                associated_id.clone(),
                "Ok",
                associated_anchor.clone(),
            ));

        assert_eq!(summary.interface_identities[0].id, interface_id);
        assert_eq!(
            summary.interface_identities[0].source_anchor,
            interface_anchor
        );
        assert_eq!(summary.associated_member_identities[0].id, associated_id);
        assert_eq!(
            summary.associated_member_identities[0].source_anchor,
            associated_anchor
        );
        assert!(summary.reserved_identity_slots.is_empty());
    }

    #[test]
    fn module_semantic_summary_represents_public_ordinary_type_and_constructor_exposure() {
        let module = module_identity();
        let type_id = TypeDeclId::ordinary(module.clone(), "Status");
        let pending_id =
            ConstructorId::variant(type_id.clone(), "Pending", ConstructorPayloadKind::Unit);
        let done_id =
            ConstructorId::variant(type_id.clone(), "Done", ConstructorPayloadKind::Record);
        let anchor = SourceAnchor::new(
            SourceOrigin::File("/repo/src/domain.ash".into()),
            Some(Span { start: 5, end: 55 }),
            "type Status",
        );

        let ty = TypeDeclSummary::new(
            type_id.clone(),
            "Status",
            Visibility::Public,
            RepresentationExposure::Exposed,
            TypeRepresentationSummary::exposed(TypeBody::Enum(vec![
                VariantDef {
                    name: "Pending".into(),
                    fields: vec![],
                    payload: VariantPayload::Unit,
                },
                VariantDef {
                    name: "Done".into(),
                    fields: vec![("code".into(), TypeExpr::Named("Int".into()))],
                    payload: VariantPayload::Record(vec![(
                        { "code".into() },
                        TypeExpr::Named("Int".into()),
                    )]),
                },
            ])),
            anchor.clone(),
        );
        let pending = ConstructorSummary::new(
            pending_id,
            type_id.clone(),
            "Pending",
            ConstructorPayloadKind::Unit,
            Visibility::Public,
            anchor.clone(),
        );
        let done = ConstructorSummary::new(
            done_id,
            type_id,
            "Done",
            ConstructorPayloadKind::Record,
            Visibility::Public,
            anchor.clone(),
        );

        let summary = ModuleSemanticSummary::new(module)
            .with_exported_type(ty.clone())
            .with_exported_constructor(pending.clone())
            .with_exported_constructor(done.clone())
            .with_diagnostic_anchor(anchor.clone());

        assert_eq!(summary.version, SummaryVersion::SPEC057_ORDINARY_TYPE_V1);
        assert_eq!(summary.exported_types, vec![ty]);
        assert_eq!(summary.exported_constructors, vec![pending, done]);
        assert_eq!(summary.diagnostic_anchors, vec![anchor]);
    }

    #[test]
    fn reserved_extension_namespaces_default_empty_and_remain_uninterpreted() {
        let summary = ModuleSemanticSummary::new(module_identity());

        assert!(summary.reserved_identity_slots.is_empty());
        assert!(
            summary
                .reserved_identity_slots
                .future_type_functions
                .is_empty()
        );
        assert!(summary.reserved_identity_slots.sealed_domains.is_empty());
        assert!(
            summary
                .reserved_identity_slots
                .generalized_projections
                .is_empty()
        );
        assert!(
            summary
                .reserved_identity_slots
                .associated_families
                .is_empty()
        );
        assert!(summary.reserved_identity_slots.extensions.is_empty());
    }

    #[test]
    fn module_semantic_summary_deserializes_older_payloads_with_defaulted_extension_fields() {
        let mut value = serde_json::to_value(ModuleSemanticSummary::new(module_identity()))
            .expect("summary should serialize");
        let object = value.as_object_mut().expect("summary serializes as object");
        object.remove("interface_identities");
        object.remove("associated_member_identities");
        object.remove("reserved_identity_slots");
        object.remove("diagnostic_anchors");

        let summary: ModuleSemanticSummary =
            serde_json::from_value(value).expect("older summary payload should deserialize");

        assert!(summary.interface_identities.is_empty());
        assert!(summary.associated_member_identities.is_empty());
        assert!(summary.reserved_identity_slots.is_empty());
        assert!(summary.diagnostic_anchors.is_empty());
    }

    #[test]
    fn effect_row_visible_alias_preserves_immutable_provider_identity() {
        let provider_module = module_identity();
        let provider = EffectRowProviderIdentity::new(provider_module, "Audit");
        let declaration = EffectRowVisibleBinding::new(
            "Audit",
            provider.clone(),
            EffectRowBindingExposure::Declaration,
        );
        let alias = EffectRowVisibleBinding::new(
            "PublicAudit",
            provider.clone(),
            EffectRowBindingExposure::NamedImport,
        );

        assert_eq!(declaration.provider, provider);
        assert_eq!(alias.provider, provider);
        assert_ne!(declaration.visible_name, alias.visible_name);
        assert_ne!(declaration.exposure, alias.exposure);
        assert_eq!(
            alias.provider.declaration_name, "Audit",
            "an import alias must not manufacture a facade-owned provider identity"
        );
    }

    #[test]
    fn opaque_inaccessible_dependency_serializes_no_private_dependency_data() {
        let public_provider = EffectRowProviderIdentity::new(module_identity(), "Published");
        let binding = EffectRowVisibleBinding::new(
            "PublishedThroughFacade",
            public_provider,
            EffectRowBindingExposure::PublicReExport,
        )
        .with_closure_status(EffectRowClosureStatus::OpaqueInaccessibleDependency(
            OpaqueInaccessibleDependency,
        ));

        let json = serde_json::to_string(&binding)
            .expect("opaque inaccessible-dependency boundary should serialize");

        assert!(
            json.contains("opaque_inaccessible_dependency"),
            "the wire contract must carry only the stable opaque classification: {json}"
        );
        for private_detail in [
            "PRIVATE_DEPENDENCY_TOKEN_2025",
            "private/provider/path",
            "private-source-anchor-2025",
            "private-row-text-2025",
            "private-provider-id-2025",
        ] {
            assert!(
                !json.contains(private_detail),
                "opaque boundary serialization must not contain private dependency detail {private_detail:?}: {json}"
            );
        }
        assert!(
            !json.contains("inaccessible_named_dependencies"),
            "the old serializable Vec<Name> dependency transport is forbidden: {json}"
        );
    }

    #[test]
    fn opaque_effect_row_summary_serializes_only_its_opaque_boundary() {
        let secret = "PRIVATE_DEPENDENCY_TOKEN_2025";
        let secret_module = ModuleIdentity::new(
            Some(CrateId(99)),
            ModuleId(99),
            vec![secret.to_string()],
            ModuleSourceOrigin::Synthetic {
                reason: secret.to_string(),
            },
        );
        let mut row = EffectRowExportSummary::new(
            EffectRowExportId::new(secret_module.clone(), secret),
            secret,
            Visibility::Public,
            EffectRowExportClassification::TransparentAlias,
            vec![EffectRowItemSummary::new(secret)],
            SourceAnchor::new(
                SourceOrigin::Synthetic {
                    reason: secret.to_string(),
                },
                None,
                secret,
            ),
        );
        row.provider = EffectRowProviderIdentity::new(secret_module.clone(), secret);
        row.binding = EffectRowVisibleBinding::new(
            secret,
            EffectRowProviderIdentity::new(secret_module, secret),
            EffectRowBindingExposure::PublicReExport,
        )
        .with_closure_status(EffectRowClosureStatus::OpaqueInaccessibleDependency(
            OpaqueInaccessibleDependency,
        ));

        let json =
            serde_json::to_string(&row).expect("opaque effect-row boundary should serialize");

        assert!(json.contains("opaque_inaccessible_dependency"), "{json}");
        assert!(
            !json.contains(secret),
            "opaque row serialization must disclose neither row payload nor identity/anchor detail: {json}"
        );
        for forbidden_field in [
            "row_items",
            "source_anchor",
            "id",
            "exported_name",
            "provider",
            "binding",
        ] {
            assert!(
                !json.contains(forbidden_field),
                "opaque row serialization must not expose {forbidden_field}: {json}"
            );
        }
    }

    #[test]
    fn opaque_effect_row_summary_round_trips_as_an_unusable_opaque_boundary() {
        let mut row = EffectRowExportSummary::new(
            EffectRowExportId::new(module_identity(), "PrivateRow"),
            "PrivateRow",
            Visibility::Public,
            EffectRowExportClassification::TransparentAlias,
            vec![EffectRowItemSummary::new("PRIVATE_DEPENDENCY_TOKEN_2025")],
            SourceAnchor::new(
                SourceOrigin::Synthetic {
                    reason: "PRIVATE_DEPENDENCY_TOKEN_2025".to_string(),
                },
                None,
                "PRIVATE_DEPENDENCY_TOKEN_2025",
            ),
        );
        row.mark_opaque_inaccessible_dependency();

        let json = serde_json::to_string(&row).expect("opaque boundary should serialize");
        let round_tripped: EffectRowExportSummary =
            serde_json::from_str(&json).expect("opaque boundary should deserialize fail closed");

        assert!(matches!(
            round_tripped.binding.closure_status,
            EffectRowClosureStatus::OpaqueInaccessibleDependency(_)
        ));
        assert!(round_tripped.row_items.is_empty());
        assert_ne!(round_tripped.exported_name, "PrivateRow");
        assert!(
            !serde_json::to_string(&round_tripped)
                .expect("round-tripped opaque boundary should serialize")
                .contains("PRIVATE_DEPENDENCY_TOKEN_2025")
        );
    }

    #[test]
    fn complete_shaped_opaque_effect_row_payload_is_rejected() {
        let secret = "PRIVATE_DEPENDENCY_TOKEN_2025";
        let mut row = EffectRowExportSummary::new(
            EffectRowExportId::new(module_identity(), secret),
            secret,
            Visibility::Public,
            EffectRowExportClassification::TransparentAlias,
            vec![EffectRowItemSummary::new(secret)],
            SourceAnchor::new(
                SourceOrigin::Synthetic {
                    reason: secret.to_string(),
                },
                None,
                secret,
            ),
        );
        row.mark_opaque_inaccessible_dependency();
        let complete_shaped = serde_json::json!({
            "id": row.id,
            "exported_name": row.exported_name,
            "provider": row.provider,
            "binding": row.binding,
            "visibility": row.visibility,
            "classification": row.classification,
            "authority": row.authority,
            "row_items": row.row_items,
            "source_anchor": row.source_anchor,
        });

        let error = serde_json::from_value::<EffectRowExportSummary>(complete_shaped)
            .expect_err("only an exact opaque wire form may represent an opaque boundary");
        assert!(error.to_string().contains("opaque"));
    }

    #[test]
    fn visible_binding_without_closure_status_is_rejected() {
        let value = serde_json::json!({
            "visible_name": "Published",
            "provider": {
                "module": module_identity(),
                "declaration_name": "Published"
            },
            "exposure": "public_re_export"
        });

        let error = serde_json::from_value::<EffectRowVisibleBinding>(value)
            .expect_err("missing closure status must not silently become usable");
        assert!(error.to_string().contains("closure_status"));
    }
}
