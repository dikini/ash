//! Observe execution with sampling and pattern binding
//!
//! This module provides execution of observe and changed constructs,
//! integrating behaviour sampling with pattern matching.

use ash_core::{Changed, Observe, Value};

use crate::ExecResult;
use crate::behaviour::BehaviourContext;
use crate::context::Context;
use crate::error::ExecError;
use crate::pattern::match_pattern;

/// Execute an observe statement
///
/// Samples the behaviour using `behaviour_ctx.sample()`, applies constraints
/// during sampling, matches the pattern against the sampled value, and
/// binds variables from the pattern match.
///
/// # Arguments
///
/// * `observe` - The observe expression to execute
/// * `ctx` - The current execution context
/// * `behaviour_ctx` - The behaviour context for sampling
///
/// # Returns
///
/// A new context with the bound variables, or an error if:
/// - The provider is not available
/// - The pattern match fails
///
/// # Errors
///
/// Returns `ExecError::CapabilityNotAvailable` if no provider is found for
/// the specified capability and channel.
/// Returns `ExecError::PatternMatchFailed` if the pattern cannot match the value.
///
/// # Examples
///
/// ```
/// use ash_core::{Observe, Pattern, Value};
/// use ash_interp::behaviour::{BehaviourContext, MockBehaviourProvider};
/// use ash_interp::context::Context;
/// use ash_interp::execute_observe::execute_observe;
/// use ash_interp::typed_provider::TypedBehaviourProvider;
/// use ash_typeck::Type;
///
/// # tokio_test::block_on(async {
/// let mut behaviour_ctx = BehaviourContext::new();
/// behaviour_ctx.register(TypedBehaviourProvider::new(
///     MockBehaviourProvider::new("sensor", "temp")
///         .with_value(Value::Int(42)),
///     Type::Int,
/// ));
///
/// let observe = Observe {
///     capability: "sensor".into(),
///     channel: "temp".into(),
///     constraints: vec![],
///     pattern: Pattern::Variable("t".into()),
/// };
///
/// let ctx = Context::new();
/// let new_ctx = execute_observe(&observe, ctx, &behaviour_ctx).await.unwrap();
/// assert_eq!(new_ctx.get("t"), Some(&Value::Int(42)));
/// # });
/// ```
pub async fn execute_observe(
    observe: &Observe,
    ctx: Context,
    behaviour_ctx: &BehaviourContext,
) -> ExecResult<Context> {
    // Sample the behaviour
    let value = behaviour_ctx
        .sample(&observe.capability, &observe.channel, &observe.constraints)
        .await?;

    // Match pattern and bind variables
    let bindings =
        match_pattern(&observe.pattern, &value).map_err(|_| ExecError::PatternMatchFailed {
            pattern: format!("{:?}", observe.pattern),
            value: value.clone(),
        })?;

    // Create new context with bindings
    let mut new_ctx = ctx.extend();
    new_ctx.set_many(bindings);

    Ok(new_ctx)
}

