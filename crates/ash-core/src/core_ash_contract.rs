//! Core Ash contract predicate sidecar carriers.
//!
//! These types are the Phase 165/TASK-1694 substrate for contract-position
//! predicates. They intentionally model predicates as typed Core metadata rather
//! than executable source text: source snippets are retained for diagnostics, but
//! predicate identity is computed from the lowered tree, binders, snapshots,
//! admitted predicate functions, and type encodings.

use crate::core_ash::{CoreName, CorePath, CoreSourceSpan, CoreType};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::{Debug, Write as _};

/// Core-prefixed alias for [`PredicateId`].
pub type CorePredicateId = PredicateId;
/// Core-prefixed alias for [`PredicateHash`].
pub type CorePredicateHash = PredicateHash;
/// Core-prefixed alias for [`PredicateRef`].
pub type CorePredicateRef = PredicateRef;
/// Core-prefixed alias for [`PredicateEnvRef`].
pub type CorePredicateEnvRef = PredicateEnvRef;
/// Core-prefixed alias for [`PredicateBinderId`].
pub type CorePredicateBinderId = PredicateBinderId;
/// Core-prefixed alias for [`PredicateBinderRef`].
pub type CorePredicateBinderRef = PredicateBinderRef;
/// Core-prefixed alias for [`PredicateBinderKind`].
pub type CorePredicateBinderKind = PredicateBinderKind;
/// Core-prefixed alias for [`PredicateBinder`].
pub type CorePredicateBinder = PredicateBinder;
/// Core-prefixed alias for [`SnapshotRef`].
pub type CoreSnapshotRef = SnapshotRef;
/// Core-prefixed alias for [`PredicateFunctionRef`].
pub type CorePredicateFunctionRef = PredicateFunctionRef;
/// Core-prefixed alias for [`PredicateEnvironment`].
pub type CorePredicateEnvironment = PredicateEnvironment;
/// Core-prefixed alias for [`PredicateNode`].
pub type CorePredicateNode = PredicateNode;
/// Core-prefixed alias for [`PredicateClassification`].
pub type CorePredicateClassification = PredicateClassification;
/// Core-prefixed alias for [`DynamicPredicatePlan`].
pub type CoreDynamicPredicatePlan = DynamicPredicatePlan;
/// Core-prefixed alias for [`LoweredPredicate`].
pub type CoreLoweredPredicate = LoweredPredicate;
/// Core-prefixed alias for [`LoweredPredicateBuilder`].
pub type CoreLoweredPredicateBuilder = LoweredPredicateBuilder;
/// Core-prefixed alias for [`DiagnosticShape`].
pub type CoreDiagnosticShape = DiagnosticShape;
/// Core-prefixed alias for [`ContractRecoverability`].
pub type CoreContractRecoverability = ContractRecoverability;
/// Core-prefixed alias for [`RuntimeCheckPlan`].
pub type CoreRuntimeCheckPlan = RuntimeCheckPlan;
/// Core-prefixed alias for [`EvidenceRef`].
pub type CoreEvidenceRef = EvidenceRef;

/// Stable contract boundary identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CoreBoundaryId(String);

impl CoreBoundaryId {
    /// Creates a boundary identifier.
    #[must_use]
    pub fn new(boundary: impl Into<String>) -> Self {
        Self(boundary.into())
    }

    /// Returns the textual boundary identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for CoreBoundaryId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for CoreBoundaryId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Stable lowered-predicate identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PredicateId(String);

impl PredicateId {
    /// Creates a predicate identifier.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the textual predicate id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable content hash for lowered predicate semantics.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PredicateHash(String);

impl PredicateHash {
    /// Creates a predicate hash from a hex string.
    #[must_use]
    pub fn new(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    /// Returns the hex hash string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Reference to a lowered predicate sidecar.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PredicateRef {
    pub id: PredicateId,
    pub stable_hash: PredicateHash,
    pub boundary: CoreBoundaryId,
    pub source_span: Option<CoreSourceSpan>,
}

/// Reference to a predicate environment sidecar.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PredicateEnvRef(String);

impl PredicateEnvRef {
    /// Creates an environment reference.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the textual environment id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Predicate binder identity scoped to a contract boundary.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PredicateBinderId {
    boundary: CoreBoundaryId,
    local: String,
}

impl PredicateBinderId {
    /// Creates a boundary-local binder identity.
    #[must_use]
    pub fn new(boundary: impl Into<CoreBoundaryId>, local: impl Into<String>) -> Self {
        Self {
            boundary: boundary.into(),
            local: local.into(),
        }
    }

    /// Returns the boundary that owns this binder.
    #[must_use]
    pub fn boundary(&self) -> &CoreBoundaryId {
        &self.boundary
    }

    /// Returns the local binder id within its boundary.
    #[must_use]
    pub fn local(&self) -> &str {
        &self.local
    }
}

/// Reference to a predicate binder.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PredicateBinderRef {
    id: PredicateBinderId,
}

impl PredicateBinderRef {
    /// Creates a binder reference from a binder id.
    #[must_use]
    pub fn new(id: PredicateBinderId) -> Self {
        Self { id }
    }

    /// Returns the referenced binder id.
    #[must_use]
    pub fn id(&self) -> &PredicateBinderId {
        &self.id
    }
}

/// Boundary role for a lowered predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoundaryKind {
    /// Boundary kind has not yet been specialized by lowering.
    Unspecified,
    Requires,
    Ensures,
    Invariant,
    Guard,
    Law,
    Composition,
}

/// Predicate binder role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredicateBinderKind {
    Lexical,
    Parameter,
    Result,
    Message,
    LawParameter,
    IntermediateBindValue,
}

/// Typed binder admitted into a predicate environment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PredicateBinder {
    id: PredicateBinderId,
    name: CoreName,
    kind: PredicateBinderKind,
    ty: CoreType,
    source_span: CoreSourceSpan,
}

impl PredicateBinder {
    /// Creates a boundary-local typed predicate binder.
    #[must_use]
    pub fn new(
        boundary: impl Into<CoreBoundaryId>,
        local_id: impl Into<String>,
        name: impl Into<CoreName>,
        kind: PredicateBinderKind,
        ty: CoreType,
        source_span: CoreSourceSpan,
    ) -> Self {
        Self {
            id: PredicateBinderId::new(boundary, local_id),
            name: name.into(),
            kind,
            ty,
            source_span,
        }
    }

    /// Returns this binder's identity.
    #[must_use]
    pub fn id(&self) -> &PredicateBinderId {
        &self.id
    }

    /// Returns a reference to this binder.
    #[must_use]
    pub fn ref_(&self) -> PredicateBinderRef {
        PredicateBinderRef::new(self.id.clone())
    }

    /// Returns this binder's kind.
    #[must_use]
    pub fn kind(&self) -> PredicateBinderKind {
        self.kind
    }

    /// Returns this binder's type.
    #[must_use]
    pub fn ty(&self) -> &CoreType {
        &self.ty
    }
}

/// Reference to an admitted predicate function.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PredicateFunctionRef {
    pub path: CorePath,
    pub arg_types: Vec<CoreType>,
    pub result_type: CoreType,
}

impl PredicateFunctionRef {
    /// Creates an admitted predicate-function reference with its type encoding.
    #[must_use]
    pub fn new(path: CorePath, arg_types: Vec<CoreType>, result_type: CoreType) -> Self {
        Self {
            path,
            arg_types,
            result_type,
        }
    }
}

/// Boundary-local snapshot reference for `old(path)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotRef {
    boundary: CoreBoundaryId,
    root: PredicateBinderId,
    path: Vec<CoreName>,
    ty: CoreType,
    source_span: CoreSourceSpan,
}

impl SnapshotRef {
    /// Creates a boundary-local snapshot reference.
    #[must_use]
    pub fn new(
        boundary: impl Into<CoreBoundaryId>,
        root: PredicateBinderId,
        path: Vec<CoreName>,
        ty: CoreType,
        source_span: CoreSourceSpan,
    ) -> Self {
        Self {
            boundary: boundary.into(),
            root,
            path,
            ty,
            source_span,
        }
    }

    /// Returns the owning boundary.
    #[must_use]
    pub fn boundary(&self) -> &CoreBoundaryId {
        &self.boundary
    }

    /// Returns the root binder of this snapshot.
    #[must_use]
    pub fn root(&self) -> &PredicateBinderId {
        &self.root
    }

    /// Returns the projection path of this snapshot relative to its root binder.
    #[must_use]
    pub fn path(&self) -> &[CoreName] {
        &self.path
    }

    /// Returns the declared type of the snapshot value.
    #[must_use]
    pub fn ty(&self) -> &CoreType {
        &self.ty
    }
}

/// Predicate sidecar environment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PredicateEnvironment {
    id: PredicateEnvRef,
    boundary: CoreBoundaryId,
    binders: Vec<PredicateBinder>,
    snapshots: Vec<SnapshotRef>,
    admitted_predicate_fns: Vec<PredicateFunctionRef>,
}

