//! Core Ash direct-style intermediate representation.
//!
//! These carriers model the SPEC-099 Core layer that sits above the existing
//! CPS IR. They intentionally use `Core*` names so Core terms cannot be
//! confused with `crate::cps` terms.

use serde::{Deserialize, Serialize};

use crate::core_ash_contract::{
    ContractDiagnostic, PredicateFaultDiagnostic, TemporalContractDiagnostic,
    TemporalMonitorFaultDiagnostic,
};

/// A Core identifier.
pub type CoreName = String;

/// A Core path split into canonical name segments.
pub type CorePath = Vec<String>;

/// A continuation target in Core.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreContRef {
    /// A static continuation label.
    Label(CoreName),
    /// A continuation value bound in the Core environment.
    Var(CoreName),
}

/// Primitive atomic Core values.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreAtom {
    /// A variable reference.
    Var(CoreName),
    /// Integer literal.
    LitInt(i64),
    /// String literal.
    LitString(String),
    /// Boolean literal.
    LitBool(bool),
    /// Unit literal.
    LitUnit,
    /// A compiler-known primitive operation name.
    PrimName(CorePrimOp),
    /// A data constructor name.
    ConstructorName(CoreName),
}

/// Compiler-known pure primitive operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorePrimOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Neg,
    Not,
    RecordGet(CoreName),
    TupleGet(usize),
    ConstructorTag(CoreName),
}

/// Core multiplicity marker for continuation types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreMultiplicity {
    /// Default one-shot handler resume continuation.
    Affine,
    /// Future hook only; not operationally supported in Phase 161.
    MultiShotPure,
}

/// Core type carrier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreType {
    /// Built-in base type such as `Int`, `String`, `Bool`, or `Unit`.
    Base(CoreName),
    /// Named type reference.
    Named(CoreName),
    /// Type variable.
    Var(CoreName),
    /// Function type with an explicit requirement row.
    Function {
        params: Vec<CoreType>,
        result: Box<CoreType>,
        row: CoreRow,
    },
    /// Refinement type carrier. The predicate remains textual until later
    /// phases introduce structured predicate carriers.
    Refinement {
        base: Box<CoreType>,
        predicate: String,
    },
    /// Continuation type `Cont<A, Ans, row, multiplicity>`.
    Cont {
        input: Box<CoreType>,
        answer: Box<CoreType>,
        row: CoreRow,
        multiplicity: CoreMultiplicity,
    },
    /// Tuple type.
    Tuple(Vec<CoreType>),
    /// Record type.
    Record(Vec<(CoreName, CoreType)>),
    /// Named type application.
    App { name: CoreName, args: Vec<CoreType> },
    /// Computation mode type wrapper.
    Mode {
        mode: CoreEvalMode,
        inner: Box<CoreType>,
        latent_row: Option<CoreRow>,
    },
}

/// Core computation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreEvalMode {
    /// Strict mode (no delayed execution).
    Strict,
    /// Lazy mode.
    Lazy,
    /// Memo mode.
    Memo,
}

/// Core thunk execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreThunkMode {
    /// Lazy thunk.
    Lazy,
    /// Memo thunk.
    Memo,
}

/// Static metadata for thunk captures.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct CoreCaptureSet {
    pub values: Vec<CoreName>,
}

/// A Core parameter with explicit type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CoreParam {
    pub name: CoreName,
    pub ty: CoreType,
}

/// Core effect row.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct CoreRow {
    pub items: Vec<CoreRowItem>,
    pub tail: Option<CoreName>,
}

impl CoreRow {
    /// Builds a closed row with no row-variable tail.
    #[must_use]
    pub fn closed(items: Vec<CoreRowItem>) -> Self {
        Self { items, tail: None }
    }

    /// Builds an open row with a row-variable tail.
    #[must_use]
    pub fn open(items: Vec<CoreRowItem>, tail: impl Into<CoreName>) -> Self {
        Self {
            items,
            tail: Some(tail.into()),
        }
    }
}

/// Requirement row item.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreRowItem {
    /// Operation requirement.
    Operation {
        path: CorePath,
        operation: CoreName,
    },
    Resource {
        path: CorePath,
        mode: CoreName,
    },
    Contract {
        contract: CoreName,
    },
    Channel {
        path: CorePath,
        mode: CoreName,
        payload_type: Box<CoreType>,
    },
    Process {
        operation: CoreName,
    },
    Failure {
        ty: Option<Box<CoreType>>,
    },
    Evidence {
        path: CorePath,
    },
    EffectGroupRef {
        path: CorePath,
    },
}

impl CoreRowItem {
    /// Builds an operation requirement row item.
    #[must_use]
    pub fn operation(path: CorePath, operation: impl Into<CoreName>) -> Self {
        Self::Operation {
            path,
            operation: operation.into(),
        }
    }

    /// Builds a channel requirement row item.
    #[must_use]
    pub fn channel<I, S>(path: I, mode: impl Into<CoreName>, payload_type: CoreType) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<CoreName>,
    {
        Self::Channel {
            path: path.into_iter().map(Into::into).collect(),
            mode: mode.into(),
            payload_type: Box::new(payload_type),
        }
    }

    /// Builds a process runtime requirement row item.
    #[must_use]
    pub fn process(operation: impl Into<CoreName>) -> Self {
        Self::Process {
            operation: operation.into(),
        }
    }

    /// Returns true when this row item is an operation requirement.
    #[must_use]
    pub fn is_operation_requirement(&self) -> bool {
        matches!(self, Self::Operation { .. })
    }

