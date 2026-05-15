//! Canonical internal type-expression and normal-form substrate.
//!
//! This module owns the shared computation-grade carriers introduced by Phase 110
//! plus the Phase 112 normal-form carrier needed by the typechecker normalizer:
//! - shared computation-head identity namespace;
//! - canonical distinction between nominal applications, projections, and
//!   computation-head applications;
//! - projection rigidity metadata;
//! - sealed-domain constructor normal forms backed by `DomainConstructorId` and
//!   `SealedDomainId`, not ordinary runtime `ConstructorId` values.
//!
//! It defines only carriers. Lowering, normalization algorithms, definitional
//! equality, and diagnostics are owned by later Phase 112 tasks.

use crate::ast::Visibility;
use crate::kind::Kind;
use crate::semantic_summary::{
    AssociatedMemberIdentityId, DomainConstructorId, InterfaceIdentityId, ModuleIdentity,
    ModuleSummaryRef, PromotedConstructorId, PromotedDataKindId, PropositionPredicateId,
    SealedDomainId, SourceAnchor, TypeDeclId, ValidatedDecreasesSummary,
};
use serde::{Deserialize, Serialize};

/// Identity for future explicit type-computation heads.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeComputationHeadId {
    pub module: ModuleIdentity,
    pub name: String,
}

impl TypeComputationHeadId {
    #[must_use]
    pub fn new(module: ModuleIdentity, name: impl Into<String>) -> Self {
        Self {
            module,
            name: name.into(),
        }
    }
}

/// Canonical identity for a reducible associated-family head.
///
/// The identity is the typed pair of the declaring interface and associated
/// member. It is intentionally not a display/debug string, so imports and
/// re-exports can preserve the same family head across module boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedFamilyHeadId {
    pub interface: InterfaceIdentityId,
    pub member: AssociatedMemberIdentityId,
}

/// Mode recorded when a projection is classified for associated-family handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssociatedFamilyProjectionMode {
    /// A simple associated-type projection with no reducible family table.
    OrdinaryAssociatedProjection,
    /// A sealed associated-family head whose checked equations may reduce.
    ReducibleSealedFamilyHead,
    /// A projection arising only from generic where-bound evidence.
    RigidWhereBoundProjection,
    /// Reduction is blocked, unavailable, private, or otherwise neutral.
    NeutralBlockedOrUnavailable,
}

/// Public classification returned by associated-family projection helper APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssociatedFamilyProjectionKind {
    OrdinaryAssociatedProjection,
    ReducibleSealedFamilyHead,
    RigidWhereBoundProjection,
    NeutralBlockedOrUnavailable,
}

/// Canonical projection carrier preserving a typed family head and argument spine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedFamilyProjection {
    pub head: AssociatedFamilyHeadId,
    pub interface_args: Vec<CanonicalTypeExpr>,
    pub kind: Kind,
    pub rigidity: ProjectionRigidity,
    pub mode: AssociatedFamilyProjectionMode,
}

impl AssociatedFamilyProjection {
    #[must_use]
    pub fn classification(&self) -> AssociatedFamilyProjectionKind {
        match self.mode {
            AssociatedFamilyProjectionMode::OrdinaryAssociatedProjection => {
                AssociatedFamilyProjectionKind::OrdinaryAssociatedProjection
            }
            AssociatedFamilyProjectionMode::ReducibleSealedFamilyHead => {
                AssociatedFamilyProjectionKind::ReducibleSealedFamilyHead
            }
            AssociatedFamilyProjectionMode::RigidWhereBoundProjection => {
                AssociatedFamilyProjectionKind::RigidWhereBoundProjection
            }
            AssociatedFamilyProjectionMode::NeutralBlockedOrUnavailable => {
                AssociatedFamilyProjectionKind::NeutralBlockedOrUnavailable
            }
        }
    }

    #[must_use]
    pub fn is_ordinary_associated_projection(&self) -> bool {
        self.classification() == AssociatedFamilyProjectionKind::OrdinaryAssociatedProjection
    }

    #[must_use]
    pub fn is_reducible_family_head(&self) -> bool {
        self.classification() == AssociatedFamilyProjectionKind::ReducibleSealedFamilyHead
    }

    #[must_use]
    pub fn is_rigid_where_bound_projection(&self) -> bool {
        self.classification() == AssociatedFamilyProjectionKind::RigidWhereBoundProjection
    }

    #[must_use]
    pub fn is_neutral_blocked_or_unavailable(&self) -> bool {
        self.classification() == AssociatedFamilyProjectionKind::NeutralBlockedOrUnavailable
    }
}