impl PredicateEnvironment {
    /// Creates a predicate environment. The id is derived from semantic content.
    #[must_use]
    pub fn new(
        boundary: impl Into<CoreBoundaryId>,
        binders: Vec<PredicateBinder>,
        snapshots: Vec<SnapshotRef>,
        admitted_predicate_fns: Vec<PredicateFunctionRef>,
    ) -> Self {
        let boundary = boundary.into();
        assert!(
            binders.iter().all(|binder| binder.id.boundary == boundary),
            "predicate binders must belong to the predicate environment boundary"
        );
        assert!(
            snapshots
                .iter()
                .all(|snapshot| snapshot.boundary == boundary),
            "predicate snapshots must belong to the predicate environment boundary"
        );
        let id = PredicateEnvRef::new(stable_digest(&(
            "PredicateEnvironment",
            &boundary,
            &binders,
            &snapshots,
            &admitted_predicate_fns,
        )));
        Self {
            id,
            boundary,
            binders,
            snapshots,
            admitted_predicate_fns,
        }
    }

    /// Returns this environment's reference.
    #[must_use]
    pub fn ref_(&self) -> PredicateEnvRef {
        self.id.clone()
    }

    /// Returns the owning boundary.
    #[must_use]
    pub fn boundary(&self) -> &CoreBoundaryId {
        &self.boundary
    }

    /// Returns admitted binders.
    #[must_use]
    pub fn binders(&self) -> &[PredicateBinder] {
        &self.binders
    }

    /// Returns admitted snapshots.
    #[must_use]
    pub fn snapshots(&self) -> &[SnapshotRef] {
        &self.snapshots
    }

    /// Returns admitted predicate functions.
    #[must_use]
    pub fn admitted_predicate_fns(&self) -> &[PredicateFunctionRef] {
        &self.admitted_predicate_fns
    }
}

/// Lowered predicate expression tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredicateNode {
    BoolLit(bool),
    IntLit(i128),
    StringLit(String),
    UnitLit,
    Binder(PredicateBinderRef),
    Result(PredicateBinderRef),
    Message(PredicateBinderRef),
    Snapshot(SnapshotRef),
    Field {
        base: Box<PredicateNode>,
        field: CoreName,
    },
    TupleIndex {
        base: Box<PredicateNode>,
        index: usize,
    },
    Not(Box<PredicateNode>),
    And(Box<PredicateNode>, Box<PredicateNode>),
    Or(Box<PredicateNode>, Box<PredicateNode>),
    Implies(Box<PredicateNode>, Box<PredicateNode>),
    Eq(Box<PredicateNode>, Box<PredicateNode>),
    Ne(Box<PredicateNode>, Box<PredicateNode>),
    Lt(Box<PredicateNode>, Box<PredicateNode>),
    Le(Box<PredicateNode>, Box<PredicateNode>),
    Gt(Box<PredicateNode>, Box<PredicateNode>),
    Ge(Box<PredicateNode>, Box<PredicateNode>),
    Add(Box<PredicateNode>, Box<PredicateNode>),
    Sub(Box<PredicateNode>, Box<PredicateNode>),
    Mul(Box<PredicateNode>, Box<PredicateNode>),
    Div(Box<PredicateNode>, Box<PredicateNode>),
    Rem(Box<PredicateNode>, Box<PredicateNode>),
    PredicateCall {
        callee: PredicateFunctionRef,
        args: Vec<PredicateNode>,
    },
}

/// Static-vs-dynamic predicate classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredicateClassification {
    Static,
    Dynamic,
}

/// Placeholder proof-fragment classification for the TASK-1694 carrier layer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProofFragment {
    FirstOrder,
    SmtSafe,
    UninterpretedFunctions,
}

/// Runtime evaluator profile for dynamic predicate plans.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum DynamicPredicatePlan {
    /// Authority-free interpreter evaluation over a captured predicate
    /// environment and snapshot map.
    Interpreter {
        /// Contract boundary kind (requires, ensures, invariant, ...).
        boundary_kind: BoundaryKind,
        /// Binders admitted by the captured predicate environment.
        environment_binders: Vec<PredicateBinder>,
        /// Predicate node to evaluate in the captured environment.
        predicate_node: PredicateNode,
    },
    /// Compiled evaluator keyed by a registered implementation name.
    Compiled(CoreName),
}

impl DynamicPredicatePlan {
    /// Boundary kind for this dynamic plan (requires, ensures, invariant, ...).
    #[must_use]
    pub fn boundary_kind(&self) -> BoundaryKind {
        match self {
            Self::Interpreter { boundary_kind, .. } => *boundary_kind,
            Self::Compiled(_) => BoundaryKind::Unspecified,
        }
    }

    /// Binders admitted by the predicate environment for this plan.
    #[must_use]
    pub fn environment_binders(&self) -> &[PredicateBinder] {
        match self {
            Self::Interpreter {
                environment_binders,
                ..
            } => environment_binders,
            Self::Compiled(_) => &[],
        }
    }

    /// Predicate node evaluated by this plan.
    #[must_use]
    pub fn predicate_node(&self) -> &PredicateNode {
        match self {
            Self::Interpreter { predicate_node, .. } => predicate_node,
            Self::Compiled(_) => {
                static DUMMY: std::sync::OnceLock<PredicateNode> = std::sync::OnceLock::new();
                DUMMY.get_or_init(|| PredicateNode::BoolLit(true))
            }
        }
    }
}

/// Diagnostic shaping metadata retained with predicates and runtime checks.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagnosticShape {
    pub code: CoreName,
    pub message: CoreName,
}

impl DiagnosticShape {
    /// Creates a diagnostic shape for a false predicate result.
    #[must_use]
    pub fn predicate_false(code: impl Into<CoreName>) -> Self {
        let code = code.into();
        Self {
            message: code.clone(),
            code,
        }
    }
}

/// Lowered contract predicate sidecar.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LoweredPredicate {
    id: PredicateId,
    predicate_ref: PredicateRef,
    source_span: Option<CoreSourceSpan>,
    contract_text: String,
    boundary: CoreBoundaryId,
    boundary_kind: BoundaryKind,
    env: PredicateEnvRef,
    root: PredicateNode,
    ty: CoreType,
    free_vars: Vec<PredicateBinderRef>,
    snapshot_refs: Vec<SnapshotRef>,
    predicate_functions: Vec<PredicateFunctionRef>,
    classification: PredicateClassification,
    proof_fragment: Option<ProofFragment>,
    dynamic_plan: Option<DynamicPredicatePlan>,
    diagnostic_shape: DiagnosticShape,
}

impl LoweredPredicate {
    /// Returns this predicate's id.
    #[must_use]
    pub fn id(&self) -> &PredicateId {
        &self.id
    }

    /// Returns this predicate's reference.
    #[must_use]
    pub fn predicate_ref(&self) -> &PredicateRef {
        &self.predicate_ref
    }

    /// Returns the diagnostic source text.
    #[must_use]
    pub fn contract_text(&self) -> &str {
        &self.contract_text
    }

    /// Returns the owning boundary.
    #[must_use]
    pub fn boundary(&self) -> &CoreBoundaryId {
        &self.boundary
    }

    /// Returns the boundary kind.
    #[must_use]
    pub fn boundary_kind(&self) -> BoundaryKind {
        self.boundary_kind
    }

    /// Returns the predicate result type.
    #[must_use]
    pub fn ty(&self) -> &CoreType {
        &self.ty
    }

    /// Returns free binders used by the lowered tree.
    #[must_use]
    pub fn free_vars(&self) -> &[PredicateBinderRef] {
        &self.free_vars
    }

    /// Returns snapshot references used by the lowered tree.
    #[must_use]
    pub fn snapshot_refs(&self) -> &[SnapshotRef] {
        &self.snapshot_refs
    }

    /// Returns static/dynamic classification.
    #[must_use]
    pub fn classification(&self) -> PredicateClassification {
        self.classification
    }

    /// Returns the lowered predicate root node.
    #[must_use]
    pub fn root(&self) -> &PredicateNode {
        &self.root
    }
}

/// Builder for [`LoweredPredicate`].
pub struct LoweredPredicateBuilder {
    boundary: CoreBoundaryId,
    boundary_kind: BoundaryKind,
    env: PredicateEnvironment,
    root: PredicateNode,
    ty: CoreType,
    source_span: Option<CoreSourceSpan>,
    contract_text: String,
    classification: PredicateClassification,
    proof_fragment: Option<ProofFragment>,
    dynamic_plan: Option<DynamicPredicatePlan>,
    diagnostic_shape: DiagnosticShape,
}

impl LoweredPredicateBuilder {
    /// Creates a lowered predicate builder with the required semantic fields.
    #[must_use]
    pub fn new(
        boundary: impl Into<CoreBoundaryId>,
        env: PredicateEnvironment,
        root: PredicateNode,
        ty: CoreType,
    ) -> Self {
        let boundary = boundary.into();
        assert_eq!(
            env.boundary(),
            &boundary,
            "predicate environment boundary must match lowered predicate boundary"
        );
        Self {
            boundary,
            boundary_kind: BoundaryKind::Unspecified,
            env,
            root,
            ty,
            source_span: None,
            contract_text: String::new(),
            classification: PredicateClassification::Dynamic,
            proof_fragment: None,
            dynamic_plan: None,
            diagnostic_shape: DiagnosticShape::predicate_false("contract-predicate-false"),
        }
    }

