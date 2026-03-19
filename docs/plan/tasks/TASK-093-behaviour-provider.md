# TASK-093: Behaviour Provider Trait

## Status: ✅ Complete

## Description

Implement the behaviour provider trait for sampling observable values.

## Specification Reference

- SPEC-014: Behaviours - Section 5 Provider Interface

## Requirements

### Functional Requirements

1. `BehaviourProvider` trait with `sample` method
2. Optional `has_changed` for optimization
3. `BehaviourRegistry` for managing providers
4. `BehaviourContext` for execution-time access
5. Support for constraints (filtering)

### Property Requirements

```rust
// Provider returns current value
let value = provider.sample(&[]).await.unwrap();

// Has changed detection
let changed = provider.has_changed(&[]).await.unwrap();

// Registry manages providers
registry.register(provider);
assert!(registry.has("sensor", "temperature"));
```

## TDD Steps

### Step 1: Write Tests (Red)

Create tests in `crates/ash-interp/src/behaviour.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_sample() {
        let provider = MockBehaviourProvider::new("sensor", "temp")
            .with_value(Value::Int(42));
        
        let value = provider.sample(&[]).await.unwrap();
        assert_eq!(value, Value::Int(42));
    }

    #[tokio::test]
    async fn test_provider_with_constraint() {
        let provider = MockBehaviourProvider::new("sensor", "temp")
            .with_constraint_handler(|c| {
                if c.name == "unit" && c.value == "celsius" {
                    Value::Int(25)
                } else {
                    Value::Int(77) // fahrenheit
                }
            });
        
        let celsius = provider.sample(&[Constraint::new("unit", "celsius")]).await.unwrap();
        assert_eq!(celsius, Value::Int(25));
        
        let fahrenheit = provider.sample(&[Constraint::new("unit", "fahrenheit")]).await.unwrap();
        assert_eq!(fahrenheit, Value::Int(77));
    }

    #[tokio::test]
    async fn test_has_changed() {
        let provider = MockBehaviourProvider::new("sensor", "temp")
            .with_value(Value::Int(42));
        
        // First check should report changed (no previous value)
        assert!(provider.has_changed(&[]).await.unwrap());
        
        // Sample to establish baseline
        let _ = provider.sample(&[]).await;
        
        // Same value - should report unchanged
        assert!(!provider.has_changed(&[]).await.unwrap());
        
        // Change value
        provider.set_value(Value::Int(43));
        
        // Should report changed
        assert!(provider.has_changed(&[]).await.unwrap());
    }

    #[test]
    fn test_behaviour_registry() {
        let mut registry = BehaviourRegistry::new();
        let provider = MockBehaviourProvider::new("sensor", "temp");
        
        registry.register(Box::new(provider));
        
        assert!(registry.has("sensor", "temp"));
        assert!(!registry.has("sensor", "pressure"));
    }

    #[tokio::test]
    async fn test_behaviour_context_sample() {
        let mut ctx = BehaviourContext::new();
        let provider = MockBehaviourProvider::new("sensor", "temp")
            .with_value(Value::Int(100));
        
        ctx.register(Box::new(provider));
        
        let value = ctx.sample("sensor", "temp", &[]).await.unwrap();
        assert_eq!(value, Value::Int(100));
    }
}
```

### Step 2: Verify RED

Run: `cargo test -p ash-interp behaviour -- --nocapture`
Expected: FAIL - trait not defined

### Step 3: Implement (Green)

Create `crates/ash-interp/src/behaviour.rs`:

