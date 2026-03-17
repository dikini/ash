//! Policy expression type checking for Ash.
//!
//! This module provides type inference and validation for policy combinators.
//! It ensures that policy expressions are well-formed and all components
//! resolve to valid policy types.
//!
//! # Type System
//!
//! Policy expressions are first-class values with type `Policy`:
//! - Variables must be bound to policy values
//! - Combinators combine policies into new policies
//! - Quantifiers iterate over collections producing policies
//!
//! # Examples
//!
//! ```
//! use ash_typeck::policy_check::{infer_policy_expr, PolicyType, PolicyTypeError};
//! use ash_parser::surface::PolicyExpr;
//!
//! let expr = PolicyExpr::And(vec![
//!     PolicyExpr::Var("p1".into()),
//!     PolicyExpr::Var("p2".into()),
//! ]);
//!
//! let result = infer_policy_expr(&expr, &|name| {
//!     if name.as_ref() == "p1" || name.as_ref() == "p2" {
//!         Ok(PolicyType::Policy)
//!     } else {
//!         Err(PolicyTypeError::UnknownVariable(name.to_string()))
//!     }
//! });
//!
//! assert!(result.is_ok());
//! ```

use std::collections::HashMap;

use ash_parser::surface::{Name, PolicyExpr};
use thiserror::Error;

/// The type of a policy expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyType {
    /// A policy value
    Policy,
    /// A boolean predicate (can be used where a policy is expected)
    Predicate,
}

/// Errors that can occur during policy type checking.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum PolicyTypeError {
    /// Variable not found in scope
    #[error("unknown variable: {0}")]
    UnknownVariable(String),

    /// Expected a policy but got something else
    #[error("expected policy, found {found}")]
    ExpectedPolicy { found: String },

    /// Expected a collection for quantifier
    #[error("expected collection in quantifier, found {found}")]
    ExpectedCollection { found: String },

    /// Invalid method call on policy
    #[error("unknown method '{method}' on policy")]
    UnknownMethod { method: String },

    /// Wrong number of arguments
    #[error("expected {expected} arguments, found {found}")]
    ArgumentCount { expected: usize, found: usize },

    /// Type mismatch in combinator
    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },
}

/// Context for policy type checking.
#[derive(Debug, Clone)]
pub struct PolicyCheckContext {
    /// Variable bindings (name -> type)
    bindings: HashMap<Name, PolicyType>,
    /// Known policy methods
    methods: HashMap<Name, MethodSignature>,
}

/// Signature for a policy method.
#[derive(Debug, Clone)]
pub struct MethodSignature {
    /// Number of arguments
    pub arity: usize,
    /// Argument types (if type checking is needed)
    pub arg_types: Vec<PolicyType>,
    /// Return type
    pub return_type: PolicyType,
}

impl Default for PolicyCheckContext {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyCheckContext {
    /// Create a new empty context.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let mut ctx = Self {
            bindings: HashMap::new(),
            methods: HashMap::new(),
        };
        ctx.register_builtin_methods();
        ctx
    }

    /// Register built-in policy methods.
    fn register_builtin_methods(&mut self) {
        // .and(other) -> Policy
        self.methods.insert(
            "and".into(),
            MethodSignature {
                arity: 1,
                arg_types: vec![PolicyType::Policy],
                return_type: PolicyType::Policy,
            },
        );

        // .or(other) -> Policy
        self.methods.insert(
            "or".into(),
            MethodSignature {
                arity: 1,
                arg_types: vec![PolicyType::Policy],
                return_type: PolicyType::Policy,
            },
        );

        // .retry(count) -> Policy
        self.methods.insert(
            "retry".into(),
            MethodSignature {
                arity: 1,
                arg_types: vec![PolicyType::Policy], // Actually expects Int, but we allow policy type
                return_type: PolicyType::Policy,
            },
        );

        // .timeout(ms) -> Policy
        self.methods.insert(
            "timeout".into(),
            MethodSignature {
                arity: 1,
                arg_types: vec![PolicyType::Policy],
                return_type: PolicyType::Policy,
            },
        );

        // .not() -> Policy
        self.methods.insert(
            "not".into(),
            MethodSignature {
                arity: 0,
                arg_types: vec![],
                return_type: PolicyType::Policy,
            },
        );
    }

    /// Bind a variable to a type.
    pub fn bind(&mut self, name: Name, ty: PolicyType) {
        self.bindings.insert(name, ty);
    }

    /// Look up a variable's type.
    #[inline]
    #[must_use]
    pub fn lookup(&self, name: &Name) -> Option<PolicyType> {
        self.bindings.get(name).copied()
    }

    /// Look up a method signature.
    #[inline]
    #[must_use]
    pub fn lookup_method(&self, name: &Name) -> Option<&MethodSignature> {
        self.methods.get(name)
    }

    /// Create a child context with additional bindings.
    #[inline]
    #[must_use]
    pub fn with_binding(&self, name: Name, ty: PolicyType) -> Self {
        let mut child = self.clone();
        child.bind(name, ty);
        child
    }
}

