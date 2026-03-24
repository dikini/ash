//! Policy evaluation for Input/Output capability operations
//!
//! This module provides policy evaluation specifically for capability operations,
//! supporting both input (observe/receive) and output (set/send) directions.
//!
//! # Example
//!
//! ```
//! use ash_interp::capability_policy::{
//!     CapabilityPolicyEvaluator, CapabilityContext, Direction, CapabilityOperation,
//!     PolicyDecision, Policy, Role
//! };
//! use ash_core::{Name, Value};
//! use chrono::Utc;
//!
//! let mut evaluator = CapabilityPolicyEvaluator::new();
//! evaluator.add_input_policy(Policy {
//!     capability_pattern: "sensor:temp".into(),
//!     condition: Box::new(|_| true),
//!     decision: Box::new(|_| PolicyDecision::Permit),
//! });
//!
//! let ctx = CapabilityContext {
//!     operation: CapabilityOperation::Observe,
//!     direction: Direction::Input,
//!     capability: "sensor".into(),
//!     channel: "temp".into(),
//!     value: None,
//!     constraints: vec![],
//!     actor: Role::new("admin"),
//!     timestamp: Utc::now(),
//! };
//!
//! let decision = evaluator.evaluate(&ctx).unwrap();
//! assert_eq!(decision, PolicyDecision::Permit);
//! ```

use ash_core::{Constraint, Name, Value};
use chrono::{DateTime, Utc};

/// Direction of capability operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Input operations: observe, receive
    Input,
    /// Output operations: set, send
    Output,
}

/// Type of capability operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityOperation {
    /// Read/observe a value
    Observe,
    /// Receive a message
    Receive,
    /// Set a value
    Set,
    /// Send a message
    Send,
}

/// Context for policy evaluation
#[derive(Debug, Clone)]
pub struct CapabilityContext {
    /// The operation being performed
    pub operation: CapabilityOperation,
    /// Direction of the operation
    pub direction: Direction,
    /// Capability name
    pub capability: Name,
    /// Channel name
    pub channel: Name,
    /// Value being set/sent (for output operations)
    pub value: Option<Value>,
    /// Query constraints (for input operations)
    pub constraints: Vec<Constraint>,
    /// Actor performing the operation
    pub actor: Role,
    /// Timestamp of the operation
    pub timestamp: DateTime<Utc>,
}

/// Policy decision for capability operations
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyDecision {
    /// Allow the operation
    Permit,
    /// Deny the operation
    Deny,
    /// Require approval from a specific role
    RequireApproval { role: Role },
    /// Transform the value before proceeding
    Transform { transformation: Transformation },
}

/// Value transformation
#[derive(Debug, Clone, PartialEq)]
pub enum Transformation {
    /// No transformation, proceed as-is
    Permit,
    /// Mask specific fields in a record
    Mask { fields: Vec<Name> },
    /// Filter the value
    Filter,
    /// Replace with a different value
    Replace { value: Value },
}

/// Flat named role used for policy evaluation and approval routing.
///
/// This runtime role carrier is intentionally just a named identity. It does not encode
/// supervision, hierarchy, or inherited authority semantics.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Role(pub String);

