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

use crate::ast::{Expr, Visibility};
use crate::kind::Kind;
use crate::runtime::FailureBoundary;
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

/// Stable identity allocated for one explicit source type hole.
///
/// The numeric value is assigned by the lowering/type-checking owner and is kept
/// separate from source spans so diagnostics can move without changing the hole's
/// semantic identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TypeHoleId(u64);

impl TypeHoleId {
    /// Creates a stable type-hole identity from an already allocated number.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the stable numeric identity for this hole.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Ambiguity classification preserved for an explicit source type hole.
///
/// Later type-checking tasks may refine these states into diagnostics; this core
/// carrier only records the distinction without deciding acceptance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TypeHoleAmbiguity {
    /// The hole is attached to a known value-position type argument.
    ExpectedValueSlot,
    /// The hole has not yet been associated with a unique expected slot.
    Ambiguous,
    /// The hole is known to appear in an unsupported position for the current MVP.
    UnsupportedPosition,
}

/// Source and kind metadata attached to an explicit source type hole.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeHoleMetadata {
    /// Stable hole identity.
    pub id: TypeHoleId,
    /// Source anchor for diagnostics; never participates in identity allocation.
    pub source_anchor: SourceAnchor,
    /// Expected kind if the owner has one at carrier construction time.
    pub expected_kind: Option<Kind>,
    /// Ambiguity state preserved for later validation/diagnostics.
    pub ambiguity: TypeHoleAmbiguity,
}

impl TypeHoleMetadata {
    /// Creates metadata for an explicit source type hole.
    #[must_use]
    pub fn new(
        id: TypeHoleId,
        source_anchor: SourceAnchor,
        expected_kind: Option<Kind>,
        ambiguity: TypeHoleAmbiguity,
    ) -> Self {
        Self {
            id,
            source_anchor,
            expected_kind,
            ambiguity,
        }
    }
}

/// Typed identity for a type-constructor head used by partial applications.
///
/// This prevents partial constructor terms from being encoded as saturated
/// `CanonicalTypeExpr::NominalApp` values with fake arguments. The visible name
/// is diagnostic/display metadata; the stable identity is the typed variant data.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TypeConstructorHeadId {
    /// Ordinary nominal type declaration used as a constructor head.
    Nominal {
        origin: TypeDeclId,
        visible_name: String,
    },
    /// Computation-grade type head used as a constructor head.
    Computation(TypeComputationHeadId),
}

impl TypeConstructorHeadId {
    /// Creates a nominal type-constructor head identity.
    #[must_use]
    pub fn nominal(origin: TypeDeclId, visible_name: impl Into<String>) -> Self {
        Self::Nominal {
            origin,
            visible_name: visible_name.into(),
        }
    }

    /// Creates a computation type-constructor head identity.
    #[must_use]
    pub const fn computation(head: TypeComputationHeadId) -> Self {
        Self::Computation(head)
    }
}

/// One argument in a partial type-constructor application.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PartialTypeArg {
    /// A concrete canonical type expression supplied at this argument position.
    Applied(Box<CanonicalTypeExpr>),
    /// An explicit source hole occupying this argument position.
    Hole(TypeHoleId),
}

/// Carrier for an explicitly partial type-constructor application.
///
/// The `args` spine may mix applied arguments and explicit holes. `result_kind`
/// records the effective kind after applying supplied arguments and abstracting
/// holes; this module does not validate that kind.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartialTypeConstructorApp {
    /// Stable identity for the constructor head.
    pub head: TypeConstructorHeadId,
    /// Argument spine preserving explicit holes rather than fabricating values.
    pub args: Vec<PartialTypeArg>,
    /// Effective result kind owned by the checker/lowering producer.
    pub result_kind: Kind,
    /// Metadata for every explicit source hole referenced by `args`.
    ///
    /// This keeps source anchors, expected kinds, and ambiguity state attached to
    /// the carrier instead of requiring consumers to reconstruct hole facts from
    /// a parallel debug/string channel.
    pub hole_metadata: Vec<TypeHoleMetadata>,
    /// Optional source anchor for the application as a whole.
    pub source_anchor: Option<SourceAnchor>,
}

impl PartialTypeConstructorApp {
    /// Creates a carrier for a partial type-constructor application.
    #[must_use]
    pub fn new(
        head: TypeConstructorHeadId,
        args: Vec<PartialTypeArg>,
        result_kind: Kind,
        source_anchor: Option<SourceAnchor>,
    ) -> Self {
        Self::new_with_hole_metadata(head, args, result_kind, Vec::new(), source_anchor)
    }

