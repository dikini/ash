//! Name resolution for Ash type system (TASK-022)
//!
//! Provides scope tracking and name resolution for variables, capabilities,
//! and other named entities in workflows and expressions.

use ash_parser::surface::{Expr, Pattern, Workflow};
use std::collections::HashMap;

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
}

/// Name resolution error
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ResolutionError {
    /// Unbound variable
    #[error("Unbound variable: {0}")]
    UnboundVariable(String),
    /// Duplicate binding in same scope
    #[error("Duplicate binding: {0}")]
    DuplicateBinding(String),
    /// Undefined capability
    #[error("Undefined capability: {0}")]
    UndefinedCapability(String),
    /// Undefined policy
    #[error("Undefined policy: {0}")]
    UndefinedPolicy(String),
    /// Undefined role
    #[error("Undefined role: {0}")]
    UndefinedRole(String),
}

impl NameResolver {
    /// Create a new name resolver with a root scope
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new(0)],
            errors: Vec::new(),
        }
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
            // Check for duplicate binding in current scope
            if scope.contains(&name) {
                self.errors
                    .push(ResolutionError::DuplicateBinding(name.to_string()));
                return;
            }
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
                self.resolve_workflow_inner(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_workflow_inner(else_branch);
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
                self.resolve_workflow_inner(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_workflow_inner(else_branch);
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
                continuation,
                ..
            } => {
                for arg in &action.args {
                    self.resolve_expr(arg);
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

            Workflow::Par { branches, .. } => {
                for branch in branches {
                    self.push_scope();
                    self.resolve_workflow_inner(branch);
                    self.pop_scope();
                }
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
            Expr::Variable(name) => {
                if !self.is_bound(name) {
                    self.errors
                        .push(ResolutionError::UnboundVariable(name.to_string()));
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
                    self.resolve_expr(&arm.body);
                }
            }

            Expr::Constructor { fields, .. } => {
                for (_, expr) in fields {
                    self.resolve_expr(expr);
                }
            }

            Expr::CheckObligation { .. } => {
                // Nothing to resolve for obligation check expressions
            }
        }
    }

    /// Resolve names in a policy expression
    fn resolve_policy_expr(&mut self, expr: &ash_parser::surface::PolicyExpr) {
        use ash_parser::surface::PolicyExpr;

        match expr {
            PolicyExpr::Var(name) => {
                if !self.is_bound(name) {
                    self.errors
                        .push(ResolutionError::UnboundVariable(name.to_string()));
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
    fn bind_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Variable(name) => {
                self.bind(name.clone());
            }

            Pattern::Wildcard => {
                // Nothing to bind
            }

            Pattern::Tuple(patterns) => {
                for pat in patterns {
                    self.bind_pattern(pat);
                }
            }

            Pattern::Record(fields) => {
                for (_, pat) in fields {
                    self.bind_pattern(pat);
                }
            }

            Pattern::List { elements, rest } => {
                for elem in elements {
                    self.bind_pattern(elem);
                }
                if let Some(rest_name) = rest {
                    self.bind(rest_name.clone());
                }
            }

            Pattern::Literal(_) => {
                // Nothing to bind
            }

            Pattern::Variant { fields, .. } => {
                if let Some(fields) = fields {
                    for (_, pat) in fields {
                        self.bind_pattern(pat);
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
    use ash_parser::surface::{ActionRef, CheckTarget, Literal, ObligationRef};
    use ash_parser::token::Span;

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

        let expr = Expr::Variable("x".into());
        resolver.resolve_expr(&expr);

        assert!(!resolver.has_errors());
    }

    #[test]
    fn test_resolve_expr_variable_unbound() {
        let mut resolver = NameResolver::new();

        let expr = Expr::Variable("x".into());
        resolver.resolve_expr(&expr);

        assert!(resolver.has_errors());
        assert_eq!(resolver.errors().len(), 1);
        assert!(matches!(
            resolver.errors()[0],
            ResolutionError::UnboundVariable(_)
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
            left: Box::new(Expr::Variable("x".into())),
            right: Box::new(Expr::Variable("y".into())),
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
            left: Box::new(Expr::Variable("x".into())),
            right: Box::new(Expr::Variable("y".into())),
            span: test_span(),
        };
        resolver.resolve_expr(&expr);

        assert!(resolver.has_errors());
        assert_eq!(resolver.errors().len(), 2);
    }

    #[test]
    fn test_bind_pattern_variable() {
        let mut resolver = NameResolver::new();
        let pattern = Pattern::Variable("x".into());

        resolver.bind_pattern(&pattern);

        assert!(resolver.is_bound("x"));
    }

    #[test]
    fn test_bind_pattern_tuple() {
        let mut resolver = NameResolver::new();
        let pattern = Pattern::Tuple(vec![
            Pattern::Variable("x".into()),
            Pattern::Variable("y".into()),
        ]);

        resolver.bind_pattern(&pattern);

        assert!(resolver.is_bound("x"));
        assert!(resolver.is_bound("y"));
    }

    #[test]
    fn test_bind_pattern_record() {
        let mut resolver = NameResolver::new();
        let pattern = Pattern::Record(vec![
            ("a".into(), Pattern::Variable("x".into())),
            ("b".into(), Pattern::Variable("y".into())),
        ]);

        resolver.bind_pattern(&pattern);

        assert!(resolver.is_bound("x"));
        assert!(resolver.is_bound("y"));
    }

    #[test]
    fn test_bind_pattern_list() {
        let mut resolver = NameResolver::new();
        let pattern = Pattern::List {
            elements: vec![Pattern::Variable("x".into())],
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
            pattern: Pattern::Variable("x".into()),
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
            pattern: Pattern::Variable("x".into()),
            expr: Expr::Literal(Literal::Int(42)),
            continuation: Some(Box::new(Workflow::Orient {
                expr: Expr::Variable("x".into()),
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
            condition: Expr::Variable("cond".into()),
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
            condition: Expr::Variable("cond".into()),
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
                pattern: Pattern::Variable("x".into()),
                expr: Expr::Literal(Literal::Int(42)),
                continuation: None,
                span: test_span(),
            }),
            second: Box::new(Workflow::Orient {
                expr: Expr::Variable("x".into()),
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
            pattern: Pattern::Variable("item".into()),
            collection: Expr::Variable("items".into()),
            body: Box::new(Workflow::Orient {
                expr: Expr::Variable("item".into()),
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
    fn test_resolve_workflow_act() {
        let mut resolver = NameResolver::new();
        resolver.bind("arg");

        let workflow = Workflow::Act {
            action: ActionRef {
                name: "write".into(),
                args: vec![Expr::Variable("arg".into())],
            },
            guard: None,
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
    fn test_duplicate_binding_error() {
        let mut resolver = NameResolver::new();
        resolver.bind("x");
        resolver.bind("x"); // Duplicate

        assert!(resolver.has_errors());
        assert!(matches!(
            resolver.errors()[0],
            ResolutionError::DuplicateBinding(_)
        ));
    }

    #[test]
    fn test_resolution_error_display() {
        let err = ResolutionError::UnboundVariable("x".to_string());
        assert!(format!("{err}").contains("x"));

        let err = ResolutionError::DuplicateBinding("x".to_string());
        assert!(format!("{err}").contains("x"));

        let err = ResolutionError::UndefinedCapability("FileIO".to_string());
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
        let errors = vec![ResolutionError::UnboundVariable("x".to_string())];
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
        resolver.resolve_expr(&Expr::Variable("x".into()));
        assert!(resolver.has_errors());

        resolver.clear_errors();
        assert!(!resolver.has_errors());
    }
}