impl Role {
    /// Create a new role
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl AsRef<str> for Role {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for Role {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Role {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Policy evaluator for capability operations
pub struct CapabilityPolicyEvaluator {
    input_policies: Vec<Policy>,
    output_policies: Vec<Policy>,
}

impl CapabilityPolicyEvaluator {
    /// Create a new policy evaluator with no policies
    pub fn new() -> Self {
        Self {
            input_policies: vec![],
            output_policies: vec![],
        }
    }

    /// Add an input policy
    pub fn add_input_policy(&mut self, policy: Policy) {
        self.input_policies.push(policy);
    }

    /// Add an output policy
    pub fn add_output_policy(&mut self, policy: Policy) {
        self.output_policies.push(policy);
    }

    /// Evaluate policy for a capability operation
    ///
    /// Returns the first matching policy's decision, or `Permit` if no policies match.
    pub fn evaluate(&self, ctx: &CapabilityContext) -> Result<PolicyDecision, PolicyError> {
        match ctx.direction {
            Direction::Input => self.evaluate_input(ctx),
            Direction::Output => self.evaluate_output(ctx),
        }
    }

    fn evaluate_input(&self, ctx: &CapabilityContext) -> Result<PolicyDecision, PolicyError> {
        for policy in &self.input_policies {
            if policy.matches(ctx) {
                return policy.decision(ctx);
            }
        }
        Ok(PolicyDecision::Permit)
    }

    fn evaluate_output(&self, ctx: &CapabilityContext) -> Result<PolicyDecision, PolicyError> {
        for policy in &self.output_policies {
            if policy.matches(ctx) {
                return policy.decision(ctx);
            }
        }
        Ok(PolicyDecision::Permit)
    }
}

impl Default for CapabilityPolicyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple policy representation
pub struct Policy {
    /// Pattern to match capability:channel (e.g., "sensor:temp")
    pub capability_pattern: String,
    /// Condition function that must return true for policy to apply
    pub condition: Box<dyn Fn(&CapabilityContext) -> bool + Send + Sync>,
    /// Decision function that returns the policy decision
    pub decision: Box<dyn Fn(&CapabilityContext) -> PolicyDecision + Send + Sync>,
}

impl std::fmt::Debug for Policy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Policy")
            .field("capability_pattern", &self.capability_pattern)
            .finish_non_exhaustive()
    }
}

impl Policy {
    /// Check if this policy matches the given context
    pub fn matches(&self, ctx: &CapabilityContext) -> bool {
        let cap_channel = format!("{}:{}", ctx.capability, ctx.channel);
        cap_channel.starts_with(&self.capability_pattern) && (self.condition)(ctx)
    }

    /// Get the decision for this policy
    pub fn decision(&self, ctx: &CapabilityContext) -> Result<PolicyDecision, PolicyError> {
        Ok((self.decision)(ctx))
    }
}

/// Errors that can occur during policy evaluation
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    /// Policy evaluation failed
    #[error("policy evaluation failed: {0}")]
    EvaluationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_policy_permit() {
        let mut eval = CapabilityPolicyEvaluator::new();
        eval.add_input_policy(Policy {
            capability_pattern: "sensor:temp".into(),
            condition: Box::new(|_| true),
            decision: Box::new(|_| PolicyDecision::Permit),
        });

        let ctx = CapabilityContext {
            operation: CapabilityOperation::Observe,
            direction: Direction::Input,
            capability: "sensor".into(),
            channel: "temp".into(),
            value: None,
            constraints: vec![],
            actor: Role::new("admin"),
            timestamp: Utc::now(),
        };

        let decision = eval.evaluate(&ctx).unwrap();
        assert_eq!(decision, PolicyDecision::Permit);
    }

    #[test]
    fn test_output_policy_deny() {
        let mut eval = CapabilityPolicyEvaluator::new();
        eval.add_output_policy(Policy {
            capability_pattern: "hvac:target".into(),
            condition: Box::new(|ctx| {
                if let Some(Value::Int(temp)) = &ctx.value {
                    *temp > 100
                } else {
                    false
                }
            }),
            decision: Box::new(|_| PolicyDecision::Deny),
        });

        let ctx = CapabilityContext {
            operation: CapabilityOperation::Set,
            direction: Direction::Output,
            capability: "hvac".into(),
            channel: "target".into(),
            value: Some(Value::Int(150)),
            constraints: vec![],
            actor: Role::new("operator"),
            timestamp: Utc::now(),
        };

        let decision = eval.evaluate(&ctx).unwrap();
        assert_eq!(decision, PolicyDecision::Deny);
    }

    #[test]
    fn test_default_permit() {
        let eval = CapabilityPolicyEvaluator::new();

        let ctx = CapabilityContext {
            operation: CapabilityOperation::Observe,
            direction: Direction::Input,
            capability: "sensor".into(),
            channel: "temp".into(),
            value: None,
            constraints: vec![],
            actor: Role::new("user"),
            timestamp: Utc::now(),
        };

        let decision = eval.evaluate(&ctx).unwrap();
        assert_eq!(decision, PolicyDecision::Permit);
    }

    #[test]
    fn test_policy_condition_not_met() {
        let mut eval = CapabilityPolicyEvaluator::new();
        eval.add_output_policy(Policy {
            capability_pattern: "hvac:target".into(),
            condition: Box::new(|ctx| {
                // Only match if temp > 100
                if let Some(Value::Int(temp)) = &ctx.value {
                    *temp > 100
                } else {
                    false
                }
            }),
            decision: Box::new(|_| PolicyDecision::Deny),
        });

        // Value is within acceptable range, condition not met
        let ctx = CapabilityContext {
            operation: CapabilityOperation::Set,
            direction: Direction::Output,
            capability: "hvac".into(),
            channel: "target".into(),
            value: Some(Value::Int(75)),
            constraints: vec![],
            actor: Role::new("operator"),
            timestamp: Utc::now(),
        };

        // Should default to Permit since condition not met
        let decision = eval.evaluate(&ctx).unwrap();
        assert_eq!(decision, PolicyDecision::Permit);
    }

    #[test]
    fn test_capability_pattern_prefix_match() {
        let mut eval = CapabilityPolicyEvaluator::new();
        eval.add_input_policy(Policy {
            capability_pattern: "sensor".into(),
            condition: Box::new(|_| true),
            decision: Box::new(|_| PolicyDecision::Deny),
        });

        let ctx = CapabilityContext {
            operation: CapabilityOperation::Observe,
            direction: Direction::Input,
            capability: "sensor".into(),
            channel: "humidity".into(),
            value: None,
            constraints: vec![],
            actor: Role::new("user"),
            timestamp: Utc::now(),
        };

        let decision = eval.evaluate(&ctx).unwrap();
        assert_eq!(decision, PolicyDecision::Deny);
    }

    #[test]
    fn test_require_approval_decision() {
        let mut eval = CapabilityPolicyEvaluator::new();
        eval.add_output_policy(Policy {
            capability_pattern: "security:door".into(),
            condition: Box::new(|_| true),
            decision: Box::new(|_| PolicyDecision::RequireApproval {
                role: Role::new("security_admin"),
            }),
        });

        let ctx = CapabilityContext {
            operation: CapabilityOperation::Set,
            direction: Direction::Output,
            capability: "security".into(),
            channel: "door".into(),
            value: Some(Value::String("unlock".into())),
            constraints: vec![],
            actor: Role::new("user"),
            timestamp: Utc::now(),
        };

        let decision = eval.evaluate(&ctx).unwrap();
        assert_eq!(
            decision,
            PolicyDecision::RequireApproval {
                role: Role::new("security_admin")
            }
        );
    }

    #[test]
    fn test_transform_decision() {
        let mut eval = CapabilityPolicyEvaluator::new();
        eval.add_input_policy(Policy {
            capability_pattern: "user:data".into(),
            condition: Box::new(|_| true),
            decision: Box::new(|_| PolicyDecision::Transform {
                transformation: Transformation::Mask {
                    fields: vec!["ssn".into(), "dob".into()],
                },
            }),
        });

        let ctx = CapabilityContext {
            operation: CapabilityOperation::Observe,
            direction: Direction::Input,
            capability: "user".into(),
            channel: "data".into(),
            value: None,
            constraints: vec![],
            actor: Role::new("analyst"),
            timestamp: Utc::now(),
        };

        let decision = eval.evaluate(&ctx).unwrap();
        assert_eq!(
            decision,
            PolicyDecision::Transform {
                transformation: Transformation::Mask {
                    fields: vec!["ssn".into(), "dob".into()]
                }
            }
        );
    }

    #[test]
    fn test_receive_operation() {
        let mut eval = CapabilityPolicyEvaluator::new();
        eval.add_input_policy(Policy {
            capability_pattern: "queue:messages".into(),
            condition: Box::new(|_| true),
            decision: Box::new(|_| PolicyDecision::Permit),
        });

        let ctx = CapabilityContext {
            operation: CapabilityOperation::Receive,
            direction: Direction::Input,
            capability: "queue".into(),
            channel: "messages".into(),
            value: None,
            constraints: vec![],
            actor: Role::new("worker"),
            timestamp: Utc::now(),
        };

        let decision = eval.evaluate(&ctx).unwrap();
        assert_eq!(decision, PolicyDecision::Permit);
    }

    #[test]
    fn test_send_operation() {
        let mut eval = CapabilityPolicyEvaluator::new();
        eval.add_output_policy(Policy {
            capability_pattern: "notification:alert".into(),
            condition: Box::new(|_| true),
            decision: Box::new(|_| PolicyDecision::Permit),
        });

        let ctx = CapabilityContext {
            operation: CapabilityOperation::Send,
            direction: Direction::Output,
            capability: "notification".into(),
            channel: "alert".into(),
            value: Some(Value::String("system alert".into())),
            constraints: vec![],
            actor: Role::new("system"),
            timestamp: Utc::now(),
        };

        let decision = eval.evaluate(&ctx).unwrap();
        assert_eq!(decision, PolicyDecision::Permit);
    }

    #[test]
    fn test_role_conversions() {
        let role1 = Role::new("admin");
        assert_eq!(role1.as_ref(), "admin");

        let role2 = Role::from("user".to_string());
        assert_eq!(role2.as_ref(), "user");

        let role3: Role = "guest".into();
        assert_eq!(role3.as_ref(), "guest");
    }

    #[test]
    fn test_transformation_variants() {
        let t1 = Transformation::Permit;
        assert_eq!(t1, Transformation::Permit);

        let t2 = Transformation::Mask {
            fields: vec!["field1".into(), "field2".into()],
        };
        assert_eq!(
            t2,
            Transformation::Mask {
                fields: vec!["field1".into(), "field2".into()]
            }
        );

        let t3 = Transformation::Filter;
        assert_eq!(t3, Transformation::Filter);

        let t4 = Transformation::Replace {
            value: Value::Int(42),
        };
        assert_eq!(
            t4,
            Transformation::Replace {
                value: Value::Int(42)
            }
        );
    }

    #[test]
    fn test_policy_debug() {
        let policy = Policy {
            capability_pattern: "test:pattern".into(),
            condition: Box::new(|_| true),
            decision: Box::new(|_| PolicyDecision::Permit),
        };

        let debug_str = format!("{:?}", policy);
        assert!(debug_str.contains("Policy"));
        assert!(debug_str.contains("test:pattern"));
    }
}
