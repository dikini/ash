//! Type-checking boundary for Core Ash programs.
//!
//! Phase 162 starts with a deliberately small checker API over validated Core
//! programs. Later tasks extend the expression rules; unsupported forms fail
//! closed instead of being accepted optimistically.

use crate::core_ash::{CoreAtom, CoreEffectOp, CoreExpr, CoreName, CoreRow, CoreRowItem, CoreType};
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
    variables: HashSet<CoreName>,
}

impl CoreTypeEnv {
    /// Returns true when the environment has no bindings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty() && self.constructors.is_empty() && self.variables.is_empty()
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

    /// Returns true when the type variable is in scope.
    #[must_use]
    pub fn contains_variable(&self, name: &str) -> bool {
        self.variables.contains(name)
    }

    /// Inserts an in-scope type variable.
    pub fn insert_variable(&mut self, name: impl Into<CoreName>) -> bool {
        self.variables.insert(name.into())
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
    refinement_predicates: HashSet<String>,
}

impl CoreDischargeEnv {
    /// Returns true when the environment has no discharge bindings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.discharged_contracts.is_empty() && self.refinement_predicates.is_empty()
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

    /// Returns true when textual refinement predicate metadata is in scope.
    #[must_use]
    pub fn contains_refinement_predicate(&self, predicate: &str) -> bool {
        self.refinement_predicates.contains(predicate)
    }