    /// Sets diagnostic source metadata. This text is not part of stable identity.
    #[must_use]
    pub fn source(mut self, source_span: CoreSourceSpan, contract_text: impl Into<String>) -> Self {
        self.source_span = Some(source_span);
        self.contract_text = contract_text.into();
        self
    }

    /// Sets the owning boundary kind.
    #[must_use]
    pub fn boundary_kind(mut self, boundary_kind: BoundaryKind) -> Self {
        self.boundary_kind = boundary_kind;
        self
    }

    /// Sets predicate classification.
    #[must_use]
    pub fn classification(mut self, classification: PredicateClassification) -> Self {
        self.classification = classification;
        self
    }

    /// Sets proof-fragment metadata.
    #[must_use]
    pub fn proof_fragment(mut self, proof_fragment: ProofFragment) -> Self {
        self.proof_fragment = Some(proof_fragment);
        self
    }

    /// Sets dynamic predicate evaluator metadata.
    #[must_use]
    pub fn dynamic_plan(mut self, dynamic_plan: DynamicPredicatePlan) -> Self {
        self.dynamic_plan = Some(dynamic_plan);
        self
    }

    /// Sets diagnostic shape metadata.
    #[must_use]
    pub fn diagnostic_shape(mut self, diagnostic_shape: DiagnosticShape) -> Self {
        self.diagnostic_shape = diagnostic_shape;
        self
    }

    /// Builds the lowered predicate and computes stable semantic identity.
    #[must_use]
    pub fn build(self) -> LoweredPredicate {
        let mut free_vars = Vec::new();
        let mut snapshots = Vec::new();
        let mut calls = Vec::new();
        collect_node_refs(&self.root, &mut free_vars, &mut snapshots, &mut calls);
        dedup_sorted(&mut free_vars);
        dedup_sorted(&mut snapshots);
        dedup_sorted(&mut calls);

        let stable_hash = PredicateHash::new(stable_digest(&StablePredicateKey {
            boundary: self.boundary.clone(),
            boundary_kind: self.boundary_kind,
            env: self.env.ref_(),
            binders: self.env.binders().to_vec(),
            environment_snapshots: self.env.snapshots().to_vec(),
            admitted_predicate_fns: self.env.admitted_predicate_fns().to_vec(),
            root: self.root.clone(),
            ty: self.ty.clone(),
            free_vars: free_vars.clone(),
            snapshot_refs: snapshots.clone(),
            predicate_functions: calls.clone(),
        }));
        let id = PredicateId::new(format!("pred:{}", stable_hash.as_str()));
        let predicate_ref = PredicateRef {
            id: id.clone(),
            stable_hash,
            boundary: self.boundary.clone(),
            source_span: self.source_span.clone(),
        };
        LoweredPredicate {
            id,
            predicate_ref,
            source_span: self.source_span,
            contract_text: self.contract_text,
            boundary: self.boundary,
            boundary_kind: self.boundary_kind,
            env: self.env.ref_(),
            root: self.root,
            ty: self.ty,
            free_vars,
            snapshot_refs: snapshots,
            predicate_functions: calls,
            classification: self.classification,
            proof_fragment: self.proof_fragment,
            dynamic_plan: self.dynamic_plan,
            diagnostic_shape: self.diagnostic_shape,
        }
    }
}

/// Contract blame party.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreBlameParty {
    Caller,
    Callee,
    Impl,
    Runtime,
}

/// Contract blame polarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreBlamePolarity {
    Negative,
    Positive,
}

/// Blame metadata for a runtime check plan.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CoreBlameLabel {
    pub party: CoreBlameParty,
    pub polarity: CoreBlamePolarity,
    pub boundary: CoreBoundaryId,
}

impl CoreBlameLabel {
    /// Creates a blame label.
    #[must_use]
    pub fn new(
        party: CoreBlameParty,
        polarity: CoreBlamePolarity,
        boundary: impl Into<CoreBoundaryId>,
    ) -> Self {
        Self {
            party,
            polarity,
            boundary: boundary.into(),
        }
    }
}

/// Runtime false-predicate recoverability boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractRecoverability {
    TrapDefault,
    ExplicitFail(Option<CoreType>),
}

/// Dynamic runtime check plan for a lowered predicate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeCheckPlan {
    predicate: PredicateRef,
    environment: PredicateEnvRef,
    evaluator: DynamicPredicatePlan,
    boundary: CoreBoundaryId,
    blame: CoreBlameLabel,
    snapshots: Vec<SnapshotRef>,
    diagnostic_shape: DiagnosticShape,
    recoverability: ContractRecoverability,
}

impl RuntimeCheckPlan {
    /// Creates a runtime check plan from a lowered predicate reference and captured environment.
    #[must_use]
    pub fn new(
        predicate: PredicateRef,
        environment: PredicateEnvRef,
        evaluator: DynamicPredicatePlan,
        blame: CoreBlameLabel,
        snapshots: Vec<SnapshotRef>,
        diagnostic_shape: DiagnosticShape,
        recoverability: ContractRecoverability,
    ) -> Self {
        let boundary = predicate.boundary.clone();
        Self {
            predicate,
            environment,
            evaluator,
            boundary,
            blame,
            snapshots,
            diagnostic_shape,
            recoverability,
        }
    }

    /// Returns the predicate reference checked by this plan.
    #[must_use]
    pub fn predicate(&self) -> &PredicateRef {
        &self.predicate
    }

    /// Returns the captured predicate environment reference.
    #[must_use]
    pub fn environment(&self) -> &PredicateEnvRef {
        &self.environment
    }

    /// Returns the owning boundary.
    #[must_use]
    pub fn boundary(&self) -> &CoreBoundaryId {
        &self.boundary
    }

    /// Returns false-predicate recoverability behavior.
    #[must_use]
    pub fn recoverability(&self) -> &ContractRecoverability {
        &self.recoverability
    }

    /// Returns the contract boundary kind (requires, ensures, etc.).
    #[must_use]
    pub fn boundary_kind(&self) -> BoundaryKind {
        self.evaluator.boundary_kind()
    }

    /// Returns the binders from the captured predicate environment.
    #[must_use]
    pub fn environment_binders(&self) -> &[PredicateBinder] {
        self.evaluator.environment_binders()
    }

    /// Returns the snapshot references bound at the contract boundary.
    #[must_use]
    pub fn snapshot_refs(&self) -> &[SnapshotRef] {
        &self.snapshots
    }

    /// Returns the predicate node evaluated by this plan.
    #[must_use]
    pub fn predicate_node(&self) -> &PredicateNode {
        self.evaluator.predicate_node()
    }

    /// Returns the blame label for dynamic failures of this plan.
    #[must_use]
    pub fn blame(&self) -> &CoreBlameLabel {
        &self.blame
    }
}

/// Runtime predicate evaluator fault classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredicateFault {
    EvaluatorTrap(String),
    MissingBinder(CoreName),
    MissingSnapshot(CoreName),
    TypeMismatch {
        expected: CoreType,
        actual: CoreType,
    },
}

/// Policy for presenting operation-produced values in contract diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObservationPolicy {
    Full,
    Summarize,
    Redact,
    Unavailable,
}

/// Reason an observed value is redacted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RedactionReason {
    Policy,
    Secret,
    CapabilityBoundary,
}

/// Diagnostic representation of an observed operation-produced value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObservationValue {
    Full(CoreName),
    Summary(CoreName),
    Redacted(RedactionReason),
    Unavailable(CoreName),
}

/// Sidecar evidence attached to a value produced by an authority-bearing operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObservationEvidence {
    id: CoreName,
    provider_path: CorePath,
    operation: CoreName,
    value: ObservationValue,
    policy: ObservationPolicy,
}

impl ObservationEvidence {
    /// Records that an ordinary value was produced by an operation under authority.
    #[must_use]
    pub fn operation_result(
        id: impl Into<CoreName>,
        provider_path: CorePath,
        operation: impl Into<CoreName>,
        value: ObservationValue,
        policy: ObservationPolicy,
    ) -> Self {
        Self {
            id: id.into(),
            provider_path,
            operation: operation.into(),
            value,
            policy,
        }
    }

    /// Operation name that produced the value.
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.operation
    }

    /// Diagnostic value view after policy application.
    #[must_use]
    pub fn value(&self) -> &ObservationValue {
        &self.value
    }

    /// Observation evidence never grants provider authority to predicates.
    #[must_use]
    pub fn grants_predicate_authority(&self) -> bool {
        false
    }

    /// Returns true when a diagnostic can still report that a contract failed.
    #[must_use]
    pub fn failure_visible_for_diagnostics(&self) -> bool {
        true
    }
}

/// Predicate evaluation authority environment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct PredicateAuthorityEnv {
    role_tokens: Vec<CoreName>,
}

impl PredicateAuthorityEnv {
    /// Default contract-predicate environment has no provider handles or role tokens.
    #[must_use]
    pub fn contract_predicate_default() -> Self {
        Self::default()
    }

