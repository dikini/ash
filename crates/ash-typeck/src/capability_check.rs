//! Capability declaration verification for Ash workflows.
//!
//! This module provides compile-time verification that workflows only use
//! declared capabilities. The checker ensures that any capability operation
//! (observe, act, etc.) is properly declared in the workflow context.
//!
//! # Example
//!
//! ```
//! use ash_typeck::capability_check::CapabilityChecker;
//! use ash_parser::surface::Workflow;
//! use ash_parser::token::Span;
//!
//! let checker = CapabilityChecker::new();
//! let workflow = Workflow::Done { span: Span::default() };
//! let result = checker.verify(&workflow);
//! assert!(result.is_ok());
//! ```

use ash_parser::surface::{Expr, Workflow};
use thiserror::Error;

/// Capability verification error.
///
/// Represents errors that can occur during capability checking,
/// such as using an undeclared capability or operation.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum CapabilityCheckError {
    /// Operation on a capability:channel that was not declared.
    #[error(
        "undeclared capability: operation '{operation}' on '{capability}:{channel}' not declared"
    )]
    NotDeclared {
        /// The operation being performed (e.g., "observe", "act").
        operation: String,
        /// The capability name (e.g., "sensor", "hvac").
        capability: String,
        /// The channel name (e.g., "temp", "target").
        channel: String,
    },
}

/// Result type for capability checking.
pub type CapabilityCheckResult<T> = Result<T, CapabilityCheckError>;

/// Capability checker for workflows.
///
/// Verifies that a workflow only uses capabilities that have been
/// properly declared. This is part of Ash's compile-time safety guarantees.
///
/// # Example
///
/// ```
/// use ash_typeck::capability_check::CapabilityChecker;
///
/// let checker = CapabilityChecker::new();
/// // Use checker to verify workflows...
/// ```
#[derive(Debug, Clone)]
pub struct CapabilityChecker;

impl CapabilityChecker {
    /// Creates a new capability checker.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_typeck::capability_check::CapabilityChecker;
    ///
    /// let checker = CapabilityChecker::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Verify that a workflow only uses declared capabilities.
    ///
    /// This method recursively checks all workflow constructs to ensure
    /// any capability operations are properly declared.
    ///
    /// # Arguments
    ///
    /// * `workflow` - The workflow to verify
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all capabilities are properly declared,
    /// or a `CapabilityCheckError` if an undeclared capability is used.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_typeck::capability_check::CapabilityChecker;
    /// use ash_parser::surface::Workflow;
    /// use ash_parser::token::Span;
    ///
    /// let checker = CapabilityChecker::new();
    /// let workflow = Workflow::Done { span: Span::default() };
    /// let result = checker.verify(&workflow);
    /// assert!(result.is_ok());
    /// ```
    pub fn verify(&self, workflow: &Workflow) -> CapabilityCheckResult<()> {
        self.verify_workflow(workflow)
    }

