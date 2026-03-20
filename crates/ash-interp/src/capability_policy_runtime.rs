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
