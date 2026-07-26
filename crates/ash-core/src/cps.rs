//! CPS IR Types
//!
//! Core data structures for the Ash CPS intermediate representation.

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::core_ash_contract::{
    ContractDiagnostic, PredicateFaultDiagnostic, TemporalContractDiagnostic,
    TemporalMonitorFaultDiagnostic,
};

/// Wrapper for one-shot consumed flag - serializes as bool but uses `Rc<RefCell>` internally
#[derive(Debug, Clone, PartialEq)]
pub struct ConsumedFlag(Rc<RefCell<bool>>);

impl Default for ConsumedFlag {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsumedFlag {
    pub fn new() -> Self {
        ConsumedFlag(Rc::new(RefCell::new(false)))
    }
    pub fn get(&self) -> bool {
        *self.0.borrow()
    }
    pub fn set(&self, value: bool) {
        *self.0.borrow_mut() = value;
    }
}

impl Serialize for ConsumedFlag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(self.get())
    }
}

impl<'de> Deserialize<'de> for ConsumedFlag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = bool::deserialize(deserializer)?;
        Ok(ConsumedFlag(Rc::new(RefCell::new(value))))
    }
}

/// A name (identifier) in the CPS IR
pub type Name = String;

/// An atom - primitive value or variable reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Atom {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Var(Name),
    ConstructorName(Name),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ContMultiplicity {
    #[default]
    Affine,
    MultiShotPure,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", content = "row")]
pub enum ResumeRowMetadata {
    Known(EffectRow),
    #[default]
    InheritFromTarget,
}

fn is_default_effect_row(row: &EffectRow) -> bool {
    row.items.is_empty()
}

fn is_default_cont_multiplicity(multiplicity: &ContMultiplicity) -> bool {
    *multiplicity == ContMultiplicity::Affine
}

fn is_default_resume_row_metadata(metadata: &ResumeRowMetadata) -> bool {
    matches!(metadata, ResumeRowMetadata::InheritFromTarget)
}

/// A value - inert data that can be bound to variables
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Atom(Atom),
    Lam {
        params: Vec<Name>,
        cont: Name,
        body: Box<Term>,
        captured_env: Env,
        #[serde(default)]
        rec_binding: Option<Name>,
        row: EffectRow,
    },
    Cont {
        param: Name,
        body: Box<Term>,
        captured_env: Env,
        captured_chain: HandlerChain,
        consumed: ConsumedFlag,
        #[serde(default, skip_serializing_if = "is_default_effect_row")]
        row: EffectRow,
        #[serde(default, skip_serializing_if = "is_default_cont_multiplicity")]
        multiplicity: ContMultiplicity,
    },
    Record {
        fields: Vec<(Name, Value)>,
    },
    Tuple {
        elems: Vec<Value>,
    },
    /// A named algebraic-data constructor with recursively evaluated fields.
    Constructor {
        name: Name,
        fields: Vec<(Name, Value)>,
    },
    /// A thunk closure capturing its creation-time runtime environment and handler chain.
    ThunkClosure {
        mode: ThunkMode,
        body: Box<Value>,
        captured_env: Env,
        captured_chain: HandlerChain,
        row: EffectRow,
        #[serde(skip, default)]
        memo_cell: Option<MemoCellId>,
    },
}

/// A continuation reference - either a static label or a variable
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContRef {
    Label(Name),
    Var(Name),
}