    /// Attempts to acquire a provider for a predicate call.
    pub fn require_provider(
        &self,
        _provider: &PredicateFunctionRef,
    ) -> Result<(), PredicateObservationError> {
        Err(PredicateObservationError::ProviderAuthorityUnavailable)
    }

    /// Role tokens available during predicate evaluation.
    #[must_use]
    pub fn role_tokens(&self) -> &[CoreName] {
        &self.role_tokens
    }
}

/// Distinct diagnostic classes around operation observation and predicate checking.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredicateObservationError {
    AdmissionDenied,
    OperationFailed,
    PredicateFalse,
    PredicateFault(Box<PredicateFault>),
    ProviderAuthorityUnavailable,
}

/// Reference to an evidence item attached to a diagnostic.
///
/// Evidence is metadata-only: the family names the evidence class (e.g.
/// `observation`, `test`, `law`, `proof`, `monitor`) and the identity names a
/// stable evidence item. The diagnostic never embeds the evidence value; it
/// carries only family and identity so that downstream reporting can resolve and
/// redact the evidence according to policy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvidenceRef {
    family: CoreName,
    identity: CoreName,
}

impl EvidenceRef {
    /// Creates an evidence reference from a family and identity.
    #[must_use]
    pub fn new(family: impl Into<CoreName>, identity: impl Into<CoreName>) -> Self {
        Self {
            family: family.into(),
            identity: identity.into(),
        }
    }

    /// Returns the evidence family.
    #[must_use]
    pub fn family(&self) -> &str {
        &self.family
    }

    /// Returns the evidence identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }
}

/// Structured diagnostic payload for a false dynamic contract predicate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractDiagnostic {
    predicate: PredicateRef,
    contract_text: String,
    blame: CoreBlameLabel,
    predicate_classification: PredicateClassification,
    snapshot_refs: Vec<SnapshotRef>,
    evidence_refs: Vec<EvidenceRef>,
    redacted: bool,
}

impl ContractDiagnostic {
    /// Creates a contract-violation diagnostic payload.
    ///
    /// Evidence refs default to empty and `redacted` defaults to `true`, keeping
    /// observation evidence details out of the diagnostic by default.
    #[must_use]
    pub fn new(
        predicate: PredicateRef,
        contract_text: impl Into<String>,
        blame: CoreBlameLabel,
        predicate_classification: PredicateClassification,
        snapshot_refs: Vec<SnapshotRef>,
    ) -> Self {
        Self {
            predicate,
            contract_text: contract_text.into(),
            blame,
            predicate_classification,
            snapshot_refs,
            evidence_refs: Vec::new(),
            redacted: true,
        }
    }

    /// Returns the failed predicate reference.
    #[must_use]
    pub fn predicate(&self) -> &PredicateRef {
        &self.predicate
    }

    /// Returns the contract source text retained for diagnostics.
    #[must_use]
    pub fn contract_text(&self) -> &str {
        &self.contract_text
    }

    /// Returns the blame label.
    #[must_use]
    pub fn blame(&self) -> &CoreBlameLabel {
        &self.blame
    }

    /// Returns the predicate classification.
    #[must_use]
    pub fn predicate_classification(&self) -> PredicateClassification {
        self.predicate_classification
    }

    /// Returns the snapshot references bound at the contract boundary.
    #[must_use]
    pub fn snapshot_refs(&self) -> &[SnapshotRef] {
        &self.snapshot_refs
    }

    /// Returns the evidence references attached to this diagnostic.
    #[must_use]
    pub fn evidence_refs(&self) -> &[EvidenceRef] {
        &self.evidence_refs
    }

    /// Returns true when observation evidence details are redacted in this diagnostic.
    #[must_use]
    pub fn redacted(&self) -> bool {
        self.redacted
    }

    /// Attaches evidence references to this diagnostic.
    #[must_use]
    pub fn with_evidence_refs(mut self, evidence_refs: Vec<EvidenceRef>) -> Self {
        self.evidence_refs = evidence_refs;
        self
    }

    /// Marks this diagnostic as redacted (`true`) or unredacted (`false`).
    #[must_use]
    pub fn with_redacted(mut self, redacted: bool) -> Self {
        self.redacted = redacted;
        self
    }
}

/// Structured diagnostic payload for a predicate evaluator fault.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PredicateFaultDiagnostic {
    predicate: PredicateRef,
    contract_text: String,
    blame: CoreBlameLabel,
    fault: PredicateFault,
    snapshot_refs: Vec<SnapshotRef>,
    evidence_refs: Vec<EvidenceRef>,
    redacted: bool,
}

impl PredicateFaultDiagnostic {
    /// Creates a predicate-fault diagnostic payload.
    ///
    /// Evidence refs default to empty and `redacted` defaults to `true`, keeping
    /// observation evidence details out of the diagnostic by default.
    #[must_use]
    pub fn new(
        predicate: PredicateRef,
        contract_text: impl Into<String>,
        blame: CoreBlameLabel,
        fault: PredicateFault,
        snapshot_refs: Vec<SnapshotRef>,
    ) -> Self {
        Self {
            predicate,
            contract_text: contract_text.into(),
            blame,
            fault,
            snapshot_refs,
            evidence_refs: Vec::new(),
            redacted: true,
        }
    }

    /// Returns the predicate reference for which the evaluator faulted.
    #[must_use]
    pub fn predicate(&self) -> &PredicateRef {
        &self.predicate
    }

    /// Returns the contract source text retained for diagnostics.
    #[must_use]
    pub fn contract_text(&self) -> &str {
        &self.contract_text
    }

    /// Returns the blame label.
    #[must_use]
    pub fn blame(&self) -> &CoreBlameLabel {
        &self.blame
    }

    /// Returns the predicate fault classification.
    #[must_use]
    pub fn fault(&self) -> &PredicateFault {
        &self.fault
    }

    /// Returns the snapshot references bound at the contract boundary.
    #[must_use]
    pub fn snapshot_refs(&self) -> &[SnapshotRef] {
        &self.snapshot_refs
    }

    /// Returns the evidence references attached to this diagnostic.
    #[must_use]
    pub fn evidence_refs(&self) -> &[EvidenceRef] {
        &self.evidence_refs
    }

    /// Returns true when observation evidence details are redacted in this diagnostic.
    #[must_use]
    pub fn redacted(&self) -> bool {
        self.redacted
    }

    /// Attaches evidence references to this diagnostic.
    #[must_use]
    pub fn with_evidence_refs(mut self, evidence_refs: Vec<EvidenceRef>) -> Self {
        self.evidence_refs = evidence_refs;
        self
    }

    /// Marks this diagnostic as redacted (`true`) or unredacted (`false`).
    #[must_use]
    pub fn with_redacted(mut self, redacted: bool) -> Self {
        self.redacted = redacted;
        self
    }
}

/// Minimal contract-position expression accepted by the TASK-1695 lowering boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractPredicateExpr {
    BoolLit(bool),
    IntLit(i128),
    StringLit(String),
    UnitLit,
    Binder(PredicateBinderRef),
    Result(PredicateBinderRef),
    Message(PredicateBinderRef),
    OldPath {
        root: PredicateBinderRef,
        path: Vec<CoreName>,
        ty: CoreType,
        source_span: CoreSourceSpan,
    },
    Field {
        base: Box<ContractPredicateExpr>,
        field: CoreName,
    },
    TupleIndex {
        base: Box<ContractPredicateExpr>,
        index: usize,
    },
    Not(Box<ContractPredicateExpr>),
    And(Box<ContractPredicateExpr>, Box<ContractPredicateExpr>),
    Or(Box<ContractPredicateExpr>, Box<ContractPredicateExpr>),
    Implies(Box<ContractPredicateExpr>, Box<ContractPredicateExpr>),
    Eq(Box<ContractPredicateExpr>, Box<ContractPredicateExpr>),
    Ne(Box<ContractPredicateExpr>, Box<ContractPredicateExpr>),
    Lt(Box<ContractPredicateExpr>, Box<ContractPredicateExpr>),
    Le(Box<ContractPredicateExpr>, Box<ContractPredicateExpr>),
    Gt(Box<ContractPredicateExpr>, Box<ContractPredicateExpr>),
    Ge(Box<ContractPredicateExpr>, Box<ContractPredicateExpr>),
    Add(Box<ContractPredicateExpr>, Box<ContractPredicateExpr>),
    Sub(Box<ContractPredicateExpr>, Box<ContractPredicateExpr>),
    Mul(Box<ContractPredicateExpr>, Box<ContractPredicateExpr>),
    Div(Box<ContractPredicateExpr>, Box<ContractPredicateExpr>),
    Rem(Box<ContractPredicateExpr>, Box<ContractPredicateExpr>),
    PredicateCall {
        callee: PredicateFunctionRef,
        args: Vec<ContractPredicateExpr>,
        smt_safe: bool,
    },
    CapabilityCall {
        path: CorePath,
        operation: CoreName,
        source_span: CoreSourceSpan,
    },
    ProcessOperation {
        operation: CoreName,
        source_span: CoreSourceSpan,
    },
    WorkflowOperation {
        operation: CoreName,
        source_span: CoreSourceSpan,
    },
    HandlerDispatch {
        source_span: CoreSourceSpan,
    },
    TimeOrRandomObservation {
        source_span: CoreSourceSpan,
    },
    ImplicitForce {
        source_span: CoreSourceSpan,
    },
    OldComputation {
        source_span: CoreSourceSpan,
    },
}