```rust
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ash_core::{Constraint, Name, Value};
use crate::ExecResult;
use crate::error::ExecError;

/// Behaviour provider trait
#[async_trait]
pub trait BehaviourProvider: Send + Sync {
    fn capability_name(&self) -> &str;
    fn channel_name(&self) -> &str;
    
    /// Sample the current value
    async fn sample(&self, constraints: &[Constraint]) -> ExecResult<Value>;
    
    /// Check if value has changed since last sample
    /// Default implementation always returns true
    async fn has_changed(&self, constraints: &[Constraint]) -> ExecResult<bool> {
        Ok(true)
    }
}

/// Registry of behaviour providers
#[derive(Default)]
pub struct BehaviourRegistry {
    providers: HashMap<(Name, Name), Box<dyn BehaviourProvider>>,
}

impl BehaviourRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, provider: Box<dyn BehaviourProvider>) {
        let key = (
            provider.capability_name().to_string(),
            provider.channel_name().to_string(),
        );
        self.providers.insert(key, provider);
    }
    
    pub fn get(&self, cap: &str, channel: &str) -> Option<&dyn BehaviourProvider> {
        self.providers.get(&(cap.into(), channel.into()))
            .map(|p| p.as_ref())
    }
    
    pub fn has(&self, cap: &str, channel: &str) -> bool {
        self.providers.contains_key(&(cap.into(), channel.into()))
    }
}

/// Context for behaviour sampling during execution
pub struct BehaviourContext {
    registry: BehaviourRegistry,
}

impl BehaviourContext {
    pub fn new() -> Self {
        Self {
            registry: BehaviourRegistry::new(),
        }
    }
    
    pub fn with_registry(registry: BehaviourRegistry) -> Self {
        Self { registry }
    }
    
    pub fn register(&mut self, provider: Box<dyn BehaviourProvider>) {
        self.registry.register(provider);
    }
    
    pub async fn sample(
        &self,
        cap: &str,
        channel: &str,
        constraints: &[Constraint],
    ) -> ExecResult<Value> {
        let provider = self.registry.get(cap, channel)
            .ok_or_else(|| ExecError::CapabilityNotAvailable(
                format!("{}:{}", cap, channel)
            ))?;
        provider.sample(constraints).await
    }
    
    pub async fn has_changed(
        &self,
        cap: &str,
        channel: &str,
        constraints: &[Constraint],
    ) -> ExecResult<bool> {
        let provider = self.registry.get(cap, channel)
            .ok_or_else(|| ExecError::CapabilityNotAvailable(
                format!("{}:{}", cap, channel)
            ))?;
        provider.has_changed(constraints).await
    }
}

impl Default for BehaviourContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock provider for testing
pub struct MockBehaviourProvider {
    name: (String, String),
    value: Mutex<Value>,
    last_value: Mutex<Option<Value>>,
}

impl MockBehaviourProvider {
    pub fn new(cap: &str, channel: &str) -> Self {
        Self {
            name: (cap.to_string(), channel.to_string()),
            value: Mutex::new(Value::Null),
            last_value: Mutex::new(None),
        }
    }
    
    pub fn with_value(self, value: Value) -> Self {
        *self.value.lock().unwrap() = value;
        self
    }
    
    pub fn set_value(&self, value: Value) {
        *self.value.lock().unwrap() = value;
    }
}

#[async_trait]
impl BehaviourProvider for MockBehaviourProvider {
    fn capability_name(&self) -> &str {
        &self.name.0
    }
    
    fn channel_name(&self) -> &str {
        &self.name.1
    }
    
    async fn sample(&self, _constraints: &[Constraint]) -> ExecResult<Value> {
        let value = self.value.lock().unwrap().clone();
        *self.last_value.lock().unwrap() = Some(value.clone());
        Ok(value)
    }
    
    async fn has_changed(&self, _constraints: &[Constraint]) -> ExecResult<bool> {
        let current = self.value.lock().unwrap().clone();
        let last = self.last_value.lock().unwrap().clone();
        
        match last {
            None => Ok(true), // Never sampled before
            Some(last_val) => Ok(current != last_val),
        }
    }
}
```

### Step 4: Verify GREEN

Run: `cargo test -p ash-interp behaviour -- --nocapture`
Expected: PASS

### Step 5: Commit

```bash
git add crates/ash-interp/src/behaviour.rs
git commit -m "feat: behaviour provider trait and registry"
```

## Notes

This task creates the base trait. See TASK-096 for typed wrapper with schema validation. Providers will eventually need to declare their Ash type schema for runtime validation.

## Completion Checklist

- [ ] BehaviourProvider trait defined
- [ ] Sample method implemented
- [ ] Has_changed with default
- [ ] BehaviourRegistry implemented
- [ ] BehaviourContext for execution
- [ ] MockBehaviourProvider for testing
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

3 hours

## Dependencies

None

## Blocked By

Nothing

## Blocks

- TASK-094 (Parse observe)
- TASK-095 (Observe execution)
- TASK-096 (Typed provider wrapper)