/// A term - computation that produces effects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Term {
    LetVal {
        name: Name,
        value: Value,
        body: Box<Term>,
    },
    LetPrim {
        name: Name,
        op: PrimOp,
        args: Vec<Atom>,
        body: Box<Term>,
    },
    LetCont {
        name: Name,
        param: Name,
        cont_body: Box<Term>,
        body: Box<Term>,
        #[serde(default, skip_serializing_if = "is_default_effect_row")]
        row: EffectRow,
        #[serde(default, skip_serializing_if = "is_default_cont_multiplicity")]
        multiplicity: ContMultiplicity,
    },
    LetContCall {
        name: Name,
        cont: Name,
        arg: Atom,
        #[serde(default, skip_serializing_if = "is_default_effect_row")]
        row: EffectRow,
        body: Box<Term>,
    },
    Jump {
        cont: ContRef,
        arg: Atom,
        row: EffectRow,
    },
    /// Value-bearing continuation jump for a non-atomic answer.
    ///
    /// `Jump` remains the compact atom-only form used by existing CPS terms;
    /// this form preserves a checked recursive value at a continuation
    /// boundary instead of forcing it through an atom projection.
    JumpValue {
        cont: ContRef,
        arg: Value,
        row: EffectRow,
    },
    Call {
        func: Atom,
        args: Vec<Atom>,
        cont: ContRef,
        row: EffectRow,
    },
    If {
        cond: Atom,
        then_branch: Box<Term>,
        else_branch: Box<Term>,
        row: EffectRow,
    },
    LetRec {
        name: Name,
        value: Value,
        body: Box<Term>,
    },
    Match {
        scrutinee: Atom,
        arms: Vec<(Name, Box<Term>)>,
        default: Option<Box<Term>>,
    },
    Raise {
        op: EffectOp,
        args: Vec<Atom>,
        resume: ContRef,
        row: EffectRow,
    },
    Handle {
        clause: HandlerClause,
        body: Box<Term>,
        cont: ContRef,
        row: EffectRow,
    },
    RecordDischarge {
        discharge: ContractDischarge,
        body: Box<Term>,
    },
    /// Return a value directly (terminal success).
    ///
    /// Terminal answers are values rather than atoms: a checked CPS program may
    /// therefore observe a recursively constructed record, tuple, or closure
    /// without first (and incorrectly) collapsing it through `eval_atom`.
    Return { value: Value },
    /// Halt with a trap reason
    Trap { reason: TrapReason },
}

/// Primitive operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Neg,
    Not,
    RecordGet(Name),
    TupleGet(usize),
    ForceThunk,
}

/// The thunk evaluation mode carried in `ThunkClosure`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThunkMode {
    /// Lazy thunk semantics.
    Lazy,
    /// Memo thunk semantics with optional memo-cell state.
    Memo,
}

/// Opaque identifier for memo cells (process-local runtime state).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoCellId {
    raw_id: u64,
}

impl MemoCellId {
    #[must_use]
    pub fn new(raw: u64) -> Self {
        Self { raw_id: raw }
    }

    #[must_use]
    pub fn raw(self) -> u64 {
        self.raw_id
    }
}

/// An effect operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectOp {
    pub item: EffectItem,
    pub arg_types: Vec<Name>,
    pub result_type: Name,
}

/// A handler clause
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HandlerClause {
    pub op: EffectOp,
    pub params: Vec<Name>,
    pub resume: Name,
    pub body: Box<Term>,
    pub row: EffectRow,
    #[serde(default, skip_serializing_if = "is_default_resume_row_metadata")]
    pub resume_row: ResumeRowMetadata,
    #[serde(default, skip_serializing_if = "is_default_cont_multiplicity")]
    pub resume_multiplicity: ContMultiplicity,
}

/// Contract discharge metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractDischarge {
    pub contract: Name,
    pub discharge_type: DischargeType,
}

/// Types of discharge
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DischargeType {
    Dynamic,
    Static,
}

/// Reasons for trapping
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrapReason {
    ContractViolation,
    ContractViolationDiagnostic(Box<ContractDiagnostic>),
    ContractPredicateFault(Box<PredicateFaultDiagnostic>),
    TemporalContractViolation(Box<TemporalContractDiagnostic>),
    TemporalMonitorFault(Box<TemporalMonitorFaultDiagnostic>),
    Custom(String),
}

/// An effect item in a row
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectItem {
    pub namespace: Name,
    pub name: Name,
    pub kind: EffectItemKind,
}

/// Kinds of effect items
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EffectItemKind {
    Capability,
    Role,
    Policy,
    Contract,
    Resource,
    Channel,
    Process,
    Evidence,
    Failure,
    Alias,
    Group,
}

/// An effect row - collection of effect items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EffectRow {
    pub items: Vec<EffectItem>,
}

/// Runtime environment - immutable frame stack
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Env {
    pub bindings: HashMap<Name, Value>,
    pub parent: Option<Box<Env>>,
}

/// A handler frame
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HandlerFrame {
    Shallow { clause: HandlerClause },
    Provider { op: EffectOp, handler: Name },
}

/// Frame-ordered operation discharge found in a handler chain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandlerFrameMatch<'a> {
    /// A shallow handler clause matched the operation.
    Shallow {
        /// Matching handler clause.
        clause: &'a HandlerClause,
        /// Frame index in the chain.
        frame_index: usize,
    },
    /// A provider frame matched the operation.
    Provider {
        /// Provider handler name.
        handler: &'a str,
        /// Frame index in the chain.
        frame_index: usize,
    },
}

/// Handler chain - explicit stack of handler frames
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HandlerChain {
    pub frames: Vec<HandlerFrame>,
}

