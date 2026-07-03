//! Name resolution for Ash type system (TASK-022)
//!
//! Provides scope tracking and name resolution for variables, capabilities,
//! and other named entities in workflows and expressions.

use ash_parser::surface::{Expr, OperationalTarget, Pattern, Workflow};
use ash_parser::token::Span;
use std::collections::HashMap;

/// Resolved capability target - canonical (provider, action) pair.
///
/// This represents a fully resolved operational target where symbolic capability
/// names have been mapped to their concrete provider:action implementations.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityTarget {
    /// Provider name (e.g., "io", "http", "db")
    pub provider: Box<str>,
    /// Action name (e.g., "fs_read", "get", "query")
    pub action: Box<str>,
}

impl CapabilityTarget {
    /// Create a new capability target from provider and action names.
    pub fn new(provider: impl Into<Box<str>>, action: impl Into<Box<str>>) -> Self {
        Self {
            provider: provider.into(),
            action: action.into(),
        }
    }

    /// Format as "provider:action" string.
    pub fn format(&self) -> String {
        format!("{}:{}", self.provider, self.action)
    }
}

impl std::fmt::Display for CapabilityTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.provider, self.action)
    }
}

/// Capability resolver that maps symbolic capability names to (provider, action) pairs.
///
/// This is used during type checking to resolve capability names like `fs_read`
/// to their concrete implementations like `io:fs_read`.
#[derive(Debug, Clone, Default)]
pub struct CapabilityResolver {
    /// Maps symbolic capability names to (provider, action) pairs.
    /// For example: "fs_read" -> ("io", "fs_read")
    mappings: HashMap<Box<str>, (Box<str>, Box<str>)>,
}

impl CapabilityResolver {
    /// Create a new empty capability resolver.
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Register a capability mapping.
    ///
    /// # Arguments
    /// * `capability_name` - The symbolic name (e.g., "fs_read")
    /// * `provider` - The provider name (e.g., "io")
    /// * `action` - The action name (e.g., "fs_read")
    pub fn register(
        &mut self,
        capability_name: impl Into<Box<str>>,
        provider: impl Into<Box<str>>,
        action: impl Into<Box<str>>,
    ) {
        let name = capability_name.into();
        self.mappings.insert(name, (provider.into(), action.into()));
    }

    /// Resolve a symbolic capability name to a (provider, action) pair.
    ///
    /// Returns `None` if the capability name is not registered.
    pub fn resolve(&self, capability_name: &str) -> Option<(Box<str>, Box<str>)> {
        self.mappings.get(capability_name).cloned()
    }

    /// Resolve an operational target to a canonical capability target.
    ///
    /// For symbolic targets, looks up the capability name in the mapping.
    /// For explicit targets, uses the provider and action directly.
    ///
    /// # Arguments
    /// * `target` - The operational target to resolve
    ///
    /// # Returns
    /// * `Some(CapabilityTarget)` - The resolved target
    /// * `None` - If the symbolic capability name is not found
    pub fn resolve_target(&self, target: &OperationalTarget) -> Option<CapabilityTarget> {
        match target {
            OperationalTarget::Symbolic { capability_name } => {
                let name = capability_name.as_ref();
                // Try to resolve via mapping
                self.resolve(name)
                    .map(|(provider, action)| CapabilityTarget { provider, action })
            }
            OperationalTarget::Qualified {
                module,
                capability_name,
            } => {
                // Module-qualified names like io::fs_read are resolved as
                // "module::capability_name" in the mapping
                let qualified_name = format!("{}::{}", module, capability_name);
                self.resolve(&qualified_name)
                    .map(|(provider, action)| CapabilityTarget { provider, action })
            }
            OperationalTarget::Explicit { provider, action } => {
                Some(CapabilityTarget::new(provider.as_ref(), action.as_ref()))
            }
        }
    }

    /// Check if a capability name is registered.
    pub fn contains(&self, capability_name: &str) -> bool {
        self.mappings.contains_key(capability_name)
    }

    /// Get all registered capability names.
    pub fn capability_names(&self) -> Vec<&str> {
        self.mappings.keys().map(|k| k.as_ref()).collect()
    }
}

/// A scope containing variable bindings
#[derive(Debug, Clone, Default)]
pub struct Scope {
    /// Variable name to type/definition mapping
    bindings: HashMap<Box<str>, BindingInfo>,
    /// Parent scope depth (0 for root)
    depth: usize,
}

/// Information about a binding
#[derive(Debug, Clone, PartialEq)]
pub struct BindingInfo {
    /// The name of the binding
    pub name: Box<str>,
    /// Whether the binding is mutable (for future use)
    pub mutable: bool,
    /// The scope depth where this binding was created
    pub depth: usize,
}

impl Scope {
    /// Create a new empty scope
    pub fn new(depth: usize) -> Self {
        Self {
            bindings: HashMap::new(),
            depth,
        }
    }