    /// Creates a carrier for a partial type-constructor application with
    /// explicit metadata for each hole in the argument spine.
    #[must_use]
    pub fn new_with_hole_metadata(
        head: TypeConstructorHeadId,
        args: Vec<PartialTypeArg>,
        result_kind: Kind,
        hole_metadata: Vec<TypeHoleMetadata>,
        source_anchor: Option<SourceAnchor>,
    ) -> Self {
        Self {
            head,
            args,
            result_kind,
            hole_metadata,
            source_anchor,
        }
    }

    /// Returns metadata for the requested hole identity if it is carried by this
    /// partial application.
    #[must_use]
    pub fn metadata_for_hole(&self, hole: TypeHoleId) -> Option<&TypeHoleMetadata> {
        self.hole_metadata
            .iter()
            .find(|metadata| metadata.id == hole)
    }
}

/// Shared source/core binder for a named type parameter with an explicit kind.
///
/// This carrier is intentionally semantic substrate only. Parser and TypeEnv
/// tasks decide when source syntax may create it and how bounds are validated.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KindedTypeBinder {
    /// Source-visible binder name.
    pub name: String,
    /// Kind assigned to the binder, such as `*` or `* -> *`.
    pub kind: Kind,
    /// Optional source anchor for diagnostics.
    pub source_anchor: Option<SourceAnchor>,
    /// Interface/proposition bounds attached to this binder when available.
    pub bounds: Vec<KindedTypeBound>,
}

impl KindedTypeBinder {
    /// Creates a kinded type binder carrier.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        kind: Kind,
        source_anchor: Option<SourceAnchor>,
        bounds: Vec<KindedTypeBound>,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            source_anchor,
            bounds,
        }
    }
}

/// Interface-bound metadata attached to a kinded type binder.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KindedTypeBound {
    /// Interface identity for the bound.
    pub interface: InterfaceIdentityId,
    /// Additional interface argument spine, if the bound is not unary.
    pub args: Vec<TypeConstructorExpr>,
    /// Optional source anchor for diagnostics.
    pub source_anchor: Option<SourceAnchor>,
}

impl KindedTypeBound {
    /// Creates kinded binder bound metadata.
    #[must_use]
    pub fn new(
        interface: InterfaceIdentityId,
        args: Vec<TypeConstructorExpr>,
        source_anchor: Option<SourceAnchor>,
    ) -> Self {
        Self {
            interface,
            args,
            source_anchor,
        }
    }
}

/// Reference to a constructor-kinded type variable.
///
/// The name is scoped by the owning binder environment; this carrier is separate
/// from nominal type identities so `M<A>` cannot be confused with a type named
/// `M` applied to `A`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConstructorVariableRef {
    /// Binder name for the constructor variable.
    pub name: String,
    /// Kind assigned to the constructor variable.
    pub kind: Kind,
    /// Optional source anchor for diagnostics.
    pub source_anchor: Option<SourceAnchor>,
}

impl ConstructorVariableRef {
    /// Creates a constructor-variable reference carrier.
    #[must_use]
    pub fn new(name: impl Into<String>, kind: Kind, source_anchor: Option<SourceAnchor>) -> Self {
        Self {
            name: name.into(),
            kind,
            source_anchor,
        }
    }

    /// Creates a constructor-variable reference from a kinded binder.
    #[must_use]
    pub fn from_binder(binder: &KindedTypeBinder) -> Self {
        Self {
            name: binder.name.clone(),
            kind: binder.kind.clone(),
            source_anchor: binder.source_anchor.clone(),
        }
    }
}

/// Canonical application of a constructor variable to a type argument spine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConstructorVariableApp {
    /// Constructor-variable head being applied.
    pub constructor: ConstructorVariableRef,
    /// Proper type argument spine.
    pub args: Vec<CanonicalTypeExpr>,
    /// Result kind after applying the argument spine.
    pub kind: Kind,
    /// Optional source anchor for diagnostics.
    pub source_anchor: Option<SourceAnchor>,
}

impl ConstructorVariableApp {
    /// Creates a constructor-variable application carrier.
    #[must_use]
    pub fn new(
        constructor: ConstructorVariableRef,
        args: Vec<CanonicalTypeExpr>,
        kind: Kind,
        source_anchor: Option<SourceAnchor>,
    ) -> Self {
        Self {
            constructor,
            args,
            kind,
            source_anchor,
        }
    }
}