/// Whether a projection is currently rigid or neutral/stuck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionRigidity {
    Rigid,
    Neutral,
}

/// Canonical sealed-domain constructor application.
///
/// This names SPEC-059 marker constructors only. It must not carry runtime ADT
/// constructor identities or promoted data-constructor identities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DomainConstructorApp {
    pub constructor: DomainConstructorId,
    pub domain: SealedDomainId,
    pub args: Vec<CanonicalTypeExpr>,
    pub kind: Kind,
}

/// Canonical promoted data-constructor application.
///
/// This is distinct from ordinary nominal type applications and sealed-domain
/// marker-constructor applications.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PromotedConstructorApp {
    pub constructor: PromotedConstructorId,
    pub data_kind: PromotedDataKindId,
    pub args: Vec<CanonicalTypeExpr>,
    pub kind: Kind,
}

/// Closed type-level constructor application families.
///
/// This carrier prevents consumers from collapsing sealed-domain marker
/// constructors and promoted runtime ADT constructors into one namespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeLevelConstructorApp {
    SealedDomainConstructor(Box<DomainConstructorApp>),
    PromotedDataConstructor(Box<PromotedConstructorApp>),
}

/// Canonical internal type-expression substrate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CanonicalTypeExpr {
    Primitive(String),
    Var(String),
    NominalApp {
        origin: TypeDeclId,
        visible_name: String,
        args: Vec<CanonicalTypeExpr>,
        kind: Kind,
    },
    PromotedDataConstructorApp(Box<PromotedConstructorApp>),
    Projection {
        interface: InterfaceIdentityId,
        member: AssociatedMemberIdentityId,
        args: Vec<CanonicalTypeExpr>,
        kind: Kind,
        rigidity: ProjectionRigidity,
    },
    ComputationHeadApp {
        head: TypeComputationHeadId,
        args: Vec<CanonicalTypeExpr>,
        kind: Kind,
    },
}

/// Canonical type-level proposition crossing crate/module/cache/summary or stable
/// diagnostic boundaries.
///
/// This is a structural carrier only. It records the four proposition families
/// accepted by SPEC-064 without performing lowering, normalization, impl search,
/// or proof search in `ash-core`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeProposition {
    Equality(TypeEqualityProposition),
    Disequality(TypeDisequalityProposition),
    InterfaceBound(InterfaceBoundProposition),
    NamedPredicate(NamedPredicateProposition),
}

/// Operand for canonical propositions.
///
/// `CanonicalTypeExpr` intentionally carries only canonical type expressions,
/// including nominal/projection/computation-head applications and promoted
/// data-constructor applications. Sealed-domain marker constructors
/// such as `Cons<A, T>` are represented honestly by `DomainConstructorApp`
/// instead of being encoded as ordinary nominal types or debug strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypePropositionTerm {
    Canonical(CanonicalTypeExpr),
    DomainConstructorApp {
        constructor: DomainConstructorId,
        domain: SealedDomainId,
        args: Vec<TypePropositionTerm>,
        kind: Kind,
    },
}

/// Type-level equality proposition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeEqualityProposition {
    pub lhs: TypePropositionTerm,
    pub rhs: TypePropositionTerm,
}

/// Type-level disequality proposition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeDisequalityProposition {
    pub lhs: TypePropositionTerm,
    pub rhs: TypePropositionTerm,
}

/// Type-level interface-bound proposition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InterfaceBoundProposition {
    pub subject: TypePropositionTerm,
    pub interface: InterfaceIdentityId,
    pub interface_args: Vec<TypePropositionTerm>,
}

/// Explicit named predicate proposition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamedPredicateProposition {
    pub predicate: PropositionPredicateId,
    pub args: Vec<TypePropositionTerm>,
}

/// Boundary provenance for proposition outcomes.
///
/// Solver-private traces that never leave `ash-typeck` do not need this carrier;
/// it exists for facts crossing stable boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropositionBoundary {
    Local,
    ImportedSummary(ModuleSummaryRef),
}

/// Normalized term pair used by boundary evidence/refutation for equality and
/// disequality propositions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropositionTypeComparisonEvidence {
    pub lhs: NormalTypeExpr,
    pub rhs: NormalTypeExpr,
}

/// Shared boundary evidence rule names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropositionEvidenceRule {
    DefinitionalEquality,
    SealedDomainConstructorDisjointness,
    NominalHeadDisjointness,
    InScopeInterfaceBound,
    ConcreteImplEvidence,
    NamedPredicateAssumption,
    ImportedSummaryFact,
}