    /// Insert a binding into this scope
    pub fn insert(&mut self, name: Box<str>) {
        self.bindings.insert(
            name.clone(),
            BindingInfo {
                name,
                mutable: false,
                depth: self.depth,
            },
        );
    }

    /// Lookup a binding in this scope only
    pub fn lookup_local(&self, name: &str) -> Option<&BindingInfo> {
        self.bindings.get(name)
    }

    /// Check if this scope contains a binding
    pub fn contains(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    /// Get all bindings in this scope
    pub fn bindings(&self) -> &HashMap<Box<str>, BindingInfo> {
        &self.bindings
    }

    /// Get the depth of this scope
    pub fn depth(&self) -> usize {
        self.depth
    }
}

/// Name resolver with scope stack
#[derive(Debug, Clone, Default)]
pub struct NameResolver {
    /// Stack of scopes (innermost last)
    scopes: Vec<Scope>,
    /// Resolution errors collected
    errors: Vec<ResolutionError>,
    /// Track bindings in the current pattern being processed
    /// This is used to detect duplicate bindings within a single pattern
    pattern_bindings: std::collections::HashSet<Box<str>>,
    /// Capability resolver for mapping symbolic names to provider:action pairs
    capability_resolver: CapabilityResolver,
}

/// Name resolution error
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ResolutionError {
    /// Unbound variable
    #[error("Unbound variable: {0}")]
    UnboundVariable(String, Span),
    /// Duplicate binding in same scope
    #[error("Duplicate binding: {0}")]
    DuplicateBinding(String, Span),
    /// Undefined capability
    #[error("Undefined capability: {0}")]
    UndefinedCapability(String, Span),
    /// Unresolved symbolic capability - capability name could not be mapped to provider:action
    #[error("Unresolved capability '{capability}': no mapping to provider:action found")]
    UnresolvedSymbolicCapability {
        /// The symbolic capability name that could not be resolved
        capability: String,
        /// Source span
        span: Span,
    },
    /// Undefined policy
    #[error("Undefined policy: {0}")]
    UndefinedPolicy(String, Span),
    /// Undefined role
    #[error("Undefined role: {0}")]
    UndefinedRole(String, Span),
}

impl NameResolver {
    /// Create a new name resolver with a root scope
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new(0)],
            errors: Vec::new(),
            pattern_bindings: std::collections::HashSet::new(),
            capability_resolver: CapabilityResolver::new(),
        }
    }

    /// Get a reference to the capability resolver
    pub fn capability_resolver(&self) -> &CapabilityResolver {
        &self.capability_resolver
    }

    /// Get a mutable reference to the capability resolver
    pub fn capability_resolver_mut(&mut self) -> &mut CapabilityResolver {
        &mut self.capability_resolver
    }

    /// Resolve an operational target to a capability target.
    ///
    /// Returns `Some(CapabilityTarget)` on success, or `None` if the symbolic
    /// capability cannot be resolved. Note: This does NOT add errors for
    /// unresolved symbolic capabilities - those are resolved during lowering.
    pub fn resolve_operational_target(
        &mut self,
        target: &OperationalTarget,
    ) -> Option<CapabilityTarget> {
        self.capability_resolver.resolve_target(target)
    }

    /// Enter a new scope
    pub fn push_scope(&mut self) {
        let depth = self.scopes.len();
        self.scopes.push(Scope::new(depth));
    }

    /// Exit the current scope
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Get the current scope depth
    pub fn current_depth(&self) -> usize {
        self.scopes.len() - 1
    }

    /// Bind a name in the current scope
    pub fn bind(&mut self, name: impl Into<Box<str>>) {
        let name = name.into();
        if let Some(scope) = self.scopes.last_mut() {
            // Allow shadowing: replace existing binding if present
            scope.insert(name);
        }
    }

    /// Lookup a name in all scopes (innermost first)
    pub fn lookup(&self, name: &str) -> Option<&BindingInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.lookup_local(name) {
                return Some(info);
            }
        }
        None
    }

    /// Check if a name is bound
    pub fn is_bound(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// Resolve all names in a workflow, collecting errors
    pub fn resolve_workflow(&mut self, workflow: &Workflow) -> Result<(), Vec<ResolutionError>> {
        self.resolve_workflow_inner(workflow);

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Internal method to resolve a workflow
    fn resolve_workflow_inner(&mut self, workflow: &Workflow) {
        match workflow {
            Workflow::Observe {
                capability,
                binding,
                continuation,
                ..
            } => {
                // Check capability exists (for now, we accept any)
                let _ = capability;

                // Bind pattern if present
                if let Some(pat) = binding {
                    self.bind_pattern(pat);
                }

                // Continue with rest
                if let Some(cont) = continuation {
                    self.resolve_workflow_inner(cont);
                }
            }

            Workflow::Act { action, .. } => {
                // Resolve the operational target (symbolic -> provider:action)
                let _ = self.resolve_operational_target(&action.target);

                // Resolve arguments
                for arg in &action.args {
                    self.resolve_expr(arg);
                }
            }

            Workflow::Let {
                pattern,
                expr,
                continuation,
                ..
            } => {
                // First resolve the expression (in current scope)
                self.resolve_expr(expr);

                // Then bind the pattern
                self.bind_pattern(pattern);

                // Continue with rest
                if let Some(cont) = continuation {
                    self.resolve_workflow_inner(cont);
                }
            }

            Workflow::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.resolve_expr(condition);
                // Create new scope for then branch
                self.push_scope();
                self.resolve_workflow_inner(then_branch);
                self.pop_scope();
                if let Some(else_branch) = else_branch {
                    // Create new scope for else branch
                    self.push_scope();
                    self.resolve_workflow_inner(else_branch);
                    self.pop_scope();
                }
            }

            Workflow::Orient {
                expr,
                binding,
                continuation,
                ..
            } => {
                self.resolve_expr(expr);

                if let Some(pat) = binding {
                    self.bind_pattern(pat);
                }

                if let Some(cont) = continuation {
                    self.resolve_workflow_inner(cont);
                }
            }

            Workflow::Decide {
                expr,
                then_branch,
                else_branch,
                ..
            } => {
                self.resolve_expr(expr);
                // Create new scope for then branch
                self.push_scope();
                self.resolve_workflow_inner(then_branch);
                self.pop_scope();
                if let Some(else_branch) = else_branch {
                    // Create new scope for else branch
                    self.push_scope();
                    self.resolve_workflow_inner(else_branch);
                    self.pop_scope();
                }
            }

            Workflow::Check {
                target,
                continuation,
                ..
            } => {
                // Resolve obligation condition if it's an obligation target
                if let ash_parser::surface::CheckTarget::Obligation(obligation) = target {
                    self.resolve_expr(&obligation.condition);
                }

                if let Some(cont) = continuation {
                    self.resolve_workflow_inner(cont);
                }
            }

            Workflow::Propose {
                action,
                binding,
                continuation,
                ..
            } => {
                // Resolve the operational target (symbolic -> provider:action)
                let _ = self.resolve_operational_target(&action.target);

                for arg in &action.args {
                    self.resolve_expr(arg);
                }

                // Bind pattern if present
                if let Some(pat) = binding {
                    self.bind_pattern(pat);
                }

                if let Some(cont) = continuation {
                    self.resolve_workflow_inner(cont);
                }
            }

            Workflow::For {
                pattern,
                collection,
                body,
                ..
            } => {
                self.resolve_expr(collection);

                // New scope for loop variable
                self.push_scope();
                self.bind_pattern(pattern);
                self.resolve_workflow_inner(body);
                self.pop_scope();
            }

            Workflow::With { body, .. } => {
                self.push_scope();
                self.resolve_workflow_inner(body);
                self.pop_scope();
            }

            Workflow::Maybe {
                primary, fallback, ..
            } => {
                self.push_scope();
                self.resolve_workflow_inner(primary);
                self.pop_scope();

                self.push_scope();
                self.resolve_workflow_inner(fallback);
                self.pop_scope();
            }

            Workflow::Must { body, .. } => {
                self.resolve_workflow_inner(body);
            }

            Workflow::Seq { first, second, .. } => {
                self.resolve_workflow_inner(first);
                self.resolve_workflow_inner(second);
            }

            Workflow::Done { .. } => {
                // Nothing to resolve
            }

            Workflow::Ret { expr, .. } => {
                // Resolve the return expression
                self.resolve_expr(expr);
            }

            Workflow::Oblige { .. } => {
                // Nothing to resolve for obligation creation
            }

            Workflow::Set {
                value,
                continuation,
                ..
            } => {
                self.resolve_expr(value);
                if let Some(cont) = continuation {
                    self.resolve_workflow_inner(cont);
                }
            }

            Workflow::Send {
                value,
                continuation,
                ..
            } => {
                self.resolve_expr(value);
                if let Some(cont) = continuation {
                    self.resolve_workflow_inner(cont);
                }
            }

            Workflow::Receive { arms, .. } => {
                // Resolve bindings from receive arm patterns and their bodies
                for arm in arms {
                    // Bind pattern variables
                    self.resolve_receive_pattern(&arm.pattern);
                    // Resolve guard if present
                    if let Some(guard) = &arm.guard {
                        self.resolve_expr(guard);
                    }
                    // Resolve arm body
                    self.resolve_workflow_inner(&arm.body);
                }
            }

            Workflow::Yield { expr, arms, .. } => {
                // Resolve the request expression
                self.resolve_expr(expr);
                // Resolve all arm bodies
                for arm in arms {
                    self.resolve_workflow_inner(&arm.body);
                }
            }

            Workflow::Resume { expr, .. } => {
                // Resolve the response value expression
                self.resolve_expr(expr);
            }
        }
    }

    /// Resolve bindings from a receive pattern
    fn resolve_receive_pattern(&mut self, pattern: &ash_parser::surface::StreamPattern) {
        use ash_parser::surface::StreamPattern;
        match pattern {
            StreamPattern::Wildcard => {
                // No binding
            }
            StreamPattern::Literal(_) => {
                // No binding
            }
            StreamPattern::Binding { pattern, .. } => {
                // Bind the inner pattern
                self.bind_pattern(pattern);
            }
        }
    }

    /// Resolve names in an expression
    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::OperatorSection { section } => {
                if let Some(left) = &section.left {
                    self.resolve_expr(left);
                }
                if let Some(right) = &section.right {
                    self.resolve_expr(right);
                }
            }
            Expr::Variable { name, span, .. } => {
                if !self.is_bound(name) {
                    self.errors
                        .push(ResolutionError::UnboundVariable(name.to_string(), *span));
                }
            }

            Expr::Literal(_) => {
                // Literals don't contain names
            }

            Expr::FieldAccess { base, .. } => {
                self.resolve_expr(base);
            }

            Expr::IndexAccess { base, index, .. } => {
                self.resolve_expr(base);
                self.resolve_expr(index);
            }

            Expr::Unary { operand, .. } => {
                self.resolve_expr(operand);
            }

            Expr::Binary { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }

            Expr::Call { args, .. } => {
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            Expr::MacroInvocation { .. } => {}

            Expr::Policy(policy_expr) => {
                self.resolve_policy_expr(policy_expr);
            }

            Expr::IfLet {
                expr,
                then_branch,
                else_branch,
                ..
            } => {
                self.resolve_expr(expr);
                self.resolve_expr(then_branch);
                self.resolve_expr(else_branch);
            }

            Expr::Match {
                scrutinee, arms, ..
            } => {
                self.resolve_expr(scrutinee);
                for arm in arms {
                    self.push_scope();
                    self.bind_pattern(&arm.pattern);
                    self.resolve_expr(&arm.body);
                    self.pop_scope();
                }
            }

            Expr::Constructor { fields, .. } => {
                for (_, expr) in fields {
                    self.resolve_expr(expr);
                }
            }

            Expr::Record { fields, .. } => {
                for (_, expr) in fields {
                    self.resolve_expr(expr);
                }
            }

            Expr::CheckObligation { .. } => {
                // Nothing to resolve for obligation check expressions
            }

            Expr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.resolve_expr(condition);
                self.resolve_expr(then_branch);
                if let Some(e) = else_branch {
                    self.resolve_expr(e);
                }
            }

            Expr::Panic { .. } => {
                // Nothing to resolve for panic
            }

            Expr::Fail { payload, .. } => {
                self.resolve_expr(payload);
            }

            Expr::WithError { body, arms, .. } => {
                self.resolve_expr(body);
                for arm in arms {
                    self.push_scope();
                    self.bind_pattern(&arm.pattern);
                    self.resolve_expr(&arm.body);
                    self.pop_scope();
                }
            }

            Expr::Block {
                statements,
                tail_expr,
                ..
            } => {
                for stmt in statements {
                    match stmt {
                        ash_parser::surface::BlockStmt::Let { pattern, expr, .. } => {
                            self.resolve_expr(expr);
                            self.bind_pattern(pattern);
                        }
                        ash_parser::surface::BlockStmt::Expr { expr, .. } => {
                            self.resolve_expr(expr);
                        }
                    }
                }
                if let Some(e) = tail_expr {
                    self.resolve_expr(e);
                }
            }

            Expr::FnDef { params, body, .. } => {
                self.push_scope();
                for (name, _ty) in params {
                    self.bind(name.as_ref());
                }
                self.resolve_expr(body);
                self.pop_scope();
            }

            Expr::FnApply { func, args, .. } => {
                self.resolve_expr(func);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }

            Expr::ActBlock { stmts, .. } => {
                for stmt in stmts {
                    let value = match stmt {
                        ash_parser::surface::ActStmt::Bind { value, .. } => value,
                        ash_parser::surface::ActStmt::Return { value, .. } => value,
                    };
                    self.resolve_expr(value);
                }
            }

            Expr::DoBlock { stmts, .. } => {
                for stmt in stmts {
                    match stmt {
                        ash_parser::surface::DoStmt::Let { name, value, .. }
                        | ash_parser::surface::DoStmt::Bind { name, value, .. } => {
                            self.resolve_expr(value);
                            self.bind(name.as_ref());
                        }
                        ash_parser::surface::DoStmt::Return { value, .. } => {
                            self.resolve_expr(value);
                        }
                        ash_parser::surface::DoStmt::Expr { value, .. } => {
                            self.resolve_expr(value);
                        }
                        ash_parser::surface::DoStmt::WorkflowRequires { .. }
                        | ash_parser::surface::DoStmt::WorkflowEnsures { .. } => {
                            // Contract statements are raw workflow-contract syntax until the
                            // Workflow elaborator classifies them. Do not resolve role symbols
                            // or the delayed `result` binder as ordinary lexical variables here.
                        }
                    }
                }
            }

            Expr::Comprehension {
                result, qualifiers, ..
            } => {
                use ash_parser::surface::ComprehensionQualifier;

                self.push_scope();
                for qualifier in qualifiers {
                    match qualifier {
                        ComprehensionQualifier::Let { name, value, .. }
                        | ComprehensionQualifier::Bind { name, value, .. } => {
                            self.resolve_expr(value);
                            self.bind(name.as_ref());
                        }
                        ComprehensionQualifier::DiscardBind { value, .. } => {
                            self.resolve_expr(value);
                        }
                    }
                }
                self.resolve_expr(result);
                self.pop_scope();
            }
            Expr::List { items, .. } => {
                for item in items {
                    self.resolve_expr(item);
                }
            }
        }
    }

    /// Resolve names in a policy expression
    fn resolve_policy_expr(&mut self, expr: &ash_parser::surface::PolicyExpr) {
        use ash_parser::surface::PolicyExpr;

        match expr {
            PolicyExpr::Var { name, span, .. } => {
                if !self.is_bound(name) {
                    self.errors
                        .push(ResolutionError::UnboundVariable(name.to_string(), *span));
                }
            }

            PolicyExpr::And(exprs)
            | PolicyExpr::Or(exprs)
            | PolicyExpr::Sequential(exprs)
            | PolicyExpr::Concurrent(exprs) => {
                for e in exprs {
                    self.resolve_policy_expr(e);
                }
            }

            PolicyExpr::Not(inner) | PolicyExpr::Implies(inner, _) => {
                self.resolve_policy_expr(inner);
            }

            PolicyExpr::ForAll { items, body, .. } | PolicyExpr::Exists { items, body, .. } => {
                self.resolve_expr(items);
                self.resolve_policy_expr(body);
            }

            PolicyExpr::MethodCall { receiver, args, .. } => {
                self.resolve_policy_expr(receiver);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }

            PolicyExpr::Call { args, .. } => {
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
        }
    }

    /// Bind names from a pattern
    ///
    /// This method checks for duplicate bindings within the pattern and rejects them.
    /// Duplicate bindings across different patterns (e.g., in different statements) are allowed.
    fn bind_pattern(&mut self, pattern: &Pattern) {
        // Clear the pattern bindings set at the start of processing a pattern
        self.pattern_bindings.clear();
        self.bind_pattern_recursive(pattern);
        // Clear the pattern bindings set after processing (not strictly necessary but clean)
        self.pattern_bindings.clear();
    }

    /// Recursively bind names from a pattern, checking for duplicates
    fn bind_pattern_recursive(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Variable { name, span, .. } => {
                // Check if this name is already bound in the current pattern
                if self.pattern_bindings.contains(name.as_ref()) {
                    self.errors
                        .push(ResolutionError::DuplicateBinding(name.to_string(), *span));
                } else {
                    self.pattern_bindings.insert(name.clone());
                    self.bind(name.clone());
                }
            }

            Pattern::Wildcard => {
                // Nothing to bind
            }

            Pattern::Tuple(patterns) => {
                for pat in patterns {
                    self.bind_pattern_recursive(pat);
                }
            }

            Pattern::Record(fields) => {
                for (_, pat) in fields {
                    self.bind_pattern_recursive(pat);
                }
            }

            Pattern::List { elements, rest } => {
                for elem in elements {
                    self.bind_pattern_recursive(elem);
                }
                if let Some(rest_name) = rest {
                    // Check if this name is already bound in the current pattern
                    if self.pattern_bindings.contains(rest_name.as_ref()) {
                        self.errors.push(ResolutionError::DuplicateBinding(
                            rest_name.to_string(),
                            Span::default(),
                        ));
                    } else {
                        self.pattern_bindings.insert(rest_name.clone());
                        self.bind(rest_name.clone());
                    }
                }
            }

            Pattern::Literal(_) => {
                // Nothing to bind
            }

            Pattern::Variant { fields, .. } => {
                if let Some(fields) = fields {
                    for (_, pat) in fields {
                        self.bind_pattern_recursive(pat);
                    }
                }
            }
        }
    }

    /// Get collected errors
    pub fn errors(&self) -> &[ResolutionError] {
        &self.errors
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Clear all errors
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    /// Get all bound names in current scope chain
    pub fn all_bindings(&self) -> Vec<&BindingInfo> {
        let mut result = Vec::new();
        for scope in &self.scopes {
            for info in scope.bindings().values() {
                result.push(info);
            }
        }
        result
    }
}

