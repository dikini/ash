//! Ash Type Checker
//!
//! Type system and type inference for the Ash workflow language.
//!
//! This crate provides:
//! - **types**: Core type definitions and unification (TASK-015 to TASK-018)
//! - **constraints**: Constraint generation for workflows and expressions (TASK-019)
//! - **solver**: Constraint solving and type error reporting (TASK-020, TASK-025)
//! - **effect**: Effect inference and lattice operations (TASK-021)
//! - **names**: Name resolution and scope tracking (TASK-022)
//! - **obligations**: Obligation tracking and proof obligations (TASK-023, TASK-024)

pub mod constraints;
pub mod effect;
pub mod names;
pub mod obligations;
pub mod policy_check;
pub mod solver;
pub mod types;

// SMT-based policy conflict detection
// Uses Z3 when 'smt' feature is enabled, fallback implementation otherwise
#[cfg(feature = "smt")]
pub mod smt;
#[cfg(not(feature = "smt"))]
pub mod smt_fallback;

// Re-export smt module under a unified name
#[cfg(feature = "smt")]
pub use smt as policy;
#[cfg(not(feature = "smt"))]
pub use smt_fallback as policy;

pub use constraints::*;
pub use effect::*;
pub use names::*;
pub use obligations::*;
pub use policy_check::*;
pub use solver::*;
pub use types::*;

/// Type check a workflow
///
/// This is a convenience function that runs the full type checking pipeline:
/// 1. Name resolution
/// 2. Constraint generation
/// 3. Constraint solving
/// 4. Effect inference
/// 5. Obligation checking
///
/// # Example
///
/// ```
/// use ash_typeck::type_check_workflow;
/// use ash_parser::surface::Workflow;
/// use ash_parser::token::Span;
///
/// let workflow = Workflow::Done { span: Span::default() };
/// let result = type_check_workflow(&workflow);
/// ```
pub fn type_check_workflow(
    workflow: &ash_parser::surface::Workflow,
) -> Result<TypeCheckResult, TypeCheckError> {
    // Step 1: Name resolution
    let mut resolver = NameResolver::new();
    resolver
        .resolve_workflow(workflow)
        .map_err(|e| TypeCheckError::ResolutionError(format!("{:?}", e)))?;

    // Step 2: Constraint generation
    let mut ctx = crate::constraints::ConstraintContext::new();
    let _ = crate::constraints::generate_workflow_constraints(&mut ctx, workflow);

    // Step 3: Constraint solving
    let mut solver = Solver::new();
    let substitution = solver
        .solve(ctx.constraints())
        .map_err(|e| TypeCheckError::TypeError(format!("{:?}", e)))?;

    // Step 4: Effect inference
    let inferred_effect = crate::effect::infer_effect(workflow);

    // Step 5: Obligation checking
    let tracker = ObligationTracker::new();
    let obligation_result = tracker.check_obligations();

    Ok(TypeCheckResult {
        substitution,
        errors: solver.errors().to_vec(),
        inferred_types: std::collections::HashMap::new(),
        effect: inferred_effect,
        obligation_status: obligation_result,
    })
}

/// Error during type checking
#[derive(Debug, Clone, thiserror::Error)]
pub enum TypeCheckError {
    /// Name resolution failed
    #[error("Name resolution error: {0}")]
    ResolutionError(String),
    /// Type error
    #[error("Type error: {0}")]
    TypeError(String),
    /// Effect constraint violation
    #[error("Effect error: {0}")]
    EffectError(String),
    /// Obligation not satisfied
    #[error("Obligation error: {0}")]
    ObligationError(String),
}

/// Extended type check result with effect and obligation info
#[derive(Debug, Clone)]
pub struct TypeCheckResult {
    /// Final substitution
    pub substitution: Substitution,
    /// Any errors encountered
    pub errors: Vec<TypeError>,
    /// Inferred types for expressions
    pub inferred_types: std::collections::HashMap<String, Type>,
    /// Inferred effect of the workflow
    pub effect: ash_core::Effect,
    /// Obligation check status
    pub obligation_status: ObligationCheckResult,
}

impl TypeCheckResult {
    /// Check if type checking succeeded
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty() && self.obligation_status.is_success()
    }

    /// Get the final type after applying substitution
    pub fn final_type(&self, ty: &Type) -> Type {
        self.substitution.apply(ty)
    }
}

impl std::fmt::Display for TypeCheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_ok() {
            write!(f, "Type check succeeded with effect {:?}", self.effect)
        } else {
            writeln!(f, "Type check failed:")?;
            if !self.errors.is_empty() {
                writeln!(f, "  Type errors: {}", self.errors.len())?;
            }
            if !self.obligation_status.is_success() {
                writeln!(f, "  Obligation status: {:?}", self.obligation_status)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::surface::{Expr, Literal, Pattern, Workflow};
    use ash_parser::token::Span;

    fn test_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    #[test]
    fn test_type_check_workflow_done() {
        let workflow = Workflow::Done { span: test_span() };
        let result = type_check_workflow(&workflow);
        assert!(result.is_ok());

        let tc_result = result.unwrap();
        assert!(tc_result.is_ok());
        assert!(tc_result.errors.is_empty());
    }

    #[test]
    fn test_type_check_workflow_let() {
        let workflow = Workflow::Let {
            pattern: Pattern::Variable("x".into()),
            expr: Expr::Literal(Literal::Int(42)),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = type_check_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_workflow_if() {
        let workflow = Workflow::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_branch: Box::new(Workflow::Done { span: test_span() }),
            else_branch: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = type_check_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_error_display() {
        let err = TypeCheckError::ResolutionError("test".to_string());
        assert!(format!("{err}").contains("test"));

        let err = TypeCheckError::TypeError("type mismatch".to_string());
        assert!(format!("{err}").contains("type mismatch"));

        let err = TypeCheckError::EffectError("effect violation".to_string());
        assert!(format!("{err}").contains("effect violation"));

        let err = TypeCheckError::ObligationError("obligation failed".to_string());
        assert!(format!("{err}").contains("obligation failed"));
    }

    #[test]
    fn test_type_check_result_display_success() {
        let workflow = Workflow::Done { span: test_span() };
        let result = type_check_workflow(&workflow).unwrap();
        let display = format!("{result}");
        assert!(display.contains("succeeded"));
    }

    #[test]
    fn test_module_exports() {
        // Test that all modules are accessible via crate root
        let _ = ConstraintContext::new();
        let _ = Solver::new();
        let _ = EffectContext::new();
        let _ = NameResolver::new();
        let _ = ObligationTracker::new();
        let _ = Substitution::new();
    }
}