/// Canonical carrier for expressions that may denote proper types or constructors.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TypeConstructorExpr {
    /// A fully proper type expression of kind `*`.
    ProperType(CanonicalTypeExpr),
    /// A constructor head without partial arguments.
    ConstructorHead(TypeConstructorHeadId),
    /// An explicitly partial application that preserves holes structurally.
    PartialApplication(PartialTypeConstructorApp),
}

/// Stable identity for a TCIR statement in one typed computation expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TcirStatementId(u64);

impl TcirStatementId {
    /// Creates a TCIR statement identity from an already allocated number.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the numeric statement identity.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Typed computation-expression carrier for source `do:K` lowering.
///
/// TCIR preserves the source, target, evidence, boundary, lift, failure-boundary,
/// and entry-artifact provenance needed by later AMIR/bytecode lowering. It
/// is a structural carrier only; executable lowering remains owned by later
/// crates/tasks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TcirComputationExpression {
    /// Source anchor for the whole computation expression.
    pub source_anchor: SourceAnchor,
    /// Source target constructor and structural constructor identity.
    pub target: TcirDoTarget,
    /// Selected sequencing evidence used by lowering.
    pub evidence: TcirSelectedEvidence,
    /// Semantic boundary attributed to this computation expression.
    pub boundary_level: FailureBoundary,
    /// Result type of the typed computation expression.
    pub result_type: CanonicalTypeExpr,
    /// Source-order statement carriers with stable per-expression IDs.
    pub statements: Vec<TcirStatement>,
    /// Explicit cross-boundary lift provenance requested by source/library calls.
    pub explicit_lifts: Vec<TcirExplicitLiftProvenance>,
    /// Failure-boundary provenance retained for runtime/report lowering.
    pub failure_boundaries: Vec<TcirFailureBoundaryProvenance>,
}

/// TCIR source `do` target identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TcirDoTarget {
    /// Structural target constructor identity; display is not semantic identity.
    pub constructor: TypeConstructorExpr,
    /// Source-style display text for diagnostics.
    pub display: String,
    /// Source anchor for the target syntax.
    pub source_anchor: SourceAnchor,
}

/// Selected sequencing evidence retained at the TCIR boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TcirSelectedEvidence {
    /// Interface selected for sequencing, usually `Monad`.
    pub interface: String,
    /// Stable selected-evidence key from the producing typechecker.
    pub evidence_key: String,
    /// Return operation selected for final return lowering.
    pub return_op: TcirOperation,
    /// Bind operation selected for `<-` lowering.
    pub bind_op: TcirOperation,
}

/// Operation reference retained by TCIR for evidence and boundary operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TcirOperation {
    /// Operation identity. This is deliberately typed instead of debug-string only.
    pub kind: TcirOperationKind,
    /// Optional source/evidence anchor for diagnostics and traceability.
    pub source_anchor: Option<SourceAnchor>,
}

impl TcirOperation {
    /// Creates a hidden compiler-prelude operation reference.
    #[must_use]
    pub fn hidden_compiler_prelude(
        name: impl Into<String>,
        source_anchor: Option<SourceAnchor>,
    ) -> Self {
        Self {
            kind: TcirOperationKind::HiddenCompilerPrelude { name: name.into() },
            source_anchor,
        }
    }

    /// Creates a visible Ash operation reference.
    #[must_use]
    pub fn visible_operation(
        module_path: Vec<String>,
        name: impl Into<String>,
        source_anchor: Option<SourceAnchor>,
    ) -> Self {
        Self {
            kind: TcirOperationKind::VisibleOperation {
                module_path,
                name: name.into(),
            },
            source_anchor,
        }
    }

    /// Creates an evidence-selected method-body operation reference.
    #[must_use]
    pub fn evidence_method(
        evidence_key: impl Into<String>,
        method: impl Into<String>,
        params: Vec<String>,
        body: Expr,
        source_anchor: Option<SourceAnchor>,
    ) -> Self {
        Self {
            kind: TcirOperationKind::EvidenceMethod {
                evidence_key: evidence_key.into(),
                method: method.into(),
                params,
                body: Box::new(body),
            },
            source_anchor,
        }
    }

    /// Creates an evidence-selected intrinsic operation reference.
    #[must_use]
    pub fn evidence_intrinsic(
        evidence_key: impl Into<String>,
        method: impl Into<String>,
        module_path: Vec<String>,
        name: impl Into<String>,
        source_anchor: Option<SourceAnchor>,
    ) -> Self {
        Self {
            kind: TcirOperationKind::EvidenceIntrinsic {
                evidence_key: evidence_key.into(),
                method: method.into(),
                module_path,
                name: name.into(),
            },
            source_anchor,
        }
    }
}