    /// Returns true when this row item is a channel requirement.
    #[must_use]
    pub fn is_channel_requirement(&self) -> bool {
        matches!(self, Self::Channel { .. })
    }

    /// Returns true when this row item is a process runtime requirement.
    #[must_use]
    pub fn is_process_requirement(&self) -> bool {
        matches!(self, Self::Process { .. })
    }
}

/// Raised operation kinds representable by SPEC-096b/SPEC-098b.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreEffectOp {
    Operation {
        path: CorePath,
        operation: CoreName,
        arg_types: Vec<CoreType>,
        result_type: CoreType,
    },
    Channel {
        path: CorePath,
        mode: CoreName,
        payload_type: CoreType,
        result_type: CoreType,
    },
    Process {
        operation: CoreName,
        arg_types: Vec<CoreType>,
        result_type: CoreType,
    },
    Failure {
        ty: Option<CoreType>,
    },
}

impl CoreEffectOp {
    /// Returns true for all currently representable raised operation kinds.
    #[must_use]
    pub fn is_raised_operation(&self) -> bool {
        matches!(
            self,
            Self::Operation { .. }
                | Self::Channel { .. }
                | Self::Process { .. }
                | Self::Failure { .. }
        )
    }
}

/// Core value carrier for inert data.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreValue {
    Atom(CoreAtom),
    Lam {
        params: Vec<CoreParam>,
        body: Box<CoreExpr>,
        row: CoreRow,
    },
    Thunk {
        mode: CoreThunkMode,
        result_ty: CoreType,
        body: Box<CoreExpr>,
        row: CoreRow,
        captures: CoreCaptureSet,
    },
    Record {
        fields: Vec<(CoreName, CoreAtom)>,
    },
    Tuple {
        elems: Vec<CoreAtom>,
    },
    DischargeMarker {
        discharge: CoreContractDischarge,
    },
}

/// Core expression carrier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreExpr {
    Atom(CoreAtom),
    LetVal {
        name: CoreName,
        ty: CoreType,
        value: CoreValue,
        body: Box<CoreExpr>,
    },
    LetRec {
        name: CoreName,
        ty: CoreType,
        value: CoreValue,
        body: Box<CoreExpr>,
    },
    LetPrim {
        name: CoreName,
        op: CorePrimOp,
        args: Vec<CoreAtom>,
        body: Box<CoreExpr>,
    },
    LetCall {
        name: CoreName,
        func: CoreAtom,
        args: Vec<CoreAtom>,
        body: Box<CoreExpr>,
    },
    If {
        cond: CoreAtom,
        then_branch: Box<CoreExpr>,
        else_branch: Box<CoreExpr>,
    },
    Call {
        func: CoreAtom,
        args: Vec<CoreAtom>,
    },
    Jump {
        cont: CoreContRef,
        arg: CoreAtom,
    },
    /// Answer-binding continuation invocation (SPEC-102 §8.7-8.8).
    ///
    /// Invokes `cont` with `arg`, binds the continuation answer to `name`,
    /// then evaluates `body` with `name` in scope. Affine continuations are
    /// consumed; multi-shot-pure continuations remain reusable.
    LetContCall {
        name: CoreName,
        cont: CoreContRef,
        arg: CoreAtom,
        body: Box<CoreExpr>,
    },
    Raise {
        op: CoreEffectOp,
        args: Vec<CoreAtom>,
    },
    Handle {
        clause: CoreHandlerClause,
        body: Box<CoreExpr>,
    },
    RecordDischarge {
        discharge: CoreContractDischarge,
        body: Box<CoreExpr>,
    },
    Trap {
        reason: CoreTrapReason,
    },
    LetMode {
        name: CoreName,
        mode: CoreEvalMode,
        ty: CoreType,
        expr: Box<CoreExpr>,
        body: Box<CoreExpr>,
    },
    Force {
        name: CoreName,
        thunk: CoreAtom,
        body: Box<CoreExpr>,
    },
}

/// Core handler clause. The resume parameter must have a continuation type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CoreHandlerClause {
    pub op: CoreEffectOp,
    pub params: Vec<CoreParam>,
    pub resume: CoreParam,
    pub body: Box<CoreExpr>,
    pub row: CoreRow,
}

/// Contract and evidence discharge metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CoreContractDischarge {
    pub contract: CoreName,
    pub mode: CoreDischargeMode,
    pub evidence: Option<CoreRefinementEvidence>,
    pub source_span: Option<CoreSourceSpan>,
}

/// Core discharge mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreDischargeMode {
    Static,
    Evidence,
    Dynamic,
}

/// Refinement evidence metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CoreRefinementEvidence {
    pub source: CoreEvidenceSource,
    pub status: CoreEvidenceStatus,
    pub predicate: String,
    pub diagnostic: Option<CoreName>,
}

/// Evidence source classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreEvidenceSource {
    HoareClause,
    Law(CorePath),
    ExternalProof(CorePath),
    Assumption(CorePath),
}

/// Evidence status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreEvidenceStatus {
    Proven,
    Disproved,
    Unknown,
    Statistical,
}

/// Lightweight source span carrier for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CoreSourceSpan {
    pub file: Option<String>,
    pub start: usize,
    pub end: usize,
}

/// Reasons a Core expression can trap outside ordinary row accounting.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreTrapReason {
    ContractViolation(CoreName),
    ContractViolationDiagnostic(ContractDiagnostic),
    ContractPredicateFault(PredicateFaultDiagnostic),
    TemporalContractViolation(TemporalContractDiagnostic),
    TemporalMonitorFault(TemporalMonitorFaultDiagnostic),
    UnhandledEffect(CoreEffectOp),
    Panic(String),
    NonExhaustiveMatch,
}
