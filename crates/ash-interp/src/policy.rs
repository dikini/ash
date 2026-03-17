//! Policy evaluation for runtime permission and obligation checks
//!
//! Policies control what workflows are allowed to do based on runtime conditions.

use ash_core::{Decision, Expr, Name, Value};
use std::collections::HashMap;

use crate::ExecResult;
use crate::context::Context;
use crate::error::ExecError;
use crate::eval::eval_expr;

/// A policy rule with a condition and decision
#[derive(Debug, Clone)]
pub struct PolicyRule {
    /// Human-readable description of the rule
    pub description: String,
    /// Condition expression - if true, this rule applies
    pub condition: Expr,
    /// Decision when condition is true
    pub decision: Decision,
}

impl PolicyRule {
    /// Create a new policy rule
    pub fn new(description: &str, condition: Expr, decision: Decision) -> Self {
        Self {
            description: description.to_string(),
            condition,
            decision,
        }
    }
}

/// A policy is a named set of rules evaluated in order
#[derive(Debug, Clone)]
pub struct Policy {
    pub name: Name,
    pub rules: Vec<PolicyRule>,
    /// Default decision if no rules match
    pub default: Decision,
}

impl Policy {
    /// Create a new policy
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            rules: Vec::new(),
            default: Decision::Deny,
        }
    }

    /// Add a rule to the policy
    pub fn with_rule(mut self, rule: PolicyRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Set the default decision
    pub fn with_default(mut self, decision: Decision) -> Self {
        self.default = decision;
        self
    }
}

/// Evaluator for policies
pub struct PolicyEvaluator {
    policies: HashMap<Name, Policy>,
}

impl PolicyEvaluator {
    /// Create a new policy evaluator
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }

    /// Register a policy
    pub fn register(&mut self, policy: Policy) {
        self.policies.insert(policy.name.clone(), policy);
    }

    /// Get a policy by name
    pub fn get(&self, name: &str) -> Option<&Policy> {
        self.policies.get(name)
    }

    /// Evaluate a policy in the given context
    ///
    /// Rules are evaluated in order; the first matching rule's decision is returned.
    /// If no rules match, the default decision is returned.
    pub fn evaluate(&self, policy_name: &str, ctx: &Context) -> ExecResult<Decision> {
        let policy = self.policies.get(policy_name).ok_or_else(|| {
            ExecError::ExecutionFailed(format!("policy '{}' not found", policy_name))
        })?;

        for rule in &policy.rules {
            match eval_expr(&rule.condition, ctx) {
                Ok(Value::Bool(true)) => return Ok(rule.decision),
                Ok(Value::Bool(false)) => continue,
                Ok(other) => {
                    return Err(ExecError::Eval(crate::error::EvalError::TypeMismatch {
                        expected: "bool".to_string(),
                        actual: format!("{:?}", other),
                    }));
                }
                Err(e) => return Err(ExecError::Eval(e)),
            }
        }

        Ok(policy.default)
    }

    /// Check if a decision permits the action
    pub fn is_permitted(&self, decision: Decision) -> bool {
        matches!(decision, Decision::Permit)
    }

    /// Check if a decision requires escalation
    pub fn requires_escalation(&self, decision: Decision) -> bool {
        matches!(decision, Decision::Escalate | Decision::RequireApproval)
    }
}

impl Default for PolicyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple policy check result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyResult {
    Allow,
    Deny,
    Escalate,
}