/// Shared boundary refutation reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropositionRefutationReason {
    DefinitionalEquality,
    ClosedHeadMismatch,
    InterfaceEvidenceNotFound,
    NamedPredicateRefuted,
    ImportedSummaryRefutation,
}

/// Shared boundary deferred reasons.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropositionDeferredKind {
    BlockedByNeutrality { blocker: NormalFormBlockReason },
    RigidAssociatedProjection,
    RequiresTypeFunctionInversion,
    RequiresAssociatedFamilyInversion,
    UnsupportedNamedPredicate,
    MissingInterfaceEvidence,
    UnsupportedProofSearch,
}

/// Boundary evidence for a satisfied proposition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropositionEvidence {
    pub proposition: TypeProposition,
    pub normalized_terms: Option<PropositionTypeComparisonEvidence>,
    pub rule: PropositionEvidenceRule,
    pub source_anchor: Option<SourceAnchor>,
    pub boundary: PropositionBoundary,
}

/// Boundary refutation for a proposition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropositionRefutation {
    pub proposition: TypeProposition,
    pub normalized_terms: Option<PropositionTypeComparisonEvidence>,
    pub reason: PropositionRefutationReason,
    pub source_anchor: Option<SourceAnchor>,
    pub boundary: PropositionBoundary,
}

/// Boundary deferred reason for a proposition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropositionDeferredReason {
    pub proposition: TypeProposition,
    pub kind: PropositionDeferredKind,
    pub source_anchor: Option<SourceAnchor>,
    pub no_inversion_boundary: bool,
}

/// Conservative proposition outcome crossing a stable boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropositionOutcome {
    Satisfied(PropositionEvidence),
    Refuted(PropositionRefutation),
    Deferred(PropositionDeferredReason),
}

/// Checked source-backed `type fn` declaration carrier.
///
/// The carrier is intentionally semantic-only: it preserves already-resolved
/// computation-head and sealed-domain identities for later TypeEnv validation and
/// normalizer registration, but it does not perform lowering, coverage checking,
/// recursion validation, or cross-module summary transport.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeFunctionDef {
    pub visibility: Visibility,
    pub head: TypeComputationHeadId,
    pub name: String,
    pub params: Vec<TypeFunctionParam>,
    pub return_type: CanonicalTypeExpr,
    pub return_kind: Kind,
    pub result_constraint: TypeFunctionResultConstraint,
    pub decreases: Option<String>,
    pub source_anchors: TypeFunctionSourceAnchors,
    pub equations: Vec<TypeFunctionEquation>,
}

/// Diagnostic anchors attached to a checked `type fn` definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeFunctionSourceAnchors {
    pub definition: SourceAnchor,
    pub decreases: Option<SourceAnchor>,
}

/// Checked type-function parameter metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeFunctionParam {
    pub name: String,
    pub ty: CanonicalTypeExpr,
    pub kind: Kind,
    pub domain_constraint: Option<SealedDomainId>,
    pub source_anchor: SourceAnchor,
}

/// One checked source equation, preserving source order and case-head anchoring.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeFunctionEquation {
    pub head: TypeComputationHeadId,
    pub ordinal: usize,
    pub patterns: Vec<TypeFunctionPattern>,
    pub result: TypeFunctionResultExpr,
    pub source_anchor: SourceAnchor,
    pub case_head_anchor: SourceAnchor,
}

/// Kind/domain constraint inherited by a checked type-level pattern position.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeFunctionPatternConstraint {
    Kind(Kind),
    Domain(SealedDomainId),
}

/// Checked source type-level pattern carrier for `type fn` equations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeFunctionPattern {
    DomainConstructor {
        constructor: Box<DomainConstructorId>,
        domain: Box<SealedDomainId>,
        fields: Vec<TypeFunctionPattern>,
        constraint: TypeFunctionPatternConstraint,
        source_anchor: SourceAnchor,
    },
    Var {
        name: String,
        constraint: TypeFunctionPatternConstraint,
        source_anchor: SourceAnchor,
    },
    Wildcard {
        constraint: TypeFunctionPatternConstraint,
        source_anchor: SourceAnchor,
    },
}

/// Kind/domain constraint for a checked type-function result expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeFunctionResultConstraint {
    Kind(Kind),
    Domain(SealedDomainId),
}