    /// Inserts a placeholder for scoped textual refinement predicate metadata.
    pub fn insert_refinement_predicate(&mut self, predicate: impl Into<String>) -> bool {
        self.refinement_predicates.insert(predicate.into())
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

    /// A type variable was referenced but not in scope.
    #[error("unknown type variable `{name}`")]
    UnknownTypeVariable { name: CoreName },

    /// A row variable was referenced but not in scope.
    #[error("unknown row variable `{name}`")]
    UnknownRowVariable { name: CoreName },

    /// A continuation binding was referenced but not present in the continuation environment.
    #[error("unknown continuation `{name}`")]
    UnknownContinuation { name: CoreName },

    /// An effect operation was referenced but not present in the operation environment.
    #[error("unknown operation: {detail}")]
    UnknownOperation { detail: String },

    /// A type application was supplied the wrong number of arguments.
    #[error("type application `{name}` expected {expected} arguments, got {actual}")]
    TypeApplicationArityMismatch {
        name: CoreName,
        expected: usize,
        actual: usize,
    },

    /// A textual refinement predicate had no scoped metadata placeholder.
    #[error("unknown refinement predicate `{predicate}`")]
    UnknownRefinementPredicate { predicate: String },

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

/// Checks that a Core type is well formed under the type-checking environment.
///
/// # Errors
///
/// Returns [`CoreTypeCheckError`] when a type, type variable, row variable, or
/// refinement predicate cannot be resolved, or when a type application has the
/// wrong arity.
pub fn check_core_type_well_formed(
    ty: &CoreType,
    env: &CoreTypeCheckEnv,
) -> Result<(), CoreTypeCheckError> {
    match ty {
        CoreType::Base(name) => {
            if is_builtin_base_type(name) {
                Ok(())
            } else {
                Err(CoreTypeCheckError::UnknownType { name: name.clone() })
            }
        }
        CoreType::Named(name) => {
            if env.types().contains_name(name) {
                Ok(())
            } else {
                Err(CoreTypeCheckError::UnknownType { name: name.clone() })
            }
        }
        CoreType::Var(name) => {
            if env.types().contains_variable(name) {
                Ok(())
            } else {
                Err(CoreTypeCheckError::UnknownTypeVariable { name: name.clone() })
            }
        }
        CoreType::Function {
            params,
            result,
            row,
        } => {
            check_types_well_formed(params, env)?;
            check_core_type_well_formed(result, env)?;
            check_core_row_well_formed(row, env)
        }
        CoreType::Refinement { base, predicate } => {
            check_core_type_well_formed(base, env)?;
            if env.discharges().contains_refinement_predicate(predicate) {
                Ok(())
            } else {
                Err(CoreTypeCheckError::UnknownRefinementPredicate {
                    predicate: predicate.clone(),
                })
            }
        }
        CoreType::Cont {
            input, answer, row, ..
        } => {
            check_core_type_well_formed(input, env)?;
            check_core_type_well_formed(answer, env)?;
            check_core_row_well_formed(row, env)
        }
        CoreType::Tuple(elems) => check_types_well_formed(elems, env),
        CoreType::Record(fields) => {
            for (_, field_ty) in fields {
                check_core_type_well_formed(field_ty, env)?;
            }
            Ok(())
        }
        CoreType::App { name, args } => {
            let Some(expected) = env.types().constructor_arity(name) else {
                return Err(CoreTypeCheckError::UnknownType { name: name.clone() });
            };
            if expected != args.len() {
                return Err(CoreTypeCheckError::TypeApplicationArityMismatch {
                    name: name.clone(),
                    expected,
                    actual: args.len(),
                });
            }
            check_types_well_formed(args, env)
        }
    }
}

/// Compares Core types using the Phase 162 definitional equality scaffold.
///
/// Record fields are compared by field name rather than source order.
///
/// # Errors
///
/// Returns [`CoreTypeCheckError`] when either type is not well formed.
pub fn core_types_equivalent(
    lhs: &CoreType,
    rhs: &CoreType,
    env: &CoreTypeCheckEnv,
) -> Result<bool, CoreTypeCheckError> {
    check_core_type_well_formed(lhs, env)?;
    check_core_type_well_formed(rhs, env)?;
    Ok(types_equivalent_unchecked(lhs, rhs, env))
}

fn check_types_well_formed(
    types: &[CoreType],
    env: &CoreTypeCheckEnv,
) -> Result<(), CoreTypeCheckError> {
    for ty in types {
        check_core_type_well_formed(ty, env)?;
    }
    Ok(())
}

fn check_core_row_well_formed(
    row: &CoreRow,
    env: &CoreTypeCheckEnv,
) -> Result<(), CoreTypeCheckError> {
    if let Some(tail) = &row.tail
        && env.rows().lookup(tail).is_none()
    {
        return Err(CoreTypeCheckError::UnknownRowVariable { name: tail.clone() });
    }

    for item in &row.items {
        match item {
            CoreRowItem::Channel { payload_type, .. } => {
                check_core_type_well_formed(payload_type, env)?;
            }
            CoreRowItem::Failure { ty: Some(ty) } => {
                check_core_type_well_formed(ty, env)?;
            }
            CoreRowItem::Capability { .. }
            | CoreRowItem::Resource { .. }
            | CoreRowItem::Role { .. }
            | CoreRowItem::Policy { .. }
            | CoreRowItem::Contract { .. }
            | CoreRowItem::Process { .. }
            | CoreRowItem::Failure { ty: None }
            | CoreRowItem::Evidence { .. }
            | CoreRowItem::EffectGroupRef { .. } => {}
        }
    }

    Ok(())
}

fn is_builtin_base_type(name: &str) -> bool {
    matches!(name, "Int" | "String" | "Bool" | "Unit")
}

fn types_equivalent_unchecked(lhs: &CoreType, rhs: &CoreType, env: &CoreTypeCheckEnv) -> bool {
    match (lhs, rhs) {
        (CoreType::Base(left), CoreType::Base(right))
        | (CoreType::Named(left), CoreType::Named(right))
        | (CoreType::Var(left), CoreType::Var(right)) => left == right,
        (
            CoreType::Function {
                params: left_params,
                result: left_result,
                row: left_row,
            },
            CoreType::Function {
                params: right_params,
                result: right_result,
                row: right_row,
            },
        ) => {
            type_slices_equivalent_unchecked(left_params, right_params, env)
                && types_equivalent_unchecked(left_result, right_result, env)
                && left_row == right_row
        }
        (
            CoreType::Refinement {
                base: left_base,
                predicate: left_predicate,
            },
            CoreType::Refinement {
                base: right_base,
                predicate: right_predicate,
            },
        ) => {
            left_predicate == right_predicate
                && types_equivalent_unchecked(left_base, right_base, env)
        }
        (
            CoreType::Cont {
                input: left_input,
                answer: left_answer,
                row: left_row,
                multiplicity: left_multiplicity,
            },
            CoreType::Cont {
                input: right_input,
                answer: right_answer,
                row: right_row,
                multiplicity: right_multiplicity,
            },
        ) => {
            left_multiplicity == right_multiplicity
                && types_equivalent_unchecked(left_input, right_input, env)
                && types_equivalent_unchecked(left_answer, right_answer, env)
                && left_row == right_row
        }
        (CoreType::Tuple(left), CoreType::Tuple(right)) => {
            type_slices_equivalent_unchecked(left, right, env)
        }
        (CoreType::Record(left), CoreType::Record(right)) => {
            record_fields_equivalent_unchecked(left, right, env)
        }
        (
            CoreType::App {
                name: left_name,
                args: left_args,
            },
            CoreType::App {
                name: right_name,
                args: right_args,
            },
        ) => {
            left_name == right_name && type_slices_equivalent_unchecked(left_args, right_args, env)
        }
        _ => false,
    }
}

fn type_slices_equivalent_unchecked(
    lhs: &[CoreType],
    rhs: &[CoreType],
    env: &CoreTypeCheckEnv,
) -> bool {
    lhs.len() == rhs.len()
        && lhs
            .iter()
            .zip(rhs)
            .all(|(left, right)| types_equivalent_unchecked(left, right, env))
}

fn record_fields_equivalent_unchecked(
    lhs: &[(CoreName, CoreType)],
    rhs: &[(CoreName, CoreType)],
    env: &CoreTypeCheckEnv,
) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }

    lhs.iter().all(|(left_name, left_ty)| {
        rhs.iter()
            .find(|(right_name, _)| right_name == left_name)
            .is_some_and(|(_, right_ty)| types_equivalent_unchecked(left_ty, right_ty, env))
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