impl From<Decision> for PolicyResult {
    fn from(d: Decision) -> Self {
        match d {
            Decision::Permit => PolicyResult::Allow,
            Decision::Deny => PolicyResult::Deny,
            Decision::RequireApproval | Decision::Escalate => PolicyResult::Escalate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::{BinaryOp, Expr};

    #[test]
    fn test_policy_evaluation_permit() {
        let mut evaluator = PolicyEvaluator::new();

        // Policy: if x > 10, permit
        let policy = Policy::new("test")
            .with_rule(PolicyRule::new(
                "allow large values",
                Expr::Binary {
                    op: BinaryOp::Gt,
                    left: Box::new(Expr::Variable("x".to_string())),
                    right: Box::new(Expr::Literal(Value::Int(10))),
                },
                Decision::Permit,
            ))
            .with_default(Decision::Deny);

        evaluator.register(policy);

        // Test with x = 15 (should permit)
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Int(15));
        let decision = evaluator.evaluate("test", &ctx).unwrap();
        assert_eq!(decision, Decision::Permit);

        // Test with x = 5 (should deny - default)
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Int(5));
        let decision = evaluator.evaluate("test", &ctx).unwrap();
        assert_eq!(decision, Decision::Deny);
    }

    #[test]
    fn test_policy_evaluation_multiple_rules() {
        let mut evaluator = PolicyEvaluator::new();

        // Policy: if x < 0, deny; if x > 100, escalate; otherwise permit
        let policy = Policy::new("bounded")
            .with_rule(PolicyRule::new(
                "deny negative",
                Expr::Binary {
                    op: BinaryOp::Lt,
                    left: Box::new(Expr::Variable("x".to_string())),
                    right: Box::new(Expr::Literal(Value::Int(0))),
                },
                Decision::Deny,
            ))
            .with_rule(PolicyRule::new(
                "escalate large",
                Expr::Binary {
                    op: BinaryOp::Gt,
                    left: Box::new(Expr::Variable("x".to_string())),
                    right: Box::new(Expr::Literal(Value::Int(100))),
                },
                Decision::Escalate,
            ))
            .with_default(Decision::Permit);

        evaluator.register(policy);

        // Test negative (should deny - first rule)
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Int(-5));
        let decision = evaluator.evaluate("bounded", &ctx).unwrap();
        assert_eq!(decision, Decision::Deny);

        // Test large (should escalate)
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Int(150));
        let decision = evaluator.evaluate("bounded", &ctx).unwrap();
        assert_eq!(decision, Decision::Escalate);

        // Test normal (should permit - default)
        let mut ctx = Context::new();
        ctx.set("x".to_string(), Value::Int(50));
        let decision = evaluator.evaluate("bounded", &ctx).unwrap();
        assert_eq!(decision, Decision::Permit);
    }

    #[test]
    fn test_policy_not_found() {
        let evaluator = PolicyEvaluator::new();
        let ctx = Context::new();
        assert!(evaluator.evaluate("missing", &ctx).is_err());
    }

    #[test]
    fn test_policy_result_conversion() {
        assert_eq!(PolicyResult::from(Decision::Permit), PolicyResult::Allow);
        assert_eq!(PolicyResult::from(Decision::Deny), PolicyResult::Deny);
        assert_eq!(
            PolicyResult::from(Decision::Escalate),
            PolicyResult::Escalate
        );
        assert_eq!(
            PolicyResult::from(Decision::RequireApproval),
            PolicyResult::Escalate
        );
    }

    #[test]
    fn test_policy_evaluator_helpers() {
        let evaluator = PolicyEvaluator::new();

        assert!(evaluator.is_permitted(Decision::Permit));
        assert!(!evaluator.is_permitted(Decision::Deny));

        assert!(evaluator.requires_escalation(Decision::Escalate));
        assert!(evaluator.requires_escalation(Decision::RequireApproval));
        assert!(!evaluator.requires_escalation(Decision::Permit));
    }

    #[test]
    fn test_policy_non_boolean_condition() {
        let mut evaluator = PolicyEvaluator::new();

        // Policy with non-boolean condition (returns int instead of bool)
        let policy = Policy::new("bad").with_rule(PolicyRule::new(
            "always returns int",
            Expr::Literal(Value::Int(42)),
            Decision::Permit,
        ));

        evaluator.register(policy);

        let ctx = Context::new();
        assert!(evaluator.evaluate("bad", &ctx).is_err());
    }
}
