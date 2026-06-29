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

use ash_parser::capability_export::{CapabilityResolutionContext, ModuleId};
use ash_parser::surface::{Expr, OperationalTarget, Workflow};
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
    /// Workflow decide is missing the required explicit policy binding.
    #[error("invalid decide workflow: missing explicit policy reference")]
    MissingPolicyReference,
    /// Workflow check target is not part of the canonical contract.
    #[error("invalid check workflow: only obligation targets are permitted")]
    InvalidCheckTarget,
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
    /// Declared actions: (provider, action) pairs
    actions: Vec<(String, String)>,
    /// Optional shared capability resolution context from module/import pipeline.
    /// This is the authoritative source for capability name resolution.
    resolution_context: Option<CapabilityResolutionContext>,
    /// Optional current module ID for module-scoped capability resolution.
    /// Used with resolution_context to resolve unqualified names within the correct module scope.
    current_module: Option<ModuleId>,
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
            resolution_context: None,
            current_module: None,
        }
    }

    /// Creates a capability checker with the shared module-owned resolution context.
    ///
    /// This is the preferred constructor for module-owned capability resolution,
    /// as it uses the authoritative context from the module/import pipeline.
    ///
    /// # Arguments
    /// * `context` - The capability resolution context from the parser pipeline
    ///
    /// # Example
    ///
    /// ```
    /// use ash_typeck::capability_check::CapabilityChecker;
    /// use ash_parser::capability_export::CapabilityResolutionContext;
    ///
    /// let context = CapabilityResolutionContext::new();
    /// let checker = CapabilityChecker::with_resolution_context(context);
    /// ```
    pub fn with_resolution_context(context: CapabilityResolutionContext) -> Self {
        Self {
            observes: Vec::new(),
            sets: Vec::new(),
            receives: Vec::new(),
            sends: Vec::new(),
            actions: Vec::new(),
            resolution_context: Some(context),
            current_module: None,
        }
    }

    /// Creates a capability checker with the shared module-owned resolution context and module ID.
    ///
    /// This is the preferred constructor for module-owned capability resolution,
    /// as it uses the authoritative context from the module/import pipeline with
    /// proper module scoping for unqualified name resolution.
    ///
    /// # Arguments
    /// * `context` - The capability resolution context from the parser pipeline
    /// * `module_id` - The ID of the current module for scoping resolution
    ///
    /// # Example
    ///
    /// ```
    /// use ash_typeck::capability_check::CapabilityChecker;
    /// use ash_parser::capability_export::{CapabilityResolutionContext, ModuleId};
    ///
    /// let context = CapabilityResolutionContext::new();
    /// let checker = CapabilityChecker::with_resolution_context_for_module(context, ModuleId(1));
    /// ```
    pub fn with_resolution_context_for_module(
        context: CapabilityResolutionContext,
        module_id: ModuleId,
    ) -> Self {
        Self {
            observes: Vec::new(),
            sets: Vec::new(),
            receives: Vec::new(),
            sends: Vec::new(),
            actions: Vec::new(),
            resolution_context: Some(context),
            current_module: Some(module_id),
        }
    }

    /// Resolve a capability name to (provider, action) using the resolution context.
    ///
    /// Requires both resolution_context and current_module to be set.
    /// Returns None if either is missing or if the name cannot be resolved.
    fn resolve_capability(&self, name: &str) -> Option<(String, String)> {
        // Require both resolution_context and current_module for resolution
        let context = self.resolution_context.as_ref()?;
        let module_id = self.current_module?;
        context.resolve_unqualified(module_id, name)
    }

    /// Resolve a qualified capability name to (provider, action) using the resolution context.
    ///
    /// Requires resolution_context to be set.
    /// Returns None if the context is missing or if the name cannot be resolved.
    fn resolve_qualified(
        &self,
        module_name: &str,
        capability_name: &str,
    ) -> Option<(String, String)> {
        self.resolution_context
            .as_ref()
            .and_then(|context| context.resolve_qualified_to_strings(module_name, capability_name))
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
    /// * `provider` - The provider name (e.g., "io", "http")
    /// * `action` - The action name (e.g., "fs_read", "get")
    ///
    /// # Example
    ///
    /// ```
    /// use ash_typeck::capability_check::CapabilityChecker;
    ///
    /// let checker = CapabilityChecker::new()
    ///     .action("io", "fs_read");
    /// ```
    pub fn action(mut self, provider: &str, action: &str) -> Self {
        self.actions
            .push((provider.to_string(), action.to_string()));
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
        actions: &[(String, String)],
    ) -> CapabilityCheckResult<()> {
        // Create a temporary checker with the provided context
        // Preserves the resolution context and current module if this checker has them
        let temp_checker = Self {
            observes: observes.to_vec(),
            sets: sets.to_vec(),
            receives: receives.to_vec(),
            sends: sends.to_vec(),
            actions: actions.to_vec(),
            resolution_context: self.resolution_context.clone(),
            current_module: self.current_module,
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
                policy,
                then_branch,
                else_branch,
                ..
            } => {
                if policy.is_none() {
                    return Err(CapabilityCheckError::MissingPolicyReference);
                }
                self.verify_workflow(then_branch)?;
                if let Some(else_b) = else_branch {
                    self.verify_workflow(else_b)?;
                }
                Ok(())
            }

            // Check - verify obligation or policy
            Workflow::Check {
                target,
                continuation,
                ..
            } => {
                if matches!(target, ash_parser::surface::CheckTarget::Policy(_)) {
                    return Err(CapabilityCheckError::InvalidCheckTarget);
                }
                if let Some(cont) = continuation {
                    self.verify_workflow(cont)?;
                }
                Ok(())
            }

            // Act - executes an action with potential side effects
            Workflow::Act { action, .. } => {
                // Resolve symbolic/qualified names to (provider, action) pairs
                // Per Phase 71: Uses shared resolution context when available
                let (provider_name, action_name): (String, String) = match &action.target {
                    OperationalTarget::Symbolic { capability_name } => {
                        // Symbolic names MUST resolve through resolver or shared context
                        match self.resolve_capability(capability_name.as_ref()) {
                            Some((provider, action)) => (provider, action),
                            None => {
                                return Err(CapabilityCheckError::ActionNotDeclared {
                                    action: capability_name.to_string(),
                                });
                            }
                        }
                    }
                    OperationalTarget::Qualified {
                        module,
                        capability_name,
                    } => {
                        // Qualified names (io::fs_read) MUST resolve through resolver
                        match self.resolve_qualified(module.as_ref(), capability_name.as_ref()) {
                            Some((provider, action)) => (provider, action),
                            None => {
                                return Err(CapabilityCheckError::ActionNotDeclared {
                                    action: format!("{}::{}", module, capability_name),
                                });
                            }
                        }
                    }
                    OperationalTarget::Explicit { provider, action } => {
                        // Explicit targets use the provided (provider, action)
                        (provider.to_string(), action.to_string())
                    }
                };

                // Check if (provider, action) is in the declared actions list
                if !self
                    .actions
                    .iter()
                    .any(|(p, a)| p == &provider_name && a == &action_name)
                {
                    return Err(CapabilityCheckError::ActionNotDeclared {
                        action: format!("{}:{}", provider_name, action_name),
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

            // Receive - verify all arm bodies
            // Note: receive doesn't need capability declaration check at this level
            // because receive arms use pattern matching on messages, not direct capability access
            Workflow::Receive {
                arms, is_control, ..
            } => {
                for arm in arms {
                    #[allow(clippy::collapsible_if)]
                    if !is_control {
                        if let ash_parser::surface::StreamPattern::Binding {
                            capability,
                            channel,
                            ..
                        } = &arm.pattern
                        {
                            if !self
                                .receives
                                .iter()
                                .any(|(c, ch)| c == capability.as_ref() && ch == channel.as_ref())
                            {
                                return Err(CapabilityCheckError::NotDeclared {
                                    operation: "receive".to_string(),
                                    capability: capability.to_string(),
                                    channel: channel.to_string(),
                                });
                            }
                        }
                    }
                    self.verify_workflow(&arm.body)?;
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

            // Oblige - evaluative effect, no capabilities
            Workflow::Oblige { .. } => Ok(()),

            // Yield - message passing with proxy, verify all arms
            Workflow::Yield { arms, .. } => {
                for arm in arms {
                    self.verify_workflow(&arm.body)?;
                }
                Ok(())
            }

            // Resume - responding to a yield, no continuation to verify
            Workflow::Resume { .. } => Ok(()),
        }
    }

    fn verify_expr(&self, expr: &Expr) -> CapabilityCheckResult<()> {
        // Expressions don't typically involve capabilities directly
        // but may contain observe/act calls in expressions in the future.
        // For now, we traverse expressions without capability checks.
        match expr {
            Expr::OperatorSection { section } => {
                if let Some(left) = &section.left {
                    self.verify_expr(left)?;
                }
                if let Some(right) = &section.right {
                    self.verify_expr(right)?;
                }
                Ok(())
            }
            Expr::Literal(_) => Ok(()),
            Expr::Variable { .. } => Ok(()),
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

            Expr::Match {
                scrutinee, arms, ..
            } => {
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

            Expr::CheckObligation { .. } => {
                // Check obligation expressions don't involve capabilities
                Ok(())
            }

            Expr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.verify_expr(condition)?;
                self.verify_expr(then_branch)?;
                if let Some(e) = else_branch {
                    self.verify_expr(e)?;
                }
                Ok(())
            }

            Expr::Panic { .. } => Ok(()),

            Expr::Fail { payload, .. } => self.verify_expr(payload),

            Expr::WithError { body, arms, .. } => {
                self.verify_expr(body)?;
                for arm in arms {
                    self.verify_expr(&arm.body)?;
                }
                Ok(())
            }

            Expr::Block {
                statements,
                tail_expr,
                ..
            } => {
                for stmt in statements {
                    let ash_parser::surface::BlockStmt::Let { expr, .. } = stmt;
                    self.verify_expr(expr)?;
                }
                if let Some(e) = tail_expr {
                    self.verify_expr(e)?;
                }
                Ok(())
            }

            Expr::FnDef { params, body, .. } => {
                let _ = params; // params don't involve capabilities
                self.verify_expr(body)
            }

            Expr::FnApply { func, args, .. } => {
                self.verify_expr(func)?;
                for arg in args {
                    self.verify_expr(arg)?;
                }
                Ok(())
            }

            // TODO(TASK-674): Act block capability checking
            Expr::ActBlock { stmts, .. } => {
                use ash_parser::surface::ActStmt;
                for stmt in stmts {
                    let value = match stmt {
                        ActStmt::Bind { value, .. } => value,
                        ActStmt::Return { value, .. } => value,
                    };
                    self.verify_expr(value)?;
                }
                Ok(())
            }

            Expr::DoBlock { stmts, .. } => {
                use ash_parser::surface::DoStmt;
                for stmt in stmts {
                    match stmt {
                        DoStmt::Let { value, .. }
                        | DoStmt::Bind { value, .. }
                        | DoStmt::Return { value, .. } => self.verify_expr(value)?,
                        DoStmt::WorkflowRequires { .. } | DoStmt::WorkflowEnsures { .. } => {
                            // Contract statements are classified by workflow elaboration; do not
                            // interpret role/result syntax as ordinary capability-bearing exprs.
                        }
                    }
                }
                Ok(())
            }

            Expr::Comprehension {
                result, qualifiers, ..
            } => {
                use ash_parser::surface::ComprehensionQualifier;
                for qualifier in qualifiers {
                    let value = match qualifier {
                        ComprehensionQualifier::Let { value, .. }
                        | ComprehensionQualifier::Bind { value, .. }
                        | ComprehensionQualifier::DiscardBind { value, .. } => value,
                    };
                    self.verify_expr(value)?;
                }
                self.verify_expr(result)
            }

            Expr::List { items, .. } => {
                for item in items {
                    self.verify_expr(item)?;
                }
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
    fn test_verify_let() {
        let checker = CapabilityChecker::new();
        let workflow = Workflow::Let {
            pattern: Pattern::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            },
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
            binding: Some(Pattern::Variable {
                name: "data".into(),
                span: ash_parser::token::Span::default(),
            }),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_observe_args_capability_index_declared() {
        let checker = CapabilityChecker::new().observe("Args", "0");
        let workflow = Workflow::Observe {
            capability: "Args:0".into(),
            binding: Some(Pattern::Variable {
                name: "arg".into(),
                span: ash_parser::token::Span::default(),
            }),
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
            binding: Some(Pattern::Variable {
                name: "data".into(),
                span: ash_parser::token::Span::default(),
            }),
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
        let checker = CapabilityChecker::new().action("test", "notify");
        let workflow = Workflow::Act {
            action: ActionRef {
                target: OperationalTarget::Explicit {
                    provider: "test".into(),
                    action: "notify".into(),
                },
                args: vec![],
            },
            guard: None,
            result_name: None,
            continuation: None,
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
                target: OperationalTarget::Explicit {
                    provider: "test".into(),
                    action: "notify".into(),
                },
                args: vec![],
            },
            guard: None,
            result_name: None,
            continuation: None,
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_err());
        match result {
            Err(CapabilityCheckError::ActionNotDeclared { action }) => {
                assert_eq!(action, "test:notify");
            }
            _ => panic!("Expected ActionNotDeclared error"),
        }
    }

    #[test]
    fn test_verify_act_symbolic_resolved() {
        // Test that symbolic capability calls resolve through the resolution context
        // fs_read should resolve to (io, fs_read) via the shared context
        use ash_parser::capability_export::{CapabilityEffect, CapabilityExport, ModuleId};
        use ash_parser::surface::{Name, Visibility};

        // Create a resolution context with the capability mapping
        let mut context = CapabilityResolutionContext::new();
        let module_id = ModuleId(1);
        let export = CapabilityExport {
            visible_name: Name::from("fs_read"),
            declaring_module: module_id,
            target_provider: Name::from("io"),
            target_action: Name::from("fs_read"),
            visibility: Visibility::Public,
            effect: CapabilityEffect::Act,
        };
        context.register(&export);

        let checker = CapabilityChecker::with_resolution_context_for_module(context, module_id)
            .action("io", "fs_read");
        let workflow = Workflow::Act {
            action: ActionRef {
                target: OperationalTarget::Symbolic {
                    capability_name: "fs_read".into(),
                },
                args: vec![],
            },
            guard: None,
            result_name: None,
            continuation: None,
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(
            result.is_ok(),
            "Symbolic fs_read should resolve to io:fs_read"
        );
    }

    #[test]
    fn test_verify_act_symbolic_unresolved() {
        // Test that unresolved symbolic capability calls fail
        let checker = CapabilityChecker::new();
        let workflow = Workflow::Act {
            action: ActionRef {
                target: OperationalTarget::Symbolic {
                    capability_name: "unknown_capability".into(),
                },
                args: vec![],
            },
            guard: None,
            result_name: None,
            continuation: None,
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_err(), "Unknown symbolic capability should fail");
        match result {
            Err(CapabilityCheckError::ActionNotDeclared { action }) => {
                assert_eq!(action, "unknown_capability");
            }
            _ => panic!("Expected ActionNotDeclared error for unresolved symbolic"),
        }
    }

    #[test]
    fn test_verify_act_qualified_resolved() {
        // Test that module-qualified capability calls resolve through the resolution context
        // io::fs_read should resolve to (io, fs_read) via the shared context
        use ash_parser::capability_export::{CapabilityEffect, CapabilityExport, ModuleId};
        use ash_parser::surface::{Name, Visibility};

        // Create a resolution context with the capability mapping
        let mut context = CapabilityResolutionContext::new();
        let io_module_id = ModuleId(1);
        let current_module_id = ModuleId(2);

        // Register module name for qualified resolution (Phase 72)
        context.register_module_name("io", io_module_id);

        // Register the capability export in the io module
        let export = CapabilityExport {
            visible_name: Name::from("fs_read"),
            declaring_module: io_module_id,
            target_provider: Name::from("io"),
            target_action: Name::from("fs_read"),
            visibility: Visibility::Public,
            effect: CapabilityEffect::Act,
        };
        context.register(&export);

        // Register an import alias in the current module for io::fs_read
        context.register_import(
            current_module_id,
            "io::fs_read",
            io_module_id,
            "fs_read",
            (Name::from("io"), Name::from("fs_read")),
        );

        let checker =
            CapabilityChecker::with_resolution_context_for_module(context, current_module_id)
                .action("io", "fs_read");
        let workflow = Workflow::Act {
            action: ActionRef {
                target: OperationalTarget::Qualified {
                    module: "io".into(),
                    capability_name: "fs_read".into(),
                },
                args: vec![],
            },
            guard: None,
            result_name: None,
            continuation: None,
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(
            result.is_ok(),
            "Qualified io::fs_read should resolve to io:fs_read"
        );
    }

    #[test]
    fn test_verify_act_qualified_unresolved() {
        // Test that unresolved qualified capability calls fail
        let checker = CapabilityChecker::new();
        let workflow = Workflow::Act {
            action: ActionRef {
                target: OperationalTarget::Qualified {
                    module: "unknown".into(),
                    capability_name: "capability".into(),
                },
                args: vec![],
            },
            guard: None,
            result_name: None,
            continuation: None,
            span: test_span(),
        };

        let result = checker.verify(&workflow);
        assert!(result.is_err(), "Unknown qualified capability should fail");
        match result {
            Err(CapabilityCheckError::ActionNotDeclared { action }) => {
                assert_eq!(action, "unknown::capability");
            }
            _ => panic!("Expected ActionNotDeclared error for unresolved qualified"),
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
            binding: Some(Pattern::Variable {
                name: "data".into(),
                span: ash_parser::token::Span::default(),
            }),
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
            .action("notify", "send")
            .action("logger", "log");

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