impl Env {
    /// Create a new empty environment
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a binding and return the new environment
    pub fn with_binding(mut self, name: Name, value: Value) -> Self {
        self.bindings.insert(name, value);
        self
    }

    /// Look up a name in the environment (current frame + parent chain)
    pub fn lookup(&self, name: &str) -> Option<&Value> {
        if let Some(value) = self.bindings.get(name) {
            return Some(value);
        }
        if let Some(ref parent) = self.parent {
            return parent.lookup(name);
        }
        None
    }
}

impl HandlerChain {
    /// Create a new empty handler chain
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a frame onto the chain
    pub fn push(&mut self, frame: HandlerFrame) {
        self.frames.push(frame);
    }

    /// Find the innermost frame that discharges an effect operation.
    pub fn find_operation_frame(&self, op: &EffectOp) -> Option<HandlerFrameMatch<'_>> {
        for (idx, frame) in self.frames.iter().enumerate().rev() {
            match frame {
                HandlerFrame::Shallow { clause } if clause.op == *op => {
                    return Some(HandlerFrameMatch::Shallow {
                        clause,
                        frame_index: idx,
                    });
                }
                HandlerFrame::Provider {
                    op: provider_op,
                    handler,
                } if provider_op == op => {
                    return Some(HandlerFrameMatch::Provider {
                        handler,
                        frame_index: idx,
                    });
                }
                _ => continue,
            }
        }
        None
    }

    /// Find the innermost handler for an effect operation, returning (clause, frame_index)
    pub fn find_handler(&self, op: &EffectOp) -> Option<(&HandlerClause, usize)> {
        for (idx, frame) in self.frames.iter().enumerate().rev() {
            match frame {
                HandlerFrame::Shallow { clause } if clause.op == *op => {
                    return Some((clause, idx));
                }
                _ => continue,
            }
        }
        None
    }

    /// Find the innermost provider frame for an effect operation, returning (handler_name, frame_index)
    pub fn find_provider(&self, op: &EffectOp) -> Option<(Name, usize)> {
        for (idx, frame) in self.frames.iter().enumerate().rev() {
            match frame {
                HandlerFrame::Provider {
                    op: provider_op,
                    handler,
                } if provider_op == op => {
                    return Some((handler.clone(), idx));
                }
                _ => continue,
            }
        }
        None
    }
}