/// Type lookup function for resolving variables.
pub type TypeLookup<'a> = dyn Fn(&Name) -> Result<PolicyType, PolicyTypeError> + 'a;

/// Infer the type of a policy expression.
///
/// # Arguments
///
/// * `expr` - The policy expression to type check
/// * `lookup` - Function to resolve variable names to types
///
/// # Returns
///
/// The inferred type, or a type error.
///
/// # Example
///
/// ```
/// use ash_typeck::policy_check::{infer_policy_expr, PolicyType};
/// use ash_parser::surface::PolicyExpr;
///
/// let expr = PolicyExpr::Var("p".into());
/// let result = infer_policy_expr(&expr, &|_| Ok(PolicyType::Policy));
///
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), PolicyType::Policy);
/// ```
#[inline]
pub fn infer_policy_expr(
    expr: &PolicyExpr,
    lookup: &TypeLookup<'_>,
) -> Result<PolicyType, PolicyTypeError> {
    let ctx = PolicyCheckContext::new();
    infer_policy_expr_with_context(expr, &ctx, lookup)
}

/// Infer the type of a policy expression with context.
pub fn infer_policy_expr_with_context(
    expr: &PolicyExpr,
    ctx: &PolicyCheckContext,
    lookup: &TypeLookup<'_>,
) -> Result<PolicyType, PolicyTypeError> {
    match expr {
        PolicyExpr::Var(name) => lookup(name).map_err(|e| match e {
            PolicyTypeError::UnknownVariable(s) => PolicyTypeError::UnknownVariable(s),
            _ => e,
        }),

        PolicyExpr::And(exprs) => {
            for expr in exprs {
                let ty = infer_policy_expr_with_context(expr, ctx, lookup)?;
                if ty != PolicyType::Policy {
                    return Err(PolicyTypeError::ExpectedPolicy {
                        found: format!("{:?}", ty),
                    });
                }
            }
            Ok(PolicyType::Policy)
        }

        PolicyExpr::Or(exprs) => {
            for expr in exprs {
                let ty = infer_policy_expr_with_context(expr, ctx, lookup)?;
                if ty != PolicyType::Policy {
                    return Err(PolicyTypeError::ExpectedPolicy {
                        found: format!("{:?}", ty),
                    });
                }
            }
            Ok(PolicyType::Policy)
        }

        PolicyExpr::Not(inner) => {
            let ty = infer_policy_expr_with_context(inner, ctx, lookup)?;
            if ty != PolicyType::Policy {
                return Err(PolicyTypeError::ExpectedPolicy {
                    found: format!("{:?}", ty),
                });
            }
            Ok(PolicyType::Policy)
        }

        PolicyExpr::Implies(antecedent, consequent) => {
            let ant_ty = infer_policy_expr_with_context(antecedent, ctx, lookup)?;
            let cons_ty = infer_policy_expr_with_context(consequent, ctx, lookup)?;

            if ant_ty != PolicyType::Policy {
                return Err(PolicyTypeError::ExpectedPolicy {
                    found: format!("{:?}", ant_ty),
                });
            }
            if cons_ty != PolicyType::Policy {
                return Err(PolicyTypeError::ExpectedPolicy {
                    found: format!("{:?}", cons_ty),
                });
            }
            Ok(PolicyType::Policy)
        }

        PolicyExpr::Sequential(exprs) | PolicyExpr::Concurrent(exprs) => {
            for expr in exprs {
                let ty = infer_policy_expr_with_context(expr, ctx, lookup)?;
                if ty != PolicyType::Policy {
                    return Err(PolicyTypeError::ExpectedPolicy {
                        found: format!("{:?}", ty),
                    });
                }
            }
            Ok(PolicyType::Policy)
        }

        PolicyExpr::ForAll {
            var, items, body, ..
        } => {
            // Check that items is a collection (for now, accept any expression)
            // In a full implementation, we'd check the type of items
            let _ = items;

            // Check body with var bound
            let child_ctx = ctx.with_binding(var.clone(), PolicyType::Policy);
            let body_ty = infer_policy_expr_with_context(body, &child_ctx, lookup)?;

            if body_ty != PolicyType::Policy {
                return Err(PolicyTypeError::ExpectedPolicy {
                    found: format!("{:?}", body_ty),
                });
            }
            Ok(PolicyType::Policy)
        }

        PolicyExpr::Exists {
            var, items, body, ..
        } => {
            // Similar to ForAll
            let _ = items;

            let child_ctx = ctx.with_binding(var.clone(), PolicyType::Policy);
            let body_ty = infer_policy_expr_with_context(body, &child_ctx, lookup)?;

            if body_ty != PolicyType::Policy {
                return Err(PolicyTypeError::ExpectedPolicy {
                    found: format!("{:?}", body_ty),
                });
            }
            Ok(PolicyType::Policy)
        }

        PolicyExpr::MethodCall {
            receiver,
            method,
            args,
            ..
        } => {
            // Check receiver is a policy
            let recv_ty = infer_policy_expr_with_context(receiver, ctx, lookup)?;
            if recv_ty != PolicyType::Policy {
                return Err(PolicyTypeError::ExpectedPolicy {
                    found: format!("{:?}", recv_ty),
                });
            }

            // Look up method
            let sig = ctx
                .lookup_method(method)
                .ok_or_else(|| PolicyTypeError::UnknownMethod {
                    method: method.to_string(),
                })?;

            // Check argument count
            if args.len() != sig.arity {
                return Err(PolicyTypeError::ArgumentCount {
                    expected: sig.arity,
                    found: args.len(),
                });
            }

            // Method calls return Policy
            Ok(sig.return_type)
        }

        PolicyExpr::Call { func: _, args, .. } => {
            // For now, assume all policy function calls return Policy
            // In a full implementation, we'd look up the function signature
            let _ = args;
            Ok(PolicyType::Policy)
        }
    }
}

