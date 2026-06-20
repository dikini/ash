//! Type-checking boundary for Core Ash programs.
//!
//! Phase 162 starts with a deliberately small checker API over validated Core
//! programs. Later tasks extend the expression rules; unsupported forms fail
//! closed instead of being accepted optimistically.

use crate::core_ash::{CoreAtom, CoreEffectOp, CoreExpr, CoreName, CoreRow, CoreType};
use crate::core_ash_validate::ValidCoreProgram;
use std::collections::{HashMap, HashSet};

/// Type-checking environment for Core Ash.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreTypeCheckEnv {
    types: CoreTypeEnv,
    values: CoreValueEnv,
    continuations: CoreContEnv,
    rows: CoreRowEnv,
    operations: CoreOpEnv,
    discharges: CoreDischargeEnv,
}

impl CoreTypeCheckEnv {
    /// Returns the type-name environment.
    #[must_use]
    pub fn types(&self) -> &CoreTypeEnv {
        &self.types
    }

    /// Returns a mutable reference to the type-name environment.
    #[must_use]
    pub fn types_mut(&mut self) -> &mut CoreTypeEnv {
        &mut self.types
    }

    /// Returns the value binding environment.
    #[must_use]
    pub fn values(&self) -> &CoreValueEnv {
        &self.values
    }

    /// Returns a mutable reference to the value binding environment.
    #[must_use]
    pub fn values_mut(&mut self) -> &mut CoreValueEnv {
        &mut self.values
    }

    /// Returns the continuation binding environment.
    #[must_use]
    pub fn continuations(&self) -> &CoreContEnv {
        &self.continuations
    }

    /// Returns a mutable reference to the continuation binding environment.
    #[must_use]
    pub fn continuations_mut(&mut self) -> &mut CoreContEnv {
        &mut self.continuations
    }

    /// Returns the row-variable environment.
    #[must_use]
    pub fn rows(&self) -> &CoreRowEnv {
        &self.rows
    }

    /// Returns a mutable reference to the row-variable environment.
    #[must_use]
    pub fn rows_mut(&mut self) -> &mut CoreRowEnv {
        &mut self.rows
    }

    /// Returns the operation signature environment.
    #[must_use]
    pub fn operations(&self) -> &CoreOpEnv {
        &self.operations
    }

    /// Returns a mutable reference to the operation signature environment.
    #[must_use]
    pub fn operations_mut(&mut self) -> &mut CoreOpEnv {
        &mut self.operations
    }

    /// Returns the discharge metadata environment.
    #[must_use]
    pub fn discharges(&self) -> &CoreDischargeEnv {
        &self.discharges
    }

    /// Returns a mutable reference to the discharge metadata environment.
    #[must_use]
    pub fn discharges_mut(&mut self) -> &mut CoreDischargeEnv {
        &mut self.discharges
    }
}

/// Known nominal type names and type constructors.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreTypeEnv {
    names: HashSet<CoreName>,
    constructors: HashMap<CoreName, usize>,
}

impl CoreTypeEnv {
    /// Returns true when the environment has no bindings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty() && self.constructors.is_empty()
    }

    /// Returns true when the type name is known.
    #[must_use]
    pub fn contains_name(&self, name: &str) -> bool {
        self.names.contains(name)
    }

    /// Inserts a known nominal type name.
    pub fn insert_name(&mut self, name: impl Into<CoreName>) -> bool {
        self.names.insert(name.into())
    }

    /// Returns the expected type-constructor arity, when known.
    #[must_use]
    pub fn constructor_arity(&self, name: &str) -> Option<usize> {
        self.constructors.get(name).copied()
    }

    /// Inserts a known type constructor arity.
    pub fn insert_constructor(&mut self, name: impl Into<CoreName>, arity: usize) -> Option<usize> {
        self.constructors.insert(name.into(), arity)
    }
}

/// Core value bindings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreValueEnv {
    bindings: HashMap<CoreName, CoreType>,
}

impl CoreValueEnv {
    /// Returns true when the environment has no bindings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Looks up the type of a value binding.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&CoreType> {
        self.bindings.get(name)
    }

    /// Inserts a value binding.
    pub fn insert(&mut self, name: impl Into<CoreName>, ty: CoreType) -> Option<CoreType> {
        self.bindings.insert(name.into(), ty)
    }
}

/// Core continuation bindings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreContEnv {
    bindings: HashMap<CoreName, CoreType>,
}

impl CoreContEnv {
    /// Returns true when the environment has no bindings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Looks up the type of a continuation binding.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&CoreType> {
        self.bindings.get(name)
    }

    /// Inserts a continuation binding.
    pub fn insert(&mut self, name: impl Into<CoreName>, ty: CoreType) -> Option<CoreType> {
        self.bindings.insert(name.into(), ty)
    }
}

/// Core row-variable bindings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreRowEnv {
    bindings: HashMap<CoreName, CoreRow>,
}

impl CoreRowEnv {
    /// Returns true when the environment has no bindings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Looks up a row-variable binding.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&CoreRow> {
        self.bindings.get(name)
    }

    /// Inserts a row-variable binding.
    pub fn insert(&mut self, name: impl Into<CoreName>, row: CoreRow) -> Option<CoreRow> {
        self.bindings.insert(name.into(), row)
    }
}

