//! Type-checking boundary for Core Ash programs.
//!
//! Phase 162 starts with a deliberately small checker API over validated Core
//! programs. Later tasks extend the expression rules; unsupported forms fail
//! closed instead of being accepted optimistically.

use crate::core_ash::{
    CoreAtom, CoreContRef, CoreEffectOp, CoreExpr, CoreHandlerClause, CoreMultiplicity, CoreName,
    CoreParam, CorePrimOp, CoreRow, CoreRowItem, CoreType, CoreValue,
};
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
    value_constructors: HashMap<CoreName, CoreType>,
    variables: HashSet<CoreName>,
}

impl CoreTypeEnv {
    /// Returns true when the environment has no bindings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
            && self.constructors.is_empty()
            && self.value_constructors.is_empty()
            && self.variables.is_empty()
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

    /// Looks up a value constructor type by name.
    #[must_use]
    pub fn value_constructor(&self, name: &str) -> Option<&CoreType> {
        self.value_constructors.get(name)
    }

    /// Inserts a known value constructor type.
    pub fn insert_value_constructor(
        &mut self,
        name: impl Into<CoreName>,
        ty: CoreType,
    ) -> Option<CoreType> {
        self.value_constructors.insert(name.into(), ty)
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
    facts: CoreTypeCheckFacts,
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

    /// Returns typed facts needed by later lowering and diagnostic stages.
    #[must_use]
    pub fn facts(&self) -> &CoreTypeCheckFacts {
        &self.facts
    }
}

/// Typed facts computed during Core type checking for later compiler stages.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreTypeCheckFacts {
    jump_continuation_rows: HashMap<CoreContRef, CoreRow>,
}

impl CoreTypeCheckFacts {
    /// Returns rows of target continuations reached by checked `Jump` expressions.
    #[must_use]
    pub fn jump_continuation_rows(&self) -> &HashMap<CoreContRef, CoreRow> {
        &self.jump_continuation_rows
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypedCoreExpr {
    ty: CoreType,
    row: CoreRow,
    facts: CoreTypeCheckFacts,
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

    /// Two rows were expected to match but did not.
    #[error("row mismatch")]
    RowMismatch { expected: CoreRow, actual: CoreRow },

    /// Two types were expected to match but did not.
    #[error("type mismatch")]
    TypeMismatch {
        expected: Box<CoreType>,
        actual: Box<CoreType>,
    },

    /// A callable or primitive operation received the wrong number of arguments.
    #[error("argument count mismatch: expected {expected}, got {actual}")]
    ArgumentCountMismatch { expected: usize, actual: usize },

    /// A row alias or group reference could not be normalized structurally.
    #[error("ambiguous row reference: {detail}")]
    AmbiguousRowReference { detail: String },

    /// The checker has not implemented this Core form yet.
    #[error("unsupported Core type-check form: {detail}")]
    UnsupportedCoreForm { detail: String },
}

/// A structural row-variable solution discovered during row comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreRowSolution {
    variable: CoreName,
    row: CoreRow,
}

impl CoreRowSolution {
    /// Returns the row variable that was solved.
    #[must_use]
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// Returns the structural row assigned to the variable.
    #[must_use]
    pub fn row(&self) -> &CoreRow {
        &self.row
    }
}

/// Result of a Core row inclusion comparison.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreRowComparison {
    included: bool,
    missing_items: Vec<CoreRowItem>,
    solutions: Vec<CoreRowSolution>,
}

impl CoreRowComparison {
    /// Returns true when the left row is included in the right row.
    #[must_use]
    pub fn is_included(&self) -> bool {
        self.included
    }

    /// Returns normalized left-row requirements that were not present in the right row.
    #[must_use]
    pub fn missing_items(&self) -> &[CoreRowItem] {
        &self.missing_items
    }

    /// Returns structural row-variable solutions produced by the comparison.
    #[must_use]
    pub fn solutions(&self) -> &[CoreRowSolution] {
        &self.solutions
    }
}

