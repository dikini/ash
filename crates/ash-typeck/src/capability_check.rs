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
    /// Action that was not declared.
    #[error("undeclared action: action '{action}' not declared")]
    ActionNotDeclared {
        /// The action name that was not declared.
        action: String,
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
pub struct CapabilityChecker {
    /// Declared observed capabilities: (capability, channel)
    observes: Vec<(String, String)>,
    /// Declared set capabilities: (capability, channel)
    sets: Vec<(String, String)>,
    /// Declared received streams: (capability, channel)
    receives: Vec<(String, String)>,
    /// Declared sent streams: (capability, channel)
    sends: Vec<(String, String)>,
    /// Declared actions: capability name
    actions: Vec<String>,
}

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
        Self {
            observes: Vec::new(),
            sets: Vec::new(),
            receives: Vec::new(),
            sends: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// Declares an observe capability.
    ///
    /// # Arguments
    ///
    /// * `cap` - The capability name
    /// * `channel` - The channel name
    ///
    /// # Example
    ///
    /// ```
    /// use ash_typeck::capability_check::CapabilityChecker;
    ///
    /// let checker = CapabilityChecker::new()
    ///     .observe("sensor", "temp");
    /// ```
    pub fn observe(mut self, cap: &str, channel: &str) -> Self {
        self.observes.push((cap.to_string(), channel.to_string()));
        self
    }

    /// Declares a set capability.
    ///
    /// # Arguments
    ///
    /// * `cap` - The capability name
    /// * `channel` - The channel name
    ///
    /// # Example
    ///
    /// ```
    /// use ash_typeck::capability_check::CapabilityChecker;
    ///
    /// let checker = CapabilityChecker::new()
    ///     .set("hvac", "target");
    /// ```
    pub fn set(mut self, cap: &str, channel: &str) -> Self {
        self.sets.push((cap.to_string(), channel.to_string()));
        self
    }

    /// Declares a receive capability.
    ///
    /// # Arguments
    ///
    /// * `cap` - The capability name
    /// * `channel` - The channel name
    ///
    /// # Example
    ///
    /// ```
    /// use ash_typeck::capability_check::CapabilityChecker;
    ///
    /// let checker = CapabilityChecker::new()
    ///     .receive("kafka", "orders");
    /// ```
    pub fn receive(mut self, cap: &str, channel: &str) -> Self {
        self.receives.push((cap.to_string(), channel.to_string()));
        self
    }

    /// Declares a send capability.
    ///
    /// # Arguments
    ///
    /// * `cap` - The capability name
    /// * `channel` - The channel name
    ///
    /// # Example
    ///
    /// ```
    /// use ash_typeck::capability_check::CapabilityChecker;
    ///
    /// let checker = CapabilityChecker::new()
    ///     .send("kafka", "events");
    /// ```
    pub fn send(mut self, cap: &str, channel: &str) -> Self {
        self.sends.push((cap.to_string(), channel.to_string()));
        self
    }

    /// Declares an action.
    ///
    /// # Arguments
    ///
    /// * `cap` - The action name
    ///
    /// # Example
    ///
    /// ```
    /// use ash_typeck::capability_check::CapabilityChecker;
    ///
    /// let checker = CapabilityChecker::new()
    ///     .action("notify");
    /// ```
    pub fn action(mut self, cap: &str) -> Self {
        self.actions.push(cap.to_string());
        self
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

    /// Verify a workflow against provided declaration context.
    ///
    /// This method allows verification with declarations provided at check time,
    /// rather than requiring them to be set via the builder methods.
    ///
    /// # Arguments
    ///
    /// * `workflow` - The workflow to verify
    /// * `observes` - Declared observe capabilities as (capability, channel) pairs
    /// * `sets` - Declared set capabilities as (capability, channel) pairs
    /// * `receives` - Declared receive capabilities as (capability, channel) pairs
    /// * `sends` - Declared send capabilities as (capability, channel) pairs
    /// * `actions` - Declared action names
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
    /// let result = checker.verify_with_context(
    ///     &workflow,
    ///     &[("sensor".to_string(), "temp".to_string())],
    ///     &[],
    ///     &[],
    ///     &[],
    ///     &[],
    /// );
    /// assert!(result.is_ok());
    /// ```
    pub fn verify_with_context(
        &self,
        workflow: &Workflow,
        observes: &[(String, String)],
        sets: &[(String, String)],
        receives: &[(String, String)],
        sends: &[(String, String)],
        actions: &[String],
    ) -> CapabilityCheckResult<()> {
        // Create a temporary checker with the provided context
        let temp_checker = Self {
            observes: observes.to_vec(),
            sets: sets.to_vec(),
            receives: receives.to_vec(),
            sends: sends.to_vec(),
            actions: actions.to_vec(),
        };
        temp_checker.verify_workflow(workflow)
    }

    fn verify_workflow(&self, workflow: &Workflow) -> CapabilityCheckResult<()> {
        match workflow {
            // Observation - checks if observe is declared for this capability
            Workflow::Observe {
                capability,
                continuation,
                ..
            } => {
                // Parse capability string to extract capability:channel
                let cap_str = capability.as_ref();
                let (cap_name, channel_name) = self.parse_capability_channel(cap_str);

                // Check if (capability, channel) is in the observes list
                if !self
                    .observes
                    .iter()
                    .any(|(c, ch)| c == cap_name && ch == channel_name)
                {
                    return Err(CapabilityCheckError::NotDeclared {
                        operation: "observe".to_string(),
                        capability: cap_name.to_string(),
                        channel: channel_name.to_string(),
                    });
                }

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
            Workflow::Act { action, .. } => {
                // Check if action name is in the actions list
                let action_name = action.name.as_ref();
                if !self.actions.iter().any(|a| a == action_name) {
                    return Err(CapabilityCheckError::ActionNotDeclared {
                        action: action_name.to_string(),
                    });
                }
                Ok(())
            }

            // Set - sets a value on an output capability
            Workflow::Set {
                capability,
                channel,
                continuation,
                ..
            } => {
                let cap_name = capability.as_ref();
                let channel_name = channel.as_ref();

                // Check if (capability, channel) is in the sets list
                if !self
                    .sets
                    .iter()
                    .any(|(c, ch)| c == cap_name && ch == channel_name)
                {
                    return Err(CapabilityCheckError::NotDeclared {
                        operation: "set".to_string(),
                        capability: cap_name.to_string(),
                        channel: channel_name.to_string(),
                    });
                }

                if let Some(cont) = continuation {
                    self.verify_workflow(cont)?;
                }
                Ok(())
            }

            // Send - sends a value to an output stream
            Workflow::Send {
                capability,
                channel,
                continuation,
                ..
            } => {
                let cap_name = capability.as_ref();
                let channel_name = channel.as_ref();

                // Check if (capability, channel) is in the sends list
                if !self
                    .sends
                    .iter()
                    .any(|(c, ch)| c == cap_name && ch == channel_name)
                {
                    return Err(CapabilityCheckError::NotDeclared {
                        operation: "send".to_string(),
                        capability: cap_name.to_string(),
                        channel: channel_name.to_string(),
                    });
                }

                if let Some(cont) = continuation {
                    self.verify_workflow(cont)?;
                }
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

            Expr::IfLet {
                expr,
                then_branch,
                else_branch,
                ..
            } => {
                self.verify_expr(expr)?;
                self.verify_expr(then_branch)?;
                self.verify_expr(else_branch)
            }

            Expr::Match { scrutinee, arms, .. } => {
                self.verify_expr(scrutinee)?;
                for arm in arms {
                    self.verify_expr(&arm.body)?;
                }
                Ok(())
            }

            Expr::Constructor { .. } => {
                // Constructor expressions don't involve capabilities
                Ok(())
            }
        }
    }

    /// Parse a capability string into (capability, channel) pair.
    ///
    /// Handles formats like:
    /// - "sensor:temp" -> ("sensor", "temp")
    /// - "sensor" -> ("sensor", "")
    fn parse_capability_channel<'a>(&self, cap_str: &'a str) -> (&'a str, &'a str) {
        if let Some(pos) = cap_str.find(':') {
            let (cap, channel) = cap_str.split_at(pos);
            (cap, &channel[1..]) // Skip the ':' character
        } else {
            (cap_str, "")
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
    use ash_parser::surface::{ActionRef, Literal, Pattern, Workflow};
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
    fn test_verify_observe_declared() {
        let checker = CapabilityChecker::new().observe("sensor", "temp");
        let workflow = Workflow::Observe {
            capability: "sensor:temp".into(),
            binding: Some(Pattern::Variable("data".into())),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_observe_undeclared() {
        let checker = CapabilityChecker::new();
        let workflow = Workflow::Observe {
            capability: "sensor:temp".into(),
            binding: Some(Pattern::Variable("data".into())),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_err());
        match result {
            Err(CapabilityCheckError::NotDeclared {
                operation,
                capability,
                channel,
            }) => {
                assert_eq!(operation, "observe");
                assert_eq!(capability, "sensor");
                assert_eq!(channel, "temp");
            }
            _ => panic!("Expected NotDeclared error"),
        }
    }

    #[test]
    fn test_verify_act_declared() {
        let checker = CapabilityChecker::new().action("notify");
        let workflow = Workflow::Act {
            action: ActionRef {
                name: "notify".into(),
                args: vec![],
            },
            guard: None,
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_act_undeclared() {
        let checker = CapabilityChecker::new();
        let workflow = Workflow::Act {
            action: ActionRef {
                name: "notify".into(),
                args: vec![],
            },
            guard: None,
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_err());
        match result {
            Err(CapabilityCheckError::ActionNotDeclared { action }) => {
                assert_eq!(action, "notify");
            }
            _ => panic!("Expected ActionNotDeclared error"),
        }
    }

    #[test]
    fn test_verify_set_declared() {
        let checker = CapabilityChecker::new().set("hvac", "target");
        let workflow = Workflow::Set {
            capability: "hvac".into(),
            channel: "target".into(),
            value: Expr::Literal(Literal::Int(72)),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_set_undeclared() {
        let checker = CapabilityChecker::new();
        let workflow = Workflow::Set {
            capability: "hvac".into(),
            channel: "target".into(),
            value: Expr::Literal(Literal::Int(72)),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_err());
        match result {
            Err(CapabilityCheckError::NotDeclared {
                operation,
                capability,
                channel,
            }) => {
                assert_eq!(operation, "set");
                assert_eq!(capability, "hvac");
                assert_eq!(channel, "target");
            }
            _ => panic!("Expected NotDeclared error"),
        }
    }

    #[test]
    fn test_verify_send_declared() {
        let checker = CapabilityChecker::new().send("kafka", "events");
        let workflow = Workflow::Send {
            capability: "kafka".into(),
            channel: "events".into(),
            value: Expr::Literal(Literal::String("data".into())),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_send_undeclared() {
        let checker = CapabilityChecker::new();
        let workflow = Workflow::Send {
            capability: "kafka".into(),
            channel: "events".into(),
            value: Expr::Literal(Literal::String("data".into())),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_err());
        match result {
            Err(CapabilityCheckError::NotDeclared {
                operation,
                capability,
                channel,
            }) => {
                assert_eq!(operation, "send");
                assert_eq!(capability, "kafka");
                assert_eq!(channel, "events");
            }
            _ => panic!("Expected NotDeclared error"),
        }
    }

    #[test]
    fn test_verify_with_context() {
        let checker = CapabilityChecker::new();
        let workflow = Workflow::Observe {
            capability: "sensor:temp".into(),
            binding: Some(Pattern::Variable("data".into())),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        // Test with declared capability
        let result = checker.verify_with_context(
            &workflow,
            &[("sensor".to_string(), "temp".to_string())],
            &[],
            &[],
            &[],
            &[],
        );
        assert!(result.is_ok());

        // Test without declared capability
        let result = checker.verify_with_context(&workflow, &[], &[], &[], &[], &[]);
        assert!(result.is_err());
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
    fn test_action_error_display() {
        let err = CapabilityCheckError::ActionNotDeclared {
            action: "notify".into(),
        };

        let msg = err.to_string();
        assert!(msg.contains("undeclared action"));
        assert!(msg.contains("notify"));
    }

    #[test]
    fn test_checker_default() {
        let checker: CapabilityChecker = Default::default();
        let workflow = Workflow::Done { span: test_span() };
        let result = checker.verify(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_chaining() {
        let checker = CapabilityChecker::new()
            .observe("sensor", "temp")
            .observe("sensor", "humidity")
            .set("hvac", "target")
            .send("kafka", "events")
            .action("notify")
            .action("log");

        // Verify all declarations were added
        assert_eq!(checker.observes.len(), 2);
        assert_eq!(checker.sets.len(), 1);
        assert_eq!(checker.sends.len(), 1);
        assert_eq!(checker.actions.len(), 2);
    }

    #[test]
    fn test_observe_without_channel() {
        // Test observe with capability that has no channel (no colon)
        let checker = CapabilityChecker::new().observe("sensor", "");
        let workflow = Workflow::Observe {
            capability: "sensor".into(),
            binding: None,
            continuation: None,
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_ok());
    }
}