/// Core operation signatures.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreOpEnv {
    operations: HashSet<CoreEffectOp>,
}

impl CoreOpEnv {
    /// Returns true when the environment has no operation signatures.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Returns true when an operation signature is known.
    #[must_use]
    pub fn contains(&self, op: &CoreEffectOp) -> bool {
        self.operations.contains(op)
    }

    /// Inserts an operation signature.
    pub fn insert(&mut self, op: CoreEffectOp) -> bool {
        self.operations.insert(op)
    }
}

/// Core contract/evidence discharge bindings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreDischargeEnv {
    discharged_contracts: HashSet<CoreName>,
}

impl CoreDischargeEnv {
    /// Returns true when the environment has no discharge bindings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.discharged_contracts.is_empty()
    }

    /// Returns true when a contract discharge is known.
    #[must_use]
    pub fn contains_contract(&self, name: &str) -> bool {
        self.discharged_contracts.contains(name)
    }

    /// Inserts a discharged contract name.
    pub fn insert_contract(&mut self, name: impl Into<CoreName>) -> bool {
        self.discharged_contracts.insert(name.into())
    }
}

/// A Core program with its checked result type and requirement row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedCoreProgram {
    expr: CoreExpr,
    ty: CoreType,
    row: CoreRow,
}

impl TypedCoreProgram {
    /// Returns the checked Core expression.
    #[must_use]
    pub fn expr(&self) -> &CoreExpr {
        &self.expr
    }

    /// Returns the checked result type.
    #[must_use]
    pub fn ty(&self) -> &CoreType {
        &self.ty
    }

    /// Returns the checked requirement row.
    #[must_use]
    pub fn row(&self) -> &CoreRow {
        &self.row
    }
}

/// Error returned by Core Ash type checking.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoreTypeCheckError {
    /// A value binding was referenced but not present in the value environment.
    #[error("unknown value `{name}`")]
    UnknownValue { name: CoreName },

    /// A type name or type constructor was referenced but not present in the type environment.
    #[error("unknown type `{name}`")]
    UnknownType { name: CoreName },

    /// A continuation binding was referenced but not present in the continuation environment.
    #[error("unknown continuation `{name}`")]
    UnknownContinuation { name: CoreName },

    /// An effect operation was referenced but not present in the operation environment.
    #[error("unknown operation: {detail}")]
    UnknownOperation { detail: String },

    /// The checker has not implemented this Core form yet.
    #[error("unsupported Core type-check form: {detail}")]
    UnsupportedCoreForm { detail: String },
}

/// Type-checks a validated Core program.
///
/// # Errors
///
/// Returns [`CoreTypeCheckError`] when a reference cannot be resolved or when
/// the expression uses a Core form that this implementation slice does not
/// support yet.
pub fn type_check_core_program(
    program: ValidCoreProgram,
    env: &CoreTypeCheckEnv,
) -> Result<TypedCoreProgram, CoreTypeCheckError> {
    let (ty, row) = type_check_expr(program.expr(), env)?;
    Ok(TypedCoreProgram {
        expr: program.into_expr(),
        ty,
        row,
    })
}

fn type_check_expr(
    expr: &CoreExpr,
    env: &CoreTypeCheckEnv,
) -> Result<(CoreType, CoreRow), CoreTypeCheckError> {
    match expr {
        CoreExpr::Atom(atom) => Ok((type_check_atom(atom, env)?, CoreRow::default())),
        CoreExpr::LetVal { .. } => Err(unsupported("LetVal")),
        CoreExpr::LetRec { .. } => Err(unsupported("LetRec")),
        CoreExpr::LetPrim { .. } => Err(unsupported("LetPrim")),
        CoreExpr::LetCall { .. } => Err(unsupported("LetCall")),
        CoreExpr::If { .. } => Err(unsupported("If")),
        CoreExpr::Call { .. } => Err(unsupported("Call")),
        CoreExpr::Jump { .. } => Err(unsupported("Jump")),
        CoreExpr::Raise { .. } => Err(unsupported("Raise")),
        CoreExpr::Handle { .. } => Err(unsupported("Handle")),
        CoreExpr::RecordDischarge { .. } => Err(unsupported("RecordDischarge")),
        CoreExpr::Trap { .. } => Err(unsupported("Trap")),
    }
}

fn type_check_atom(
    atom: &CoreAtom,
    env: &CoreTypeCheckEnv,
) -> Result<CoreType, CoreTypeCheckError> {
    match atom {
        CoreAtom::Var(name) => env
            .values()
            .lookup(name)
            .cloned()
            .ok_or_else(|| CoreTypeCheckError::UnknownValue { name: name.clone() }),
        CoreAtom::LitInt(_) => Ok(CoreType::Base("Int".into())),
        CoreAtom::LitString(_) => Ok(CoreType::Base("String".into())),
        CoreAtom::LitBool(_) => Ok(CoreType::Base("Bool".into())),
        CoreAtom::LitUnit => Ok(CoreType::Base("Unit".into())),
        CoreAtom::PrimName(_) => Err(unsupported("PrimName atom")),
        CoreAtom::ConstructorName(_) => Err(unsupported("ConstructorName atom")),
    }
}

fn unsupported(form: &str) -> CoreTypeCheckError {
    CoreTypeCheckError::UnsupportedCoreForm {
        detail: form.to_owned(),
    }
}
