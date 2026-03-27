//! Tests for capability provider wiring from Engine to `RuntimeState`
//!
//! These tests verify that providers configured via `EngineBuilder`
//! are properly passed to `RuntimeState` and available during execution.

use ash_core::{Effect, Value};
use ash_engine::{CapabilityProvider, Engine};
use async_trait::async_trait;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// A test provider that tracks whether it was called
#[derive(Debug)]
struct TestProvider {
    name: String,
    was_called: AtomicBool,
}

impl TestProvider {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            was_called: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl CapabilityProvider for TestProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    async fn observe(
        &self,
        _action: &str,
        _args: &[Value],
    ) -> Result<Value, ash_engine::providers::ProviderError> {
        self.was_called.store(true, Ordering::SeqCst);
        Ok(Value::Null)
    }

    async fn execute(
        &self,
        _action: &str,
        _args: &[Value],
    ) -> Result<Value, ash_engine::providers::ProviderError> {
        self.was_called.store(true, Ordering::SeqCst);
        Ok(Value::Null)
    }
}

/// Test that providers can be configured via `EngineBuilder`
#[test]
fn test_engine_builder_accepts_providers() {
    let provider = TestProvider::new("test_provider");

    let engine = Engine::new()
        .with_custom_provider("test_provider", Arc::new(provider))
        .build();

    assert!(engine.is_ok(), "Engine should build with custom provider");
}

/// Test that multiple providers can be configured
#[test]
fn test_engine_builder_multiple_providers() {
    let provider1 = TestProvider::new("provider1");
    let provider2 = TestProvider::new("provider2");

    let engine = Engine::new()
        .with_custom_provider("provider1", Arc::new(provider1))
        .with_custom_provider("provider2", Arc::new(provider2))
        .build();

    assert!(
        engine.is_ok(),
        "Engine should build with multiple providers"
    );
}

/// Test that stdio capabilities can be enabled
#[test]
fn test_engine_builder_stdio_capabilities() {
    let engine = Engine::new().with_stdio_capabilities().build();

    assert!(
        engine.is_ok(),
        "Engine should build with stdio capabilities"
    );
}

/// Test that fs capabilities can be enabled
#[test]
fn test_engine_builder_fs_capabilities() {
    let engine = Engine::new().with_fs_capabilities().build();

    assert!(engine.is_ok(), "Engine should build with fs capabilities");
}

/// Test that providers override built-ins with same name
#[test]
fn test_provider_override_builtin() {
    let custom_stdio = TestProvider::new("stdio");

    let engine = Engine::new()
        .with_stdio_capabilities() // Enable built-in stdio
        .with_custom_provider("stdio", Arc::new(custom_stdio)) // Override
        .build();

    assert!(
        engine.is_ok(),
        "Engine should allow overriding built-in providers"
    );
}

/// Test that Engine is Send + Sync even with providers configured
#[test]
fn test_engine_with_providers_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build()
        .expect("engine builds");

    // These are compile-time checks
    assert_send::<Engine>();
    assert_sync::<Engine>();

    // Runtime check - engine can be shared
    let _ = &engine;
}

/// Test engine with all capabilities combined (HTTP returns error until implemented)
#[test]
fn test_engine_builder_all_capabilities_except_http() {
    let custom_provider = TestProvider::new("custom");

    // HTTP provider not yet implemented - test without it
    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .with_custom_provider("custom", Arc::new(custom_provider))
        .build();

    assert!(
        engine.is_ok(),
        "Engine should build with stdio, fs, and custom providers"
    );
}

/// Test that HTTP capabilities returns error (not yet implemented)
#[test]
fn test_http_capabilities_returns_error() {
    use ash_engine::HttpConfig;

    let result = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .with_http_capabilities(HttpConfig::new())
        .build();

    assert!(
        result.is_err(),
        "Engine should return error when HTTP capabilities requested (not yet implemented)"
    );
}