/// Result of name resolution
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// Whether resolution succeeded
    pub success: bool,
    /// Errors encountered
    pub errors: Vec<ResolutionError>,
    /// Number of bindings found
    pub binding_count: usize,
}

impl ResolutionResult {
    /// Create a successful result
    pub fn success(count: usize) -> Self {
        Self {
            success: true,
            errors: vec![],
            binding_count: count,
        }
    }

    /// Create a failed result
    pub fn failure(errors: Vec<ResolutionError>) -> Self {
        Self {
            success: false,
            errors,
            binding_count: 0,
        }
    }
}

/// Quick resolve function for workflows
pub fn resolve_workflow(workflow: &Workflow) -> Result<ResolutionResult, Vec<ResolutionError>> {
    let mut resolver = NameResolver::new();
    resolver.resolve_workflow(workflow)?;

    Ok(ResolutionResult::success(resolver.all_bindings().len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::surface::{
        ActionRef, CheckTarget, ComprehensionQualifier, Literal, ObligationRef,
    };

    fn test_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    #[test]
    fn test_scope_creation() {
        let scope = Scope::new(0);
        assert!(scope.bindings().is_empty());
        assert_eq!(scope.depth(), 0);
    }

    #[test]
    fn test_scope_insert_and_lookup() {
        let mut scope = Scope::new(0);
        scope.insert("x".into());

        assert!(scope.contains("x"));
        assert!(!scope.contains("y"));

        let info = scope.lookup_local("x").unwrap();
        assert_eq!(info.name, "x".into());
        assert_eq!(info.depth, 0);
    }

    #[test]
    fn test_resolver_creation() {
        let resolver = NameResolver::new();
        assert!(!resolver.has_errors());
        assert_eq!(resolver.current_depth(), 0);
    }

    #[test]
    fn test_resolver_push_pop_scope() {
        let mut resolver = NameResolver::new();
        assert_eq!(resolver.current_depth(), 0);

        resolver.push_scope();
        assert_eq!(resolver.current_depth(), 1);

        resolver.pop_scope();
        assert_eq!(resolver.current_depth(), 0);

        // Can't pop root scope
        resolver.pop_scope();
        assert_eq!(resolver.current_depth(), 0);
    }

    #[test]
    fn test_resolver_bind_and_lookup() {
        let mut resolver = NameResolver::new();
        resolver.bind("x");

        assert!(resolver.is_bound("x"));
        assert!(!resolver.is_bound("y"));

        let info = resolver.lookup("x").unwrap();
        assert_eq!(info.name, "x".into());
    }

    #[test]
    fn test_resolver_lookup_across_scopes() {
        let mut resolver = NameResolver::new();
        resolver.bind("x");

        resolver.push_scope();
        resolver.bind("y");

        // Should find x in parent scope
        assert!(resolver.is_bound("x"));
        // Should find y in current scope
        assert!(resolver.is_bound("y"));

        resolver.pop_scope();

        // Should still find x
        assert!(resolver.is_bound("x"));
        // Should not find y anymore
        assert!(!resolver.is_bound("y"));
    }

    #[test]
    fn test_resolve_expr_variable_bound() {
        let mut resolver = NameResolver::new();
        resolver.bind("x");

        let expr = Expr::Variable {
            name: "x".into(),
            span: ash_parser::token::Span::default(),
        };
        resolver.resolve_expr(&expr);

        assert!(!resolver.has_errors());
    }

    #[test]
    fn test_resolve_expr_variable_unbound() {
        let mut resolver = NameResolver::new();

        let expr = Expr::Variable {
            name: "x".into(),
            span: ash_parser::token::Span::default(),
        };
        resolver.resolve_expr(&expr);

        assert!(resolver.has_errors());
        assert_eq!(resolver.errors().len(), 1);
        assert!(matches!(
            resolver.errors()[0],
            ResolutionError::UnboundVariable(_, _)
        ));
    }

    #[test]
    fn test_resolve_expr_literal() {
        let mut resolver = NameResolver::new();

        let expr = Expr::Literal(Literal::Int(42));
        resolver.resolve_expr(&expr);

        assert!(!resolver.has_errors());
    }

    #[test]
    fn test_resolve_expr_binary() {
        let mut resolver = NameResolver::new();
        resolver.bind("x");
        resolver.bind("y");

        let expr = Expr::Binary {
            op: ash_parser::surface::BinaryOp::Add,
            raw_operator: None,
            left: Box::new(Expr::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            }),
            right: Box::new(Expr::Variable {
                name: "y".into(),
                span: ash_parser::token::Span::default(),
            }),
            span: test_span(),
        };
        resolver.resolve_expr(&expr);

        assert!(!resolver.has_errors());
    }

    #[test]
    fn test_resolve_expr_binary_unbound() {
        let mut resolver = NameResolver::new();

        let expr = Expr::Binary {
            op: ash_parser::surface::BinaryOp::Add,
            raw_operator: None,
            left: Box::new(Expr::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            }),
            right: Box::new(Expr::Variable {
                name: "y".into(),
                span: ash_parser::token::Span::default(),
            }),
            span: test_span(),
        };
        resolver.resolve_expr(&expr);

        assert!(resolver.has_errors());
        assert_eq!(resolver.errors().len(), 2);
    }

    #[test]
    fn test_bind_pattern_variable() {
        let mut resolver = NameResolver::new();
        let pattern = Pattern::Variable {
            name: "x".into(),
            span: ash_parser::token::Span::default(),
        };

        resolver.bind_pattern(&pattern);

        assert!(resolver.is_bound("x"));
    }

    #[test]
    fn test_bind_pattern_tuple() {
        let mut resolver = NameResolver::new();
        let pattern = Pattern::Tuple(vec![
            Pattern::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            },
            Pattern::Variable {
                name: "y".into(),
                span: ash_parser::token::Span::default(),
            },
        ]);

        resolver.bind_pattern(&pattern);

        assert!(resolver.is_bound("x"));
        assert!(resolver.is_bound("y"));
    }

    #[test]
    fn test_bind_pattern_record() {
        let mut resolver = NameResolver::new();
        let pattern = Pattern::Record(vec![
            (
                "a".into(),
                Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
            ),
            (
                "b".into(),
                Pattern::Variable {
                    name: "y".into(),
                    span: ash_parser::token::Span::default(),
                },
            ),
        ]);

        resolver.bind_pattern(&pattern);

        assert!(resolver.is_bound("x"));
        assert!(resolver.is_bound("y"));
    }

    #[test]
    fn test_bind_pattern_list() {
        let mut resolver = NameResolver::new();
        let pattern = Pattern::List {
            elements: vec![Pattern::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            }],
            rest: Some("xs".into()),
        };

        resolver.bind_pattern(&pattern);

        assert!(resolver.is_bound("x"));
        assert!(resolver.is_bound("xs"));
    }

    #[test]
    fn test_resolve_workflow_done() {
        let mut resolver = NameResolver::new();
        let workflow = Workflow::Done { span: test_span() };

        let result = resolver.resolve_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_workflow_let() {
        let mut resolver = NameResolver::new();
        let workflow = Workflow::Let {
            pattern: Pattern::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            },
            expr: Expr::Literal(Literal::Int(42)),
            continuation: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = resolver.resolve_workflow(&workflow);
        assert!(result.is_ok());
        // x should be bound in the resolver after resolution
        assert!(resolver.is_bound("x"));
    }

    #[test]
    fn test_resolve_workflow_let_use_variable() {
        let mut resolver = NameResolver::new();
        let workflow = Workflow::Let {
            pattern: Pattern::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            },
            expr: Expr::Literal(Literal::Int(42)),
            continuation: Some(Box::new(Workflow::Orient {
                expr: Expr::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
                binding: None,
                continuation: None,
                span: test_span(),
            })),
            span: test_span(),
        };

        let result = resolver.resolve_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_workflow_if() {
        let mut resolver = NameResolver::new();
        resolver.bind("cond");

        let workflow = Workflow::If {
            condition: Expr::Variable {
                name: "cond".into(),
                span: ash_parser::token::Span::default(),
            },
            then_branch: Box::new(Workflow::Done { span: test_span() }),
            else_branch: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = resolver.resolve_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_workflow_if_unbound() {
        let mut resolver = NameResolver::new();

        let workflow = Workflow::If {
            condition: Expr::Variable {
                name: "cond".into(),
                span: ash_parser::token::Span::default(),
            },
            then_branch: Box::new(Workflow::Done { span: test_span() }),
            else_branch: Some(Box::new(Workflow::Done { span: test_span() })),
            span: test_span(),
        };

        let result = resolver.resolve_workflow(&workflow);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_workflow_seq() {
        let mut resolver = NameResolver::new();
        let workflow = Workflow::Seq {
            first: Box::new(Workflow::Let {
                pattern: Pattern::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
                expr: Expr::Literal(Literal::Int(42)),
                continuation: None,
                span: test_span(),
            }),
            second: Box::new(Workflow::Orient {
                expr: Expr::Variable {
                    name: "x".into(),
                    span: ash_parser::token::Span::default(),
                },
                binding: None,
                continuation: None,
                span: test_span(),
            }),
            span: test_span(),
        };

        let result = resolver.resolve_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_workflow_for() {
        let mut resolver = NameResolver::new();
        resolver.bind("items");

        let workflow = Workflow::For {
            pattern: Pattern::Variable {
                name: "item".into(),
                span: ash_parser::token::Span::default(),
            },
            collection: Expr::Variable {
                name: "items".into(),
                span: ash_parser::token::Span::default(),
            },
            body: Box::new(Workflow::Orient {
                expr: Expr::Variable {
                    name: "item".into(),
                    span: ash_parser::token::Span::default(),
                },
                binding: None,
                continuation: None,
                span: test_span(),
            }),
            span: test_span(),
        };

        let result = resolver.resolve_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_comprehension_qualifiers_bind_left_to_right() {
        let mut resolver = NameResolver::new();
        resolver.bind("xs");

        let workflow = Workflow::Ret {
            expr: Expr::Comprehension {
                result: Box::new(Expr::Variable {
                    name: "y".into(),
                    span: test_span(),
                }),
                qualifiers: vec![
                    ComprehensionQualifier::Bind {
                        name: "x".into(),
                        value: Box::new(Expr::Variable {
                            name: "xs".into(),
                            span: test_span(),
                        }),
                        span: test_span(),
                    },
                    ComprehensionQualifier::Let {
                        name: "y".into(),
                        value: Box::new(Expr::Variable {
                            name: "x".into(),
                            span: test_span(),
                        }),
                        span: test_span(),
                    },
                ],
                target: None,
                span: test_span(),
            },
            span: test_span(),
        };

        assert!(resolver.resolve_workflow(&workflow).is_ok());
    }

    #[test]
    fn test_resolve_comprehension_qualifier_rhs_cannot_see_later_bindings() {
        let mut resolver = NameResolver::new();
        resolver.bind("xs");

        let workflow = Workflow::Ret {
            expr: Expr::Comprehension {
                result: Box::new(Expr::Variable {
                    name: "late".into(),
                    span: test_span(),
                }),
                qualifiers: vec![
                    ComprehensionQualifier::Let {
                        name: "first".into(),
                        value: Box::new(Expr::Variable {
                            name: "late".into(),
                            span: test_span(),
                        }),
                        span: test_span(),
                    },
                    ComprehensionQualifier::Bind {
                        name: "late".into(),
                        value: Box::new(Expr::Variable {
                            name: "xs".into(),
                            span: test_span(),
                        }),
                        span: test_span(),
                    },
                ],
                target: None,
                span: test_span(),
            },
            span: test_span(),
        };

        let result = resolver.resolve_workflow(&workflow);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_workflow_act() {
        let mut resolver = NameResolver::new();
        resolver.bind("arg");

        let workflow = Workflow::Act {
            action: ActionRef {
                target: OperationalTarget::Explicit {
                    provider: "io".into(),
                    action: "write".into(),
                },
                args: vec![Expr::Variable {
                    name: "arg".into(),
                    span: ash_parser::token::Span::default(),
                }],
            },
            guard: None,
            result_name: None,
            continuation: None,
            span: test_span(),
        };

        let result = resolver.resolve_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_workflow_check() {
        let mut resolver = NameResolver::new();

        let workflow = Workflow::Check {
            target: CheckTarget::Obligation(ObligationRef {
                role: "admin".into(),
                condition: Expr::Literal(Literal::Bool(true)),
            }),
            continuation: None,
            span: test_span(),
        };

        let result = resolver.resolve_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_shadowing_allowed() {
        let mut resolver = NameResolver::new();
        resolver.bind("x");
        resolver.bind("x"); // Shadow - should be allowed

        // Shadowing is now allowed, so no errors
        assert!(!resolver.has_errors());

        // The second binding should shadow the first
        assert!(resolver.is_bound("x"));
    }

    #[test]
    fn test_resolution_error_display() {
        let err = ResolutionError::UnboundVariable("x".to_string(), Span::default());
        assert!(format!("{err}").contains("x"));

        let err = ResolutionError::DuplicateBinding("x".to_string(), Span::default());
        assert!(format!("{err}").contains("x"));

        let err = ResolutionError::UndefinedCapability("FileIO".to_string(), Span::default());
        assert!(format!("{err}").contains("FileIO"));
    }

    #[test]
    fn test_resolution_result_success() {
        let result = ResolutionResult::success(5);
        assert!(result.success);
        assert!(result.errors.is_empty());
        assert_eq!(result.binding_count, 5);
    }

    #[test]
    fn test_resolution_result_failure() {
        let errors = vec![ResolutionError::UnboundVariable(
            "x".to_string(),
            Span::default(),
        )];
        let result = ResolutionResult::failure(errors);
        assert!(!result.success);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_all_bindings() {
        let mut resolver = NameResolver::new();
        resolver.bind("x");
        resolver.push_scope();
        resolver.bind("y");

        let bindings = resolver.all_bindings();
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_resolver_clear_errors() {
        let mut resolver = NameResolver::new();
        resolver.resolve_expr(&Expr::Variable {
            name: "x".into(),
            span: ash_parser::token::Span::default(),
        });
        assert!(resolver.has_errors());

        resolver.clear_errors();
        assert!(!resolver.has_errors());
    }
}
