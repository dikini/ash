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
//! - **runtime_verification**: Runtime verification checks (TASK-116)

pub mod capability_check;
pub mod capability_typecheck;
pub mod check_expr;
pub mod check_pattern;
pub mod constraint_checking;
pub mod constraints;
pub mod effect;
pub mod effective_caps;
pub mod error;
pub mod exhaustiveness;
pub mod instantiate;
pub mod kind;
pub mod name_binding;
pub mod names;
pub mod obligation_checker;
pub mod obligations;
pub mod policy_check;
pub mod qualified_name;
pub mod requirements;
pub mod role_checking;
pub mod runtime_verification;
pub mod solver;
pub mod type_env;
pub mod types;
pub mod visibility;

// SMT-based policy conflict detection using Z3
// Provides compile-time verification of policy constraints
pub mod smt;

// Re-export smt module under a unified name
pub use smt as policy;

pub use ash_core::ast::{TypeDef, VariantDef};
pub use capability_check::*;
pub use check_pattern::{Bindings, TypeEnv, check_pattern};
pub use constraint_checking::*;
pub use constraints::*;
pub use effect::*;
pub use effective_caps::{
    CapabilitySource, CompositionError, EffectiveCapabilitySet, MergedCapability,
};
pub use instantiate::{InstantiateError, InstantiateSubst, instantiate};
pub use kind::Kind;
pub use name_binding::{NameBinder, NameError};
pub use names::*;
pub use obligation_checker::*;
pub use obligations::*;
pub use policy_check::*;
pub use qualified_name::QualifiedName;
pub use requirements::{
    CheckResult, ContractCheckResult, RequirementContext, RequirementError, check_contract,
    check_requirement,
};
pub use runtime_verification::{
    AggregateVerificationInputs, CapabilityOperation, CapabilitySchema, CapabilitySchemaRegistry,
    CapabilityVerifier, EffectChecker, ObligationRequirements, OperationError, OperationResult,
    OperationVerifier, RateLimiter, RuntimeObligationChecker, RuntimeObligations, StaticPolicy,
    StaticPolicyValidator, VerificationError, VerificationResult, VerificationWarning,
};
pub use solver::{Solver, TypeError};
pub use types::*;
pub use visibility::{ModulePath, VisibilityChecker, VisibilityError, VisibilityExt};

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

    // Step 5: Obligation checking using ObligationCollector (TASK-275)
    let mut obligation_ctx = crate::obligations::LinearObligationContext::new();
    let mut collector = crate::obligations::ObligationCollector::new();

    // Collect and verify obligations from the workflow AST
    let obligation_result = collector
        .collect(workflow, &mut obligation_ctx)
        .and_then(|()| collector.finalize(&obligation_ctx))
        .map(|()| crate::obligations::ObligationCheckResult::Success)
        .unwrap_or_else(|_e| {
            // Convert TypeError to obligation check result
            // For now, we track it as a failed obligation
            crate::obligations::ObligationCheckResult::Failed(vec![])
        });

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
