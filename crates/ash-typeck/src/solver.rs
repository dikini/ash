//! Constraint solver for Ash type system (TASK-020, TASK-025)
//!
//! Provides constraint solving using unification and error reporting.

use crate::constraints::Constraint;
use crate::types::{Substitution, Type, TypeVar, UnifyError, unify};
use ash_core::Effect;
use ash_parser::token::Span;
use std::fmt;

/// Type error with detailed information (TASK-025)
///
/// Type values are boxed to keep the error type small on the stack.
/// See: https://docs.rs/serde_json/latest/src/serde_json/error.rs.html
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TypeError {
    /// Types cannot be unified
    #[error("Type mismatch: expected {expected:?}, found {found:?}")]
    Mismatch { expected: Box<Type>, found: Box<Type> },
    /// Infinite type detected (occurs check failed)
    #[error("Infinite type: type variable {var:?} occurs in {typ:?}")]
    InfiniteType { var: TypeVar, typ: Box<Type> },
    /// Constructor name mismatch
    #[error("Constructor name mismatch: expected {expected}, found {found}")]
    ConstructorNameMismatch { expected: String, found: String },
    /// Constructor arity mismatch
    #[error(
        "Constructor arity mismatch for {name}: expected {expected_arity}, found {found_arity}"
    )]
    ConstructorArityMismatch {
        name: String,
        expected_arity: usize,
        found_arity: usize,
    },
    /// Unbound variable
    #[error("Unbound variable: {0}")]
    UnboundVariable(String),
    /// Effect constraint violated
    #[error("Effect constraint violated: {required} required but {actual} found")]
    EffectViolation { required: Effect, actual: Effect },
    /// Missing capability
    #[error("Missing capability: {0}")]
    MissingCapability(String),
    /// Unsatisfied obligation
    #[error("Unsatisfied obligation: {0}")]
    UnsatisfiedObligation(String),
    /// Obligation error from workflow contracts
    #[error("Obligation error: {0}")]
    Obligation(#[from] ash_core::workflow_contract::ObligationError),
    /// Undischarged obligations at workflow end
    #[error("Undischarged obligations: {obligations:?}")]
    UndischargedObligations { obligations: Vec<String> },
    /// Unknown obligation checked without being obliged first
    #[error("Unknown obligation: '{name}' was not obliged")]
    UnknownObligation { name: String, span: Span },
    /// Obligation already satisfied (linear obligation tracking)
    #[error("Obligation already satisfied: '{name}'")]
    ObligationAlreadySatisfied { name: String, span: Span },
    /// Unsatisfied obligations at workflow end
    #[error("Unsatisfied obligations: {obligations:?}")]
    UnsatisfiedObligations { obligations: Vec<String> },
    /// Pattern type mismatch
    #[error("Pattern mismatch: expected {expected:?}, got {actual:?}")]
    PatternMismatch { expected: Box<Type>, actual: Box<Type> },
    /// Unknown variant in pattern
    #[error("Unknown variant: {0}")]
    UnknownVariant(String),
    /// Pattern arity mismatch
    #[error("Pattern arity mismatch: expected {expected} elements, got {actual}")]
    PatternArityMismatch { expected: usize, actual: usize },
    /// Invalid pattern
    #[error("Invalid pattern: {message}")]
    InvalidPattern { message: String },
    /// Not a constructor
    #[error("Not a constructor: {0}")]
    NotAConstructor(String),
    /// Unknown capability referenced
    #[error("Unknown capability: '{name}' does not exist")]
    UnknownCapability { name: String, span: Span },
    /// Invalid constraint field for capability
    #[error("Invalid constraint field '{field}' for capability '{capability}'")]
    InvalidConstraintField {
        capability: String,
        field: String,
        span: Span,
    },
    /// Constraint value type mismatch
    #[error("Constraint type mismatch for field '{field}': expected {expected}, found {found}")]
    ConstraintTypeMismatch {
        field: String,
        expected: String,
        found: String,
    },
}