/// Errors produced before predicate proof/runtime artifacts are allocated.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractPredicateLoweringError {
    NonBooleanPredicate {
        ty: CoreType,
        source_span: CoreSourceSpan,
    },
    InvalidSnapshotPath {
        source_span: CoreSourceSpan,
    },
    UnknownPredicateBinder {
        binder: PredicateBinderRef,
    },
    UnadmittedPredicateFunction {
        function: Box<PredicateFunctionRef>,
    },
    ForbiddenCapabilityCall {
        source_span: CoreSourceSpan,
    },
    ForbiddenProcessOperation {
        source_span: CoreSourceSpan,
    },
    ForbiddenWorkflowOperation {
        source_span: CoreSourceSpan,
    },
    ForbiddenHandlerDispatch {
        source_span: CoreSourceSpan,
    },
    ForbiddenEnvironmentObservation {
        source_span: CoreSourceSpan,
    },
    ForbiddenImplicitForce {
        source_span: CoreSourceSpan,
    },
}

/// Proof obligation sidecar emitted for accepted static predicates.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PredicateProofObligation {
    pub predicate: PredicateRef,
    pub boundary: CoreBoundaryId,
    pub fragment: ProofFragment,
    pub source_span: CoreSourceSpan,
}

/// Successful lowering result for an accepted contract predicate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractPredicateLowering {
    pub predicate: LoweredPredicate,
    pub proof_obligations: Vec<PredicateProofObligation>,
    pub runtime_check: Option<RuntimeCheckPlan>,
}

/// Stable reference to external or internal contract evidence.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractEvidenceRef(String);

impl ContractEvidenceRef {
    /// Creates an evidence reference.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Stable reference to a contract discharge record.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractDischargeRef(String);

/// Contract discharge status retained for diagnostics and optimizer safety.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractDischargeStatus {
    StaticProven { evidence: ContractEvidenceRef },
    Disproved { diagnostic: CoreName },
    EvidenceSurvivedTesting { evidence: ContractEvidenceRef },
    Dynamic { plan: Box<RuntimeCheckPlan> },
    Deferred { reason: CoreName },
}

/// Runtime monitor evidence row attached to a contract discharge.
///
/// This record is intentionally authority-free: it carries a monitor observation
/// at a boundary, but does not grant predicate authority or discharge operation,
/// resource, role, or policy rows.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeMonitorEvidence {
    monitor_ref: CoreName,
    contract_ref: CoreName,
    boundary: CoreBoundaryId,
    outcome: MonitorEvaluationResult,
    redacted: bool,
}

impl RuntimeMonitorEvidence {
    /// Creates a new runtime monitor evidence row.
    #[must_use]
    pub fn new(
        monitor_ref: impl Into<CoreName>,
        contract_ref: impl Into<CoreName>,
        boundary: impl Into<CoreBoundaryId>,
        outcome: MonitorEvaluationResult,
    ) -> Self {
        Self {
            monitor_ref: monitor_ref.into(),
            contract_ref: contract_ref.into(),
            boundary: boundary.into(),
            outcome,
            redacted: true,
        }
    }

    /// The monitor plan identity that produced this evidence.
    #[must_use]
    pub fn monitor_ref(&self) -> &CoreName {
        &self.monitor_ref
    }

    /// The trace contract identity the monitor was checking.
    #[must_use]
    pub fn contract_ref(&self) -> &CoreName {
        &self.contract_ref
    }

    /// The boundary where the monitor was attached.
    #[must_use]
    pub fn boundary(&self) -> &CoreBoundaryId {
        &self.boundary
    }

    /// The evaluation result at this boundary.
    #[must_use]
    pub fn outcome(&self) -> &MonitorEvaluationResult {
        &self.outcome
    }

    /// Whether the observation trace is redacted (default true).
    #[must_use]
    pub fn redacted(&self) -> bool {
        self.redacted
    }

    /// Returns a copy with the requested redaction flag.
    #[must_use]
    pub fn with_redacted(mut self, redacted: bool) -> Self {
        self.redacted = redacted;
        self
    }
}

/// Contract discharge sidecar record.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractDischargeRecord {
    discharge_ref: ContractDischargeRef,
    contract: CoreName,
    boundary: CoreBoundaryId,
    status: ContractDischargeStatus,
    source_span: CoreSourceSpan,
    blame: Option<CoreBlameLabel>,
    monitor_evidence: Vec<RuntimeMonitorEvidence>,
}

impl ContractDischargeRecord {
    /// Records a statically proven contract discharge.
    #[must_use]
    pub fn static_proven(
        contract: impl Into<CoreName>,
        boundary: impl Into<CoreBoundaryId>,
        evidence: ContractEvidenceRef,
        source_span: CoreSourceSpan,
        blame: Option<CoreBlameLabel>,
    ) -> Self {
        Self::new(
            contract,
            boundary,
            ContractDischargeStatus::StaticProven { evidence },
            source_span,
            blame,
        )
    }

    /// Records a survived-testing/evidence discharge.
    #[must_use]
    pub fn evidence_survived_testing(
        contract: impl Into<CoreName>,
        boundary: impl Into<CoreBoundaryId>,
        evidence: ContractEvidenceRef,
        source_span: CoreSourceSpan,
    ) -> Self {
        Self::new(
            contract,
            boundary,
            ContractDischargeStatus::EvidenceSurvivedTesting { evidence },
            source_span,
            None,
        )
    }

    /// Records a dynamic contract check discharge.
    #[must_use]
    pub fn dynamic(
        contract: impl Into<CoreName>,
        boundary: impl Into<CoreBoundaryId>,
        plan: RuntimeCheckPlan,
        source_span: CoreSourceSpan,
        blame: Option<CoreBlameLabel>,
    ) -> Self {
        Self::new(
            contract,
            boundary,
            ContractDischargeStatus::Dynamic {
                plan: Box::new(plan),
            },
            source_span,
            blame,
        )
    }

    /// Records a deferred contract discharge.
    #[must_use]
    pub fn deferred(
        contract: impl Into<CoreName>,
        boundary: impl Into<CoreBoundaryId>,
        reason: impl Into<CoreName>,
        source_span: CoreSourceSpan,
    ) -> Self {
        Self::new(
            contract,
            boundary,
            ContractDischargeStatus::Deferred {
                reason: reason.into(),
            },
            source_span,
            None,
        )
    }

    /// Creates a contract discharge record with the supplied status.
    #[must_use]
    pub fn new(
        contract: impl Into<CoreName>,
        boundary: impl Into<CoreBoundaryId>,
        status: ContractDischargeStatus,
        source_span: CoreSourceSpan,
        blame: Option<CoreBlameLabel>,
    ) -> Self {
        let contract = contract.into();
        let boundary = boundary.into();
        let monitor_evidence = Vec::new();
        let discharge_ref = ContractDischargeRef(stable_digest(&(
            "ContractDischarge",
            &contract,
            &boundary,
            &status,
            &source_span,
            &monitor_evidence,
        )));
        Self {
            discharge_ref,
            contract,
            boundary,
            status,
            source_span,
            blame,
            monitor_evidence,
        }
    }

    /// Returns this discharge record reference.
    #[must_use]
    pub fn discharge_ref(&self) -> ContractDischargeRef {
        self.discharge_ref.clone()
    }

    /// Returns the discharge status.
    #[must_use]
    pub fn status(&self) -> &ContractDischargeStatus {
        &self.status
    }

    /// Runtime monitor evidence attached to this discharge record.
    #[must_use]
    pub fn monitor_evidence(&self) -> &[RuntimeMonitorEvidence] {
        &self.monitor_evidence
    }

    /// Returns a copy with the supplied monitor evidence attached.
    #[must_use]
    pub fn with_monitor_evidence(mut self, monitor_evidence: Vec<RuntimeMonitorEvidence>) -> Self {
        self.monitor_evidence = monitor_evidence;
        self
    }
}

/// Metadata connecting producer postconditions to continuation preconditions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComposedContract {
    producer_postcondition: ContractDischargeRef,
    continuation_precondition: ContractDischargeRef,
    intermediate_binder: CoreName,
    proof_obligation: PredicateRef,
    composed_postcondition: Option<PredicateRef>,
    evidence: Option<ContractEvidenceRef>,
    source_span: CoreSourceSpan,
}

impl ComposedContract {
    /// Creates composed-contract metadata for bind/sequencing obligations.
    #[must_use]
    pub fn new(
        producer_postcondition: ContractDischargeRef,
        continuation_precondition: ContractDischargeRef,
        intermediate_binder: impl Into<CoreName>,
        proof_obligation: PredicateRef,
        composed_postcondition: Option<PredicateRef>,
        evidence: Option<ContractEvidenceRef>,
        source_span: CoreSourceSpan,
    ) -> Self {
        Self {
            producer_postcondition,
            continuation_precondition,
            intermediate_binder: intermediate_binder.into(),
            proof_obligation,
            composed_postcondition,
            evidence,
            source_span,
        }
    }