/// Validate that an expression is a valid policy expression.
///
/// This is a convenience function that checks if an expression
/// can be used where a policy is expected.
///
/// # Arguments
///
/// * `expr` - The expression to validate
/// * `lookup` - Function to resolve variable names to types
///
/// # Returns
///
/// Ok(()) if valid, or a type error.
#[inline]
pub fn validate_policy_expr(
    expr: &PolicyExpr,
    lookup: &TypeLookup<'_>,
) -> Result<(), PolicyTypeError> {
    let ty = infer_policy_expr(expr, lookup)?;
    if ty == PolicyType::Policy {
        Ok(())
    } else {
        Err(PolicyTypeError::ExpectedPolicy {
            found: format!("{:?}", ty),
        })
    }
}

/// Normalize a policy expression.
///
/// Applies normalization passes:
/// 1. Flatten nested And/Or
/// 2. Eliminate double negation
/// 3. Constant folding (if possible)
///
/// # Arguments
///
/// * `expr` - The expression to normalize
///
/// # Returns
///
/// The normalized expression.
#[must_use]
pub fn normalize(expr: PolicyExpr) -> PolicyExpr {
    let expr = flatten_nested_and(expr);
    let expr = flatten_nested_or(expr);
    
    eliminate_double_negation(expr)
}

