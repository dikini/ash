//! Shared runtime handling for capability-level policy outcomes.

use ash_core::{Constraint, Name, Value};
use chrono::Utc;

use crate::capability_policy::{
    CapabilityContext, CapabilityOperation, CapabilityPolicyEvaluator, Direction, PolicyDecision,
    Role, Transformation,
};
use crate::error::{ExecError, ExecResult};

/// Build a capability-policy evaluation context for a single operation.
pub(crate) fn build_policy_context(
    operation: CapabilityOperation,
    direction: Direction,
    capability: &str,
    channel: &str,
    value: Option<Value>,
    constraints: &[Constraint],
    actor: &Role,
) -> CapabilityContext {
    CapabilityContext {
        operation,
        direction,
        capability: Name::from(capability),
        channel: Name::from(channel),
        value,
        constraints: constraints.to_vec(),
        actor: actor.clone(),
        timestamp: Utc::now(),
    }
}

/// Evaluate capability policy and convert blocking outcomes into runtime errors.
pub(crate) fn check_policy(
    policy_eval: &CapabilityPolicyEvaluator,
    policy_ctx: &CapabilityContext,
) -> ExecResult<PolicyDecision> {
    let decision = policy_eval
        .evaluate(policy_ctx)
        .map_err(|err| ExecError::ExecutionFailed(err.to_string()))?;

    match decision {
        PolicyDecision::Permit | PolicyDecision::Transform { .. } => Ok(decision),
        PolicyDecision::Deny => Err(ExecError::PolicyDenied {
            policy: format!("{}:{}", policy_ctx.capability, policy_ctx.channel),
        }),
        PolicyDecision::RequireApproval { role } => Err(ExecError::RequiresApproval {
            role,
            operation: policy_ctx.operation.operation_name().to_string(),
            capability: format!("{}:{}", policy_ctx.capability, policy_ctx.channel),
        }),
    }
}

/// Apply a transformation to a runtime value.
pub(crate) fn apply_transformation(value: Value, transformation: &Transformation) -> Value {
    match transformation {
        Transformation::Permit => value,
        Transformation::Mask { fields } => mask_fields(value, fields),
        Transformation::Filter => Value::Null,
        Transformation::Replace { value } => value.clone(),
    }
}

fn mask_fields(mut value: Value, fields: &[Name]) -> Value {
    if let Value::Record(record) = &mut value {
        for field in fields {
            if let Some(entry) = record.get_mut(field.as_str()) {
                *entry = Value::Null;
            }
        }
    }

    value
}

trait CapabilityOperationExt {
    fn operation_name(self) -> &'static str;
}

impl CapabilityOperationExt for CapabilityOperation {
    fn operation_name(self) -> &'static str {
        match self {
            CapabilityOperation::Observe => "observe",
            CapabilityOperation::Receive => "receive",
            CapabilityOperation::Set => "set",
            CapabilityOperation::Send => "send",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability_policy::{Policy, Role};

    #[test]
    fn require_approval_preserves_the_explicit_named_role() {
        let mut eval = CapabilityPolicyEvaluator::new();
        eval.add_output_policy(Policy {
            capability_pattern: "repo:merge".into(),
            condition: Box::new(|_| true),
            decision: Box::new(|_| PolicyDecision::RequireApproval {
                role: Role::new("senior_reviewer"),
            }),
        });

        let actor = Role::new("reviewer_delegate");
        let ctx = build_policy_context(
            CapabilityOperation::Set,
            Direction::Output,
            "repo",
            "merge",
            Some(Value::Bool(true)),
            &[],
            &actor,
        );

        let error = check_policy(&eval, &ctx).expect_err("approval should be surfaced");

        assert_eq!(ctx.actor, actor);
        assert_ne!(ctx.actor.as_ref(), "senior_reviewer");
        assert_eq!(
            error,
            ExecError::RequiresApproval {
                role: Role::new("senior_reviewer"),
                operation: "set".into(),
                capability: "repo:merge".into(),
            }
        );
    }
}