    /// Producer postcondition discharge used by the composition rule.
    #[must_use]
    pub fn producer_postcondition(&self) -> &ContractDischargeRef {
        &self.producer_postcondition
    }

    /// Continuation precondition discharge produced by the composition rule.
    #[must_use]
    pub fn continuation_precondition(&self) -> &ContractDischargeRef {
        &self.continuation_precondition
    }

    /// Intermediate binder connecting the two computations.
    #[must_use]
    pub fn intermediate_binder(&self) -> &str {
        &self.intermediate_binder
    }

    /// Evidence associated with this composition, if any.
    #[must_use]
    pub fn evidence(&self) -> Option<&ContractEvidenceRef> {
        self.evidence.as_ref()
    }
}

/// Textual predicate entailment known to the summary-level subsumption checker.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PredicateEntailment {
    antecedent: CoreName,
    consequent: CoreName,
}

impl PredicateEntailment {
    /// Creates an entailment `antecedent ⇒ consequent`.
    #[must_use]
    pub fn new(antecedent: impl Into<CoreName>, consequent: impl Into<CoreName>) -> Self {
        Self {
            antecedent: antecedent.into(),
            consequent: consequent.into(),
        }
    }

    /// Entailment antecedent.
    #[must_use]
    pub fn antecedent(&self) -> &str {
        &self.antecedent
    }

    /// Entailment consequent.
    #[must_use]
    pub fn consequent(&self) -> &str {
        &self.consequent
    }
}

/// A requires/ensures clause summary for interface/impl subsumption checks.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractClauseSummary {
    predicate: CoreName,
    blame: CoreBlameLabel,
}

impl ContractClauseSummary {
    /// Creates a `requires` clause summary with caller/negative blame.
    #[must_use]
    pub fn requires(predicate: impl Into<CoreName>) -> Self {
        Self {
            predicate: predicate.into(),
            blame: CoreBlameLabel::new(
                CoreBlameParty::Caller,
                CoreBlamePolarity::Negative,
                "requires",
            ),
        }
    }

    /// Creates an `ensures` clause summary with callee/positive blame.
    #[must_use]
    pub fn ensures(predicate: impl Into<CoreName>) -> Self {
        Self {
            predicate: predicate.into(),
            blame: CoreBlameLabel::new(
                CoreBlameParty::Callee,
                CoreBlamePolarity::Positive,
                "ensures",
            ),
        }
    }

    /// Predicate identifier/text for this summary-level clause.
    #[must_use]
    pub fn predicate(&self) -> &str {
        &self.predicate
    }

    /// Blame label for dynamic failures of this clause.
    #[must_use]
    pub fn blame(&self) -> &CoreBlameLabel {
        &self.blame
    }
}

/// Contract summary attached to an interface or impl method.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractSummary {
    name: CoreName,
    requires: ContractClauseSummary,
    ensures: ContractClauseSummary,
}

impl ContractSummary {
    /// Creates a method-level contract summary.
    #[must_use]
    pub fn new(
        name: impl Into<CoreName>,
        requires: ContractClauseSummary,
        ensures: ContractClauseSummary,
    ) -> Self {
        Self {
            name: name.into(),
            requires,
            ensures,
        }
    }
}

/// Subsumption proof obligations checked for an impl contract against its interface.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractSubsumptionProof {
    precondition_obligation: PredicateEntailment,
    postcondition_obligation: PredicateEntailment,
}

impl ContractSubsumptionProof {
    /// Interface precondition must imply impl precondition.
    #[must_use]
    pub fn precondition_obligation(&self) -> &PredicateEntailment {
        &self.precondition_obligation
    }

    /// Impl postcondition must imply interface postcondition.
    #[must_use]
    pub fn postcondition_obligation(&self) -> &PredicateEntailment {
        &self.postcondition_obligation
    }
}

/// Errors for interface/impl contract behavioral subtyping.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractSubsumptionError {
    PreconditionNotWeakened { required: PredicateEntailment },
    PostconditionNotStrengthened { required: PredicateEntailment },
}

/// Checks `{P} C {Q} ⊑ {P'} C {Q'}` iff `P ⇒ P'` and `Q' ⇒ Q`.
pub fn check_contract_subsumption(
    interface: &ContractSummary,
    impl_contract: &ContractSummary,
    entailments: &[PredicateEntailment],
) -> Result<ContractSubsumptionProof, ContractSubsumptionError> {
    let pre = PredicateEntailment::new(
        interface.requires.predicate(),
        impl_contract.requires.predicate(),
    );
    let post = PredicateEntailment::new(
        impl_contract.ensures.predicate(),
        interface.ensures.predicate(),
    );

    if !entailment_holds(&pre, entailments) {
        return Err(ContractSubsumptionError::PreconditionNotWeakened { required: pre });
    }
    if !entailment_holds(&post, entailments) {
        return Err(ContractSubsumptionError::PostconditionNotStrengthened { required: post });
    }
    Ok(ContractSubsumptionProof {
        precondition_obligation: pre,
        postcondition_obligation: post,
    })
}

fn entailment_holds(required: &PredicateEntailment, entailments: &[PredicateEntailment]) -> bool {
    required.antecedent == required.consequent || entailments.iter().any(|known| known == required)
}

/// Fact kinds available to trace/temporal contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraceFactKind {
    Process,
    Channel,
    Resource,
    Operation,
    Service,
    ExternalActor,
    Contract,
    Workflow,
    Evidence,
    Time,
}

/// Interpretation class for a trace contract alphabet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraceInterpretation {
    Operational,
    Normative,
    Mixed,
}

/// Trace alphabet sidecar.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceAlphabet {
    facts: Vec<TraceFactKind>,
}

impl TraceAlphabet {
    /// Creates a trace alphabet.
    #[must_use]
    pub fn new(facts: Vec<TraceFactKind>) -> Self {
        Self { facts }
    }

    /// Classifies the alphabet as operational, normative, or mixed.
    #[must_use]
    pub fn interpretation(&self) -> TraceInterpretation {
        let operational = self.facts.iter().any(|fact| {
            matches!(
                fact,
                TraceFactKind::Process
                    | TraceFactKind::Channel
                    | TraceFactKind::Resource
                    | TraceFactKind::Operation
                    | TraceFactKind::Service
                    | TraceFactKind::ExternalActor
                    | TraceFactKind::Time
            )
        });
        let normative = self.facts.iter().any(|fact| {
            matches!(
                fact,
                TraceFactKind::Contract | TraceFactKind::Workflow | TraceFactKind::Evidence
            )
        });
        match (operational, normative) {
            (true, true) => TraceInterpretation::Mixed,
            (true, false) => TraceInterpretation::Operational,
            (false, true) | (false, false) => TraceInterpretation::Normative,
        }
    }

    fn contains(&self, fact: &TraceFactKind) -> bool {
        self.facts.contains(fact)
    }
}

/// Temporal formula carrier for trace contracts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemporalFormula {
    Always(TraceFactKind),
    Eventually(TraceFactKind),
    EventuallyAfter {
        after: TraceFactKind,
        event: TraceFactKind,
    },
    Not(Box<TemporalFormula>),
}

/// Monitor scope restricting consumed facts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MonitorScope {
    alphabet: TraceAlphabet,
}

impl MonitorScope {
    /// Creates a monitor scope.
    #[must_use]
    pub fn new(alphabet: TraceAlphabet) -> Self {
        Self { alphabet }
    }

    /// Returns whether a fact is in scope for this monitor.
    #[must_use]
    pub fn accepts(&self, fact: &TraceFactKind) -> bool {
        self.alphabet.contains(fact)
    }
}

/// Monitor plan sidecar reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MonitorPlan {
    monitor_ref: CoreName,
    scope: MonitorScope,
}

impl MonitorPlan {
    /// Creates a monitor plan.
    #[must_use]
    pub fn new(monitor_ref: impl Into<CoreName>, scope: MonitorScope) -> Self {
        Self {
            monitor_ref: monitor_ref.into(),
            scope,
        }
    }

    /// Returns this monitor plan reference.
    #[must_use]
    pub fn monitor_ref(&self) -> &str {
        &self.monitor_ref
    }

    /// Returns this monitor's fact scope.
    #[must_use]
    pub fn scope(&self) -> &MonitorScope {
        &self.scope
    }
}

/// Trace-contract discharge mode.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraceContractDischarge {
    StaticModelChecked { evidence: ContractEvidenceRef },
    StaticProved { evidence: ContractEvidenceRef },
    EvidenceSurvivedTesting { evidence: ContractEvidenceRef },
    RuntimeMonitor { plan: CoreName },
    Deferred { reason: CoreName },
}

/// Trace contract sidecar.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceContract {
    id: CoreName,
    alphabet: TraceAlphabet,
    formula: TemporalFormula,
    discharge: TraceContractDischarge,
}

impl TraceContract {
    /// Creates a trace contract sidecar.
    #[must_use]
    pub fn new(
        id: impl Into<CoreName>,
        alphabet: TraceAlphabet,
        formula: TemporalFormula,
        discharge: TraceContractDischarge,
    ) -> Self {
        Self {
            id: id.into(),
            alphabet,
            formula,
            discharge,
        }
    }