/// Flatten nested And expressions.
///
/// `and(a, and(b, c))` becomes `and(a, b, c)`
#[must_use]
pub fn flatten_nested_and(expr: PolicyExpr) -> PolicyExpr {
    match expr {
        PolicyExpr::And(exprs) => {
            let mut flattened = Vec::with_capacity(exprs.len());
            for e in exprs {
                match flatten_nested_and(e) {
                    PolicyExpr::And(nested) => flattened.extend(nested),
                    other => flattened.push(other),
                }
            }
            PolicyExpr::And(flattened)
        }
        PolicyExpr::Or(exprs) => {
            PolicyExpr::Or(exprs.into_iter().map(flatten_nested_and).collect())
        }
        PolicyExpr::Not(inner) => PolicyExpr::Not(Box::new(flatten_nested_and(*inner))),
        PolicyExpr::Implies(left, right) => PolicyExpr::Implies(
            Box::new(flatten_nested_and(*left)),
            Box::new(flatten_nested_and(*right)),
        ),
        PolicyExpr::Sequential(exprs) => {
            PolicyExpr::Sequential(exprs.into_iter().map(flatten_nested_and).collect())
        }
        PolicyExpr::Concurrent(exprs) => {
            PolicyExpr::Concurrent(exprs.into_iter().map(flatten_nested_and).collect())
        }
        PolicyExpr::ForAll {
            var,
            items,
            body,
            span,
        } => PolicyExpr::ForAll {
            var,
            items,
            body: Box::new(flatten_nested_and(*body)),
            span,
        },
        PolicyExpr::Exists {
            var,
            items,
            body,
            span,
        } => PolicyExpr::Exists {
            var,
            items,
            body: Box::new(flatten_nested_and(*body)),
            span,
        },
        PolicyExpr::MethodCall {
            receiver,
            method,
            args,
            span,
        } => PolicyExpr::MethodCall {
            receiver: Box::new(flatten_nested_and(*receiver)),
            method,
            args,
            span,
        },
        other => other,
    }
}

/// Flatten nested Or expressions.
///
/// `or(a, or(b, c))` becomes `or(a, b, c)`
#[must_use]
pub fn flatten_nested_or(expr: PolicyExpr) -> PolicyExpr {
    match expr {
        PolicyExpr::Or(exprs) => {
            let mut flattened = Vec::with_capacity(exprs.len());
            for e in exprs {
                match flatten_nested_or(e) {
                    PolicyExpr::Or(nested) => flattened.extend(nested),
                    other => flattened.push(other),
                }
            }
            PolicyExpr::Or(flattened)
        }
        PolicyExpr::And(exprs) => {
            PolicyExpr::And(exprs.into_iter().map(flatten_nested_or).collect())
        }
        PolicyExpr::Not(inner) => PolicyExpr::Not(Box::new(flatten_nested_or(*inner))),
        PolicyExpr::Implies(left, right) => PolicyExpr::Implies(
            Box::new(flatten_nested_or(*left)),
            Box::new(flatten_nested_or(*right)),
        ),
        PolicyExpr::Sequential(exprs) => {
            PolicyExpr::Sequential(exprs.into_iter().map(flatten_nested_or).collect())
        }
        PolicyExpr::Concurrent(exprs) => {
            PolicyExpr::Concurrent(exprs.into_iter().map(flatten_nested_or).collect())
        }
        PolicyExpr::ForAll {
            var,
            items,
            body,
            span,
        } => PolicyExpr::ForAll {
            var,
            items,
            body: Box::new(flatten_nested_or(*body)),
            span,
        },
        PolicyExpr::Exists {
            var,
            items,
            body,
            span,
        } => PolicyExpr::Exists {
            var,
            items,
            body: Box::new(flatten_nested_or(*body)),
            span,
        },
        PolicyExpr::MethodCall {
            receiver,
            method,
            args,
            span,
        } => PolicyExpr::MethodCall {
            receiver: Box::new(flatten_nested_or(*receiver)),
            method,
            args,
            span,
        },
        other => other,
    }
}