impl From<UnifyError> for TypeError {
    fn from(err: UnifyError) -> Self {
        match err {
            UnifyError::Mismatch(t1, t2) => TypeError::Mismatch {
                expected: Box::new(t1),
                found: Box::new(t2),
            },
            UnifyError::InfiniteType(var, typ) => TypeError::InfiniteType {
                var,
                typ: Box::new(typ),
            },
            UnifyError::ConstructorNameMismatch { expected, found } => {
                TypeError::ConstructorNameMismatch { expected, found }
            }
            UnifyError::ConstructorArityMismatch {
                name,
                expected_arity,
                found_arity,
            } => TypeError::ConstructorArityMismatch {
                name,
                expected_arity,
                found_arity,
            },
        }
    }
}

/// Solver state for constraint solving
#[derive(Debug, Clone)]
pub struct Solver {
    /// Current substitution
    pub substitution: Substitution,
    /// Collected errors
    pub errors: Vec<TypeError>,
    /// Effect constraints
    pub effect_constraints: Vec<(Effect, Effect)>,
}

impl Default for Solver {
    fn default() -> Self {
        Self::new()
    }
}

impl Solver {
    /// Create a new solver
    pub fn new() -> Self {
        Self {
            substitution: Substitution::new(),
            errors: Vec::new(),
            effect_constraints: Vec::new(),
        }
    }

