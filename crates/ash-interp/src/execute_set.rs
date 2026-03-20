//! Set statement execution for output behaviours
//!
//! This module provides execution of set statements, enabling workflows to
//! set values on writable channels through behaviour providers.

use ash_core::Value;

use crate::ExecResult;
use crate::behaviour::{BehaviourContext, SettableBehaviourProvider};
use crate::capability_policy::{
    CapabilityOperation, CapabilityPolicyEvaluator, Direction, PolicyDecision, Role,
};
use crate::capability_policy_runtime::{apply_transformation, build_policy_context, check_policy};
use crate::error::ExecError;

/// Execute a set statement
///
/// Sets a value on a writable channel through the behaviour context.
/// The value is validated before being set, and errors are returned
/// if the provider is not available or validation fails.
///
/// # Arguments
///
/// * `capability` - The capability name (e.g., "hvac" in "hvac:target")
/// * `channel` - The channel name (e.g., "target" in "hvac:target")
/// * `value` - The value to set (already evaluated)
/// * `behaviour_ctx` - The behaviour context for provider lookup
///
/// # Returns
///
/// Returns `Ok(())` if the value was successfully set, or an error if:
/// - The provider is not available (`ExecError::CapabilityNotAvailable`)
/// - The value validation fails (`ExecError::ValidationFailed`)
/// - The set operation fails (`ExecError::ExecutionFailed`)
///
/// # Examples
///
/// ```
/// use ash_core::Value;
/// use ash_interp::behaviour::{BehaviourContext, MockSettableProvider, TypedSettableProvider};
/// use ash_interp::execute_set::execute_set;
/// use ash_typeck::Type;
///
/// # tokio_test::block_on(async {
/// let mut ctx = BehaviourContext::new();
/// let provider = MockSettableProvider::new("actuator", "led");
/// ctx.register_settable(TypedSettableProvider::new(provider, Type::Bool));
///
/// let policy_eval = ash_interp::CapabilityPolicyEvaluator::new();
/// let actor = ash_interp::Role::new("system");
/// execute_set("actuator", "led", Value::Bool(true), &ctx, &policy_eval, &actor).await.unwrap();
/// # });
/// ```
pub async fn execute_set(
    capability: &str,
    channel: &str,
    value: Value,
    behaviour_ctx: &BehaviourContext,
    policy_eval: &CapabilityPolicyEvaluator,
    actor: &Role,
) -> ExecResult<()> {
    let policy_ctx = build_policy_context(
        CapabilityOperation::Set,
        Direction::Output,
        capability,
        channel,
        Some(value.clone()),
        &[],
        actor,
    );
    let decision = check_policy(policy_eval, &policy_ctx)?;
    let value = match decision {
        PolicyDecision::Permit => value,
        PolicyDecision::Transform { transformation } => {
            apply_transformation(value, &transformation)
        }
        PolicyDecision::Deny | PolicyDecision::RequireApproval { .. } => unreachable!(),
    };

    // Get settable provider
    let provider = behaviour_ctx
        .get_settable(capability, channel)
        .ok_or_else(|| ExecError::CapabilityNotAvailable(format!("{capability}:{channel}")))?;

    // Validate the value
    provider
        .validate(&value)
        .map_err(|e| ExecError::ValidationFailed(e.to_string()))?;

    // Set the value
    provider.set(value).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::Value;

    use crate::behaviour::{
        BehaviourContext, BehaviourProvider, MockSettableProvider, TypedSettableProvider,
    };
    use ash_typeck::Type;

    #[tokio::test]
    async fn test_set_executes() {
        let mut ctx = BehaviourContext::new();
        let provider = MockSettableProvider::new("hvac", "target");
        ctx.register_settable(TypedSettableProvider::new(provider, Type::Int));

        // Execute set
        let policy_eval = CapabilityPolicyEvaluator::new();
        let actor = Role::new("system");
        execute_set("hvac", "target", Value::Int(72), &ctx, &policy_eval, &actor)
            .await
            .unwrap();

        // Verify provider state changed by retrieving from context
        let provider = ctx.get_settable("hvac", "target").unwrap();
        let current = provider.sample(&[]).await.unwrap();
        assert_eq!(current, Value::Int(72));
    }

    #[tokio::test]
    async fn test_set_changes_value_multiple_times() {
        let mut ctx = BehaviourContext::new();
        let provider = MockSettableProvider::new("actuator", "brightness");
        ctx.register_settable(TypedSettableProvider::new(provider, Type::Int));

        // Set initial value
        let policy_eval = CapabilityPolicyEvaluator::new();
        let actor = Role::new("system");
        execute_set(
            "actuator",
            "brightness",
            Value::Int(50),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await
        .unwrap();
        let provider = ctx.get_settable("actuator", "brightness").unwrap();
        assert_eq!(provider.sample(&[]).await.unwrap(), Value::Int(50));

        // Change to new value
        execute_set(
            "actuator",
            "brightness",
            Value::Int(75),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await
        .unwrap();
        let provider = ctx.get_settable("actuator", "brightness").unwrap();
        assert_eq!(provider.sample(&[]).await.unwrap(), Value::Int(75));

        // Change to another value
        execute_set(
            "actuator",
            "brightness",
            Value::Int(100),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await
        .unwrap();
        let provider = ctx.get_settable("actuator", "brightness").unwrap();
        assert_eq!(provider.sample(&[]).await.unwrap(), Value::Int(100));
    }

    #[tokio::test]
    async fn test_set_missing_provider() {
        let ctx = BehaviourContext::new(); // Empty - no providers registered

        let policy_eval = CapabilityPolicyEvaluator::new();
        let actor = Role::new("system");
        let result = execute_set(
            "nonexistent",
            "channel",
            Value::Int(42),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExecError::CapabilityNotAvailable(name) if name == "nonexistent:channel"
        ));
    }

    #[tokio::test]
    async fn test_set_validation_failure() {
        let mut ctx = BehaviourContext::new();

        // Create provider with custom validation that rejects values > 100
        let provider =
            MockSettableProvider::new("actuator", "brightness").with_validator(|v| match v {
                Value::Int(n) if (0..=100).contains(n) => Ok(()),
                _ => Err(crate::error::ValidationError::InvalidValue(
                    "brightness must be 0-100".to_string(),
                )),
            });

        ctx.register_settable(TypedSettableProvider::new(provider, Type::Int));

        // Valid value should succeed
        let policy_eval = CapabilityPolicyEvaluator::new();
        let actor = Role::new("system");
        execute_set(
            "actuator",
            "brightness",
            Value::Int(50),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await
        .unwrap();

        // Verify value was set
        let provider = ctx.get_settable("actuator", "brightness").unwrap();
        assert_eq!(provider.sample(&[]).await.unwrap(), Value::Int(50));

        // Invalid value should fail
        let result = execute_set(
            "actuator",
            "brightness",
            Value::Int(150),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExecError::ValidationFailed(_)
        ));

        // Verify valid value is still there (set wasn't called with invalid value)
        let provider = ctx.get_settable("actuator", "brightness").unwrap();
        assert_eq!(provider.sample(&[]).await.unwrap(), Value::Int(50));
    }

    #[tokio::test]
    async fn test_set_with_different_types() {
        let mut ctx = BehaviourContext::new();

        // Bool provider
        let bool_provider = MockSettableProvider::new("device", "enabled");
        ctx.register_settable(TypedSettableProvider::new(bool_provider, Type::Bool));

        // String provider
        let string_provider = MockSettableProvider::new("device", "name");
        ctx.register_settable(TypedSettableProvider::new(string_provider, Type::String));

        // Set bool value
        let policy_eval = CapabilityPolicyEvaluator::new();
        let actor = Role::new("system");
        execute_set(
            "device",
            "enabled",
            Value::Bool(true),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await
        .unwrap();
        let bool_provider = ctx.get_settable("device", "enabled").unwrap();
        assert_eq!(bool_provider.sample(&[]).await.unwrap(), Value::Bool(true));

        // Set string value
        execute_set(
            "device",
            "name",
            Value::String("sensor-01".to_string()),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await
        .unwrap();
        let string_provider = ctx.get_settable("device", "name").unwrap();
        assert_eq!(
            string_provider.sample(&[]).await.unwrap(),
            Value::String("sensor-01".to_string())
        );
    }
}