    /// Returns trace interpretation classification.
    #[must_use]
    pub fn interpretation(&self) -> TraceInterpretation {
        self.alphabet.interpretation()
    }

    /// Returns this trace contract id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns this trace contract formula.
    #[must_use]
    pub fn formula(&self) -> &TemporalFormula {
        &self.formula
    }

    /// Trace contracts are not value-level predicate artifacts.
    #[must_use]
    pub fn predicate_ref(&self) -> Option<&PredicateRef> {
        None
    }
}

/// Evaluates a runtime trace contract monitor against recorded facts.
#[must_use]
pub fn evaluate_temporal_monitor(
    contract: &TraceContract,
    plan: &MonitorPlan,
    observed: &[TraceFactKind],
) -> MonitorEvaluationResult {
    if observed.iter().any(|fact| !plan.scope().accepts(fact)) {
        return MonitorEvaluationResult::Faulted(TemporalMonitorFaultDiagnostic::new(
            contract.id(),
            MonitorFault::OutOfScopeFact,
        ));
    }

    if formula_satisfied(contract.formula(), observed) {
        MonitorEvaluationResult::Satisfied
    } else {
        MonitorEvaluationResult::Violated(TemporalContractDiagnostic::new(
            contract.id(),
            contract.formula().clone(),
            contract.interpretation(),
        ))
    }
}

fn formula_satisfied(formula: &TemporalFormula, observed: &[TraceFactKind]) -> bool {
    match formula {
        TemporalFormula::Always(fact) => observed.iter().all(|observed| observed == fact),
        TemporalFormula::Eventually(fact) => observed.contains(fact),
        TemporalFormula::EventuallyAfter { after, event } => observed
            .iter()
            .position(|fact| fact == after)
            .is_some_and(|start| observed[start + 1..].contains(event)),
        TemporalFormula::Not(inner) => !formula_satisfied(inner, observed),
    }
}

/// Workflow ledger fact linked to an originating trace fact.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowLedgerFact {
    id: CoreName,
    source_trace_ref: CoreName,
}

impl WorkflowLedgerFact {
    /// Creates a workflow ledger fact.
    #[must_use]
    pub fn new(id: impl Into<CoreName>, source_trace_ref: impl Into<CoreName>) -> Self {
        Self {
            id: id.into(),
            source_trace_ref: source_trace_ref.into(),
        }
    }

    /// Returns the source trace reference.
    #[must_use]
    pub fn source_trace_ref(&self) -> &str {
        &self.source_trace_ref
    }
}

/// Temporal monitor fault classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonitorFault {
    OutOfScopeFact,
    EvaluatorTrap(CoreName),
    WindowUnavailable,
}

/// Temporal contract violation diagnostic payload.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemporalContractDiagnostic {
    contract_ref: CoreName,
    formula: TemporalFormula,
    interpretation: TraceInterpretation,
}

impl TemporalContractDiagnostic {
    /// Creates a temporal contract violation diagnostic.
    #[must_use]
    pub fn new(
        contract_ref: impl Into<CoreName>,
        formula: TemporalFormula,
        interpretation: TraceInterpretation,
    ) -> Self {
        Self {
            contract_ref: contract_ref.into(),
            formula,
            interpretation,
        }
    }

    /// Returns the violated contract reference.
    #[must_use]
    pub fn contract_ref(&self) -> &str {
        &self.contract_ref
    }
}

/// Temporal monitor evaluator fault diagnostic payload.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemporalMonitorFaultDiagnostic {
    contract_ref: CoreName,
    fault: MonitorFault,
}

impl TemporalMonitorFaultDiagnostic {
    /// Creates a temporal monitor fault diagnostic.
    #[must_use]
    pub fn new(contract_ref: impl Into<CoreName>, fault: MonitorFault) -> Self {
        Self {
            contract_ref: contract_ref.into(),
            fault,
        }
    }

    /// Returns the monitor fault classification.
    #[must_use]
    pub fn fault(&self) -> &MonitorFault {
        &self.fault
    }
}

/// Runtime monitor evaluation result states.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonitorEvaluationResult {
    Satisfied,
    Violated(TemporalContractDiagnostic),
    Pending,
    Inconclusive(CoreName),
    Faulted(TemporalMonitorFaultDiagnostic),
}

/// Authority-free temporal monitor environment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MonitorAuthorityEnv {
    facts: Vec<TraceFactKind>,
}

impl MonitorAuthorityEnv {
    /// Creates an authority-free environment over recorded facts only.
    #[must_use]
    pub fn recorded_facts_only(facts: Vec<TraceFactKind>) -> Self {
        Self { facts }
    }

    /// Returns whether the monitor can consume a recorded fact.
    #[must_use]
    pub fn can_consume(&self, fact: &TraceFactKind) -> bool {
        self.facts.contains(fact)
    }

    /// Monitors do not acquire provider/process authority.
    #[must_use]
    pub fn has_provider_authority(&self) -> bool {
        false
    }
}