    fn verify_workflow(&self, workflow: &Workflow) -> CapabilityCheckResult<()> {
        match workflow {
            // Observation - checks if observe is declared for this capability
            Workflow::Observe { continuation, .. } => {
                // TODO: Check if observe is declared for this capability:channel
                // For now, we just verify the syntax - actual declaration checking
                // would require the workflow definition context
                if let Some(cont) = continuation {
                    self.verify_workflow(cont)?;
                }
                Ok(())
            }

            // Orientation - pure evaluation, no capabilities
            Workflow::Orient { continuation, .. } => {
                if let Some(cont) = continuation {
                    self.verify_workflow(cont)?;
                }
                Ok(())
            }

            // Proposal - deliberative phase
            Workflow::Propose { continuation, .. } => {
                if let Some(cont) = continuation {
                    self.verify_workflow(cont)?;
                }
                Ok(())
            }

            // Decision - check both branches
            Workflow::Decide {
                then_branch,
                else_branch,
                ..
            } => {
                self.verify_workflow(then_branch)?;
                if let Some(else_b) = else_branch {
                    self.verify_workflow(else_b)?;
                }
                Ok(())
            }

            // Check - verify obligation or policy
            Workflow::Check { continuation, .. } => {
                if let Some(cont) = continuation {
                    self.verify_workflow(cont)?;
                }
                Ok(())
            }

            // Act - executes an action with potential side effects
            Workflow::Act { .. } => {
                // TODO: Check if act is declared for this capability:channel
                Ok(())
            }

            // Let binding - verify expression and continuation
            Workflow::Let {
                expr, continuation, ..
            } => {
                self.verify_expr(expr)?;
                if let Some(cont) = continuation {
                    self.verify_workflow(cont)?;
                }
                Ok(())
            }

            // Conditional - verify condition and branches
            Workflow::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.verify_expr(condition)?;
                self.verify_workflow(then_branch)?;
                if let Some(else_b) = else_branch {
                    self.verify_workflow(else_b)?;
                }
                Ok(())
            }

            // For loop - verify collection and body
            Workflow::For {
                collection, body, ..
            } => {
                self.verify_expr(collection)?;
                self.verify_workflow(body)
            }

            // Parallel composition - verify all branches
            Workflow::Par { branches, .. } => {
                for branch in branches {
                    self.verify_workflow(branch)?;
                }
                Ok(())
            }

            // With clause - verify capability usage and body
            Workflow::With { body, .. } => self.verify_workflow(body),

            // Maybe - verify primary and fallback
            Workflow::Maybe {
                primary, fallback, ..
            } => {
                self.verify_workflow(primary)?;
                self.verify_workflow(fallback)
            }

            // Must - verify body
            Workflow::Must { body, .. } => self.verify_workflow(body),

            // Sequential composition - verify both parts
            Workflow::Seq { first, second, .. } => {
                self.verify_workflow(first)?;
                self.verify_workflow(second)
            }

            // Done - pure workflow, no capabilities
            Workflow::Done { .. } => Ok(()),

            // Return - pure workflow, no capabilities
            Workflow::Ret { .. } => Ok(()),
        }
    }

    fn verify_expr(&self, expr: &Expr) -> CapabilityCheckResult<()> {
        // Expressions don't typically involve capabilities directly
        // but may contain observe/act calls in expressions in the future.
        // For now, we traverse expressions without capability checks.
        match expr {
            Expr::Literal(_) => Ok(()),
            Expr::Variable(_) => Ok(()),
            Expr::FieldAccess { base, .. } => self.verify_expr(base),
            Expr::IndexAccess { base, index, .. } => {
                self.verify_expr(base)?;
                self.verify_expr(index)
            }
            Expr::Unary { operand, .. } => self.verify_expr(operand),
            Expr::Binary { left, right, .. } => {
                self.verify_expr(left)?;
                self.verify_expr(right)
            }
            Expr::Call { args, .. } => {
                for arg in args {
                    self.verify_expr(arg)?;
                }
                Ok(())
            }
            Expr::Policy(_) => {
                // Policy expressions don't involve capability operations
                Ok(())
            }
        }
    }
}

impl Default for CapabilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::surface::{Literal, Pattern, Workflow};
    use ash_parser::token::Span;

    fn test_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    #[test]
    fn test_checker_creation() {
        let _checker = CapabilityChecker::new();
        // Just verify it can be created
    }

    #[test]
    fn test_verify_pure_workflow() {
        let checker = CapabilityChecker::new();
        let workflow = Workflow::Done { span: test_span() };

        let result = checker.verify(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_seq() {
        let checker = CapabilityChecker::new();
        let workflow = Workflow::Seq {
            first: Box::new(Workflow::Done { span: test_span() }),
            second: Box::new(Workflow::Done { span: test_span() }),
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_par() {
        let checker = CapabilityChecker::new();
        let workflow = Workflow::Par {
            branches: vec![
                Workflow::Done { span: test_span() },
                Workflow::Done { span: test_span() },
            ],
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_let() {
        let checker = CapabilityChecker::new();
        let workflow = Workflow::Let {
            pattern: Pattern::Variable("x".into()),
            expr: Expr::Literal(Literal::Int(42)),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_if() {
        let checker = CapabilityChecker::new();
        let workflow = Workflow::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_branch: Box::new(Workflow::Done { span: test_span() }),
            else_branch: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_observe() {
        let checker = CapabilityChecker::new();
        let workflow = Workflow::Observe {
            capability: "sensor".into(),
            binding: Some(Pattern::Variable("data".into())),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        // Currently passes since we don't have declaration context
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_display() {
        let err = CapabilityCheckError::NotDeclared {
            operation: "observe".into(),
            capability: "sensor".into(),
            channel: "temp".into(),
        };

        let msg = err.to_string();
        assert!(msg.contains("undeclared capability"));
        assert!(msg.contains("observe"));
        assert!(msg.contains("sensor:temp"));
    }

    #[test]
    fn test_checker_default() {
        let checker: CapabilityChecker = Default::default();
        let workflow = Workflow::Done { span: test_span() };
        let result = checker.verify(&workflow);
        assert!(result.is_ok());
    }
}