/// Typed TCIR operation identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TcirOperationKind {
    /// Compiler-prelude operation kept hidden from ordinary source data.
    HiddenCompilerPrelude { name: String },
    /// Visible Ash operation such as `proc::from_act`.
    VisibleOperation {
        module_path: Vec<String>,
        name: String,
    },
    /// Selected user evidence method body, preserved as a parameterized closure.
    EvidenceMethod {
        evidence_key: String,
        method: String,
        params: Vec<String>,
        body: Box<Expr>,
    },
    /// Selected compiler intrinsic implementing an evidence method.
    EvidenceIntrinsic {
        evidence_key: String,
        method: String,
        module_path: Vec<String>,
        name: String,
    },
}

/// One source statement in a typed computation expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TcirStatement {
    /// Stable statement identity within the containing TCIR expression.
    pub id: TcirStatementId,
    /// Source anchor for this statement.
    pub source_anchor: SourceAnchor,
    /// Statement payload.
    pub kind: TcirStatementKind,
}

/// TCIR statement payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TcirStatementKind {
    /// Pure lexical binding inside a computation expression.
    Let {
        binder: TcirBinder,
        value: Box<Expr>,
    },
    /// Monadic bind with selected bind operation and continuation closure facts.
    Bind {
        binder: TcirBinder,
        source: Box<Expr>,
        bind_op: Box<TcirOperation>,
        closure: TcirClosure,
    },
    /// Final return with selected return operation.
    Return {
        value: Box<Expr>,
        return_op: Box<TcirOperation>,
    },
    /// Explicit cross-boundary lift requested by source/library operation.
    ExplicitLift { lift: TcirExplicitLiftProvenance },
    /// Failure-boundary provenance retained for later runtime/report lowering.
    FailureBoundary {
        boundary: TcirFailureBoundaryProvenance,
    },
}

/// TCIR source binder.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TcirBinder {
    /// Binder name.
    pub name: String,
    /// Optional binder source anchor.
    pub source_anchor: Option<SourceAnchor>,
}

/// Continuation closure facts for a TCIR bind.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TcirClosure {
    /// Source anchor for the continuation closure.
    pub source_anchor: SourceAnchor,
    /// Closure parameters.
    pub params: Vec<TcirBinder>,
    /// Statement IDs that belong to this continuation body.
    pub body_statement_ids: Vec<TcirStatementId>,
}

/// Provenance for an explicit cross-boundary lift.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TcirExplicitLiftProvenance {
    /// Operation that requested the lift.
    pub operation: TcirOperation,
    /// Source boundary.
    pub from_boundary: FailureBoundary,
    /// Destination boundary.
    pub to_boundary: FailureBoundary,
    /// Source anchor for the lift expression.
    pub source_anchor: SourceAnchor,
}

/// Failure-boundary provenance retained by TCIR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TcirFailureBoundaryProvenance {
    /// Boundary owning the boundary.
    pub boundary: FailureBoundary,
    /// Optional runtime entity identity when already known by the producer.
    ///
    /// Typechecking does not fabricate runtime UUIDs. Later runtime lowering may
    /// bind this boundary to an execution entity once an actual run/process/workflow
    /// identity exists.
    pub entity: Option<crate::runtime::FailureEntity>,
    /// Source anchor for the failure boundary.
    pub source_anchor: SourceAnchor,
    /// Human-readable notes for diagnostics; not semantic identity.
    pub notes: Vec<String>,
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
    ConstructorVariableApp(Box<ConstructorVariableApp>),
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
    ConstructorVariableApp {
        constructor: Box<ConstructorVariableRef>,
        args: Vec<NormalTypeExpr>,
        kind: Kind,
        /// Reason preserved while constructor-variable kinding and unification
        /// remain owned by later HKT TypeEnv tasks.
        reason: NormalFormBlockReason,
    },
    NeutralComputationApp {
        head: TypeComputationHeadId,
        args: Vec<NormalTypeExpr>,
        kind: Kind,
        /// Reason preserved when this neutral computation app is stuck.
        ///
        /// SPEC-060 requires neutral computation applications to carry a blocker
        /// reason; projections remain optional because imported projection carriers
        /// may predate diagnostic attribution.
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
        /// `None` is reserved for imported carriers that predate reason attribution;
        /// the Phase 112 normalizer always fills `Some(...)` for carriers it constructs.
        reason: Option<NormalFormBlockReason>,
    },
}