/// Execute a changed check
///
/// Checks if the value has changed since the last sample using
/// `behaviour_ctx.has_changed()`. Returns a new context with the
/// "changed" variable bound to the boolean result.
///
/// # Arguments
///
/// * `changed` - The changed expression to execute
/// * `ctx` - The current execution context
/// * `behaviour_ctx` - The behaviour context for checking changes
///
/// # Returns
///
/// A new context with the "changed" variable bound to a boolean indicating
/// whether the value has changed since the last sample.
///
/// # Errors
///
/// Returns `ExecError::CapabilityNotAvailable` if no provider is found for
/// the specified capability and channel.
///
/// # Examples
///
/// ```
/// use ash_core::{Changed, Value};
/// use ash_interp::behaviour::{BehaviourContext, MockBehaviourProvider};
/// use ash_interp::context::Context;
/// use ash_interp::execute_observe::execute_changed;
/// use ash_interp::typed_provider::TypedBehaviourProvider;
/// use ash_typeck::Type;
///
/// # tokio_test::block_on(async {
/// let mut behaviour_ctx = BehaviourContext::new();
/// let provider = MockBehaviourProvider::new("sensor", "temp")
///     .with_value(Value::Int(42));
/// behaviour_ctx.register(TypedBehaviourProvider::new(provider.clone(), Type::Int));
///
/// // First sample to establish baseline
/// let _ = behaviour_ctx.sample("sensor", "temp", &[]).await;
///
/// // Change value
/// provider.set_value(Value::Int(43));
///
/// let changed = Changed {
///     capability: "sensor".into(),
///     channel: "temp".into(),
///     constraints: vec![],
/// };
///
/// let ctx = Context::new();
/// let new_ctx = execute_changed(&changed, ctx, &behaviour_ctx).await.unwrap();
/// assert_eq!(new_ctx.get("changed"), Some(&Value::Bool(true)));
/// # });
/// ```
pub async fn execute_changed(
    changed: &Changed,
    ctx: Context,
    behaviour_ctx: &BehaviourContext,
) -> ExecResult<Context> {
    // Check if value has changed
    let has_changed = behaviour_ctx
        .has_changed(&changed.capability, &changed.channel, &changed.constraints)
        .await?;

    // Create new context with result bound to "changed"
    let mut new_ctx = ctx.extend();
    new_ctx.set("changed".to_string(), Value::Bool(has_changed));

    Ok(new_ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::{Constraint, Pattern, Predicate, Value};
    use std::collections::HashMap;

    use crate::behaviour::MockBehaviourProvider;
    use crate::typed_provider::TypedBehaviourProvider;
    use ash_typeck::Type;

    #[tokio::test]
    async fn test_execute_observe_simple() {
        let mut behaviour_ctx = BehaviourContext::new();
        behaviour_ctx.register(TypedBehaviourProvider::new(
            MockBehaviourProvider::new("sensor", "temp").with_value(Value::Int(42)),
            Type::Int,
        ));

        let observe = Observe {
            capability: "sensor".into(),
            channel: "temp".into(),
            constraints: vec![],
            pattern: Pattern::Variable("t".into()),
        };

        let ctx = Context::new();
        let new_ctx = execute_observe(&observe, ctx, &behaviour_ctx)
            .await
            .unwrap();

        assert_eq!(new_ctx.get("t"), Some(&Value::Int(42)));
    }

    #[tokio::test]
    async fn test_execute_observe_with_constraints() {
        let mut behaviour_ctx = BehaviourContext::new();
        let mut record = HashMap::new();
        record.insert("price".into(), Value::Int(150));
        record.insert("symbol".into(), Value::String("AAPL".into()));

        let record_type = Type::Record(vec![
            (Box::from("price"), Type::Int),
            (Box::from("symbol"), Type::String),
        ]);
        behaviour_ctx.register(TypedBehaviourProvider::new(
            MockBehaviourProvider::new("market", "stock").with_value(Value::Record(Box::new(record))),
            record_type,
        ));

        let observe = Observe {
            capability: "market".into(),
            channel: "stock".into(),
            constraints: vec![Constraint {
                predicate: Predicate {
                    name: "symbol".into(),
                    arguments: vec![ash_core::Expr::Literal(Value::String("AAPL".into()))],
                },
            }],
            pattern: Pattern::Variable("stock".into()),
        };

        let ctx = Context::new();
        let new_ctx = execute_observe(&observe, ctx, &behaviour_ctx)
            .await
            .unwrap();

        // Should have bound the record
        assert!(matches!(new_ctx.get("stock"), Some(Value::Record(_))));
    }

    #[tokio::test]
    async fn test_execute_observe_destructuring() {
        let mut behaviour_ctx = BehaviourContext::new();
        let mut record = HashMap::new();
        record.insert("value".into(), Value::Int(25));
        record.insert("unit".into(), Value::String("celsius".into()));

        let record_type = Type::Record(vec![
            (Box::from("value"), Type::Int),
            (Box::from("unit"), Type::String),
        ]);
        behaviour_ctx.register(TypedBehaviourProvider::new(
            MockBehaviourProvider::new("sensor", "temp").with_value(Value::Record(Box::new(record))),
            record_type,
        ));

        let observe = Observe {
            capability: "sensor".into(),
            channel: "temp".into(),
            constraints: vec![],
            pattern: Pattern::Record(vec![
                ("value".into(), Pattern::Variable("t".into())),
                ("unit".into(), Pattern::Variable("u".into())),
            ]),
        };

        let ctx = Context::new();
        let new_ctx = execute_observe(&observe, ctx, &behaviour_ctx)
            .await
            .unwrap();

        assert_eq!(new_ctx.get("t"), Some(&Value::Int(25)));
        assert_eq!(new_ctx.get("u"), Some(&Value::String("celsius".into())));
    }

    #[tokio::test]
    async fn test_execute_observe_missing_provider() {
        let behaviour_ctx = BehaviourContext::new(); // Empty

        let observe = Observe {
            capability: "sensor".into(),
            channel: "temp".into(),
            constraints: vec![],
            pattern: Pattern::Variable("t".into()),
        };

        let ctx = Context::new();
        let result = execute_observe(&observe, ctx, &behaviour_ctx).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExecError::CapabilityNotAvailable(_)
        ));
    }

    #[tokio::test]
    async fn test_execute_changed_true() {
        let mut behaviour_ctx = BehaviourContext::new();
        let provider = MockBehaviourProvider::new("sensor", "temp").with_value(Value::Int(42));
        behaviour_ctx.register(TypedBehaviourProvider::new(provider.clone(), Type::Int));

        // First sample to establish baseline
        let _ = behaviour_ctx.sample("sensor", "temp", &[]).await;

        // Change value
        provider.set_value(Value::Int(43));

        let changed = Changed {
            capability: "sensor".into(),
            channel: "temp".into(),
            constraints: vec![],
        };

        let ctx = Context::new();
        let new_ctx = execute_changed(&changed, ctx, &behaviour_ctx)
            .await
            .unwrap();

        assert_eq!(new_ctx.get("changed"), Some(&Value::Bool(true)));
    }

    #[tokio::test]
    async fn test_execute_changed_false() {
        let mut behaviour_ctx = BehaviourContext::new();
        let provider = MockBehaviourProvider::new("sensor", "temp").with_value(Value::Int(42));
        behaviour_ctx.register(TypedBehaviourProvider::new(provider.clone(), Type::Int));

        // Sample to establish baseline
        let _ = behaviour_ctx.sample("sensor", "temp", &[]).await;

        // Value unchanged

        let changed = Changed {
            capability: "sensor".into(),
            channel: "temp".into(),
            constraints: vec![],
        };

        let ctx = Context::new();
        let new_ctx = execute_changed(&changed, ctx, &behaviour_ctx)
            .await
            .unwrap();

        assert_eq!(new_ctx.get("changed"), Some(&Value::Bool(false)));
    }
}
