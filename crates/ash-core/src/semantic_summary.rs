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

use crate::ast::{Name, Span, TypeBody, TypeVar, Visibility};
use crate::kind::Kind;
use crate::module_graph::{CrateId, ModuleId};
use crate::type_ir::{
    CanonicalTypeExpr, TypeComputationHeadId, TypeFunctionEquation, TypeFunctionResultConstraint,
    TypeFunctionSourceAnchors,
};
use serde::{Deserialize, Serialize};

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

/// Summary for one ordinary type declaration visible in a module summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDeclSummary {
    pub id: TypeDeclId,
    pub exported_name: Name,
    pub visibility: Visibility,
    pub params: Vec<TypeVar>,
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
}

/// Core schema-level validation failures for semantic-summary version contracts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleSemanticSummaryValidationError {
    /// Public type-computation facts are only valid in SPEC-062/V3 summaries.
    TypeFunctionsRequireV3 { version: SummaryVersion },
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
            source_anchor,
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
    /// Public type-function summaries exported from this module (SPEC-062 §6).
    ///
    /// `#[serde(default)]` preserves V1/V2 wire compatibility. Non-empty values
    /// are valid only when `version` is SPEC-062/V3.
    #[serde(default)]
    pub exported_type_functions: Vec<TypeFunctionSummary>,
}

impl ModuleSemanticSummary {
    #[must_use]
    pub fn new(module: ModuleIdentity) -> Self {
        Self {
            module,
            version: SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
            exported_types: Vec::new(),
            exported_constructors: Vec::new(),
            re_exports: Vec::new(),
            imported_summary_refs: Vec::new(),
            interface_identities: Vec::new(),
            associated_member_identities: Vec::new(),
            reserved_identity_slots: ReservedSemanticIdentitySlots::default(),
            diagnostic_anchors: Vec::new(),
            exported_sealed_domains: Vec::new(),
            exported_type_functions: Vec::new(),
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

    /// Add a public type-function summary to this module summary.
    #[must_use]
    pub fn with_exported_type_function(mut self, type_function: TypeFunctionSummary) -> Self {
        self.exported_type_functions.push(type_function);
        self
    }

    /// Build the semantic content key used by in-memory import dedup/cache boundaries.
    ///
    /// The key is intentionally structural and process-local: it is suitable for
    /// comparing summaries already decoded into this core version, but it is not a
    /// stable persistent-cache digest format. A future persistent cache should feed
    /// at least these inputs into an explicit digest algorithm: summary schema
    /// version, module identity, ordinary exported type/constructor facts,
    /// imported summary refs, sealed-domain summaries, public type-function
    /// signatures/equations, type-function dependency refs/digests, and compiler
    /// algorithm version metadata.
    #[must_use]
    pub fn semantic_cache_key(&self) -> Vec<String> {
        let mut key = Vec::new();
        key.push(format!("version::{:?}", self.version));
        key.push(format!("module::{:?}", self.module));
        key.extend(self.exported_types.iter().map(|ty| {
            format!(
                "type::{}::{:?}::{:?}::{:?}::{:?}::{:?}",
                ty.exported_name,
                ty.id,
                ty.visibility,
                ty.params,
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
        match self.version {
            SummaryVersion::SPEC057_ORDINARY_TYPE_V1 | SummaryVersion::SPEC059_SEALED_DOMAIN_V2 => {
                if self.exported_type_functions.is_empty() {
                    Ok(())
                } else {
                    Err(
                        ModuleSemanticSummaryValidationError::TypeFunctionsRequireV3 {
                            version: self.version,
                        },
                    )
                }
            }
            SummaryVersion::SPEC062_TYPE_COMPUTATION_V3 => Ok(()),
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
    fn ordinary_type_summary_does_not_replace_workflow_summary_carrier() {
        let summary = ModuleSemanticSummary::new(module_identity());
        let workflow_summary = crate::workflow_carrier::PublicWorkflowSummary::default();

        assert!(summary.exported_types.is_empty());
        assert_eq!(workflow_summary.node_count, 0);
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
}