impl EffectRow {
    /// Validate that the row has no duplicate (namespace, name) pairs
    pub fn validate_row(&self) -> Result<(), String> {
        let mut seen = std::collections::HashSet::new();
        for item in &self.items {
            let key = (&item.namespace, &item.name);
            if !seen.insert(key) {
                return Err(format!(
                    "Duplicate effect item: ({}, {})",
                    item.namespace, item.name
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_int() {
        let atom = Atom::Int(42);
        assert_eq!(atom, Atom::Int(42));
    }

    #[test]
    fn test_value_atom() {
        let value = Value::Atom(Atom::Int(42));
        assert_eq!(value, Value::Atom(Atom::Int(42)));
    }

    #[test]
    fn test_env_lookup_nested() {
        let env = Env::new()
            .with_binding("x".to_string(), Value::Atom(Atom::Int(1)))
            .with_binding("y".to_string(), Value::Atom(Atom::Int(2)));
        assert_eq!(env.lookup("x"), Some(&Value::Atom(Atom::Int(1))));
        assert_eq!(env.lookup("y"), Some(&Value::Atom(Atom::Int(2))));
        assert_eq!(env.lookup("z"), None);
    }

    #[test]
    fn test_handler_chain_find_no_match() {
        let chain = HandlerChain::new();
        let op = EffectOp {
            item: EffectItem {
                namespace: "cap".to_string(),
                name: "db.read".to_string(),
                kind: EffectItemKind::Capability,
            },
            arg_types: vec!["String".to_string()],
            result_type: "Int".to_string(),
        };
        assert_eq!(chain.find_handler(&op), None);
    }

    #[test]
    fn test_handler_chain_push() {
        let mut chain = HandlerChain::new();
        let op = EffectOp {
            item: EffectItem {
                namespace: "cap".to_string(),
                name: "db.read".to_string(),
                kind: EffectItemKind::Capability,
            },
            arg_types: vec!["String".to_string()],
            result_type: "Int".to_string(),
        };
        let clause = HandlerClause {
            op: op.clone(),
            params: vec!["table".to_string()],
            resume: "resume".to_string(),
            body: Box::new(Term::Trap {
                reason: TrapReason::Custom("test".to_string()),
            }),
            row: EffectRow::default(),
            resume_row: ResumeRowMetadata::default(),
            resume_multiplicity: ContMultiplicity::default(),
        };
        chain.push(HandlerFrame::Shallow {
            clause: clause.clone(),
        });
        let (found_clause, found_idx) = chain.find_handler(&op).unwrap();
        assert_eq!(found_clause, &clause);
        assert_eq!(found_idx, 0);
    }

    #[test]
    fn test_effect_row_with_items() {
        let row = EffectRow {
            items: vec![
                EffectItem {
                    namespace: "cap".to_string(),
                    name: "db.read".to_string(),
                    kind: EffectItemKind::Capability,
                },
                EffectItem {
                    namespace: "cap".to_string(),
                    name: "db.write".to_string(),
                    kind: EffectItemKind::Capability,
                },
            ],
        };
        assert!(row.validate_row().is_ok());
    }

    #[test]
    fn test_prim_op_display() {
        assert_eq!(format!("{:?}", PrimOp::Add), "Add");
    }

    #[test]
    fn test_trap_reason() {
        let reason = TrapReason::Custom("test".to_string());
        assert_eq!(format!("{:?}", reason), "Custom(\"test\")");
    }

    #[test]
    fn test_contract_discharge() {
        let discharge = ContractDischarge {
            contract: "test".to_string(),
            discharge_type: DischargeType::Dynamic,
        };
        assert_eq!(discharge.contract, "test");
    }

    #[test]
    fn test_let_val_term() {
        let term = Term::LetVal {
            name: "x".to_string(),
            value: Value::Atom(Atom::Int(42)),
            body: Box::new(Term::Trap {
                reason: TrapReason::Custom("test".to_string()),
            }),
        };
        assert_eq!(term, term.clone());
    }

    #[test]
    fn test_let_prim_term() {
        let term = Term::LetPrim {
            name: "y".to_string(),
            op: PrimOp::Add,
            args: vec![Atom::Int(1), Atom::Int(2)],
            body: Box::new(Term::Trap {
                reason: TrapReason::Custom("test".to_string()),
            }),
        };
        assert_eq!(term, term.clone());
    }

    #[test]
    fn test_let_cont_term() {
        let term = Term::LetCont {
            name: "k".to_string(),
            param: "v".to_string(),
            cont_body: Box::new(Term::Trap {
                reason: TrapReason::Custom("test".to_string()),
            }),
            body: Box::new(Term::Trap {
                reason: TrapReason::Custom("test".to_string()),
            }),
            row: EffectRow::default(),
            multiplicity: ContMultiplicity::Affine,
        };
        assert_eq!(term, term.clone());
    }

    #[test]
    fn test_jump_term() {
        let term = Term::Jump {
            cont: ContRef::Label("exit".to_string()),
            arg: Atom::Int(42),
            row: EffectRow::default(),
        };
        assert_eq!(term, term.clone());
    }

    #[test]
    fn test_call_term() {
        let term = Term::Call {
            func: Atom::Var("f".to_string()),
            args: vec![Atom::Int(1)],
            cont: ContRef::Label("k".to_string()),
            row: EffectRow::default(),
        };
        assert_eq!(term, term.clone());
    }

    #[test]
    fn test_lam_value() {
        let value = Value::Lam {
            params: vec!["x".to_string()],
            cont: "k".to_string(),
            body: Box::new(Term::Trap {
                reason: TrapReason::Custom("test".to_string()),
            }),
            captured_env: Env::new(),
            rec_binding: None,
            row: EffectRow::default(),
        };
        assert_eq!(value, value.clone());
    }

    #[test]
    fn test_cont_value() {
        let value = Value::Cont {
            param: "v".to_string(),
            body: Box::new(Term::Trap {
                reason: TrapReason::Custom("test".to_string()),
            }),
            captured_env: Env::new(),
            captured_chain: HandlerChain::new(),
            consumed: ConsumedFlag::new(),
            row: EffectRow::default(),
            multiplicity: ContMultiplicity::Affine,
        };
        assert_eq!(value, value.clone());
    }

    #[test]
    fn test_cont_ref_label() {
        let cont_ref = ContRef::Label("exit".to_string());
        assert_eq!(cont_ref, ContRef::Label("exit".to_string()));
    }

    #[test]
    fn test_cont_ref_var() {
        let cont_ref = ContRef::Var("k".to_string());
        assert_eq!(cont_ref, ContRef::Var("k".to_string()));
    }
}
