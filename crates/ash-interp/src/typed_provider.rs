//! Typed provider wrappers for runtime type validation
//!
//! This module provides wrapper structs that carry type schemas for
//! [`BehaviourProvider`] and [`StreamProvider`], enabling runtime validation
//! of sampled and streamed values.

use ash_core::{Constraint, Value};
use ash_typeck::Type;
use async_trait::async_trait;

use crate::behaviour::BehaviourProvider;
use crate::error::{ExecError, ExecResult};
use crate::stream::StreamProvider;

/// Typed wrapper for [`BehaviourProvider`] with associated type schema
///
/// This struct wraps a behaviour provider and carries a [`Type`] schema
/// that describes the expected type of values returned by the provider.
/// The schema can be used for runtime type validation.
///
/// # Example
///
/// ```
/// use ash_interp::typed_provider::TypedBehaviourProvider;
/// use ash_interp::behaviour::MockBehaviourProvider;
/// use ash_core::Value;
/// use ash_typeck::Type;
///
/// let inner = MockBehaviourProvider::new("sensor", "temp").with_value(Value::Int(42));
/// let typed = TypedBehaviourProvider::new(inner, Type::Int);
///
/// assert_eq!(typed.schema(), &Type::Int);
/// ```
pub struct TypedBehaviourProvider {
    inner: Box<dyn BehaviourProvider>,
    schema: Type,
}

impl TypedBehaviourProvider {
    /// Create a new typed behaviour provider
    ///
    /// # Arguments
    ///
    /// * `provider` - The behaviour provider to wrap
    /// * `schema` - The expected type schema for values from this provider
    ///
    /// # Type Parameters
    ///
    /// * `P` - The concrete provider type, must implement [`BehaviourProvider`] + `'static`
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::typed_provider::TypedBehaviourProvider;
    /// use ash_interp::behaviour::MockBehaviourProvider;
    /// use ash_core::Value;
    /// use ash_typeck::Type;
    ///
    /// let inner = MockBehaviourProvider::new("sensor", "temp").with_value(Value::Int(42));
    /// let typed = TypedBehaviourProvider::new(inner, Type::Int);
    /// ```
    pub fn new<P>(provider: P, schema: Type) -> Self
    where
        P: BehaviourProvider + 'static,
    {
        Self {
            inner: Box::new(provider),
            schema,
        }
    }

    /// Get the type schema for this provider
    ///
    /// Returns a reference to the [`Type`] schema that describes the
    /// expected type of values from this provider.
    #[must_use]
    pub fn schema(&self) -> &Type {
        &self.schema
    }
}

#[async_trait]
impl BehaviourProvider for TypedBehaviourProvider {
    fn capability_name(&self) -> &str {
        self.inner.capability_name()
    }

    fn channel_name(&self) -> &str {
        self.inner.channel_name()
    }

    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value> {
        let value = self.inner.sample(constraints).await?;

        if !self.schema.matches(&value) {
            return Err(ExecError::type_mismatch(
                format!("{}:{}", self.capability_name(), self.channel_name()),
                self.schema.to_string(),
                value.to_string(),
            ));
        }

        Ok(value)
    }

    async fn has_changed(&self, constraints: &[Constraint]) -> ExecResult<bool> {
        self.inner.has_changed(constraints).await
    }
}

/// Typed wrapper for [`StreamProvider`] with associated type schema
///
/// This struct wraps a stream provider and carries a [`Type`] schema
/// that describes the expected type of values returned by the provider.
/// The schema can be used for runtime type validation.
///
/// # Example
///
/// ```
/// use ash_interp::typed_provider::TypedStreamProvider;
/// use ash_interp::stream::MockStreamProvider;
/// use ash_core::Value;
/// use ash_typeck::Type;
///
/// let inner = MockStreamProvider::with_values("kafka", "orders", vec![Value::Int(1)]);
/// let typed = TypedStreamProvider::new(inner, Type::Int);
///
/// assert_eq!(typed.schema(), &Type::Int);
/// ```
pub struct TypedStreamProvider {
    inner: Box<dyn StreamProvider>,
    schema: Type,
}

impl TypedStreamProvider {
    /// Create a new typed stream provider
    ///
    /// # Arguments
    ///
    /// * `provider` - The stream provider to wrap
    /// * `schema` - The expected type schema for values from this provider
    ///
    /// # Type Parameters
    ///
    /// * `P` - The concrete provider type, must implement [`StreamProvider`] + `'static`
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::typed_provider::TypedStreamProvider;
    /// use ash_interp::stream::MockStreamProvider;
    /// use ash_core::Value;
    /// use ash_typeck::Type;
    ///
    /// let inner = MockStreamProvider::with_values("kafka", "orders", vec![Value::Int(1)]);
    /// let typed = TypedStreamProvider::new(inner, Type::Int);
    /// ```
    pub fn new<P>(provider: P, schema: Type) -> Self
    where
        P: StreamProvider + 'static,
    {
        Self {
            inner: Box::new(provider),
            schema,
        }
    }