/// Checked source-equation result expression carrier.
///
/// This deliberately mirrors the canonical type-expression head families while
/// adding `DomainConstructorApp`, so sealed-domain marker constructors are never
/// encoded as ordinary nominal constructors before normalization.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeFunctionResultExpr {
    Primitive {
        name: String,
        kind: Kind,
        constraint: TypeFunctionResultConstraint,
        source_anchor: SourceAnchor,
    },
    Var {
        name: String,
        kind: Kind,
        constraint: TypeFunctionResultConstraint,
        source_anchor: SourceAnchor,
    },
    NominalApp {
        origin: TypeDeclId,
        visible_name: String,
        args: Vec<TypeFunctionResultExpr>,
        kind: Kind,
        constraint: TypeFunctionResultConstraint,
        source_anchor: SourceAnchor,
    },
    DomainConstructorApp {
        constructor: DomainConstructorId,
        domain: SealedDomainId,
        args: Vec<TypeFunctionResultExpr>,
        kind: Kind,
        constraint: TypeFunctionResultConstraint,
        source_anchor: SourceAnchor,
    },
    PromotedDataConstructorApp {
        constructor: Box<PromotedConstructorId>,
        data_kind: Box<PromotedDataKindId>,
        args: Vec<TypeFunctionResultExpr>,
        kind: Kind,
        constraint: TypeFunctionResultConstraint,
        source_anchor: SourceAnchor,
    },
    Projection {
        interface: InterfaceIdentityId,
        member: AssociatedMemberIdentityId,
        args: Vec<TypeFunctionResultExpr>,
        kind: Kind,
        constraint: TypeFunctionResultConstraint,
        rigidity: ProjectionRigidity,
        source_anchor: SourceAnchor,
    },
    ComputationHeadApp {
        head: TypeComputationHeadId,
        args: Vec<TypeFunctionResultExpr>,
        kind: Kind,
        constraint: TypeFunctionResultConstraint,
        source_anchor: SourceAnchor,
    },
}

/// Checked source-backed associated-family scheme carrier.
///
/// This is a structural carrier only. It preserves typed family identities,
/// sealed-domain constructor patterns, RHS projections, decreases evidence, and
/// source anchors for later TypeEnv validation/normalizer registration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedFamilyScheme {
    pub head: AssociatedFamilyHeadId,
    pub params: Vec<AssociatedFamilySchemeParam>,
    pub result_domain: CanonicalTypeExpr,
    pub result_kind: Kind,
    pub equations: Vec<AssociatedFamilyEquation>,
    pub source_anchor: SourceAnchor,
}

/// Checked associated-family scheme parameter metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedFamilySchemeParam {
    pub name: String,
    pub ty: CanonicalTypeExpr,
    pub kind: Kind,
    pub domain_constraint: Option<SealedDomainId>,
    pub source_anchor: SourceAnchor,
}

/// One checked associated-family equation, preserving source order and case head.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssociatedFamilyEquation {
    pub head: AssociatedFamilyHeadId,
    pub ordinal: usize,
    pub interface_arg_patterns: Vec<AssociatedFamilyPattern>,
    pub result: AssociatedFamilyResultExpr,
    pub decreases: Option<ValidatedDecreasesSummary>,
    pub source_anchor: SourceAnchor,
    pub case_head_anchor: SourceAnchor,
}

/// Kind/domain constraint for associated-family patterns and result expressions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssociatedFamilyResultConstraint {
    Kind(Kind),
    Domain(SealedDomainId),
}

/// Checked source type-level pattern carrier for associated-family equations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssociatedFamilyPattern {
    DomainConstructor {
        constructor: Box<DomainConstructorId>,
        domain: Box<SealedDomainId>,
        fields: Vec<AssociatedFamilyPattern>,
        constraint: AssociatedFamilyResultConstraint,
        source_anchor: SourceAnchor,
    },
    NominalApp {
        origin: TypeDeclId,
        visible_name: String,
        args: Vec<AssociatedFamilyPattern>,
        constraint: AssociatedFamilyResultConstraint,
        source_anchor: SourceAnchor,
    },
    Primitive {
        name: String,
        constraint: AssociatedFamilyResultConstraint,
        source_anchor: SourceAnchor,
    },
    Var {
        name: String,
        constraint: AssociatedFamilyResultConstraint,
        source_anchor: SourceAnchor,
    },
    Wildcard {
        constraint: AssociatedFamilyResultConstraint,
        source_anchor: SourceAnchor,
    },
}

