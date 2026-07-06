//! Type-checking boundary for Core Ash programs.
//!
//! Phase 162 starts with a deliberately small checker API over validated Core
//! programs. Later tasks extend the expression rules; unsupported forms fail
//! closed instead of being accepted optimistically.

use crate::core_ash::{
    CoreAtom, CoreContRef, CoreContractDischarge, CoreDischargeMode, CoreEffectOp, CoreEvalMode,
    CoreEvidenceStatus, CoreExpr, CoreHandlerClause, CoreMultiplicity, CoreName, CoreParam,
    CorePrimOp, CoreRow, CoreRowItem, CoreThunkMode, CoreType, CoreValue,
};
use crate::core_ash_lower::{
    CoreLoweringContext, CoreLoweringError, lower_core_program_with_context_and_letcall_rows,
};
use crate::core_ash_validate::ValidCoreProgram;
use crate::cps::Term;
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

    /// Returns refinement proof obligations emitted while checking the program.
    #[must_use]
    pub fn obligations(&self) -> &[CoreRefinementObligation] {
        self.facts.refinement_obligations()
    }

    /// Returns validated discharge records encountered while checking the program.
    #[must_use]
    pub fn discharges(&self) -> &[CoreContractDischarge] {
        self.facts.discharges()
    }
}

/// A Core program that has been type-checked and lowered using checked facts.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckedLoweredCoreProgram {
    typed: TypedCoreProgram,
    lowered: Term,
}

impl CheckedLoweredCoreProgram {
    /// Returns the typed Core program.
    #[must_use]
    pub fn typed(&self) -> &TypedCoreProgram {
        &self.typed
    }

    /// Returns the lowered CPS term.
    #[must_use]
    pub fn lowered(&self) -> &Term {
        &self.lowered
    }

    /// Consumes the wrapper and returns both checked and lowered artifacts.
    #[must_use]
    pub fn into_parts(self) -> (TypedCoreProgram, Term) {
        (self.typed, self.lowered)
    }
}

/// Typed facts computed during Core type checking for later compiler stages.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreTypeCheckFacts {
    jump_continuation_rows: HashMap<CoreContRef, CoreRow>,
    mode_binding_latent_rows: HashMap<CoreName, CoreRow>,
    refinement_obligations: Vec<CoreRefinementObligation>,
    discharges: Vec<CoreContractDischarge>,
}

impl CoreTypeCheckFacts {
    /// Returns rows of target continuations reached by checked `Jump` expressions.
    #[must_use]
    pub fn jump_continuation_rows(&self) -> &HashMap<CoreContRef, CoreRow> {
        &self.jump_continuation_rows
    }

    /// Returns latent rows inferred for local `LetMode` mode bindings.
    #[must_use]
    pub fn mode_binding_latent_rows(&self) -> &HashMap<CoreName, CoreRow> {
        &self.mode_binding_latent_rows
    }

    /// Returns refinement obligations emitted by annotation checking.
    #[must_use]
    pub fn refinement_obligations(&self) -> &[CoreRefinementObligation] {
        &self.refinement_obligations
    }

    /// Returns validated discharge metadata records.
    #[must_use]
    pub fn discharges(&self) -> &[CoreContractDischarge] {
        &self.discharges
    }
}

/// A refinement proof obligation emitted by Core annotation checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreRefinementObligation {
    predicate: String,
    value_name: Option<CoreName>,
    base_type: CoreType,
    refinement_type: CoreType,
}

impl CoreRefinementObligation {
    /// Returns the textual refinement predicate.
    #[must_use]
    pub fn predicate(&self) -> &str {
        &self.predicate
    }

    /// Returns the binding name associated with the obligation, when known.
    #[must_use]
    pub fn value_name(&self) -> Option<&str> {
        self.value_name.as_deref()
    }

    /// Returns the base type being refined.
    #[must_use]
    pub fn base_type(&self) -> &CoreType {
        &self.base_type
    }

    /// Returns the target refinement type.
    #[must_use]
    pub fn refinement_type(&self) -> &CoreType {
        &self.refinement_type
    }
}

/// Public summary of one Core function type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorePublicFunctionSummary {
    exported_name: CoreName,
    params: Vec<CoreType>,
    result: CoreType,
    row: CorePublicRowSummary,
    type_constructors: Vec<CorePublicTypeConstructorSummary>,
    refinement_obligations: Vec<CoreRefinementObligation>,
    discharges: Vec<CoreContractDischarge>,
}

impl CorePublicFunctionSummary {
    /// Returns the exported function name represented by this summary.
    #[must_use]
    pub fn exported_name(&self) -> &str {
        &self.exported_name
    }

    /// Returns the public parameter types.
    #[must_use]
    pub fn params(&self) -> &[CoreType] {
        &self.params
    }

    /// Returns the public result type.
    #[must_use]
    pub fn result(&self) -> &CoreType {
        &self.result
    }

    /// Returns the normalized public requirement row summary.
    #[must_use]
    pub fn row(&self) -> &CorePublicRowSummary {
        &self.row
    }

    /// Returns named type constructors referenced by the public function type.
    #[must_use]
    pub fn type_constructors(&self) -> &[CorePublicTypeConstructorSummary] {
        &self.type_constructors
    }

    /// Returns public refinement obligations retained for downstream checking.
    #[must_use]
    pub fn refinement_obligations(&self) -> &[CoreRefinementObligation] {
        &self.refinement_obligations
    }

    /// Returns discharge metadata retained for downstream checking.
    #[must_use]
    pub fn discharges(&self) -> &[CoreContractDischarge] {
        &self.discharges
    }
}

/// Public summary of a normalized Core requirement row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorePublicRowSummary {
    items: Vec<CorePublicRowItemSummary>,
    tail: Option<CoreName>,
}

impl CorePublicRowSummary {
    /// Returns normalized public row items.
    #[must_use]
    pub fn items(&self) -> &[CorePublicRowItemSummary] {
        &self.items
    }

    /// Returns the open-row tail, when the public row remains polymorphic.
    #[must_use]
    pub fn tail(&self) -> Option<&str> {
        self.tail.as_deref()
    }
}

/// Public summary of one normalized Core row item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorePublicRowItemSummary {
    Capability {
        path: Vec<String>,
        operation: String,
    },
    Resource {
        path: Vec<String>,
        mode: String,
    },
    Role {
        path: Vec<String>,
    },
    Policy {
        path: Vec<String>,
    },
    Contract {
        contract: String,
    },
    Channel {
        path: Vec<String>,
        mode: String,
        payload_type: Box<CoreType>,
    },
    Process {
        operation: String,
    },
    Failure {
        ty: Option<Box<CoreType>>,
    },
    Evidence {
        path: Vec<String>,
    },
}

/// Public type-constructor identity and arity referenced by a summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorePublicTypeConstructorSummary {
    name: CoreName,
    arity: usize,
}

impl CorePublicTypeConstructorSummary {
    /// Returns the constructor name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the constructor arity visible in the public type.
    #[must_use]
    pub fn arity(&self) -> usize {
        self.arity
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

    /// A record type used the same field name more than once.
    #[error("duplicate record field `{field}`")]
    DuplicateRecordField { field: CoreName },

    /// A discharge record was malformed or did not prove a hard refinement.
    #[error("invalid discharge: {detail}")]
    InvalidDischarge { detail: String },

    /// Two rows were expected to match but did not.
    #[error("row mismatch")]
    RowMismatch { expected: CoreRow, actual: CoreRow },

    /// Two mode types are structurally incompatible.
    #[error("mode type mismatch: expected mode `{expected:?}`, got `{actual:?}`")]
    ModeTypeMismatch {
        expected: CoreEvalMode,
        actual: CoreEvalMode,
    },

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

    /// A mode type has an invalid latent-row shape.
    #[error("invalid mode type: {detail}")]
    InvalidModeType { detail: String },

    /// A mode binding annotation row does not match the inferred latent row.
    #[error("mode latent row mismatch for `{name}`")]
    ModeLatentRowMismatch {
        name: CoreName,
        expected: CoreRow,
        actual: CoreRow,
    },
}

/// Error returned while constructing public Core summaries.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CorePublicSummaryError {
    /// A private or ambiguous row reference would leak through a public summary.
    #[error("private row reference `{}` in public summary", path.join("."))]
    PrivateRowReference {
        path: Vec<String>,
        public_item: Option<CoreName>,
        detail: String,
    },

    /// A non-function type was summarized as a public function.
    #[error("invalid public function summary: {detail}")]
    InvalidFunctionType { detail: String },

    /// A row failed normalization before summary emission.
    #[error("invalid public row summary: {detail}")]
    InvalidRow { detail: String },
}