    /// Get the type schema for this provider
    ///
    /// Returns a reference to the [`Type`] schema that describes the
    /// expected type of values from this provider.
    #[must_use]
    pub fn schema(&self) -> &Type {
        &self.schema
    }
}

#[async_trait]
impl StreamProvider for TypedStreamProvider {
    fn capability_name(&self) -> &str {
        self.inner.capability_name()
    }

    fn channel_name(&self) -> &str {
        self.inner.channel_name()
    }

    fn try_recv(&self) -> Option<ExecResult<Value>> {
        self.inner.try_recv().map(|result| {
            result.and_then(|value| {
                if self.schema.matches(&value) {
                    Ok(value)
                } else {
                    Err(ExecError::type_mismatch(
                        format!("{}:{}", self.capability_name(), self.channel_name()),
                        self.schema.to_string(),
                        value.to_string(),
                    ))
                }
            })
        })
    }

    async fn recv(&self) -> ExecResult<Value> {
        let value = self.inner.recv().await?;

        if !self.schema.matches(&value) {
            return Err(ExecError::type_mismatch(
                format!("{}:{}", self.capability_name(), self.channel_name()),
                self.schema.to_string(),
                value.to_string(),
            ));
        }

        Ok(value)
    }

    fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::behaviour::MockBehaviourProvider;
    use crate::stream::MockStreamProvider;

    // ============================================================
    // TypedBehaviourProvider Tests
    // ============================================================

    #[test]
    fn test_typed_behaviour_provider_creation() {
        let inner = MockBehaviourProvider::new("sensor", "temp");
        let typed = TypedBehaviourProvider::new(inner, Type::Int);

        assert_eq!(typed.schema(), &Type::Int);
        assert_eq!(typed.capability_name(), "sensor");
        assert_eq!(typed.channel_name(), "temp");
    }

    #[tokio::test]
    async fn test_typed_behaviour_provider_delegates_sample() {
        let inner = MockBehaviourProvider::new("sensor", "temp").with_value(Value::Int(42));
        let typed = TypedBehaviourProvider::new(inner, Type::Int);

        let value = typed.sample(&[]).await.unwrap();
        assert_eq!(value, Value::Int(42));
    }

    #[tokio::test]
    async fn test_typed_behaviour_provider_delegates_has_changed() {
        let inner = MockBehaviourProvider::new("sensor", "temp").with_value(Value::Int(42));
        let typed = TypedBehaviourProvider::new(inner, Type::Int);

        // First check should report changed (no previous value)
        assert!(typed.has_changed(&[]).await.unwrap());

        // Sample to establish baseline
        let _ = typed.sample(&[]).await;

        // Same value - should report unchanged
        assert!(!typed.has_changed(&[]).await.unwrap());
    }

    #[test]
    fn test_typed_behaviour_provider_with_complex_type() {
        let inner = MockBehaviourProvider::new("db", "users");
        let record_type = Type::Record(vec![
            (Box::from("name"), Type::String),
            (Box::from("age"), Type::Int),
        ]);
        let typed = TypedBehaviourProvider::new(inner, record_type.clone());

        assert_eq!(typed.schema(), &record_type);
    }

    // ============================================================
    // TypedStreamProvider Tests
    // ============================================================

    #[test]
    fn test_typed_stream_provider_creation() {
        let inner = MockStreamProvider::new("kafka", "orders");
        let typed = TypedStreamProvider::new(inner, Type::Int);

        assert_eq!(typed.schema(), &Type::Int);
        assert_eq!(typed.capability_name(), "kafka");
        assert_eq!(typed.channel_name(), "orders");
    }

    #[tokio::test]
    async fn test_typed_stream_provider_delegates_recv() {
        let inner =
            MockStreamProvider::with_values("kafka", "orders", vec![Value::Int(1), Value::Int(2)]);
        let typed = TypedStreamProvider::new(inner, Type::Int);

        let v1 = typed.recv().await.unwrap();
        assert_eq!(v1, Value::Int(1));

        let v2 = typed.recv().await.unwrap();
        assert_eq!(v2, Value::Int(2));
    }