/// Checked source-equation RHS carrier for associated-family computation.
///
/// This mirrors the canonical type-expression head families while adding
/// `DomainConstructorApp` and preserving recursive associated-family projections
/// without encoding marker constructors or family calls as debug strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssociatedFamilyResultExpr {
    Primitive {
        name: String,
        kind: Kind,
        constraint: AssociatedFamilyResultConstraint,
        source_anchor: SourceAnchor,
    },
    Var {
        name: String,
        kind: Kind,
        constraint: AssociatedFamilyResultConstraint,
        source_anchor: SourceAnchor,
    },
    NominalApp {
        origin: TypeDeclId,
        visible_name: String,
        args: Vec<AssociatedFamilyResultExpr>,
        kind: Kind,
        constraint: AssociatedFamilyResultConstraint,
        source_anchor: SourceAnchor,
    },
    DomainConstructorApp {
        constructor: DomainConstructorId,
        domain: SealedDomainId,
        args: Vec<AssociatedFamilyResultExpr>,
        kind: Kind,
        constraint: AssociatedFamilyResultConstraint,
        source_anchor: SourceAnchor,
    },
    AssociatedFamilyProjection {
        head: AssociatedFamilyHeadId,
        interface_args: Vec<AssociatedFamilyResultExpr>,
        kind: Kind,
        constraint: AssociatedFamilyResultConstraint,
        rigidity: ProjectionRigidity,
        source_anchor: SourceAnchor,
    },
    Projection {
        interface: InterfaceIdentityId,
        member: AssociatedMemberIdentityId,
        args: Vec<AssociatedFamilyResultExpr>,
        kind: Kind,
        constraint: AssociatedFamilyResultConstraint,
        rigidity: ProjectionRigidity,
        source_anchor: SourceAnchor,
    },
    ComputationHeadApp {
        head: TypeComputationHeadId,
        args: Vec<AssociatedFamilyResultExpr>,
        kind: Kind,
        constraint: AssociatedFamilyResultConstraint,
        source_anchor: SourceAnchor,
    },
}

/// Reason metadata for normal forms that are stuck/blocked rather than reducible.
///
/// The normalizer and diagnostics may refine which reason applies in later tasks;
/// the carrier lives in `ash-core` so neutral computation applications and rigid
/// or neutral projections can transport that distinction structurally.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalFormBlockReason {
    AbstractScrutinee,
    NeutralScrutinee,
    RigidProjection,
    MissingAssociatedEvidence,
    AssociatedFamilyNotSealed,
    AmbiguousAssociatedFamilySelection,
    AssociatedFamilyLocalUnavailable,
    ImportedAssociatedFamilyUnsupported,
    Unsupported,
}

/// Shared normal-form carrier produced by the Phase 112 type-expression normalizer.
///
/// This is a structural carrier only: it does not perform normalization or encode
/// reduction rules. It deliberately keeps sealed-domain marker constructors in a
/// dedicated `DomainConstructorApp` variant backed by `DomainConstructorId` and
/// `SealedDomainId`, so they cannot be confused with ordinary runtime/ADT
/// constructor identities. Neutral computation heads and projections retain their
/// normalized argument spines for later definitional equality.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NormalTypeExpr {
    Primitive(String),
    Var(String),
    NominalApp {
        origin: TypeDeclId,
        visible_name: String,
        args: Vec<NormalTypeExpr>,
        kind: Kind,
    },
    DomainConstructorApp {
        constructor: DomainConstructorId,
        domain: SealedDomainId,
        args: Vec<NormalTypeExpr>,
        kind: Kind,
    },
    PromotedDataConstructorApp {
        constructor: Box<PromotedConstructorId>,
        data_kind: Box<PromotedDataKindId>,
        args: Vec<NormalTypeExpr>,
        kind: Kind,
    },
    NeutralComputationApp {
        head: TypeComputationHeadId,
        args: Vec<NormalTypeExpr>,
        kind: Kind,
        /// Reason preserved when this neutral computation app is stuck.
        ///
        /// SPEC-060 requires neutral computation applications to carry a blocker
        /// reason; projections remain optional because legacy/imported projection
        /// carriers may predate diagnostic attribution.
        reason: NormalFormBlockReason,
    },
    Projection {
        interface: InterfaceIdentityId,
        member: AssociatedMemberIdentityId,
        args: Vec<NormalTypeExpr>,
        kind: Kind,
        rigidity: ProjectionRigidity,
        /// Reason preserved when this projection remains stuck.
        ///
        /// `None` is reserved for imported or legacy carriers that predate reason
        /// attribution; the Phase 112 normalizer always fills `Some(...)` for
        /// carriers it constructs.
        reason: Option<NormalFormBlockReason>,
    },
}