/// A Core value with its synthesized type and construction row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedCoreValue {
    ty: CoreType,
    row: CoreRow,
}

impl TypedCoreValue {
    /// Returns the synthesized value type.
    #[must_use]
    pub fn ty(&self) -> &CoreType {
        &self.ty
    }

    /// Returns the value construction row.
    #[must_use]
    pub fn row(&self) -> &CoreRow {
        &self.row
    }
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
    let checked = type_check_expr(program.expr(), env)?;
    Ok(TypedCoreProgram {
        expr: program.into_expr(),
        ty: checked.ty,
        row: checked.row,
        facts: checked.facts,
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

/// Normalizes a Core requirement row.
///
/// Exact duplicate items are removed, item-kind namespaces are preserved by the
/// `CoreRowItem` variant identity, and an open-row tail is kept unchanged.
///
/// # Errors
///
/// Returns [`CoreTypeCheckError::AmbiguousRowReference`] when an effect group
/// reference appears before alias/group expansion has happened.
pub fn normalize_core_row(row: &CoreRow) -> Result<CoreRow, CoreTypeCheckError> {
    let mut seen = HashSet::new();
    let mut items = Vec::with_capacity(row.items.len());

    for item in &row.items {
        reject_ambiguous_row_item(item)?;
        if seen.insert(item.clone()) {
            items.push(item.clone());
        }
    }

    Ok(CoreRow {
        items,
        tail: row.tail.clone(),
    })
}

/// Checks structural inclusion of two Core requirement rows.
///
/// `actual <= expected` means every normalized requirement in `actual` appears
/// in `expected`, possibly by solving one explicit open-row tail to the
/// structural remainder demanded by the comparison.
///
/// # Errors
///
/// Returns [`CoreTypeCheckError::AmbiguousRowReference`] when either row still
/// contains an unexpanded effect group reference.
pub fn core_row_included_in(
    actual: &CoreRow,
    expected: &CoreRow,
) -> Result<CoreRowComparison, CoreTypeCheckError> {
    let actual = normalize_core_row(actual)?;
    let expected = normalize_core_row(expected)?;
    let missing_items = row_difference(&actual.items, &expected.items);

    match (&actual.tail, &expected.tail) {
        (None, None) => Ok(CoreRowComparison {
            included: missing_items.is_empty(),
            missing_items,
            solutions: Vec::new(),
        }),
        (None, Some(expected_tail)) => {
            let remainder = row_difference(&actual.items, &expected.items);
            Ok(CoreRowComparison {
                included: true,
                missing_items: Vec::new(),
                solutions: vec![CoreRowSolution {
                    variable: expected_tail.clone(),
                    row: CoreRow::closed(remainder),
                }],
            })
        }
        (Some(actual_tail), None) => {
            if !missing_items.is_empty() {
                return Ok(CoreRowComparison {
                    included: false,
                    missing_items,
                    solutions: Vec::new(),
                });
            }

            let solution_row = CoreRow::closed(row_difference(&expected.items, &actual.items));

            Ok(CoreRowComparison {
                included: true,
                missing_items: Vec::new(),
                solutions: vec![CoreRowSolution {
                    variable: actual_tail.clone(),
                    row: solution_row,
                }],
            })
        }
        (Some(actual_tail), Some(expected_tail)) => Ok(CoreRowComparison {
            included: actual_tail == expected_tail && missing_items.is_empty(),
            missing_items,
            solutions: Vec::new(),
        }),
    }
}

/// Synthesizes the type of a Core atom.
///
/// # Errors
///
/// Returns [`CoreTypeCheckError`] when a referenced value or constructor cannot
/// be resolved, or when a primitive name has no first-slice function type.
pub fn synthesize_core_atom(
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
        CoreAtom::PrimName(op) => primitive_type(op),
        CoreAtom::ConstructorName(name) => {
            let Some(ty) = env.types().value_constructor(name).cloned() else {
                return Err(CoreTypeCheckError::UnknownValue { name: name.clone() });
            };
            check_core_type_well_formed(&ty, env)?;
            Ok(ty)
        }
    }
}

/// Synthesizes the type and construction row of an inert Core value.
///
/// # Errors
///
/// Returns [`CoreTypeCheckError`] when component atoms/types are not well
/// formed, or when a lambda body row does not match its latent row annotation.
pub fn synthesize_core_value(
    value: &CoreValue,
    env: &CoreTypeCheckEnv,
) -> Result<TypedCoreValue, CoreTypeCheckError> {
    let ty = match value {
        CoreValue::Atom(atom) => synthesize_core_atom(atom, env)?,
        CoreValue::Lam { params, body, row } => {
            let mut body_env = env.clone();
            let mut param_types = Vec::with_capacity(params.len());
            for param in params {
                check_core_type_well_formed(&param.ty, env)?;
                body_env
                    .values_mut()
                    .insert(param.name.clone(), param.ty.clone());
                param_types.push(param.ty.clone());
            }

            let body_checked = type_check_expr(body, &body_env)?;
            if normalize_core_row(&body_checked.row)? != normalize_core_row(row)? {
                return Err(CoreTypeCheckError::RowMismatch {
                    expected: row.clone(),
                    actual: body_checked.row,
                });
            }

            CoreType::Function {
                params: param_types,
                result: Box::new(body_checked.ty),
                row: row.clone(),
            }
        }
        CoreValue::Record { fields } => {
            let mut typed_fields = Vec::with_capacity(fields.len());
            for (name, atom) in fields {
                typed_fields.push((name.clone(), synthesize_core_atom(atom, env)?));
            }
            CoreType::Record(typed_fields)
        }
        CoreValue::Tuple { elems } => {
            let mut elem_types = Vec::with_capacity(elems.len());
            for elem in elems {
                elem_types.push(synthesize_core_atom(elem, env)?);
            }
            CoreType::Tuple(elem_types)
        }
        CoreValue::DischargeMarker { .. } => CoreType::Base("Unit".into()),
    };

    Ok(TypedCoreValue {
        ty,
        row: CoreRow::default(),
    })
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

    normalize_core_row(row)?;
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
                && rows_equivalent_unchecked(left_row, right_row)
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
                && rows_equivalent_unchecked(left_row, right_row)
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

fn rows_equivalent_unchecked(lhs: &CoreRow, rhs: &CoreRow) -> bool {
    match (normalize_core_row(lhs), normalize_core_row(rhs)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn row_difference(left: &[CoreRowItem], right: &[CoreRowItem]) -> Vec<CoreRowItem> {
    left.iter()
        .filter(|item| !right.contains(item))
        .cloned()
        .collect()
}

fn union_core_rows(lhs: &CoreRow, rhs: &CoreRow) -> Result<CoreRow, CoreTypeCheckError> {
    let left = normalize_core_row(lhs)?;
    let right = normalize_core_row(rhs)?;
    let mut items = left.items;
    for item in right.items {
        if !items.contains(&item) {
            items.push(item);
        }
    }

    let tail = match (left.tail, right.tail) {
        (None, None) => None,
        (Some(tail), None) | (None, Some(tail)) => Some(tail),
        (Some(left_tail), Some(right_tail)) if left_tail == right_tail => Some(left_tail),
        (Some(left_tail), Some(right_tail)) => {
            return Err(CoreTypeCheckError::AmbiguousRowReference {
                detail: format!(
                    "cannot union rows with different open tails `{left_tail}` and `{right_tail}`"
                ),
            });
        }
    };

    normalize_core_row(&CoreRow { items, tail })
}

fn reject_ambiguous_row_item(item: &CoreRowItem) -> Result<(), CoreTypeCheckError> {
    if let CoreRowItem::EffectGroupRef { path } = item {
        return Err(CoreTypeCheckError::AmbiguousRowReference {
            detail: format!(
                "effect group {} must be expanded before row comparison",
                path.join(".")
            ),
        });
    }
    Ok(())
}

fn typed_expr(ty: CoreType, row: CoreRow) -> TypedCoreExpr {
    TypedCoreExpr {
        ty,
        row,
        facts: CoreTypeCheckFacts::default(),
    }
}

fn merge_typecheck_facts(
    mut lhs: CoreTypeCheckFacts,
    rhs: CoreTypeCheckFacts,
) -> CoreTypeCheckFacts {
    lhs.jump_continuation_rows
        .extend(rhs.jump_continuation_rows);
    lhs
}

fn type_check_expr(
    expr: &CoreExpr,
    env: &CoreTypeCheckEnv,
) -> Result<TypedCoreExpr, CoreTypeCheckError> {
    match expr {
        CoreExpr::Atom(atom) => Ok(typed_expr(type_check_atom(atom, env)?, CoreRow::default())),
        CoreExpr::LetVal {
            name,
            ty,
            value,
            body,
        } => {
            check_core_type_well_formed(ty, env)?;
            check_value_against(value, ty, env)?;
            let mut body_env = env.clone();
            body_env.values_mut().insert(name.clone(), ty.clone());
            type_check_expr(body, &body_env)
        }
        CoreExpr::LetRec {
            name,
            ty,
            value,
            body,
        } => {
            check_core_type_well_formed(ty, env)?;
            let mut recursive_env = env.clone();
            recursive_env.values_mut().insert(name.clone(), ty.clone());
            check_value_against(value, ty, &recursive_env)?;
            type_check_expr(body, &recursive_env)
        }
        CoreExpr::LetPrim {
            name,
            op,
            args,
            body,
        } => {
            let result_ty = check_primitive_application(op, args, env)?;
            let mut body_env = env.clone();
            body_env.values_mut().insert(name.clone(), result_ty);
            type_check_expr(body, &body_env)
        }
        CoreExpr::LetCall {
            name,
            func,
            args,
            body,
        } => {
            let (result_ty, callee_row) = check_function_application(func, args, env)?;
            let mut body_env = env.clone();
            body_env.values_mut().insert(name.clone(), result_ty);
            let body_checked = type_check_expr(body, &body_env)?;
            Ok(TypedCoreExpr {
                ty: body_checked.ty,
                row: union_core_rows(&callee_row, &body_checked.row)?,
                facts: body_checked.facts,
            })
        }
        CoreExpr::If {
            cond,
            then_branch,
            else_branch,
        } => type_check_if(cond, then_branch, else_branch, env),
        CoreExpr::Call { func, args } => {
            let (result_ty, callee_row) = check_function_application(func, args, env)?;
            Ok(typed_expr(result_ty, callee_row))
        }
        CoreExpr::Jump { cont, arg } => type_check_jump(cont, arg, env),
        CoreExpr::Raise { op, args } => type_check_raise(op, args, env),
        CoreExpr::Handle { clause, body } => type_check_handle(clause, body, env),
        CoreExpr::RecordDischarge { .. } => Err(unsupported("RecordDischarge")),
        CoreExpr::Trap { .. } => Err(unsupported("Trap")),
    }
}

fn type_check_atom(
    atom: &CoreAtom,
    env: &CoreTypeCheckEnv,
) -> Result<CoreType, CoreTypeCheckError> {
    synthesize_core_atom(atom, env)
}

fn type_check_expr_against(
    expr: &CoreExpr,
    expected: &CoreType,
    env: &CoreTypeCheckEnv,
) -> Result<TypedCoreExpr, CoreTypeCheckError> {
    check_core_type_well_formed(expected, env)?;
    if let CoreExpr::Trap { .. } = expr {
        return Ok(typed_expr(expected.clone(), CoreRow::default()));
    }

    let checked = type_check_expr(expr, env)?;
    ensure_types_equivalent(expected, &checked.ty, env)?;
    Ok(checked)
}

fn type_check_if(
    cond: &CoreAtom,
    then_branch: &CoreExpr,
    else_branch: &CoreExpr,
    env: &CoreTypeCheckEnv,
) -> Result<TypedCoreExpr, CoreTypeCheckError> {
    ensure_types_equivalent(
        &CoreType::Base("Bool".into()),
        &type_check_atom(cond, env)?,
        env,
    )?;

    match (then_branch, else_branch) {
        (CoreExpr::Trap { .. }, CoreExpr::Trap { .. }) => {
            Err(unsupported("If with only Trap branches"))
        }
        (CoreExpr::Trap { .. }, _) => {
            let else_checked = type_check_expr(else_branch, env)?;
            let then_checked = type_check_expr_against(then_branch, &else_checked.ty, env)?;
            Ok(TypedCoreExpr {
                ty: else_checked.ty,
                row: union_core_rows(&then_checked.row, &else_checked.row)?,
                facts: merge_typecheck_facts(then_checked.facts, else_checked.facts),
            })
        }
        (_, CoreExpr::Trap { .. }) => {
            let then_checked = type_check_expr(then_branch, env)?;
            let else_checked = type_check_expr_against(else_branch, &then_checked.ty, env)?;
            Ok(TypedCoreExpr {
                ty: then_checked.ty,
                row: union_core_rows(&then_checked.row, &else_checked.row)?,
                facts: merge_typecheck_facts(then_checked.facts, else_checked.facts),
            })
        }
        _ => {
            let then_checked = type_check_expr(then_branch, env)?;
            let else_checked = type_check_expr(else_branch, env)?;
            ensure_types_equivalent(&then_checked.ty, &else_checked.ty, env)?;
            Ok(TypedCoreExpr {
                ty: then_checked.ty,
                row: union_core_rows(&then_checked.row, &else_checked.row)?,
                facts: merge_typecheck_facts(then_checked.facts, else_checked.facts),
            })
        }
    }
}

fn check_value_against(
    value: &CoreValue,
    expected: &CoreType,
    env: &CoreTypeCheckEnv,
) -> Result<(), CoreTypeCheckError> {
    let typed = synthesize_core_value(value, env)?;
    ensure_types_equivalent(expected, typed.ty(), env)?;
    if normalize_core_row(typed.row())? != CoreRow::default() {
        return Err(CoreTypeCheckError::RowMismatch {
            expected: CoreRow::default(),
            actual: typed.row().clone(),
        });
    }
    Ok(())
}

fn check_function_application(
    func: &CoreAtom,
    args: &[CoreAtom],
    env: &CoreTypeCheckEnv,
) -> Result<(CoreType, CoreRow), CoreTypeCheckError> {
    let func_ty = type_check_atom(func, env)?;
    check_core_type_well_formed(&func_ty, env)?;
    let CoreType::Function {
        params,
        result,
        row,
    } = func_ty
    else {
        return Err(unsupported("non-function call target"));
    };

    check_arguments(&params, args, env)?;
    Ok((*result, row))
}

fn type_check_raise(
    op: &CoreEffectOp,
    args: &[CoreAtom],
    env: &CoreTypeCheckEnv,
) -> Result<TypedCoreExpr, CoreTypeCheckError> {
    let (arg_types, result_type, row) = effect_operation_signature(op, env)?;
    if !env.operations().contains(op) {
        return Err(CoreTypeCheckError::UnknownOperation {
            detail: effect_operation_detail(op),
        });
    }

    check_arguments(&arg_types, args, env)?;
    Ok(typed_expr(result_type, row))
}

fn type_check_handle(
    clause: &CoreHandlerClause,
    body: &CoreExpr,
    env: &CoreTypeCheckEnv,
) -> Result<TypedCoreExpr, CoreTypeCheckError> {
    let (arg_types, op_result_ty, op_row) = effect_operation_signature(&clause.op, env)?;
    if !env.operations().contains(&clause.op) {
        return Err(CoreTypeCheckError::UnknownOperation {
            detail: effect_operation_detail(&clause.op),
        });
    }

    check_handler_params(&clause.params, &arg_types, env)?;
    let resume_row = check_handler_resume(&clause.resume.ty, &op_result_ty, env)?;

    let mut clause_env = env.clone();
    for param in &clause.params {
        clause_env
            .values_mut()
            .insert(param.name.clone(), param.ty.clone());
    }
    clause_env
        .continuations_mut()
        .insert(clause.resume.name.clone(), clause.resume.ty.clone());

    let clause_checked = type_check_expr(&clause.body, &clause_env)?;
    let expected_clause_row = normalize_core_row(&clause.row)?;
    let actual_clause_row = normalize_core_row(&clause_checked.row)?;
    if actual_clause_row != expected_clause_row {
        return Err(CoreTypeCheckError::RowMismatch {
            expected: clause.row.clone(),
            actual: clause_checked.row,
        });
    }

    let body_checked = type_check_expr(body, env)?;
    let residual = handle_residual_row(&body_checked.row, &op_row, &resume_row, &clause.row)?;
    Ok(TypedCoreExpr {
        ty: body_checked.ty,
        row: residual,
        facts: merge_typecheck_facts(body_checked.facts, clause_checked.facts),
    })
}

fn check_handler_params(
    params: &[CoreParam],
    expected_types: &[CoreType],
    env: &CoreTypeCheckEnv,
) -> Result<(), CoreTypeCheckError> {
    if params.len() != expected_types.len() {
        return Err(CoreTypeCheckError::ArgumentCountMismatch {
            expected: expected_types.len(),
            actual: params.len(),
        });
    }

    for (param, expected_ty) in params.iter().zip(expected_types) {
        check_core_type_well_formed(&param.ty, env)?;
        ensure_types_equivalent(expected_ty, &param.ty, env)?;
    }
    Ok(())
}

fn check_handler_resume(
    resume_ty: &CoreType,
    op_result_ty: &CoreType,
    env: &CoreTypeCheckEnv,
) -> Result<CoreRow, CoreTypeCheckError> {
    check_core_type_well_formed(resume_ty, env)?;
    let CoreType::Cont {
        input,
        row,
        multiplicity,
        ..
    } = resume_ty
    else {
        return Err(unsupported("handler resume without continuation type"));
    };

    if *multiplicity != CoreMultiplicity::Affine {
        return Err(unsupported("handler resume with non-affine multiplicity"));
    }

    ensure_types_equivalent(op_result_ty, input, env)?;
    Ok(row.clone())
}

fn handle_residual_row(
    body_row: &CoreRow,
    op_row: &CoreRow,
    resume_row: &CoreRow,
    clause_row: &CoreRow,
) -> Result<CoreRow, CoreTypeCheckError> {
    let body_without_op = subtract_core_row(body_row, op_row)?;
    union_core_rows(&union_core_rows(&body_without_op, resume_row)?, clause_row)
}

fn subtract_core_row(lhs: &CoreRow, rhs: &CoreRow) -> Result<CoreRow, CoreTypeCheckError> {
    let left = normalize_core_row(lhs)?;
    let right = normalize_core_row(rhs)?;
    Ok(CoreRow {
        items: row_difference(&left.items, &right.items),
        tail: left.tail,
    })
}

fn effect_operation_signature(
    op: &CoreEffectOp,
    env: &CoreTypeCheckEnv,
) -> Result<(Vec<CoreType>, CoreType, CoreRow), CoreTypeCheckError> {
    match op {
        CoreEffectOp::Capability {
            path,
            operation,
            arg_types,
            result_type,
        } => {
            check_types_well_formed(arg_types, env)?;
            check_core_type_well_formed(result_type, env)?;
            Ok((
                arg_types.clone(),
                result_type.clone(),
                CoreRow::closed(vec![CoreRowItem::Capability {
                    path: path.clone(),
                    operation: operation.clone(),
                }]),
            ))
        }
        CoreEffectOp::Channel {
            path,
            mode,
            payload_type,
            result_type,
        } => {
            check_core_type_well_formed(payload_type, env)?;
            check_core_type_well_formed(result_type, env)?;
            Ok((
                vec![payload_type.clone()],
                result_type.clone(),
                CoreRow::closed(vec![CoreRowItem::Channel {
                    path: path.clone(),
                    mode: mode.clone(),
                    payload_type: Box::new(payload_type.clone()),
                }]),
            ))
        }
        CoreEffectOp::Process {
            operation,
            arg_types,
            result_type,
        } => {
            check_types_well_formed(arg_types, env)?;
            check_core_type_well_formed(result_type, env)?;
            Ok((
                arg_types.clone(),
                result_type.clone(),
                CoreRow::closed(vec![CoreRowItem::Process {
                    operation: operation.clone(),
                }]),
            ))
        }
        CoreEffectOp::Failure { ty } => {
            if let Some(ty) = ty {
                check_core_type_well_formed(ty, env)?;
            }
            let arg_types = ty.iter().cloned().collect();
            let result_type = CoreType::Named("Never".into());
            check_core_type_well_formed(&result_type, env)?;
            Ok((
                arg_types,
                result_type,
                CoreRow::closed(vec![CoreRowItem::Failure {
                    ty: ty.clone().map(Box::new),
                }]),
            ))
        }
    }
}

fn effect_operation_detail(op: &CoreEffectOp) -> String {
    match op {
        CoreEffectOp::Capability {
            path, operation, ..
        } => format!("cap {}", dotted_name(path, operation)),
        CoreEffectOp::Channel { path, mode, .. } => {
            format!("channel {}", dotted_name(path, mode))
        }
        CoreEffectOp::Process { operation, .. } => format!("proc {operation}"),
        CoreEffectOp::Failure { ty } => ty
            .as_ref()
            .map(|ty| format!("fail {}", type_detail(ty)))
            .unwrap_or_else(|| "fail failure".to_owned()),
    }
}

fn dotted_name(path: &[String], leaf: &str) -> String {
    if path.is_empty() {
        leaf.to_owned()
    } else {
        format!("{}.{}", path.join("."), leaf)
    }
}

fn type_detail(ty: &CoreType) -> String {
    match ty {
        CoreType::Base(name) | CoreType::Named(name) | CoreType::Var(name) => name.clone(),
        CoreType::Function { .. } => "function".to_owned(),
        CoreType::Refinement { base, predicate } => {
            format!("{}|{predicate}", type_detail(base))
        }
        CoreType::Cont { .. } => "continuation".to_owned(),
        CoreType::Tuple(elems) => format!("tuple/{}", elems.len()),
        CoreType::Record(fields) => format!("record/{}", fields.len()),
        CoreType::App { name, args } => format!("{name}/{}", args.len()),
    }
}

fn check_primitive_application(
    op: &CorePrimOp,
    args: &[CoreAtom],
    env: &CoreTypeCheckEnv,
) -> Result<CoreType, CoreTypeCheckError> {
    let CoreType::Function {
        params,
        result,
        row,
    } = primitive_type(op)?
    else {
        return Err(unsupported("primitive without function type"));
    };

    check_arguments(&params, args, env)?;

    if normalize_core_row(&row)? != CoreRow::default() {
        return Err(CoreTypeCheckError::RowMismatch {
            expected: CoreRow::default(),
            actual: row,
        });
    }

    Ok(*result)
}

fn check_arguments(
    params: &[CoreType],
    args: &[CoreAtom],
    env: &CoreTypeCheckEnv,
) -> Result<(), CoreTypeCheckError> {
    if params.len() != args.len() {
        return Err(CoreTypeCheckError::ArgumentCountMismatch {
            expected: params.len(),
            actual: args.len(),
        });
    }

    for (arg, expected) in args.iter().zip(params) {
        let actual = type_check_atom(arg, env)?;
        ensure_types_equivalent(expected, &actual, env)?;
    }

    Ok(())
}

fn type_check_jump(
    cont: &CoreContRef,
    arg: &CoreAtom,
    env: &CoreTypeCheckEnv,
) -> Result<TypedCoreExpr, CoreTypeCheckError> {
    let Some(cont_ty) = env.continuations().lookup(cont_ref_name(cont)).cloned() else {
        return Err(CoreTypeCheckError::UnknownContinuation {
            name: cont_ref_name(cont).to_owned(),
        });
    };
    check_core_type_well_formed(&cont_ty, env)?;
    let CoreType::Cont {
        input, answer, row, ..
    } = cont_ty
    else {
        return Err(unsupported("non-continuation jump target"));
    };

    let actual = type_check_atom(arg, env)?;
    ensure_types_equivalent(&input, &actual, env)?;

    let mut facts = CoreTypeCheckFacts::default();
    facts
        .jump_continuation_rows
        .insert(cont.clone(), row.clone());
    Ok(TypedCoreExpr {
        ty: *answer,
        row: CoreRow::default(),
        facts,
    })
}

fn cont_ref_name(cont: &CoreContRef) -> &str {
    match cont {
        CoreContRef::Label(name) | CoreContRef::Var(name) => name,
    }
}

fn ensure_types_equivalent(
    expected: &CoreType,
    actual: &CoreType,
    env: &CoreTypeCheckEnv,
) -> Result<(), CoreTypeCheckError> {
    if core_types_equivalent(expected, actual, env)? {
        Ok(())
    } else {
        Err(CoreTypeCheckError::TypeMismatch {
            expected: Box::new(expected.clone()),
            actual: Box::new(actual.clone()),
        })
    }
}

fn unsupported(form: &str) -> CoreTypeCheckError {
    CoreTypeCheckError::UnsupportedCoreForm {
        detail: form.to_owned(),
    }
}

fn primitive_type(op: &CorePrimOp) -> Result<CoreType, CoreTypeCheckError> {
    let int = CoreType::Base("Int".into());
    let bool_ty = CoreType::Base("Bool".into());
    let unary_int = || CoreType::Function {
        params: vec![int.clone()],
        result: Box::new(int.clone()),
        row: CoreRow::default(),
    };
    let binary_int = || CoreType::Function {
        params: vec![int.clone(), int.clone()],
        result: Box::new(int.clone()),
        row: CoreRow::default(),
    };
    let binary_int_to_bool = || CoreType::Function {
        params: vec![int.clone(), int.clone()],
        result: Box::new(bool_ty.clone()),
        row: CoreRow::default(),
    };

    match op {
        CorePrimOp::Add | CorePrimOp::Sub | CorePrimOp::Mul | CorePrimOp::Div => Ok(binary_int()),
        CorePrimOp::Eq
        | CorePrimOp::Ne
        | CorePrimOp::Lt
        | CorePrimOp::Le
        | CorePrimOp::Gt
        | CorePrimOp::Ge => Ok(binary_int_to_bool()),
        CorePrimOp::Neg => Ok(unary_int()),
        CorePrimOp::Not => Ok(CoreType::Function {
            params: vec![bool_ty.clone()],
            result: Box::new(bool_ty),
            row: CoreRow::default(),
        }),
        CorePrimOp::RecordGet(_) | CorePrimOp::TupleGet(_) | CorePrimOp::ConstructorTag(_) => {
            Err(unsupported("structural primitive name atom"))
        }
    }
}