    #[test]
    fn test_typed_stream_provider_delegates_try_recv() {
        let inner = MockStreamProvider::with_values("kafka", "orders", vec![Value::Int(42)]);
        let typed = TypedStreamProvider::new(inner, Type::Int);

        let value = typed.try_recv().unwrap().unwrap();
        assert_eq!(value, Value::Int(42));

        // No more values - should return None
        assert!(typed.try_recv().is_none());
    }

    #[test]
    fn test_typed_stream_provider_delegates_is_closed() {
        let inner = MockStreamProvider::with_values("kafka", "orders", vec![Value::Int(1)]);
        let typed = TypedStreamProvider::new(inner, Type::Int);

        // Initially not closed
        assert!(!typed.is_closed());

        // Close the stream
        typed.inner.is_closed(); // Accessing inner to close would require interior mutability
        // The mock's close method requires mutable access, but is_closed is through trait
        // This test verifies delegation works for the initial state
    }

    #[test]
    fn test_typed_stream_provider_with_complex_type() {
        let inner = MockStreamProvider::new("kafka", "events");
        let list_type = Type::List(Box::new(Type::String));
        let typed = TypedStreamProvider::new(inner, list_type.clone());

        assert_eq!(typed.schema(), &list_type);
    }

    #[test]
    fn test_typed_stream_provider_with_function_type() {
        let inner = MockStreamProvider::new("stream", "callbacks");
        let func_type = Type::Fun(
            vec![Type::Int, Type::Int],
            Box::new(Type::Int),
            ash_core::Effect::Operational,
        );
        let typed = TypedStreamProvider::new(inner, func_type.clone());

        assert_eq!(typed.schema(), &func_type);
    }

    // ============================================================
    // Runtime Validation Tests (TASK-099)
    // ============================================================

    #[tokio::test]
    async fn test_validation_passes() {
        let inner = MockBehaviourProvider::new("sensor", "temp").with_value(Value::Int(42));
        let typed = TypedBehaviourProvider::new(inner, Type::Int);

        let result = typed.sample(&[]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[tokio::test]
    async fn test_validation_fails() {
        let inner = MockBehaviourProvider::new("sensor", "temp")
            .with_value(Value::String("not a number".into()));
        let typed = TypedBehaviourProvider::new(inner, Type::Int);

        let result = typed.sample(&[]).await;
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("type mismatch"));
        assert!(err.contains("sensor:temp"));
    }

    #[tokio::test]
    async fn test_stream_validation_passes() {
        let inner = MockStreamProvider::with_values("queue", "events", vec![Value::Int(42)]);
        let typed = TypedStreamProvider::new(inner, Type::Int);

        let result = typed.recv().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[tokio::test]
    async fn test_stream_validation_fails() {
        let inner = MockStreamProvider::with_values(
            "queue",
            "events",
            vec![Value::String("not a number".into())],
        );
        let typed = TypedStreamProvider::new(inner, Type::Int);

        let result = typed.recv().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_stream_try_recv_validation_passes() {
        let inner = MockStreamProvider::with_values("queue", "events", vec![Value::Int(42)]);
        let typed = TypedStreamProvider::new(inner, Type::Int);

        let result = typed.try_recv().unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[test]
    fn test_stream_try_recv_validation_fails() {
        let inner = MockStreamProvider::with_values(
            "queue",
            "events",
            vec![Value::String("not a number".into())],
        );
        let typed = TypedStreamProvider::new(inner, Type::Int);

        let result = typed.try_recv().unwrap();
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("type mismatch"));
        assert!(err.contains("queue:events"));
    }

    // ============================================================
    // Error Display Tests (TASK-100)
    // ============================================================

    #[test]
    fn test_error_display_simple() {
        let err = ExecError::type_mismatch("sensor:temp", "Int", "String(\"hello\")");

        let msg = err.to_string();
        assert!(msg.contains("sensor:temp"));
        assert!(msg.contains("Int"));
        assert!(msg.contains("String"));
    }

    #[test]
    fn test_error_display_with_path() {
        let err =
            ExecError::type_mismatch("sensor:temp", "Int", "String(\"x\")").with_path("point.x");

        let msg = err.to_string();
        assert!(msg.contains("point.x"));
        assert!(msg.contains("sensor:temp"));
    }

    #[tokio::test]
    async fn test_validation_error_message() {
        let inner = MockBehaviourProvider::new("sensor", "temp")
            .with_value(Value::String("not a number".into()));
        let typed = TypedBehaviourProvider::new(inner, Type::Int);

        let result = typed.sample(&[]).await;
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("sensor:temp"),
            "Error should mention provider: {err_msg}"
        );
        assert!(
            err_msg.contains("Int"),
            "Error should mention expected type: {err_msg}"
        );
        assert!(
            err_msg.contains("not a number"),
            "Error should mention actual value: {err_msg}"
        );
    }
}