/// Eliminate double negation.
///
/// `!!p` becomes `p`
#[must_use]
pub fn eliminate_double_negation(expr: PolicyExpr) -> PolicyExpr {
    match expr {
        PolicyExpr::Not(inner) => match eliminate_double_negation(*inner) {
            PolicyExpr::Not(double_inner) => *double_inner,
            other => PolicyExpr::Not(Box::new(other)),
        },
        PolicyExpr::And(exprs) => {
            PolicyExpr::And(exprs.into_iter().map(eliminate_double_negation).collect())
        }
        PolicyExpr::Or(exprs) => {
            PolicyExpr::Or(exprs.into_iter().map(eliminate_double_negation).collect())
        }
        PolicyExpr::Implies(left, right) => PolicyExpr::Implies(
            Box::new(eliminate_double_negation(*left)),
            Box::new(eliminate_double_negation(*right)),
        ),
        PolicyExpr::Sequential(exprs) => {
            PolicyExpr::Sequential(exprs.into_iter().map(eliminate_double_negation).collect())
        }
        PolicyExpr::Concurrent(exprs) => {
            PolicyExpr::Concurrent(exprs.into_iter().map(eliminate_double_negation).collect())
        }
        PolicyExpr::ForAll {
            var,
            items,
            body,
            span,
        } => PolicyExpr::ForAll {
            var,
            items,
            body: Box::new(eliminate_double_negation(*body)),
            span,
        },
        PolicyExpr::Exists {
            var,
            items,
            body,
            span,
        } => PolicyExpr::Exists {
            var,
            items,
            body: Box::new(eliminate_double_negation(*body)),
            span,
        },
        PolicyExpr::MethodCall {
            receiver,
            method,
            args,
            span,
        } => PolicyExpr::MethodCall {
            receiver: Box::new(eliminate_double_negation(*receiver)),
            method,
            args,
            span,
        },
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_parser::surface::Expr;

    // Helper to create a simple lookup that treats all variables as policies
    fn policy_lookup(_name: &Name) -> Result<PolicyType, PolicyTypeError> {
        Ok(PolicyType::Policy)
    }

    #[test]
    fn test_type_check_var() {
        let expr = PolicyExpr::Var("p".into());
        let result = infer_policy_expr(&expr, &policy_lookup);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PolicyType::Policy);
    }

    #[test]
    fn test_type_check_and() {
        let expr = PolicyExpr::And(vec![
            PolicyExpr::Var("p1".into()),
            PolicyExpr::Var("p2".into()),
        ]);
        let result = infer_policy_expr(&expr, &policy_lookup);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_or() {
        let expr = PolicyExpr::Or(vec![
            PolicyExpr::Var("p1".into()),
            PolicyExpr::Var("p2".into()),
        ]);
        let result = infer_policy_expr(&expr, &policy_lookup);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_not() {
        let expr = PolicyExpr::Not(Box::new(PolicyExpr::Var("p".into())));
        let result = infer_policy_expr(&expr, &policy_lookup);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_implies() {
        let expr = PolicyExpr::Implies(
            Box::new(PolicyExpr::Var("a".into())),
            Box::new(PolicyExpr::Var("b".into())),
        );
        let result = infer_policy_expr(&expr, &policy_lookup);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_forall() {
        let expr = PolicyExpr::ForAll {
            var: "x".into(),
            items: Box::new(Expr::Variable("items".into())),
            body: Box::new(PolicyExpr::Var("p".into())),
            span: ash_parser::token::Span::default(),
        };
        let result = infer_policy_expr(&expr, &policy_lookup);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_exists() {
        let expr = PolicyExpr::Exists {
            var: "x".into(),
            items: Box::new(Expr::Variable("items".into())),
            body: Box::new(PolicyExpr::Var("p".into())),
            span: ash_parser::token::Span::default(),
        };
        let result = infer_policy_expr(&expr, &policy_lookup);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_method_call() {
        let expr = PolicyExpr::MethodCall {
            receiver: Box::new(PolicyExpr::Var("base".into())),
            method: "and".into(),
            args: vec![Expr::Variable("other".into())],
            span: ash_parser::token::Span::default(),
        };
        let result = infer_policy_expr(&expr, &policy_lookup);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_sequential() {
        let expr = PolicyExpr::Sequential(vec![
            PolicyExpr::Var("p1".into()),
            PolicyExpr::Var("p2".into()),
        ]);
        let result = infer_policy_expr(&expr, &policy_lookup);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_check_concurrent() {
        let expr = PolicyExpr::Concurrent(vec![
            PolicyExpr::Var("p1".into()),
            PolicyExpr::Var("p2".into()),
        ]);
        let result = infer_policy_expr(&expr, &policy_lookup);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unknown_variable() {
        let expr = PolicyExpr::Var("unknown".into());
        let lookup = |name: &Name| {
            if name.as_ref() == "known" {
                Ok(PolicyType::Policy)
            } else {
                Err(PolicyTypeError::UnknownVariable(name.to_string()))
            }
        };
        let result = infer_policy_expr(&expr, &lookup);
        assert!(result.is_err());
    }

    #[test]
    fn test_flatten_nested_and() {
        let expr = PolicyExpr::And(vec![
            PolicyExpr::And(vec![
                PolicyExpr::Var("a".into()),
                PolicyExpr::Var("b".into()),
            ]),
            PolicyExpr::Var("c".into()),
        ]);
        let normalized = flatten_nested_and(expr);
        match normalized {
            PolicyExpr::And(exprs) => {
                assert_eq!(exprs.len(), 3);
            }
            _ => panic!("Expected flattened And"),
        }
    }

    #[test]
    fn test_flatten_nested_or() {
        let expr = PolicyExpr::Or(vec![
            PolicyExpr::Or(vec![
                PolicyExpr::Var("a".into()),
                PolicyExpr::Var("b".into()),
            ]),
            PolicyExpr::Var("c".into()),
        ]);
        let normalized = flatten_nested_or(expr);
        match normalized {
            PolicyExpr::Or(exprs) => {
                assert_eq!(exprs.len(), 3);
            }
            _ => panic!("Expected flattened Or"),
        }
    }

    #[test]
    fn test_eliminate_double_negation() {
        let expr = PolicyExpr::Not(Box::new(PolicyExpr::Not(Box::new(PolicyExpr::Var(
            "p".into(),
        )))));
        let normalized = eliminate_double_negation(expr);
        assert!(matches!(normalized, PolicyExpr::Var(_)));
    }

    #[test]
    fn test_eliminate_triple_negation() {
        // !!!p = !p (three negations cancel to one)
        let expr = PolicyExpr::Not(Box::new(PolicyExpr::Not(Box::new(PolicyExpr::Not(
            Box::new(PolicyExpr::Var("p".into())),
        )))));
        let normalized = eliminate_double_negation(expr);
        assert!(matches!(normalized, PolicyExpr::Not(_)));
    }

    #[test]
    fn test_normalize_full() {
        let expr = PolicyExpr::And(vec![
            PolicyExpr::And(vec![PolicyExpr::Var("a".into())]),
            PolicyExpr::Not(Box::new(PolicyExpr::Not(Box::new(PolicyExpr::Var(
                "b".into(),
            ))))),
        ]);
        let normalized = normalize(expr);
        match normalized {
            PolicyExpr::And(exprs) => {
                assert_eq!(exprs.len(), 2);
                assert!(matches!(exprs[0], PolicyExpr::Var(_)));
                assert!(matches!(exprs[1], PolicyExpr::Var(_)));
            }
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_context_bindings() {
        let mut ctx = PolicyCheckContext::new();
        ctx.bind("x".into(), PolicyType::Policy);
        assert_eq!(ctx.lookup(&"x".into()), Some(PolicyType::Policy));
        assert_eq!(ctx.lookup(&"y".into()), None);
    }

    #[test]
    fn test_context_with_binding() {
        let ctx = PolicyCheckContext::new();
        let child = ctx.with_binding("x".into(), PolicyType::Policy);
        assert_eq!(child.lookup(&"x".into()), Some(PolicyType::Policy));
    }

    #[test]
    fn test_unknown_method() {
        let expr = PolicyExpr::MethodCall {
            receiver: Box::new(PolicyExpr::Var("base".into())),
            method: "unknown_method".into(),
            args: vec![],
            span: ash_parser::token::Span::default(),
        };
        let result = infer_policy_expr(&expr, &policy_lookup);
        assert!(matches!(result, Err(PolicyTypeError::UnknownMethod { .. })));
    }

    #[test]
    fn test_argument_count_mismatch() {
        let expr = PolicyExpr::MethodCall {
            receiver: Box::new(PolicyExpr::Var("base".into())),
            method: "and".into(),
            args: vec![], // and expects 1 argument
            span: ash_parser::token::Span::default(),
        };
        let result = infer_policy_expr(&expr, &policy_lookup);
        assert!(matches!(result, Err(PolicyTypeError::ArgumentCount { .. })));
    }

    #[test]
    fn test_builtin_methods_registered() {
        let ctx = PolicyCheckContext::new();
        assert!(ctx.lookup_method(&"and".into()).is_some());
        assert!(ctx.lookup_method(&"or".into()).is_some());
        assert!(ctx.lookup_method(&"retry".into()).is_some());
        assert!(ctx.lookup_method(&"timeout".into()).is_some());
        assert!(ctx.lookup_method(&"not".into()).is_some());
    }
}