/// Validates and lowers a contract-position expression into Core predicate artifacts.
#[allow(clippy::too_many_arguments)]
pub fn lower_contract_predicate(
    boundary: impl Into<CoreBoundaryId>,
    env: PredicateEnvironment,
    expr: ContractPredicateExpr,
    ty: CoreType,
    source_span: CoreSourceSpan,
    contract_text: impl Into<String>,
    blame: CoreBlameLabel,
    recoverability: ContractRecoverability,
) -> Result<ContractPredicateLowering, ContractPredicateLoweringError> {
    let boundary = boundary.into();
    let bool_ty = CoreType::Base("Bool".to_string());
    if ty != bool_ty {
        return Err(ContractPredicateLoweringError::NonBooleanPredicate { ty, source_span });
    }

    let lowered = lower_expr(&boundary, &env, &expr)?;
    let classification = if lowered.smt_safe {
        PredicateClassification::Static
    } else {
        PredicateClassification::Dynamic
    };
    let mut builder =
        LoweredPredicateBuilder::new(boundary.clone(), env.clone(), lowered.node, bool_ty)
            .source(source_span.clone(), contract_text)
            .classification(classification)
            .diagnostic_shape(DiagnosticShape::predicate_false("contract-predicate-false"));

    if classification == PredicateClassification::Static {
        builder = builder.proof_fragment(ProofFragment::SmtSafe);
    }

    let predicate = builder.build();
    let proof_obligations = if classification == PredicateClassification::Static {
        vec![PredicateProofObligation {
            predicate: predicate.predicate_ref().clone(),
            boundary: boundary.clone(),
            fragment: ProofFragment::SmtSafe,
            source_span,
        }]
    } else {
        Vec::new()
    };
    let runtime_check = if classification == PredicateClassification::Dynamic {
        Some(RuntimeCheckPlan::new(
            predicate.predicate_ref().clone(),
            predicate.env.clone(),
            DynamicPredicatePlan::Interpreter {
                boundary_kind: predicate.boundary_kind(),
                environment_binders: predicate
                    .free_vars()
                    .iter()
                    .filter_map(|b| env.binders().iter().find(|eb| eb.id() == b.id()))
                    .cloned()
                    .collect(),
                predicate_node: predicate.root().clone(),
            },
            blame,
            predicate.snapshot_refs().to_vec(),
            DiagnosticShape::predicate_false("contract-predicate-false"),
            recoverability,
        ))
    } else {
        None
    };

    Ok(ContractPredicateLowering {
        predicate,
        proof_obligations,
        runtime_check,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoweredExpr {
    node: PredicateNode,
    smt_safe: bool,
}

fn lower_expr(
    boundary: &CoreBoundaryId,
    env: &PredicateEnvironment,
    expr: &ContractPredicateExpr,
) -> Result<LoweredExpr, ContractPredicateLoweringError> {
    use ContractPredicateExpr as E;
    let lowered = match expr {
        E::BoolLit(value) => LoweredExpr {
            node: PredicateNode::BoolLit(*value),
            smt_safe: true,
        },
        E::IntLit(value) => LoweredExpr {
            node: PredicateNode::IntLit(*value),
            smt_safe: true,
        },
        E::StringLit(value) => LoweredExpr {
            node: PredicateNode::StringLit(value.clone()),
            smt_safe: true,
        },
        E::UnitLit => LoweredExpr {
            node: PredicateNode::UnitLit,
            smt_safe: true,
        },
        E::Binder(binder) => {
            validate_binder_ref(env, binder)?;
            LoweredExpr {
                node: PredicateNode::Binder(binder.clone()),
                smt_safe: true,
            }
        }
        E::Result(binder) => {
            validate_binder_ref(env, binder)?;
            LoweredExpr {
                node: PredicateNode::Result(binder.clone()),
                smt_safe: true,
            }
        }
        E::Message(binder) => {
            validate_binder_ref(env, binder)?;
            LoweredExpr {
                node: PredicateNode::Message(binder.clone()),
                smt_safe: true,
            }
        }
        E::OldPath {
            root,
            path,
            ty,
            source_span,
        } => {
            validate_binder_ref(env, root)?;
            if path.is_empty() {
                return Err(ContractPredicateLoweringError::InvalidSnapshotPath {
                    source_span: source_span.clone(),
                });
            }
            LoweredExpr {
                node: PredicateNode::Snapshot(SnapshotRef::new(
                    boundary.clone(),
                    root.id().clone(),
                    path.clone(),
                    ty.clone(),
                    source_span.clone(),
                )),
                smt_safe: true,
            }
        }
        E::Field { base, field } => {
            let base = lower_expr(boundary, env, base)?;
            LoweredExpr {
                node: PredicateNode::Field {
                    base: Box::new(base.node),
                    field: field.clone(),
                },
                smt_safe: base.smt_safe,
            }
        }
        E::TupleIndex { base, index } => {
            let base = lower_expr(boundary, env, base)?;
            LoweredExpr {
                node: PredicateNode::TupleIndex {
                    base: Box::new(base.node),
                    index: *index,
                },
                smt_safe: base.smt_safe,
            }
        }
        E::Not(inner) => unary(boundary, env, inner, PredicateNode::Not)?,
        E::And(left, right) => binary(boundary, env, left, right, PredicateNode::And)?,
        E::Or(left, right) => binary(boundary, env, left, right, PredicateNode::Or)?,
        E::Implies(left, right) => binary(boundary, env, left, right, PredicateNode::Implies)?,
        E::Eq(left, right) => binary(boundary, env, left, right, PredicateNode::Eq)?,
        E::Ne(left, right) => binary(boundary, env, left, right, PredicateNode::Ne)?,
        E::Lt(left, right) => binary(boundary, env, left, right, PredicateNode::Lt)?,
        E::Le(left, right) => binary(boundary, env, left, right, PredicateNode::Le)?,
        E::Gt(left, right) => binary(boundary, env, left, right, PredicateNode::Gt)?,
        E::Ge(left, right) => binary(boundary, env, left, right, PredicateNode::Ge)?,
        E::Add(left, right) => binary(boundary, env, left, right, PredicateNode::Add)?,
        E::Sub(left, right) => binary(boundary, env, left, right, PredicateNode::Sub)?,
        E::Mul(left, right) => binary(boundary, env, left, right, PredicateNode::Mul)?,
        E::Div(left, right) => binary(boundary, env, left, right, PredicateNode::Div)?,
        E::Rem(left, right) => binary(boundary, env, left, right, PredicateNode::Rem)?,
        E::PredicateCall {
            callee,
            args,
            smt_safe,
        } => {
            validate_predicate_function(env, callee)?;
            let args = args
                .iter()
                .map(|arg| lower_expr(boundary, env, arg))
                .collect::<Result<Vec<_>, _>>()?;
            let args_smt_safe = args.iter().all(|arg| arg.smt_safe);
            LoweredExpr {
                node: PredicateNode::PredicateCall {
                    callee: callee.clone(),
                    args: args.into_iter().map(|arg| arg.node).collect(),
                },
                smt_safe: *smt_safe && args_smt_safe,
            }
        }
        E::CapabilityCall { source_span, .. } => {
            return Err(ContractPredicateLoweringError::ForbiddenCapabilityCall {
                source_span: source_span.clone(),
            });
        }
        E::ProcessOperation { source_span, .. } => {
            return Err(ContractPredicateLoweringError::ForbiddenProcessOperation {
                source_span: source_span.clone(),
            });
        }
        E::WorkflowOperation { source_span, .. } => {
            return Err(ContractPredicateLoweringError::ForbiddenWorkflowOperation {
                source_span: source_span.clone(),
            });
        }
        E::HandlerDispatch { source_span } => {
            return Err(ContractPredicateLoweringError::ForbiddenHandlerDispatch {
                source_span: source_span.clone(),
            });
        }
        E::TimeOrRandomObservation { source_span } => {
            return Err(
                ContractPredicateLoweringError::ForbiddenEnvironmentObservation {
                    source_span: source_span.clone(),
                },
            );
        }
        E::ImplicitForce { source_span } => {
            return Err(ContractPredicateLoweringError::ForbiddenImplicitForce {
                source_span: source_span.clone(),
            });
        }
        E::OldComputation { source_span } => {
            return Err(ContractPredicateLoweringError::InvalidSnapshotPath {
                source_span: source_span.clone(),
            });
        }
    };
    Ok(lowered)
}

fn validate_binder_ref(
    env: &PredicateEnvironment,
    binder: &PredicateBinderRef,
) -> Result<(), ContractPredicateLoweringError> {
    if env
        .binders()
        .iter()
        .any(|admitted| admitted.ref_() == *binder)
    {
        Ok(())
    } else {
        Err(ContractPredicateLoweringError::UnknownPredicateBinder {
            binder: binder.clone(),
        })
    }
}

fn validate_predicate_function(
    env: &PredicateEnvironment,
    function: &PredicateFunctionRef,
) -> Result<(), ContractPredicateLoweringError> {
    if env.admitted_predicate_fns().contains(function) {
        Ok(())
    } else {
        Err(
            ContractPredicateLoweringError::UnadmittedPredicateFunction {
                function: Box::new(function.clone()),
            },
        )
    }
}

fn unary(
    boundary: &CoreBoundaryId,
    env: &PredicateEnvironment,
    inner: &ContractPredicateExpr,
    wrap: impl FnOnce(Box<PredicateNode>) -> PredicateNode,
) -> Result<LoweredExpr, ContractPredicateLoweringError> {
    let inner = lower_expr(boundary, env, inner)?;
    Ok(LoweredExpr {
        node: wrap(Box::new(inner.node)),
        smt_safe: inner.smt_safe,
    })
}

fn binary(
    boundary: &CoreBoundaryId,
    env: &PredicateEnvironment,
    left: &ContractPredicateExpr,
    right: &ContractPredicateExpr,
    wrap: impl FnOnce(Box<PredicateNode>, Box<PredicateNode>) -> PredicateNode,
) -> Result<LoweredExpr, ContractPredicateLoweringError> {
    let left = lower_expr(boundary, env, left)?;
    let right = lower_expr(boundary, env, right)?;
    Ok(LoweredExpr {
        node: wrap(Box::new(left.node), Box::new(right.node)),
        smt_safe: left.smt_safe && right.smt_safe,
    })
}

#[derive(Debug, Clone, Serialize)]
struct StablePredicateKey {
    boundary: CoreBoundaryId,
    boundary_kind: BoundaryKind,
    env: PredicateEnvRef,
    binders: Vec<PredicateBinder>,
    environment_snapshots: Vec<SnapshotRef>,
    admitted_predicate_fns: Vec<PredicateFunctionRef>,
    root: PredicateNode,
    ty: CoreType,
    free_vars: Vec<PredicateBinderRef>,
    snapshot_refs: Vec<SnapshotRef>,
    predicate_functions: Vec<PredicateFunctionRef>,
}

fn collect_node_refs(
    node: &PredicateNode,
    binders: &mut Vec<PredicateBinderRef>,
    snapshots: &mut Vec<SnapshotRef>,
    calls: &mut Vec<PredicateFunctionRef>,
) {
    match node {
        PredicateNode::BoolLit(_)
        | PredicateNode::IntLit(_)
        | PredicateNode::StringLit(_)
        | PredicateNode::UnitLit => {}
        PredicateNode::Binder(binder)
        | PredicateNode::Result(binder)
        | PredicateNode::Message(binder) => binders.push(binder.clone()),
        PredicateNode::Snapshot(snapshot) => snapshots.push(snapshot.clone()),
        PredicateNode::Field { base, .. }
        | PredicateNode::TupleIndex { base, .. }
        | PredicateNode::Not(base) => collect_node_refs(base, binders, snapshots, calls),
        PredicateNode::And(left, right)
        | PredicateNode::Or(left, right)
        | PredicateNode::Implies(left, right)
        | PredicateNode::Eq(left, right)
        | PredicateNode::Ne(left, right)
        | PredicateNode::Lt(left, right)
        | PredicateNode::Le(left, right)
        | PredicateNode::Gt(left, right)
        | PredicateNode::Ge(left, right)
        | PredicateNode::Add(left, right)
        | PredicateNode::Sub(left, right)
        | PredicateNode::Mul(left, right)
        | PredicateNode::Div(left, right)
        | PredicateNode::Rem(left, right) => {
            collect_node_refs(left, binders, snapshots, calls);
            collect_node_refs(right, binders, snapshots, calls);
        }
        PredicateNode::PredicateCall { callee, args } => {
            calls.push(callee.clone());
            for arg in args {
                collect_node_refs(arg, binders, snapshots, calls);
            }
        }
    }
}

fn dedup_sorted<T>(items: &mut Vec<T>)
where
    T: Clone + Debug,
{
    let mut keyed = BTreeMap::new();
    for item in items.iter().cloned() {
        keyed.insert(stable_digest(&item), item);
    }
    *items = keyed.into_values().collect();
}

fn stable_digest<T: Debug>(value: &T) -> String {
    let encoded = format!("{value:#?}");
    let digest = Sha256::digest(encoded.as_bytes());
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut out, "{byte:02x}").expect("writing to String cannot fail");
    }
    out
}