/// Error returned by the checked Core type-check-and-lower integration path.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CoreCheckedLoweringError {
    /// Core type checking failed before lowering started.
    #[error(transparent)]
    TypeCheck(#[from] CoreTypeCheckError),

    /// Core-to-CPS lowering failed after successful type checking.
    #[error(transparent)]
    Lower(#[from] CoreLoweringError),
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
    facts: CoreTypeCheckFacts,
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

    /// Returns metadata facts produced while synthesizing this value.
    #[must_use]
    pub fn facts(&self) -> &CoreTypeCheckFacts {
        &self.facts
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

/// Type-checks a validated Core program, then lowers it using checked facts.
///
/// This is the integration boundary for consumers that need lowering to honor
/// typechecker facts such as continuation rows discovered for `Jump`.
///
/// # Errors
///
/// Returns [`CoreCheckedLoweringError::TypeCheck`] if type checking fails, and
/// [`CoreCheckedLoweringError::Lower`] if lowering fails after successful type
/// checking.
pub fn type_check_and_lower_core_program(
    program: ValidCoreProgram,
    env: &CoreTypeCheckEnv,
    context: CoreLoweringContext,
) -> Result<CheckedLoweredCoreProgram, CoreCheckedLoweringError> {
    let typed = type_check_core_program(program.clone(), env)?;
    let mut letcall_function_rows = HashMap::new();
    collect_letcall_function_rows(
        program.expr(),
        env,
        &mut Vec::new(),
        &mut letcall_function_rows,
    )?;
    let context = lowering_context_with_checked_facts(context, env, typed.facts());
    let lowered =
        lower_core_program_with_context_and_letcall_rows(program, context, &letcall_function_rows)?;
    Ok(CheckedLoweredCoreProgram { typed, lowered })
}

fn collect_letcall_function_rows(
    expr: &CoreExpr,
    env: &CoreTypeCheckEnv,
    path: &mut Vec<usize>,
    rows: &mut HashMap<Vec<usize>, CoreRow>,
) -> Result<(), CoreTypeCheckError> {
    match expr {
        CoreExpr::Atom(_)
        | CoreExpr::Call { .. }
        | CoreExpr::Jump { .. }
        | CoreExpr::LetContCall { .. }
        | CoreExpr::Trap { .. } => Ok(()),
        CoreExpr::LetVal {
            name,
            ty,
            value,
            body,
        } => {
            let mut body_env = env.clone();
            body_env.values_mut().insert(name.clone(), ty.clone());

            path.push(0);
            collect_letcall_function_rows_in_value(value, env, path, rows)?;
            path.pop();

            path.push(1);
            collect_letcall_function_rows(body, &body_env, path, rows)?;
            path.pop();
            Ok(())
        }
        CoreExpr::LetRec {
            name,
            ty,
            value,
            body,
        } => {
            let mut recursive_env = env.clone();
            recursive_env.values_mut().insert(name.clone(), ty.clone());

            path.push(0);
            collect_letcall_function_rows_in_value(value, &recursive_env, path, rows)?;
            path.pop();

            path.push(1);
            collect_letcall_function_rows(body, &recursive_env, path, rows)?;
            path.pop();
            Ok(())
        }
        CoreExpr::LetPrim { body, .. } => {
            path.push(0);
            let result = collect_letcall_function_rows(body, env, path, rows);
            path.pop();
            result
        }
        CoreExpr::LetMode {
            name,
            ty,
            expr,
            body,
            ..
        } => {
            path.push(0);
            let left = collect_letcall_function_rows(expr, env, path, rows);
            path.pop();

            let mut body_env = env.clone();
            body_env.values_mut().insert(name.clone(), ty.clone());

            path.push(1);
            let right = collect_letcall_function_rows(body, &body_env, path, rows);
            path.pop();

            left.and(right)
        }
        CoreExpr::LetCall {
            name,
            func,
            args,
            body,
            ..
        } => {
            let (result_ty, _, _) = check_function_application(func, args, env)?;
            if let CoreType::Function { row, .. } = &result_ty {
                rows.insert(path.clone(), row.clone());
            }

            let mut body_env = env.clone();
            body_env
                .values_mut()
                .insert(name.clone(), result_ty.clone());

            path.push(0);
            let result = collect_letcall_function_rows(body, &body_env, path, rows);
            path.pop();
            result
        }
        CoreExpr::If {
            then_branch,
            else_branch,
            ..
        } => {
            path.push(0);
            let then_result = collect_letcall_function_rows(then_branch, env, path, rows);
            path.pop();

            path.push(1);
            let else_result = collect_letcall_function_rows(else_branch, env, path, rows);
            path.pop();

            then_result.and(else_result)
        }
        CoreExpr::Raise { .. } => Ok(()),
        CoreExpr::Handle { clause, body } => {
            let mut clause_env = env.clone();
            for param in &clause.params {
                clause_env
                    .values_mut()
                    .insert(param.name.clone(), param.ty.clone());
            }

            path.push(0);
            collect_letcall_function_rows(&clause.body, &clause_env, path, rows)?;
            path.pop();

            path.push(1);
            collect_letcall_function_rows(body, env, path, rows)?;
            path.pop();
            Ok(())
        }
        CoreExpr::Force { name, thunk, body } => {
            let mut body_env = env.clone();
            if let CoreAtom::Var(thunk_name) = thunk
                && let Some(CoreType::Mode {
                    mode: CoreEvalMode::Lazy | CoreEvalMode::Memo,
                    inner,
                    ..
                }) = env.values().lookup(thunk_name)
            {
                body_env
                    .values_mut()
                    .insert(name.clone(), inner.as_ref().clone());
            }

            path.push(0);
            let result = collect_letcall_function_rows(body, &body_env, path, rows);
            path.pop();
            result
        }
        CoreExpr::RecordDischarge { body, .. } => {
            path.push(0);
            let result = collect_letcall_function_rows(body, env, path, rows);
            path.pop();
            result
        }
    }
}

fn collect_letcall_function_rows_in_value(
    value: &CoreValue,
    env: &CoreTypeCheckEnv,
    path: &mut Vec<usize>,
    rows: &mut HashMap<Vec<usize>, CoreRow>,
) -> Result<(), CoreTypeCheckError> {
    match value {
        CoreValue::Atom(_) => Ok(()),
        CoreValue::Lam { params, body, .. } => {
            let mut body_env = env.clone();
            for param in params {
                body_env
                    .values_mut()
                    .insert(param.name.clone(), param.ty.clone());
            }

            path.push(0);
            let result = collect_letcall_function_rows(body, &body_env, path, rows);
            path.pop();
            result
        }
        CoreValue::Record { .. } | CoreValue::Tuple { .. } | CoreValue::DischargeMarker { .. } => {
            Ok(())
        }
        CoreValue::Thunk { body, .. } => collect_letcall_function_rows(body, env, path, rows),
    }
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
            input,
            answer,
            row,
            multiplicity,
        } => {
            check_core_type_well_formed(input, env)?;
            check_core_type_well_formed(answer, env)?;
            check_core_row_well_formed(row, env)?;
            // SPEC-102 §4/§8: MultiShotPure requires a normalized closed empty row.
            // Affine is valid with any row.
            if *multiplicity == CoreMultiplicity::MultiShotPure {
                if !row.items.is_empty() {
                    return Err(unsupported(
                        "multi-shot-pure continuation must have a closed empty row, \
                         but the row has non-empty items",
                    ));
                }
                if row.tail.is_some() {
                    return Err(unsupported(
                        "multi-shot-pure continuation must have a closed empty row, \
                         but the row has an open tail",
                    ));
                }
            }
            Ok(())
        }
        CoreType::Tuple(elems) => check_types_well_formed(elems, env),
        CoreType::Record(fields) => {
            if let Some(field) = duplicate_record_field_name(fields) {
                return Err(CoreTypeCheckError::DuplicateRecordField { field });
            }
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
        CoreType::Mode {
            mode,
            inner,
            latent_row,
        } => {
            check_core_type_well_formed(inner, env)?;
            match mode {
                CoreEvalMode::Strict => {
                    if latent_row.is_some() {
                        return Err(CoreTypeCheckError::InvalidModeType {
                            detail: "strict mode requires no latent row".to_owned(),
                        });
                    }
                }
                CoreEvalMode::Lazy | CoreEvalMode::Memo => {
                    let Some(row) = latent_row else {
                        return Err(CoreTypeCheckError::InvalidModeType {
                            detail: match mode {
                                CoreEvalMode::Lazy => "lazy mode requires a latent row".to_owned(),
                                CoreEvalMode::Memo => "memo mode requires a latent row".to_owned(),
                                CoreEvalMode::Strict => unreachable!(),
                            },
                        });
                    };
                    check_core_row_well_formed(row, env)?;
                }
            }
            Ok(())
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

/// Checks exact inclusion of two Core requirement rows.
///
/// `actual <= expected` means every normalized requirement in `actual` appears
/// in `expected`, possibly by solving one explicit open-row tail to the
/// exact remainder demanded by the comparison.
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

fn core_row_included_in_env(
    actual: &CoreRow,
    expected: &CoreRow,
    env: &CoreTypeCheckEnv,
) -> Result<CoreRowComparison, CoreTypeCheckError> {
    let actual = normalize_core_row_structural(actual, env)?;
    let expected = normalize_core_row_structural(expected, env)?;
    let missing_items = row_difference_structural(&actual.items, &expected.items, env)?;

    match (&actual.tail, &expected.tail) {
        (None, None) => Ok(CoreRowComparison {
            included: missing_items.is_empty(),
            missing_items,
            solutions: Vec::new(),
        }),
        (None, Some(expected_tail)) => {
            let remainder = row_difference_structural(&actual.items, &expected.items, env)?;
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

            let solution_row = CoreRow::closed(row_difference_structural(
                &expected.items,
                &actual.items,
                env,
            )?);

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

/// Builds a public summary for a normalized Core requirement row.
///
/// # Errors
///
/// Returns [`CorePublicSummaryError`] when the row contains a private or
/// ambiguous group reference, or when normalization fails.
pub fn summarize_core_public_row(
    row: &CoreRow,
) -> Result<CorePublicRowSummary, CorePublicSummaryError> {
    for item in &row.items {
        if let CoreRowItem::EffectGroupRef { path } = item {
            return Err(CorePublicSummaryError::PrivateRowReference {
                path: path.clone(),
                public_item: None,
                detail: format!(
                    "private effect group {} must be expanded or exported before summary",
                    path.join(".")
                ),
            });
        }
    }

    let normalized = normalize_core_row(row).map_err(|err| CorePublicSummaryError::InvalidRow {
        detail: err.to_string(),
    })?;

    let items = normalized
        .items
        .iter()
        .map(public_row_item_summary)
        .collect();

    Ok(CorePublicRowSummary {
        items,
        tail: normalized.tail,
    })
}

/// Builds a public summary for a Core function type.
///
/// # Errors
///
/// Returns [`CorePublicSummaryError`] when `ty` is not a function type or when
/// its row cannot be exported safely.
pub fn summarize_core_public_function_type(
    exported_name: impl Into<CoreName>,
    ty: &CoreType,
    obligations: &[CoreRefinementObligation],
    discharges: &[CoreContractDischarge],
) -> Result<CorePublicFunctionSummary, CorePublicSummaryError> {
    let CoreType::Function {
        params,
        result,
        row,
    } = ty
    else {
        return Err(CorePublicSummaryError::InvalidFunctionType {
            detail: "public function summary requires a function type".to_owned(),
        });
    };

    let mut type_constructors = Vec::new();
    for param in params {
        collect_public_type_constructors(param, &mut type_constructors)?;
    }
    collect_public_type_constructors(result, &mut type_constructors)?;
    for item in &row.items {
        collect_public_row_item_type_constructors(item, &mut type_constructors)?;
    }

    Ok(CorePublicFunctionSummary {
        exported_name: exported_name.into(),
        params: params.clone(),
        result: (**result).clone(),
        row: summarize_core_public_row(row)?,
        type_constructors,
        refinement_obligations: obligations.to_vec(),
        discharges: discharges.to_vec(),
    })
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
        CoreAtom::Var(name) => {
            let ty = env
                .values()
                .lookup(name)
                .cloned()
                .ok_or_else(|| CoreTypeCheckError::UnknownValue { name: name.clone() })?;
            check_core_type_well_formed(&ty, env)?;
            Ok(ty)
        }
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
    let (ty, facts) = match value {
        CoreValue::Atom(atom) => (
            synthesize_core_atom(atom, env)?,
            CoreTypeCheckFacts::default(),
        ),
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

            check_core_row_well_formed(row, env)?;

            let body_checked = type_check_expr(body, &body_env)?;
            let comparison = core_row_included_in_env(&body_checked.row, row, env)?;
            if !comparison.is_included() {
                return Err(CoreTypeCheckError::RowMismatch {
                    expected: row.clone(),
                    actual: body_checked.row,
                });
            }

            (
                CoreType::Function {
                    params: param_types,
                    result: Box::new(body_checked.ty),
                    row: row.clone(),
                },
                body_checked.facts,
            )
        }
        CoreValue::Record { fields } => {
            let mut typed_fields = Vec::with_capacity(fields.len());
            for (name, atom) in fields {
                typed_fields.push((name.clone(), synthesize_core_atom(atom, env)?));
            }
            (
                CoreType::Record(typed_fields),
                CoreTypeCheckFacts::default(),
            )
        }
        CoreValue::Tuple { elems } => {
            let mut elem_types = Vec::with_capacity(elems.len());
            for elem in elems {
                elem_types.push(synthesize_core_atom(elem, env)?);
            }
            (CoreType::Tuple(elem_types), CoreTypeCheckFacts::default())
        }
        CoreValue::DischargeMarker { discharge } => {
            validate_discharge_marker(discharge, env)?;
            let mut facts = CoreTypeCheckFacts::default();
            facts.discharges.push(discharge.clone());
            return Ok(TypedCoreValue {
                ty: CoreType::Base("Unit".into()),
                row: CoreRow::default(),
                facts,
            });
        }
        CoreValue::Thunk {
            mode: thunk_mode,
            result_ty,
            body,
            row,
            captures: _,
        } => {
            if matches!(result_ty, CoreType::Mode { .. }) {
                return Err(CoreTypeCheckError::InvalidModeType {
                    detail: "thunk result type must not be a mode type".to_owned(),
                });
            }

            check_core_row_well_formed(row, env)?;
            let body_checked = type_check_expr_against(body, result_ty, env)?;

            if !rows_equivalent(row, &body_checked.row, env)? {
                return Err(CoreTypeCheckError::RowMismatch {
                    expected: row.clone(),
                    actual: body_checked.row,
                });
            }

            let mode = match thunk_mode {
                CoreThunkMode::Lazy => CoreEvalMode::Lazy,
                CoreThunkMode::Memo => CoreEvalMode::Memo,
            };
            (
                CoreType::Mode {
                    mode,
                    inner: Box::new(result_ty.clone()),
                    latent_row: Some(row.clone()),
                },
                body_checked.facts,
            )
        }
    };

    Ok(TypedCoreValue {
        ty,
        row: CoreRow::default(),
        facts,
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
                && rows_equivalent_unchecked(left_row, right_row, env)
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
                && rows_equivalent_unchecked(left_row, right_row, env)
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
        (
            CoreType::Mode {
                mode: left_mode,
                inner: left_inner,
                latent_row: left_latent_row,
            },
            CoreType::Mode {
                mode: right_mode,
                inner: right_inner,
                latent_row: right_latent_row,
            },
        ) => {
            left_mode == right_mode
                && types_equivalent_unchecked(left_inner, right_inner, env)
                && match (left_latent_row, right_latent_row) {
                    (None, None) => true,
                    (Some(left_row), Some(right_row)) => {
                        rows_equivalent_unchecked(left_row, right_row, env)
                    }
                    _ => false,
                }
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
    if lhs.len() != rhs.len()
        || has_duplicate_record_field_names(lhs)
        || has_duplicate_record_field_names(rhs)
    {
        return false;
    }

    lhs.iter().all(|(left_name, left_ty)| {
        rhs.iter()
            .find(|(right_name, _)| right_name == left_name)
            .is_some_and(|(_, right_ty)| types_equivalent_unchecked(left_ty, right_ty, env))
    })
}

fn has_duplicate_record_field_names(fields: &[(CoreName, CoreType)]) -> bool {
    duplicate_record_field_name(fields).is_some()
}

fn duplicate_record_field_name(fields: &[(CoreName, CoreType)]) -> Option<CoreName> {
    let mut seen = HashSet::with_capacity(fields.len());
    fields
        .iter()
        .find_map(|(field_name, _)| (!seen.insert(field_name)).then(|| field_name.clone()))
}

fn rows_equivalent(
    lhs: &CoreRow,
    rhs: &CoreRow,
    env: &CoreTypeCheckEnv,
) -> Result<bool, CoreTypeCheckError> {
    let left = normalize_core_row_structural(lhs, env)?;
    let right = normalize_core_row_structural(rhs, env)?;

    if left.tail != right.tail || left.items.len() != right.items.len() {
        return Ok(false);
    };

    let mut used = vec![false; right.items.len()];
    for lhs_item in &left.items {
        let mut found = None;
        for (index, rhs_item) in right.items.iter().enumerate() {
            if used[index] {
                continue;
            }

            if row_items_equivalent(lhs_item, rhs_item, env)? {
                found = Some(index);
                break;
            }
        }

        let Some(index) = found else {
            return Ok(false);
        };

        used[index] = true;
    }

    Ok(used.iter().all(|used| *used))
}

fn rows_equivalent_unchecked(lhs: &CoreRow, rhs: &CoreRow, env: &CoreTypeCheckEnv) -> bool {
    rows_equivalent(lhs, rhs, env).unwrap_or(false)
}

fn normalize_core_row_structural(
    row: &CoreRow,
    env: &CoreTypeCheckEnv,
) -> Result<CoreRow, CoreTypeCheckError> {
    let normalized = normalize_core_row(row)?;
    let items = structural_dedup_row_items(&normalized.items, env)?;
    Ok(CoreRow {
        items,
        tail: normalized.tail,
    })
}

fn structural_dedup_row_items(
    items: &[CoreRowItem],
    env: &CoreTypeCheckEnv,
) -> Result<Vec<CoreRowItem>, CoreTypeCheckError> {
    let mut deduped = Vec::new();
    'outer: for item in items {
        for existing in &deduped {
            if row_items_equivalent(item, existing, env)? {
                continue 'outer;
            }
        }
        deduped.push(item.clone());
    }
    Ok(deduped)
}

fn row_items_equivalent(
    lhs: &CoreRowItem,
    rhs: &CoreRowItem,
    env: &CoreTypeCheckEnv,
) -> Result<bool, CoreTypeCheckError> {
    Ok(match (lhs, rhs) {
        (
            CoreRowItem::Capability {
                path: left_path,
                operation: left_op,
            },
            CoreRowItem::Capability {
                path: right_path,
                operation: right_op,
            },
        ) => left_path == right_path && left_op == right_op,
        (
            CoreRowItem::Resource {
                path: left_path,
                mode: left_mode,
            },
            CoreRowItem::Resource {
                path: right_path,
                mode: right_mode,
            },
        ) => left_path == right_path && left_mode == right_mode,
        (CoreRowItem::Role { path: left_path }, CoreRowItem::Role { path: right_path }) => {
            left_path == right_path
        }
        (CoreRowItem::Policy { path: left_path }, CoreRowItem::Policy { path: right_path }) => {
            left_path == right_path
        }
        (
            CoreRowItem::Contract {
                contract: left_contract,
            },
            CoreRowItem::Contract {
                contract: right_contract,
            },
        ) => left_contract == right_contract,
        (
            CoreRowItem::Channel {
                path: left_path,
                mode: left_mode,
                payload_type: left_payload_type,
            },
            CoreRowItem::Channel {
                path: right_path,
                mode: right_mode,
                payload_type: right_payload_type,
            },
        ) => {
            left_path == right_path
                && left_mode == right_mode
                && core_types_equivalent(left_payload_type, right_payload_type, env)?
        }
        (
            CoreRowItem::Process {
                operation: left_operation,
            },
            CoreRowItem::Process {
                operation: right_operation,
            },
        ) => left_operation == right_operation,
        (
            CoreRowItem::Failure { ty: Some(left_ty) },
            CoreRowItem::Failure { ty: Some(right_ty) },
        ) => core_types_equivalent(left_ty, right_ty, env)?,
        (CoreRowItem::Failure { ty: None }, CoreRowItem::Failure { ty: None }) => true,
        (CoreRowItem::Evidence { path: left_path }, CoreRowItem::Evidence { path: right_path }) => {
            left_path == right_path
        }
        (
            CoreRowItem::EffectGroupRef { path: left_path },
            CoreRowItem::EffectGroupRef { path: right_path },
        ) => left_path == right_path,
        _ => false,
    })
}

fn row_difference(left: &[CoreRowItem], right: &[CoreRowItem]) -> Vec<CoreRowItem> {
    left.iter()
        .filter(|item| !right.contains(item))
        .cloned()
        .collect()
}

fn row_difference_structural(
    left: &[CoreRowItem],
    right: &[CoreRowItem],
    env: &CoreTypeCheckEnv,
) -> Result<Vec<CoreRowItem>, CoreTypeCheckError> {
    let mut used = vec![false; right.len()];
    let mut difference = Vec::new();

    for item in left {
        let mut matched_index = None;
        for (index, expected_item) in right.iter().enumerate() {
            if used[index] {
                continue;
            }

            if row_items_equivalent(item, expected_item, env)? {
                matched_index = Some(index);
                break;
            }
        }

        if let Some(index) = matched_index {
            used[index] = true;
        } else {
            difference.push(item.clone());
        }
    }

    Ok(difference)
}

fn public_row_item_summary(item: &CoreRowItem) -> CorePublicRowItemSummary {
    match item {
        CoreRowItem::Capability { path, operation } => CorePublicRowItemSummary::Capability {
            path: path.clone(),
            operation: operation.clone(),
        },
        CoreRowItem::Resource { path, mode } => CorePublicRowItemSummary::Resource {
            path: path.clone(),
            mode: mode.clone(),
        },
        CoreRowItem::Role { path } => CorePublicRowItemSummary::Role { path: path.clone() },
        CoreRowItem::Policy { path } => CorePublicRowItemSummary::Policy { path: path.clone() },
        CoreRowItem::Contract { contract } => CorePublicRowItemSummary::Contract {
            contract: contract.clone(),
        },
        CoreRowItem::Channel {
            path,
            mode,
            payload_type,
        } => CorePublicRowItemSummary::Channel {
            path: path.clone(),
            mode: mode.clone(),
            payload_type: payload_type.clone(),
        },
        CoreRowItem::Process { operation } => CorePublicRowItemSummary::Process {
            operation: operation.clone(),
        },
        CoreRowItem::Failure { ty } => CorePublicRowItemSummary::Failure { ty: ty.clone() },
        CoreRowItem::Evidence { path } => CorePublicRowItemSummary::Evidence { path: path.clone() },
        CoreRowItem::EffectGroupRef { .. } => {
            unreachable!("effect group refs are rejected before summary mapping")
        }
    }
}

fn collect_public_type_constructors(
    ty: &CoreType,
    constructors: &mut Vec<CorePublicTypeConstructorSummary>,
) -> Result<(), CorePublicSummaryError> {
    match ty {
        CoreType::Base(_) | CoreType::Named(_) | CoreType::Var(_) => {}
        CoreType::Function {
            params,
            result,
            row,
        } => {
            for param in params {
                collect_public_type_constructors(param, constructors)?;
            }
            collect_public_type_constructors(result, constructors)?;
            for item in &row.items {
                collect_public_row_item_type_constructors(item, constructors)?;
            }
        }
        CoreType::Refinement { base, .. } => {
            collect_public_type_constructors(base, constructors)?;
        }
        CoreType::Cont {
            input, answer, row, ..
        } => {
            collect_public_type_constructors(input, constructors)?;
            collect_public_type_constructors(answer, constructors)?;
            for item in &row.items {
                collect_public_row_item_type_constructors(item, constructors)?;
            }
        }
        CoreType::Mode {
            inner, latent_row, ..
        } => {
            collect_public_type_constructors(inner, constructors)?;
            let Some(latent_row) = latent_row else {
                return Ok(());
            };
            for item in &latent_row.items {
                collect_public_row_item_type_constructors_with_privacy(item, constructors)?;
            }
        }
        CoreType::Tuple(elems) => {
            for elem in elems {
                collect_public_type_constructors(elem, constructors)?;
            }
        }
        CoreType::Record(fields) => {
            for (_, field_ty) in fields {
                collect_public_type_constructors(field_ty, constructors)?;
            }
        }
        CoreType::App { name, args } => {
            push_public_type_constructor(constructors, name.clone(), args.len());
            for arg in args {
                collect_public_type_constructors(arg, constructors)?;
            }
        }
    }

    Ok(())
}

fn collect_public_row_item_type_constructors(
    item: &CoreRowItem,
    constructors: &mut Vec<CorePublicTypeConstructorSummary>,
) -> Result<(), CorePublicSummaryError> {
    match item {
        CoreRowItem::Channel { payload_type, .. } => {
            collect_public_type_constructors(payload_type, constructors)
        }
        CoreRowItem::Failure { ty: Some(ty) } => collect_public_type_constructors(ty, constructors),
        CoreRowItem::Capability { .. }
        | CoreRowItem::Resource { .. }
        | CoreRowItem::Role { .. }
        | CoreRowItem::Policy { .. }
        | CoreRowItem::Contract { .. }
        | CoreRowItem::Process { .. }
        | CoreRowItem::Failure { ty: None }
        | CoreRowItem::Evidence { .. }
        | CoreRowItem::EffectGroupRef { .. } => Ok(()),
    }
}

fn push_public_type_constructor(
    constructors: &mut Vec<CorePublicTypeConstructorSummary>,
    name: CoreName,
    arity: usize,
) {
    if constructors
        .iter()
        .any(|constructor| constructor.name == name && constructor.arity == arity)
    {
        return;
    }

    constructors.push(CorePublicTypeConstructorSummary { name, arity });
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
    lhs.mode_binding_latent_rows
        .extend(rhs.mode_binding_latent_rows);
    lhs.refinement_obligations
        .extend(rhs.refinement_obligations);
    lhs.discharges.extend(rhs.discharges);
    lhs
}

fn record_mode_binding_latent_row(ty: &CoreType, name: &str, facts: &mut CoreTypeCheckFacts) {
    if let CoreType::Mode {
        mode: CoreEvalMode::Lazy | CoreEvalMode::Memo,
        latent_row: Some(row),
        ..
    } = ty
    {
        facts
            .mode_binding_latent_rows
            .insert(name.to_string(), row.clone());
    }
}

fn lowering_context_with_checked_facts(
    mut context: CoreLoweringContext,
    env: &CoreTypeCheckEnv,
    facts: &CoreTypeCheckFacts,
) -> CoreLoweringContext {
    for (name, ty) in &env.values.bindings {
        if let CoreType::Function { row, .. } = ty {
            context = context.with_function_row(name.clone(), row.clone());
        }

        if let CoreType::Mode {
            mode: CoreEvalMode::Lazy | CoreEvalMode::Memo,
            inner,
            latent_row: Some(row),
            ..
        } = ty
        {
            context = context.with_mode_binding_latent_row(name.clone(), row.clone());
            if let CoreType::Function { row, .. } = inner.as_ref() {
                context = context.with_mode_binding_function_row(name.clone(), row.clone());
            }
        }
    }

    for (cont, row) in facts.jump_continuation_rows() {
        context = context.with_cont_row(cont_ref_name(cont), row.clone());
    }

    for (name, row) in facts.mode_binding_latent_rows() {
        context = context.with_mode_binding_latent_row(name, row.clone());
    }

    context
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
            let mut value_facts = check_value_against(value, ty, env, Some(name.clone()))?;
            record_mode_binding_latent_row(ty, name, &mut value_facts);
            let mut body_env = env.clone();
            body_env.values_mut().insert(name.clone(), ty.clone());
            let body_checked = type_check_expr(body, &body_env)?;
            Ok(TypedCoreExpr {
                ty: body_checked.ty,
                row: body_checked.row,
                facts: merge_typecheck_facts(value_facts, body_checked.facts),
            })
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
            let mut value_facts =
                check_value_against(value, ty, &recursive_env, Some(name.clone()))?;
            record_mode_binding_latent_row(ty, name, &mut value_facts);
            let body_checked = type_check_expr(body, &recursive_env)?;
            Ok(TypedCoreExpr {
                ty: body_checked.ty,
                row: body_checked.row,
                facts: merge_typecheck_facts(value_facts, body_checked.facts),
            })
        }
        CoreExpr::LetPrim {
            name,
            op,
            args,
            body,
        } => {
            let (result_ty, arg_facts) = check_primitive_application(op, args, env)?;
            let mut body_env = env.clone();
            body_env.values_mut().insert(name.clone(), result_ty);
            let body_checked = type_check_expr(body, &body_env)?;
            Ok(TypedCoreExpr {
                ty: body_checked.ty,
                row: body_checked.row,
                facts: merge_typecheck_facts(arg_facts, body_checked.facts),
            })
        }
        CoreExpr::LetCall {
            name,
            func,
            args,
            body,
        } => {
            let (result_ty, callee_row, arg_facts) = check_function_application(func, args, env)?;
            let mut body_env = env.clone();
            body_env.values_mut().insert(name.clone(), result_ty);
            let body_checked = type_check_expr(body, &body_env)?;
            Ok(TypedCoreExpr {
                ty: body_checked.ty,
                row: union_core_rows_structural(&callee_row, &body_checked.row, env)?,
                facts: merge_typecheck_facts(arg_facts, body_checked.facts),
            })
        }
        CoreExpr::If {
            cond,
            then_branch,
            else_branch,
        } => type_check_if(cond, then_branch, else_branch, env),
        CoreExpr::Call { func, args } => {
            let (result_ty, callee_row, facts) = check_function_application(func, args, env)?;
            Ok(TypedCoreExpr {
                ty: result_ty,
                row: callee_row,
                facts,
            })
        }
        CoreExpr::LetMode {
            name,
            mode,
            ty,
            expr,
            body,
        } => {
            check_core_type_well_formed(ty, env)?;
            let (inner_ty, letmode_row) = check_letmode_type(ty, *mode, name)?;
            let expr_checked = type_check_expr_against(expr, &inner_ty, env)?;

            if matches!(mode, CoreEvalMode::Lazy | CoreEvalMode::Memo)
                && let Some(expected_row) = &letmode_row
                && !rows_equivalent(&expr_checked.row, expected_row, env)?
            {
                return Err(CoreTypeCheckError::ModeLatentRowMismatch {
                    name: name.clone(),
                    expected: expected_row.clone(),
                    actual: expr_checked.row,
                });
            }

            let mut body_env = env.clone();
            body_env.values_mut().insert(name.clone(), ty.clone());
            let body_checked = type_check_expr(body, &body_env)?;

            let mut facts = merge_typecheck_facts(expr_checked.facts, body_checked.facts);
            let row = match mode {
                CoreEvalMode::Strict => {
                    union_core_rows_structural(&expr_checked.row, &body_checked.row, env)?
                }
                CoreEvalMode::Lazy | CoreEvalMode::Memo => {
                    if let Some(latent_row) = letmode_row {
                        facts
                            .mode_binding_latent_rows
                            .insert(name.clone(), latent_row);
                    }

                    body_checked.row
                }
            };

            Ok(TypedCoreExpr {
                ty: body_checked.ty,
                row,
                facts,
            })
        }
        CoreExpr::Force { name, thunk, body } => {
            let CoreAtom::Var(_) = thunk else {
                return Err(unsupported("Force requires variable thunk"));
            };

            let thunk_ty = type_check_atom(thunk, env)?;
            let (result_ty, thunk_row) = match thunk_ty {
                CoreType::Mode {
                    mode: CoreEvalMode::Lazy,
                    inner,
                    latent_row,
                } => (inner.as_ref().clone(), latent_row),
                CoreType::Mode {
                    mode: CoreEvalMode::Memo,
                    inner,
                    latent_row,
                } => (inner.as_ref().clone(), latent_row),
                CoreType::Mode {
                    mode: CoreEvalMode::Strict,
                    ..
                } => {
                    return Err(unsupported("cannot force strict mode"));
                }
                _ => return Err(unsupported("Force requires mode-typed thunk")),
            };

            let thunk_row = thunk_row.ok_or_else(|| CoreTypeCheckError::InvalidModeType {
                detail: "forced thunk mode must carry a latent row".to_owned(),
            })?;
            let mut body_env = env.clone();
            body_env
                .values_mut()
                .insert(name.clone(), result_ty.clone());
            let body_checked = type_check_expr(body, &body_env)?;
            let row = union_core_rows_structural(&body_checked.row, &thunk_row, env)?;

            Ok(TypedCoreExpr {
                ty: body_checked.ty,
                row,
                facts: body_checked.facts,
            })
        }
        CoreExpr::Jump { cont, arg } => type_check_jump(cont, arg, env),
        CoreExpr::LetContCall {
            name,
            cont,
            arg,
            body,
        } => type_check_letcontcall(name, cont, arg, body, env),
        CoreExpr::Raise { op, args } => type_check_raise(op, args, env),
        CoreExpr::Handle { clause, body } => type_check_handle(clause, body, env),
        CoreExpr::RecordDischarge { discharge, body } => {
            validate_contract_discharge(discharge, env)?;
            let body_checked = type_check_expr(body, env)?;
            let residual_row = subtract_core_row(
                &body_checked.row,
                &CoreRow::closed(vec![CoreRowItem::Contract {
                    contract: discharge.contract.clone(),
                }]),
                env,
            )?;
            let mut facts = body_checked.facts;
            facts.discharges.push(discharge.clone());
            Ok(TypedCoreExpr {
                ty: body_checked.ty,
                row: residual_row,
                facts,
            })
        }
        CoreExpr::Trap { .. } => Ok(typed_expr(
            CoreType::Base("Unit".into()),
            CoreRow::default(),
        )),
    }
}

fn type_check_atom(
    atom: &CoreAtom,
    env: &CoreTypeCheckEnv,
) -> Result<CoreType, CoreTypeCheckError> {
    synthesize_core_atom(atom, env)
}

fn check_letmode_type(
    ty: &CoreType,
    mode: CoreEvalMode,
    name: &str,
) -> Result<(CoreType, Option<CoreRow>), CoreTypeCheckError> {
    let CoreType::Mode {
        mode: actual_mode,
        inner,
        latent_row,
    } = ty
    else {
        return Err(unsupported("let-mode requires a mode-typed annotation"));
    };

    if *actual_mode != mode {
        return Err(CoreTypeCheckError::ModeTypeMismatch {
            expected: mode,
            actual: *actual_mode,
        });
    }

    match (mode, latent_row) {
        (CoreEvalMode::Strict, Some(_)) => {
            return Err(CoreTypeCheckError::InvalidModeType {
                detail: format!("strict mode binding `{name}` requires no latent row",),
            });
        }
        (CoreEvalMode::Strict, None) => {}
        (CoreEvalMode::Lazy | CoreEvalMode::Memo, None) => {
            return Err(CoreTypeCheckError::InvalidModeType {
                detail: format!("{:?} mode binding `{name}` requires a latent row", mode),
            });
        }
        (CoreEvalMode::Lazy | CoreEvalMode::Memo, Some(_)) => {}
    }

    Ok((*inner.clone(), latent_row.clone()))
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
    let cond_ty = type_check_atom(cond, env)?;
    let cond_facts = check_type_against_annotation(&CoreType::Base("Bool".into()), &cond_ty, env)?;

    match (then_branch, else_branch) {
        (CoreExpr::Trap { .. }, CoreExpr::Trap { .. }) => {
            Err(unsupported("If with only Trap branches"))
        }
        (CoreExpr::Trap { .. }, _) => {
            let else_checked = type_check_expr(else_branch, env)?;
            let then_checked = type_check_expr_against(then_branch, &else_checked.ty, env)?;
            Ok(TypedCoreExpr {
                ty: else_checked.ty,
                row: union_core_rows_structural(&then_checked.row, &else_checked.row, env)?,
                facts: merge_typecheck_facts(
                    cond_facts,
                    merge_typecheck_facts(then_checked.facts, else_checked.facts),
                ),
            })
        }
        (_, CoreExpr::Trap { .. }) => {
            let then_checked = type_check_expr(then_branch, env)?;
            let else_checked = type_check_expr_against(else_branch, &then_checked.ty, env)?;
            Ok(TypedCoreExpr {
                ty: then_checked.ty,
                row: union_core_rows_structural(&then_checked.row, &else_checked.row, env)?,
                facts: merge_typecheck_facts(
                    cond_facts,
                    merge_typecheck_facts(then_checked.facts, else_checked.facts),
                ),
            })
        }
        _ => {
            let then_checked = type_check_expr(then_branch, env)?;
            let else_checked = type_check_expr(else_branch, env)?;
            ensure_types_equivalent(&then_checked.ty, &else_checked.ty, env)?;
            Ok(TypedCoreExpr {
                ty: then_checked.ty,
                row: union_core_rows_structural(&then_checked.row, &else_checked.row, env)?,
                facts: merge_typecheck_facts(
                    cond_facts,
                    merge_typecheck_facts(then_checked.facts, else_checked.facts),
                ),
            })
        }
    }
}

fn collect_public_row_item_type_constructors_with_privacy(
    item: &CoreRowItem,
    constructors: &mut Vec<CorePublicTypeConstructorSummary>,
) -> Result<(), CorePublicSummaryError> {
    match item {
        CoreRowItem::EffectGroupRef { path } => Err(CorePublicSummaryError::PrivateRowReference {
            path: path.clone(),
            public_item: None,
            detail: format!(
                "private effect group {} must be expanded or exported before summary",
                path.join(".")
            ),
        }),
        CoreRowItem::Channel { payload_type, .. } => {
            collect_public_type_constructors(payload_type, constructors)?;
            Ok(())
        }
        CoreRowItem::Failure { ty: Some(ty) } => {
            collect_public_type_constructors(ty, constructors)?;
            Ok(())
        }
        CoreRowItem::Capability { .. }
        | CoreRowItem::Resource { .. }
        | CoreRowItem::Role { .. }
        | CoreRowItem::Policy { .. }
        | CoreRowItem::Contract { .. }
        | CoreRowItem::Process { .. }
        | CoreRowItem::Failure { ty: None }
        | CoreRowItem::Evidence { .. } => Ok(()),
    }
}

fn check_value_against(
    value: &CoreValue,
    expected: &CoreType,
    env: &CoreTypeCheckEnv,
    value_name: Option<CoreName>,
) -> Result<CoreTypeCheckFacts, CoreTypeCheckError> {
    let typed = synthesize_core_value(value, env)?;
    let mut facts = check_type_against_annotation(expected, typed.ty(), env)?;
    facts = merge_typecheck_facts(typed.facts().clone(), facts);
    if normalize_core_row(typed.row())? != CoreRow::default() {
        return Err(CoreTypeCheckError::RowMismatch {
            expected: CoreRow::default(),
            actual: typed.row().clone(),
        });
    }
    for obligation in &mut facts.refinement_obligations {
        if obligation.value_name.is_none() {
            obligation.value_name.clone_from(&value_name);
        }
    }
    Ok(facts)
}

fn check_type_against_annotation(
    expected: &CoreType,
    actual: &CoreType,
    env: &CoreTypeCheckEnv,
) -> Result<CoreTypeCheckFacts, CoreTypeCheckError> {
    check_core_type_well_formed(expected, env)?;
    check_core_type_well_formed(actual, env)?;

    if let (
        CoreType::Function {
            params: expected_params,
            result: expected_result,
            row: expected_row,
        },
        CoreType::Function {
            params: actual_params,
            result: actual_result,
            row: actual_row,
        },
    ) = (expected, actual)
    {
        let result_facts = check_type_against_annotation(expected_result, actual_result, env)?;

        if !type_slices_equivalent_unchecked(expected_params, actual_params, env) {
            return Err(CoreTypeCheckError::TypeMismatch {
                expected: Box::new(expected.clone()),
                actual: Box::new(actual.clone()),
            });
        }

        if !core_row_included_in_env(actual_row, expected_row, env)?.is_included() {
            return Err(CoreTypeCheckError::RowMismatch {
                expected: expected_row.clone(),
                actual: actual_row.clone(),
            });
        }

        return Ok(result_facts);
    }

    if let (
        CoreType::Mode {
            mode: expected_mode,
            inner: expected_inner,
            latent_row: expected_row,
        },
        CoreType::Mode {
            mode: actual_mode,
            inner: actual_inner,
            latent_row: actual_row,
        },
    ) = (expected, actual)
    {
        if expected_mode != actual_mode {
            return Err(CoreTypeCheckError::ModeTypeMismatch {
                expected: *expected_mode,
                actual: *actual_mode,
            });
        }

        match (expected_row, actual_row) {
            (None, None) => {}
            (Some(expected_row), Some(actual_row)) => {
                if !rows_equivalent(expected_row, actual_row, env)? {
                    return Err(CoreTypeCheckError::RowMismatch {
                        expected: expected_row.clone(),
                        actual: actual_row.clone(),
                    });
                }
            }
            _ => {
                return Err(CoreTypeCheckError::InvalidModeType {
                    detail: "mode latent-row shape differs".to_owned(),
                });
            }
        }

        return check_type_against_annotation(expected_inner, actual_inner, env);
    }

    if types_equivalent_unchecked(expected, actual, env) {
        return Ok(CoreTypeCheckFacts::default());
    }

    if let CoreType::Refinement { base, predicate } = expected
        && actual_refines_base_or_matches_base(actual, base, env)
    {
        let mut facts = CoreTypeCheckFacts::default();
        facts.refinement_obligations.push(CoreRefinementObligation {
            predicate: predicate.clone(),
            value_name: None,
            base_type: (**base).clone(),
            refinement_type: expected.clone(),
        });
        return Ok(facts);
    }

    if let CoreType::Refinement { base, .. } = actual
        && types_equivalent_unchecked(expected, base, env)
    {
        return Ok(CoreTypeCheckFacts::default());
    }

    Err(CoreTypeCheckError::TypeMismatch {
        expected: Box::new(expected.clone()),
        actual: Box::new(actual.clone()),
    })
}

fn actual_refines_base_or_matches_base(
    actual: &CoreType,
    expected_base: &CoreType,
    env: &CoreTypeCheckEnv,
) -> bool {
    if types_equivalent_unchecked(actual, expected_base, env) {
        return true;
    }

    if let CoreType::Refinement { base, .. } = actual {
        return types_equivalent_unchecked(base, expected_base, env);
    }

    false
}

fn validate_contract_discharge(
    discharge: &CoreContractDischarge,
    env: &CoreTypeCheckEnv,
) -> Result<(), CoreTypeCheckError> {
    match discharge.mode {
        CoreDischargeMode::Static | CoreDischargeMode::Evidence => {
            let Some(evidence) = &discharge.evidence else {
                return Err(invalid_discharge(
                    "static and evidence discharge modes require evidence metadata",
                ));
            };

            if !env
                .discharges()
                .contains_refinement_predicate(&evidence.predicate)
            {
                return Err(CoreTypeCheckError::UnknownRefinementPredicate {
                    predicate: evidence.predicate.clone(),
                });
            }

            if evidence.status != CoreEvidenceStatus::Proven {
                return Err(invalid_discharge(
                    "hard refinement discharges require proven evidence",
                ));
            }
        }
        CoreDischargeMode::Dynamic => {
            if discharge.evidence.is_some() {
                return Err(invalid_discharge(
                    "dynamic discharge mode must not carry static evidence",
                ));
            }
        }
    }

    Ok(())
}

fn validate_discharge_marker(
    discharge: &CoreContractDischarge,
    env: &CoreTypeCheckEnv,
) -> Result<(), CoreTypeCheckError> {
    match discharge.mode {
        CoreDischargeMode::Static if discharge.evidence.is_none() => Ok(()),
        CoreDischargeMode::Evidence if discharge.evidence.is_none() => Err(invalid_discharge(
            "evidence discharge mode requires evidence metadata",
        )),
        _ => validate_contract_discharge(discharge, env),
    }
}

fn invalid_discharge(detail: &str) -> CoreTypeCheckError {
    CoreTypeCheckError::InvalidDischarge {
        detail: detail.to_owned(),
    }
}

fn check_function_application(
    func: &CoreAtom,
    args: &[CoreAtom],
    env: &CoreTypeCheckEnv,
) -> Result<(CoreType, CoreRow, CoreTypeCheckFacts), CoreTypeCheckError> {
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

    let facts = check_arguments(&params, args, env)?;
    Ok((*result, row, facts))
}

fn type_check_raise(
    op: &CoreEffectOp,
    args: &[CoreAtom],
    env: &CoreTypeCheckEnv,
) -> Result<TypedCoreExpr, CoreTypeCheckError> {
    let op = lookup_effect_operation_structural(op, env)?.ok_or_else(|| {
        CoreTypeCheckError::UnknownOperation {
            detail: effect_operation_detail(op),
        }
    })?;

    let (arg_types, result_type, row) = effect_operation_signature(op, env)?;
    let facts = check_arguments(&arg_types, args, env)?;
    Ok(TypedCoreExpr {
        ty: result_type,
        row,
        facts,
    })
}

fn type_check_handle(
    clause: &CoreHandlerClause,
    body: &CoreExpr,
    env: &CoreTypeCheckEnv,
) -> Result<TypedCoreExpr, CoreTypeCheckError> {
    let clause_op = lookup_effect_operation_structural(&clause.op, env)?.ok_or_else(|| {
        CoreTypeCheckError::UnknownOperation {
            detail: effect_operation_detail(&clause.op),
        }
    })?;

    let (arg_types, op_result_ty, op_row) = effect_operation_signature(clause_op, env)?;
    if !operation_signature_matches(clause_op, &clause.op, env)? {
        return Err(CoreTypeCheckError::UnknownOperation {
            detail: effect_operation_detail(&clause.op),
        });
    }

    check_handler_params(&clause.params, &arg_types, env)?;
    let (resume_row, resume_answer_ty) =
        check_handler_resume(&clause.resume.ty, &op_result_ty, env)?;

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
    if !rows_equivalent(&clause.row, &clause_checked.row, env)? {
        return Err(CoreTypeCheckError::RowMismatch {
            expected: clause.row.clone(),
            actual: clause_checked.row,
        });
    }

    let body_checked = type_check_expr(body, env)?;
    ensure_types_equivalent(&body_checked.ty, &resume_answer_ty, env)?;
    let expected_clause_ty =
        if clause_may_complete_without_resume(&clause.body, &clause.resume.name) {
            &body_checked.ty
        } else {
            &resume_answer_ty
        };
    let result_facts = check_type_against_annotation(expected_clause_ty, &clause_checked.ty, env)?;
    let residual = handle_residual_row(&body_checked.row, &op_row, &resume_row, &clause.row, env)?;
    Ok(TypedCoreExpr {
        ty: body_checked.ty,
        row: residual,
        facts: merge_typecheck_facts(
            merge_typecheck_facts(body_checked.facts, clause_checked.facts),
            result_facts,
        ),
    })
}

fn operation_signature_matches(
    declared: &CoreEffectOp,
    registered: &CoreEffectOp,
    env: &CoreTypeCheckEnv,
) -> Result<bool, CoreTypeCheckError> {
    match (declared, registered) {
        (
            CoreEffectOp::Channel {
                path: declared_path,
                mode: declared_mode,
                payload_type: declared_payload_type,
                result_type: declared_result_type,
                ..
            },
            CoreEffectOp::Channel {
                path: registered_path,
                mode: registered_mode,
                payload_type: registered_payload_type,
                result_type: registered_result_type,
                ..
            },
        ) => Ok(declared_path == registered_path
            && declared_mode == registered_mode
            && core_types_equivalent(declared_payload_type, registered_payload_type, env)?
            && types_equivalent_unchecked(declared_result_type, registered_result_type, env)),
        (
            CoreEffectOp::Capability {
                path: declared_path,
                operation: declared_operation,
                arg_types: declared_args,
                result_type: declared_result_type,
                ..
            },
            CoreEffectOp::Capability {
                path: declared_path2,
                operation: declared_operation2,
                arg_types: registered_args,
                result_type: registered_result_type,
                ..
            },
        ) => Ok(declared_path == declared_path2
            && declared_operation == declared_operation2
            && type_slices_equivalent_unchecked(declared_args, registered_args, env)
            && types_equivalent_unchecked(declared_result_type, registered_result_type, env)),
        (
            CoreEffectOp::Process {
                operation: declared_operation,
                ..
            },
            CoreEffectOp::Process {
                operation: registered_operation,
                ..
            },
        ) => Ok(declared_operation == registered_operation
            && args_and_results_match(declared, registered, env)?),
        (
            CoreEffectOp::Failure { ty: declared_ty },
            CoreEffectOp::Failure { ty: registered_ty },
        ) => match (declared_ty, registered_ty) {
            (None, None) => Ok(true),
            (Some(declared_ty), Some(registered_ty)) => {
                core_types_equivalent(declared_ty, registered_ty, env)
            }
            _ => Ok(false),
        },
        _ => Ok(false),
    }
}

fn args_and_results_match(
    declared: &CoreEffectOp,
    registered: &CoreEffectOp,
    env: &CoreTypeCheckEnv,
) -> Result<bool, CoreTypeCheckError> {
    let (declared_args, declared_result, _) = effect_operation_signature(declared, env)?;
    let (registered_args, registered_result, _) = effect_operation_signature(registered, env)?;
    Ok(
        type_slices_equivalent_unchecked(&declared_args, &registered_args, env)
            && types_equivalent_unchecked(&declared_result, &registered_result, env),
    )
}

fn lookup_effect_operation_structural<'a>(
    op: &CoreEffectOp,
    env: &'a CoreTypeCheckEnv,
) -> Result<Option<&'a CoreEffectOp>, CoreTypeCheckError> {
    for registered in env.operations().operations.iter() {
        if operation_signature_matches(op, registered, env)? {
            return Ok(Some(registered));
        }
    }
    Ok(None)
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
) -> Result<(CoreRow, CoreType), CoreTypeCheckError> {
    // Well-formedness check enforces multiplicity/row legality:
    // MultiShotPure requires a normalized closed empty row (TASK-1684).
    check_core_type_well_formed(resume_ty, env)?;
    let CoreType::Cont {
        input,
        answer,
        row,
        multiplicity,
    } = resume_ty
    else {
        return Err(unsupported("handler resume without continuation type"));
    };

    // Both Affine and legal MultiShotPure continuations are accepted.
    // The well-formedness check above already rejected MultiShotPure with
    // non-empty or open rows.
    let _ = multiplicity;

    ensure_types_equivalent(op_result_ty, input, env)?;
    Ok((row.clone(), (**answer).clone()))
}

fn clause_may_complete_without_resume(expr: &CoreExpr, resume_name: &str) -> bool {
    match expr {
        CoreExpr::Atom(_) | CoreExpr::Call { .. } | CoreExpr::Raise { .. } => true,
        CoreExpr::Trap { .. } => false,
        CoreExpr::Jump { cont, .. } => cont_ref_name(cont) != resume_name,
        CoreExpr::LetContCall { cont, body, .. } => {
            cont_ref_name(cont) != resume_name
                || clause_may_complete_without_resume(body, resume_name)
        }
        CoreExpr::LetVal { body, .. }
        | CoreExpr::LetRec { body, .. }
        | CoreExpr::LetPrim { body, .. }
        | CoreExpr::LetCall { body, .. }
        | CoreExpr::LetMode { body, .. }
        | CoreExpr::Force { body, .. }
        | CoreExpr::RecordDischarge { body, .. } => {
            clause_may_complete_without_resume(body, resume_name)
        }
        CoreExpr::If {
            then_branch,
            else_branch,
            ..
        } => {
            clause_may_complete_without_resume(then_branch, resume_name)
                || clause_may_complete_without_resume(else_branch, resume_name)
        }
        CoreExpr::Handle { clause, body } => {
            clause_may_complete_without_resume(body, resume_name)
                || (clause.resume.name != resume_name
                    && clause_may_complete_without_resume(&clause.body, resume_name))
        }
    }
}

fn handle_residual_row(
    body_row: &CoreRow,
    op_row: &CoreRow,
    resume_row: &CoreRow,
    clause_row: &CoreRow,
    env: &CoreTypeCheckEnv,
) -> Result<CoreRow, CoreTypeCheckError> {
    let body_without_op = subtract_core_row(body_row, op_row, env)?;
    union_core_rows_structural(
        &union_core_rows_structural(&body_without_op, resume_row, env)?,
        clause_row,
        env,
    )
}

fn union_core_rows_structural(
    lhs: &CoreRow,
    rhs: &CoreRow,
    env: &CoreTypeCheckEnv,
) -> Result<CoreRow, CoreTypeCheckError> {
    let unioned = union_core_rows(lhs, rhs)?;
    normalize_core_row_structural(&unioned, env)
}

fn subtract_core_row(
    lhs: &CoreRow,
    rhs: &CoreRow,
    env: &CoreTypeCheckEnv,
) -> Result<CoreRow, CoreTypeCheckError> {
    let left = normalize_core_row_structural(lhs, env)?;
    let right = normalize_core_row_structural(rhs, env)?;
    Ok(CoreRow {
        items: row_difference_structural(&left.items, &right.items, env)?,
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
        CoreEffectOp::Process { operation, .. } => format!("process {operation}"),
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
        CoreType::Mode { inner, .. } => format!("mode/{}", type_detail(inner)),
    }
}

fn check_primitive_application(
    op: &CorePrimOp,
    args: &[CoreAtom],
    env: &CoreTypeCheckEnv,
) -> Result<(CoreType, CoreTypeCheckFacts), CoreTypeCheckError> {
    let CoreType::Function {
        params,
        result,
        row,
    } = primitive_type(op)?
    else {
        return Err(unsupported("primitive without function type"));
    };

    let facts = check_arguments(&params, args, env)?;

    if normalize_core_row(&row)? != CoreRow::default() {
        return Err(CoreTypeCheckError::RowMismatch {
            expected: CoreRow::default(),
            actual: row,
        });
    }

    Ok((*result, facts))
}

fn check_arguments(
    params: &[CoreType],
    args: &[CoreAtom],
    env: &CoreTypeCheckEnv,
) -> Result<CoreTypeCheckFacts, CoreTypeCheckError> {
    if params.len() != args.len() {
        return Err(CoreTypeCheckError::ArgumentCountMismatch {
            expected: params.len(),
            actual: args.len(),
        });
    }

    let mut facts = CoreTypeCheckFacts::default();
    for (arg, expected) in args.iter().zip(params) {
        let actual = type_check_atom(arg, env)?;
        let mut arg_facts = check_type_against_annotation(expected, &actual, env)?;
        if let CoreAtom::Var(name) = arg {
            fill_missing_obligation_owner(&mut arg_facts, name);
        }
        facts = merge_typecheck_facts(facts, arg_facts);
    }

    Ok(facts)
}

fn fill_missing_obligation_owner(facts: &mut CoreTypeCheckFacts, value_name: &str) {
    for obligation in &mut facts.refinement_obligations {
        if obligation.value_name.is_none() {
            obligation.value_name = Some(value_name.to_owned());
        }
    }
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
    let mut facts = check_type_against_annotation(&input, &actual, env)?;

    facts
        .jump_continuation_rows
        .insert(cont.clone(), row.clone());
    Ok(TypedCoreExpr {
        ty: *answer,
        row: CoreRow::default(),
        facts,
    })
}

fn type_check_letcontcall(
    name: &CoreName,
    cont: &CoreContRef,
    arg: &CoreAtom,
    body: &CoreExpr,
    env: &CoreTypeCheckEnv,
) -> Result<TypedCoreExpr, CoreTypeCheckError> {
    let Some(cont_ty) = env.continuations().lookup(cont_ref_name(cont)).cloned() else {
        return Err(CoreTypeCheckError::UnknownContinuation {
            name: cont_ref_name(cont).to_owned(),
        });
    };
    check_core_type_well_formed(&cont_ty, env)?;
    let CoreType::Cont {
        input,
        answer,
        row,
        multiplicity: _,
    } = cont_ty
    else {
        return Err(unsupported("non-continuation let-cont-call target"));
    };

    let actual = type_check_atom(arg, env)?;
    let mut facts = check_type_against_annotation(&input, &actual, env)?;
    facts
        .jump_continuation_rows
        .insert(cont.clone(), row.clone());

    // Bind the answer name and check the body.
    let mut body_env = env.clone();
    body_env
        .values_mut()
        .insert(name.clone(), (*answer).clone());
    let body_checked = type_check_expr(body, &body_env)?;

    // The overall row is the continuation invocation row plus the body row.
    let combined_row = union_core_rows_structural(&row, &body_checked.row, env)?;

    Ok(TypedCoreExpr {
        ty: body_checked.ty,
        row: combined_row,
        facts: merge_typecheck_facts(facts, body_checked.facts),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn chan(path: &[&str], mode: &str, payload: CoreType) -> CoreRowItem {
        CoreRowItem::Channel {
            path: path.iter().map(|part| (*part).to_owned()).collect(),
            mode: mode.to_owned(),
            payload_type: Box::new(payload),
        }
    }

    fn cap(path: &[&str], operation: &str) -> CoreRowItem {
        CoreRowItem::Capability {
            path: path.iter().map(|part| (*part).to_owned()).collect(),
            operation: operation.to_owned(),
        }
    }

    #[test]
    fn core_row_included_in_env_deduplicates_structural_typed_items_before_solving_open_tails() {
        let payload = CoreType::Record(vec![
            ("a".into(), CoreType::Base("Int".into())),
            ("b".into(), CoreType::Base("String".into())),
        ]);
        let swapped_payload = CoreType::Record(vec![
            ("b".into(), CoreType::Base("String".into())),
            ("a".into(), CoreType::Base("Int".into())),
        ]);

        let actual = CoreRow::closed(vec![
            chan(&["jobs"], "send", payload),
            chan(&["jobs"], "send", swapped_payload),
            cap(&["log"], "write"),
        ]);
        let expected = CoreRow::open(
            vec![chan(
                &["jobs"],
                "send",
                CoreType::Record(vec![
                    ("a".into(), CoreType::Base("Int".into())),
                    ("b".into(), CoreType::Base("String".into())),
                ]),
            )],
            "r",
        );
        let comparison = core_row_included_in_env(&actual, &expected, &CoreTypeCheckEnv::default())
            .expect("typed row inclusion should deduplicate equivalent items");

        assert!(comparison.is_included());
        assert_eq!(comparison.solutions().len(), 1);
        assert_eq!(
            comparison.solutions()[0].row(),
            &CoreRow::closed(vec![cap(&["log"], "write")])
        );
    }

    #[test]
    fn union_core_rows_structural_deduplicates_structural_typed_items() {
        let payload = CoreType::Record(vec![
            ("a".into(), CoreType::Base("Int".into())),
            ("b".into(), CoreType::Base("String".into())),
        ]);
        let reordered_payload = CoreType::Record(vec![
            ("b".into(), CoreType::Base("String".into())),
            ("a".into(), CoreType::Base("Int".into())),
        ]);

        let lhs = CoreRow::closed(vec![
            chan(&["jobs"], "send", payload.clone()),
            cap(&["cache"], "read"),
        ]);
        let rhs = CoreRow::closed(vec![
            chan(&["jobs"], "send", reordered_payload),
            cap(&["audit"], "emit"),
        ]);

        let unioned = union_core_rows_structural(&lhs, &rhs, &CoreTypeCheckEnv::default())
            .expect("typed row unions should collapse semantically equivalent items");

        assert_eq!(
            unioned,
            CoreRow::closed(vec![
                chan(&["jobs"], "send", payload),
                cap(&["cache"], "read"),
                cap(&["audit"], "emit"),
            ])
        );
    }
}
