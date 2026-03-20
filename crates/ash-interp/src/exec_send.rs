//! Send statement execution for output streams
//!
//! This module provides execution of send statements, enabling workflows to
//! send values to output streams through sendable stream providers.

use std::time::Duration;

use ash_core::Value;

use crate::ExecResult;
use crate::capability_policy::{
    CapabilityOperation, CapabilityPolicyEvaluator, Direction, PolicyDecision, Role,
};
use crate::capability_policy_runtime::{apply_transformation, build_policy_context, check_policy};
use crate::error::ExecError;
use crate::stream::{SendableStreamProvider, StreamContext};

/// Execute a send statement
///
/// Sends a value to an output stream through the stream context.
/// Handles backpressure by checking `would_block()` and waiting with a timeout.
///
/// # Arguments
///
/// * `capability` - The capability name (e.g., "queue" in "queue:output")
/// * `channel` - The channel name (e.g., "output" in "queue:output")
/// * `value` - The value to send (already evaluated)
/// * `stream_ctx` - The stream context for provider lookup
///
/// # Returns
///
/// Returns `Ok(())` if the value was successfully sent, or an error if:
/// - The provider is not available (`ExecError::CapabilityNotAvailable`)
/// - The send would block and times out (`ExecError::ExecutionFailed`)
/// - The send operation fails (`ExecError::ExecutionFailed`)
///
/// # Examples
///
/// ```
/// use ash_core::Value;
/// use ash_interp::stream::{StreamContext, MockSendableProvider, TypedSendableProvider};
/// use ash_interp::exec_send::execute_send;
/// use ash_typeck::Type;
///
/// # tokio_test::block_on(async {
/// let mut ctx = StreamContext::new();
/// let provider = MockSendableProvider::new("queue", "output");
/// ctx.register_sendable(TypedSendableProvider::new(provider, Type::Int));
///
/// let policy_eval = ash_interp::CapabilityPolicyEvaluator::new();
/// let actor = ash_interp::Role::new("system");
/// execute_send("queue", "output", Value::Int(42), &ctx, &policy_eval, &actor).await.unwrap();
/// # });
/// ```
pub async fn execute_send(
    capability: &str,
    channel: &str,
    value: Value,
    stream_ctx: &StreamContext,
    policy_eval: &CapabilityPolicyEvaluator,
    actor: &Role,
) -> ExecResult<()> {
    let policy_ctx = build_policy_context(
        CapabilityOperation::Send,
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

    // Get sendable provider
    let provider = stream_ctx
        .get_sendable(capability, channel)
        .ok_or_else(|| ExecError::CapabilityNotAvailable(format!("{capability}:{channel}")))?;

    // Optional: wait if would_block (with timeout)
    let mut attempts = 0;
    while provider.would_block() && attempts < 100 {
        tokio::time::sleep(Duration::from_millis(10)).await;
        attempts += 1;
    }

    if provider.would_block() {
        return Err(ExecError::ExecutionFailed("send would block".to_string()));
    }

    // Send value
    provider.send(value).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::Value;
    use ash_typeck::Type;

    use crate::stream::{MockSendableProvider, StreamContext, TypedSendableProvider};

    #[tokio::test]
    async fn test_send_executes() {
        let mut ctx = StreamContext::new();
        let provider = MockSendableProvider::new("queue", "output");
        ctx.register_sendable(TypedSendableProvider::new(provider.clone(), Type::Int));

        // Execute send
        let policy_eval = CapabilityPolicyEvaluator::new();
        let actor = Role::new("system");
        execute_send(
            "queue",
            "output",
            Value::Int(42),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await
        .unwrap();

        // Verify value was sent
        assert_eq!(provider.sent_count(), 1);
        assert_eq!(provider.sent_values()[0], Value::Int(42));
    }

    #[tokio::test]
    async fn test_send_missing_provider() {
        let ctx = StreamContext::new(); // Empty - no providers registered

        let policy_eval = CapabilityPolicyEvaluator::new();
        let actor = Role::new("system");
        let result = execute_send(
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
    async fn test_send_would_block() {
        let mut ctx = StreamContext::new();
        let provider = MockSendableProvider::new("queue", "output");

        // Set provider to block
        provider.set_would_block(true);

        ctx.register_sendable(TypedSendableProvider::new(provider, Type::Int));

        // Execute send - should fail after timeout
        let policy_eval = CapabilityPolicyEvaluator::new();
        let actor = Role::new("system");
        let result = execute_send(
            "queue",
            "output",
            Value::Int(42),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExecError::ExecutionFailed(msg) if msg == "send would block"
        ));
    }

    #[tokio::test]
    async fn test_send_with_different_types() {
        let mut ctx = StreamContext::new();

        // Int provider
        let int_provider = MockSendableProvider::new("queue", "numbers");
        ctx.register_sendable(TypedSendableProvider::new(int_provider.clone(), Type::Int));

        // String provider
        let string_provider = MockSendableProvider::new("queue", "messages");
        ctx.register_sendable(TypedSendableProvider::new(
            string_provider.clone(),
            Type::String,
        ));

        // Send int value
        let policy_eval = CapabilityPolicyEvaluator::new();
        let actor = Role::new("system");
        execute_send(
            "queue",
            "numbers",
            Value::Int(100),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await
        .unwrap();
        assert_eq!(int_provider.sent_count(), 1);
        assert_eq!(int_provider.sent_values()[0], Value::Int(100));

        // Send string value
        execute_send(
            "queue",
            "messages",
            Value::String("hello".to_string()),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await
        .unwrap();
        assert_eq!(string_provider.sent_count(), 1);
        assert_eq!(
            string_provider.sent_values()[0],
            Value::String("hello".to_string())
        );
    }

    #[tokio::test]
    async fn test_send_type_validation() {
        let mut ctx = StreamContext::new();
        let provider = MockSendableProvider::new("queue", "output");
        ctx.register_sendable(TypedSendableProvider::new(provider, Type::Int));

        // Valid value should succeed
        let policy_eval = CapabilityPolicyEvaluator::new();
        let actor = Role::new("system");
        execute_send(
            "queue",
            "output",
            Value::Int(42),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await
        .unwrap();

        // Invalid type should fail
        let result = execute_send(
            "queue",
            "output",
            Value::String("not an int".to_string()),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("type mismatch"));
    }

    #[tokio::test]
    async fn test_send_multiple_values() {
        let mut ctx = StreamContext::new();
        let provider = MockSendableProvider::new("queue", "output");
        ctx.register_sendable(TypedSendableProvider::new(provider.clone(), Type::Int));

        // Send multiple values
        let policy_eval = CapabilityPolicyEvaluator::new();
        let actor = Role::new("system");
        execute_send("queue", "output", Value::Int(1), &ctx, &policy_eval, &actor)
            .await
            .unwrap();
        execute_send("queue", "output", Value::Int(2), &ctx, &policy_eval, &actor)
            .await
            .unwrap();
        execute_send("queue", "output", Value::Int(3), &ctx, &policy_eval, &actor)
            .await
            .unwrap();

        // Verify all values were sent in order
        let sent = provider.sent_values();
        assert_eq!(sent.len(), 3);
        assert_eq!(sent[0], Value::Int(1));
        assert_eq!(sent[1], Value::Int(2));
        assert_eq!(sent[2], Value::Int(3));
    }

    #[tokio::test]
    async fn test_send_would_block_then_succeeds() {
        let mut ctx = StreamContext::new();
        let provider = MockSendableProvider::new("queue", "output");

        // Set provider to block initially
        provider.set_would_block(true);

        ctx.register_sendable(TypedSendableProvider::new(provider.clone(), Type::Int));

        // Start a task that will unblock after a short delay
        let unblock_handle = {
            let provider = provider.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                provider.set_would_block(false);
            })
        };

        // Execute send - should wait and then succeed
        let policy_eval = CapabilityPolicyEvaluator::new();
        let actor = Role::new("system");
        execute_send(
            "queue",
            "output",
            Value::Int(42),
            &ctx,
            &policy_eval,
            &actor,
        )
        .await
        .unwrap();

        // Wait for the unblock task to complete
        unblock_handle.await.unwrap();

        // Verify value was sent
        assert_eq!(provider.sent_count(), 1);
        assert_eq!(provider.sent_values()[0], Value::Int(42));
    }
}