    /// Solve a list of constraints
    pub fn solve(&mut self, constraints: &[Constraint]) -> Result<Substitution, Vec<TypeError>> {
        self.errors.clear();

        // First pass: solve type constraints
        for constraint in constraints {
            if let Err(err) = self.solve_constraint(constraint) {
                self.errors.push(err);
            }
        }

        // Second pass: verify effect constraints
        for (e1, e2) in &self.effect_constraints {
            if !e1.at_least(*e2) && e1 != e2 {
                // e1 should be <= e2 means e2 >= e1 (at_least)
                // If e1 is not at least e2 and they're not equal, check reverse
                if !e2.at_least(*e1) {
                    self.errors.push(TypeError::EffectViolation {
                        required: *e2,
                        actual: *e1,
                    });
                }
            }
        }

        if self.errors.is_empty() {
            Ok(self.substitution.clone())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Solve a single constraint
    pub fn solve_constraint(&mut self, constraint: &Constraint) -> Result<(), TypeError> {
        match constraint {
            Constraint::TypeEqual(t1, t2) => {
                let t1 = self.substitution.apply(t1);
                let t2 = self.substitution.apply(t2);
                let sub = unify(&t1, &t2)?;
                self.substitution = self.substitution.compose(&sub);
                Ok(())
            }

            Constraint::TypeSubtype(t1, t2) => {
                // For now, subtyping is just equality
                // In a full implementation, this would handle width subtyping for records
                let t1 = self.substitution.apply(t1);
                let t2 = self.substitution.apply(t2);
                let sub = unify(&t1, &t2)?;
                self.substitution = self.substitution.compose(&sub);
                Ok(())
            }

            Constraint::EffectLeq(e1, e2) => {
                // Store effect constraint for later verification
                // e1 <= e2 means e2 must be at least as powerful as e1
                self.effect_constraints.push((*e1, *e2));
                Ok(())
            }

            Constraint::HasType(_, ty) => {
                // Expression type constraint - already generated during constraint collection
                // In a full implementation, this would unify with actual expression type
                let _ = ty;
                Ok(())
            }

            Constraint::VarBinding(_, ty) => {
                // Variable binding - type is already applied during constraint generation
                let _ = ty;
                Ok(())
            }

            Constraint::PatternMatch(_, _) => {
                // Pattern match constraint - handled during constraint generation
                Ok(())
            }

            Constraint::HasCapability(name, required) => {
                // In a full implementation, check against capability registry
                // For now, just verify the effect level is valid
                let _ = name;
                if required.at_least(Effect::Epistemic) {
                    Ok(())
                } else {
                    Err(TypeError::MissingCapability(name.to_string()))
                }
            }

            Constraint::SatisfiesObligation(name, _) => {
                // Obligation satisfaction - would check against obligation tracker
                let _ = name;
                Ok(())
            }
        }
    }

    /// Get the current substitution
    pub fn substitution(&self) -> &Substitution {
        &self.substitution
    }

    /// Get collected errors
    pub fn errors(&self) -> &[TypeError] {
        &self.errors
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Clear the solver state
    pub fn clear(&mut self) {
        self.substitution = Substitution::new();
        self.errors.clear();
        self.effect_constraints.clear();
    }
}

/// Result of type checking
#[derive(Debug, Clone)]
pub struct TypeCheckResult {
    /// Final substitution
    pub substitution: Substitution,
    /// Any errors encountered
    pub errors: Vec<TypeError>,
    /// Inferred types for expressions
    pub inferred_types: std::collections::HashMap<String, Type>,
}

impl TypeCheckResult {
    /// Check if type checking succeeded
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the final type after applying substitution
    pub fn final_type(&self, ty: &Type) -> Type {
        self.substitution.apply(ty)
    }
}

impl fmt::Display for TypeCheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_ok() {
            write!(f, "Type check succeeded")
        } else {
            writeln!(f, "Type check failed with {} error(s):", self.errors.len())?;
            for (i, err) in self.errors.iter().enumerate() {
                writeln!(f, "  {}: {err}", i + 1)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::surface::Pattern;

    #[test]
    fn test_solver_creation() {
        let solver = Solver::new();
        assert!(solver.errors().is_empty());
    }

    #[test]
    fn test_solve_type_equal_success() {
        let mut solver = Solver::new();
        let constraints = vec![Constraint::TypeEqual(Type::Int, Type::Int)];

        let result = solver.solve(&constraints);
        assert!(result.is_ok());
    }

    #[test]
    fn test_solve_type_equal_failure() {
        let mut solver = Solver::new();
        let constraints = vec![Constraint::TypeEqual(Type::Int, Type::Bool)];

        let result = solver.solve(&constraints);
        assert!(result.is_err());
    }

    #[test]
    fn test_solve_var_unification() {
        let mut solver = Solver::new();
        let v1 = TypeVar(1);
        let constraints = vec![Constraint::TypeEqual(Type::Var(v1), Type::Int)];

        let result = solver.solve(&constraints);
        assert!(result.is_ok());

        let sub = result.unwrap();
        assert_eq!(sub.apply(&Type::Var(v1)), Type::Int);
    }

    #[test]
    fn test_solve_multiple_constraints() {
        let mut solver = Solver::new();
        let v1 = TypeVar(1);
        let v2 = TypeVar(2);

        let constraints = vec![
            Constraint::TypeEqual(Type::Var(v1), Type::Int),
            Constraint::TypeEqual(Type::Var(v2), Type::Var(v1)),
        ];

        let result = solver.solve(&constraints);
        assert!(result.is_ok());

        let sub = result.unwrap();
        assert_eq!(sub.apply(&Type::Var(v1)), Type::Int);
        assert_eq!(sub.apply(&Type::Var(v2)), Type::Int);
    }

    #[test]
    fn test_solve_effect_constraint_valid() {
        let mut solver = Solver::new();
        let constraints = vec![Constraint::EffectLeq(
            Effect::Epistemic,
            Effect::Operational,
        )];

        let result = solver.solve(&constraints);
        assert!(result.is_ok());
    }

    #[test]
    fn test_solve_effect_constraint_same() {
        let mut solver = Solver::new();
        let constraints = vec![Constraint::EffectLeq(
            Effect::Operational,
            Effect::Operational,
        )];

        let result = solver.solve(&constraints);
        assert!(result.is_ok());
    }

    #[test]
    fn test_solve_subtype_constraint() {
        let mut solver = Solver::new();
        let constraints = vec![Constraint::TypeSubtype(Type::Int, Type::Int)];

        let result = solver.solve(&constraints);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_error_display() {
        let err = TypeError::Mismatch {
            expected: Box::new(Type::Int),
            found: Box::new(Type::Bool),
        };
        let display = format!("{err}");
        assert!(display.contains("Type mismatch"));
        assert!(display.contains("Int"));
        assert!(display.contains("Bool"));
    }

    #[test]
    fn test_type_error_unbound_variable() {
        let err = TypeError::UnboundVariable("x".to_string());
        assert!(format!("{err}").contains("x"));
    }

    #[test]
    fn test_type_error_effect_violation() {
        let err = TypeError::EffectViolation {
            required: Effect::Epistemic,
            actual: Effect::Operational,
        };
        let display = format!("{err}");
        assert!(display.contains("epistemic"));
        assert!(display.contains("operational"));
    }

    #[test]
    fn test_type_error_missing_capability() {
        let err = TypeError::MissingCapability("FileIO".to_string());
        assert!(format!("{err}").contains("FileIO"));
    }

    #[test]
    fn test_type_error_unsatisfied_obligation() {
        let err = TypeError::UnsatisfiedObligation("Audit".to_string());
        assert!(format!("{err}").contains("Audit"));
    }

    #[test]
    fn test_type_check_result_ok() {
        let result = TypeCheckResult {
            substitution: Substitution::new(),
            errors: vec![],
            inferred_types: std::collections::HashMap::new(),
        };
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_result_not_ok() {
        let result = TypeCheckResult {
            substitution: Substitution::new(),
            errors: vec![TypeError::UnboundVariable("x".to_string())],
            inferred_types: std::collections::HashMap::new(),
        };
        assert!(!result.is_ok());
    }

    #[test]
    fn test_type_check_result_display_success() {
        let result = TypeCheckResult {
            substitution: Substitution::new(),
            errors: vec![],
            inferred_types: std::collections::HashMap::new(),
        };
        assert!(format!("{result}").contains("succeeded"));
    }

    #[test]
    fn test_type_check_result_display_failure() {
        let result = TypeCheckResult {
            substitution: Substitution::new(),
            errors: vec![TypeError::UnboundVariable("x".to_string())],
            inferred_types: std::collections::HashMap::new(),
        };
        assert!(format!("{result}").contains("failed"));
    }

    #[test]
    fn test_has_capability_constraint() {
        let mut solver = Solver::new();
        let constraints = vec![Constraint::HasCapability(
            "FileIO".into(),
            Effect::Operational,
        )];

        let result = solver.solve(&constraints);
        assert!(result.is_ok());
    }

    #[test]
    fn test_var_binding_constraint() {
        let mut solver = Solver::new();
        let constraints = vec![Constraint::VarBinding(
            Pattern::Variable("x".into()),
            Type::Int,
        )];

        let result = solver.solve(&constraints);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_match_constraint() {
        let mut solver = Solver::new();
        let constraints = vec![Constraint::PatternMatch(
            Pattern::Variable("x".into()),
            Type::Int,
        )];

        let result = solver.solve(&constraints);
        assert!(result.is_ok());
    }

    #[test]
    fn test_satisfies_obligation_constraint() {
        let mut solver = Solver::new();
        let constraints = vec![Constraint::SatisfiesObligation("Audit".into(), Type::Bool)];

        let result = solver.solve(&constraints);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unify_error_conversion() {
        let unify_err = UnifyError::Mismatch(Type::Int, Type::Bool);
        let type_err: TypeError = unify_err.into();

        assert!(matches!(type_err, TypeError::Mismatch { .. }));
    }

    #[test]
    fn test_infinite_type_error_conversion() {
        let v1 = TypeVar(1);
        let unify_err = UnifyError::InfiniteType(v1, Type::List(Box::new(Type::Var(v1))));
        let type_err: TypeError = unify_err.into();

        assert!(matches!(type_err, TypeError::InfiniteType { .. }));
    }

    #[test]
    fn test_solver_clear() {
        let mut solver = Solver::new();
        let constraints = vec![Constraint::TypeEqual(Type::Int, Type::Bool)];
        let _ = solver.solve(&constraints);

        assert!(!solver.errors().is_empty());

        solver.clear();
        assert!(solver.errors().is_empty());
        assert!(solver.effect_constraints.is_empty());
    }

    #[test]
    fn test_solver_has_errors() {
        let mut solver = Solver::new();
        assert!(!solver.has_errors());

        let constraints = vec![Constraint::TypeEqual(Type::Int, Type::Bool)];
        let _ = solver.solve(&constraints);

        assert!(solver.has_errors());
    }
}
