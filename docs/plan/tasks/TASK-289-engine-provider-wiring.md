# TASK-289: Wire Engine Capability Providers to Runtime

## Status: 📝 Planned

## Description

Fix the critical issue where engine capability providers are dead configuration. EngineBuilder stores providers via `with_stdio_capabilities()`, `with_fs_capabilities()`, and custom providers, but `execute()` only passes RuntimeState into the interpreter, and RuntimeState has no provider registry. This breaks the embedding/capability contract.

## Specification Reference

- SPEC-010: Engine API Specification
- SPEC-004: Runtime Semantics Specification
- SPEC-017: Capability Integration Specification

## Dependencies

- ✅ TASK-071: Engine crate structure
- ✅ TASK-075: Standard capability providers
- ✅ TASK-076: CLI engine integration

## Critical File Locations

- `crates/ash-engine/src/lib.rs:60` - EngineBuilder stores providers but they don't reach runtime
- `crates/ash-engine/src/lib.rs:235` - execute() doesn't pass providers to RuntimeState
- `crates/ash-interp/src/runtime_state.rs:17` - RuntimeState has no provider registry

## Requirements

### Functional Requirements

1. Providers configured via EngineBuilder must be accessible at runtime
2. RuntimeState must maintain a provider registry
3. The interpreter must resolve capabilities against configured providers
4. Custom providers must participate in capability resolution

### Current State (Broken)

**File:** `crates/ash-engine/src/lib.rs:235`

```rust
pub fn execute(&self, workflow: &Workflow) -> Result<ExecutionResult, EngineError> {
    let mut state = RuntimeState::new(); // No providers passed!
    
    // Providers configured in EngineBuilder are ignored
    self.interpreter.execute(
        workflow,
        &mut Context::new(),
        &mut state,  // Empty provider registry
    )
}
```

**File:** `crates/ash-interp/src/runtime_state.rs:17`

```rust
pub struct RuntimeState {
    // No provider_registry field!
    values: HashMap<VarId, Value>,
    current_effect: Effect,
    // ...
}
```

Problems:
1. EngineBuilder providers never reach the interpreter
2. RuntimeState has no provider registry
3. Capability resolution uses hardcoded defaults
4. Custom providers are effectively ignored

### Target State (Fixed)

```rust
// crates/ash-engine/src/lib.rs
pub fn execute(&self, workflow: &Workflow) -> Result<ExecutionResult, EngineError> {
    let mut state = RuntimeState::with_providers(
        self.provider_registry.clone()  // Pass configured providers
    );
    
    self.interpreter.execute(
        workflow,
        &mut Context::new(),
        &mut state,
    )
}

// crates/ash-interp/src/runtime_state.rs
pub struct RuntimeState {
    provider_registry: Arc<ProviderRegistry>,  // NEW
    values: HashMap<VarId, Value>,
    current_effect: Effect,
    // ...
}

impl RuntimeState {
    pub fn with_providers(registry: Arc<ProviderRegistry>) -> Self {
        Self {
            provider_registry: registry,
            values: HashMap::new(),
            current_effect: Effect::default(),
            // ...
        }
    }
    
    pub fn get_provider(&self, name: &str) -> Option<&dyn CapabilityProvider> {
        self.provider_registry.get(name)
    }
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-engine/tests/provider_wiring_test.rs`

```rust
//! Tests for engine capability provider wiring

use ash_engine::{Engine, EngineBuilder};
use ash_core::{CapabilityProvider, Value};

#[test]
fn test_stdio_provider_reaches_runtime() {
    let engine = Engine::builder()
        .with_stdio_capabilities()
        .build();
    
    let workflow = r#"
        workflow test {
            act print("hello");
        }
    "#;
    
    // Should use the configured stdio provider, not a stub
    let result = engine.run(workflow).unwrap();
    assert!(result.is_complete());
}

#[test]
fn test_custom_provider_reaches_runtime() {
    #[derive(Debug)]
    struct TestProvider;
    
    impl CapabilityProvider for TestProvider {
        fn name(&self) -> &str {
            "test_provider"
        }
        
        fn execute(&self, action: &str, args: &[Value]) -> Result<Value, ProviderError> {
            assert_eq!(action, "test_action");
            Ok(Value::String("test_result".to_string()))
        }
    }
    
    let engine = Engine::builder()
        .with_provider(Box::new(TestProvider))
        .build();
    
    let workflow = r#"
        workflow test {
            capabilities: [test_provider];
            act test_provider::test_action();
        }
    "#;
    
    let result = engine.run(workflow).unwrap();
    assert!(result.is_complete());
}

#[test]
fn test_multiple_providers_wired() {
    let engine = Engine::builder()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .with_network_capabilities()
        .build();
    
    // All providers should be accessible
    let state = engine.create_runtime_state();
    assert!(state.get_provider("stdio").is_some());
    assert!(state.get_provider("fs").is_some());
    assert!(state.get_provider("network").is_some());
}

#[test]
fn test_provider_resolution_at_runtime() {
    let engine = Engine::builder()
        .with_stdio_capabilities()
        .build();
    
    let workflow = r#"
        workflow test {
            // This should resolve against the configured stdio provider
            act log("test message");
        }
    "#;
    
    let result = engine.run(workflow).unwrap();
    assert!(result.is_complete());
    // Verify the actual stdio provider was used
}

proptest! {
    #[test]
    fn provider_registry_preserves_provider_count(count in 0usize..10) {
        let mut builder = Engine::builder();
        for i in 0..count {
            builder = builder.with_provider(Box::new(TestProvider::new(i)));
        }
        let engine = builder.build();
        let state = engine.create_runtime_state();
        
        assert_eq!(state.provider_count(), count);
    }
}
```

### Step 2: Add Provider Registry to RuntimeState

**File:** `crates/ash-interp/src/runtime_state.rs`

```rust
use std::sync::Arc;
use ash_core::capability::ProviderRegistry;

pub struct RuntimeState {
    provider_registry: Arc<ProviderRegistry>,  // NEW
    values: HashMap<VarId, Value>,
    current_effect: Effect,
    provenance: ProvenanceLog,
}

impl RuntimeState {
    /// Create RuntimeState without providers (backwards compatible)
    pub fn new() -> Self {
        Self::with_providers(Arc::new(ProviderRegistry::default()))
    }
    
    /// Create RuntimeState with configured providers
    pub fn with_providers(registry: Arc<ProviderRegistry>) -> Self {
        Self {
            provider_registry: registry,
            values: HashMap::new(),
            current_effect: Effect::Epistemic,
            provenance: ProvenanceLog::new(),
        }
    }
    
    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<&dyn CapabilityProvider> {
        self.provider_registry.get(name)
    }
    
    /// Check if a provider is available
    pub fn has_provider(&self, name: &str) -> bool {
        self.provider_registry.contains(name)
    }
    
    /// Get count of registered providers (for testing)
    pub fn provider_count(&self) -> usize {
        self.provider_registry.len()
    }
}
```

### Step 3: Wire Providers Through Engine

**File:** `crates/ash-engine/src/lib.rs`

```rust
impl Engine {
    pub fn execute(&self, workflow: &Workflow) -> Result<ExecutionResult, EngineError> {
        // Pass configured providers to RuntimeState
        let mut state = RuntimeState::with_providers(
            self.provider_registry.clone()
        );
        
        let mut ctx = Context::new();
        
        self.interpreter.execute(
            workflow,
            &mut ctx,
            &mut state,
        )
    }
    
    /// Create a RuntimeState for testing/inspection
    pub fn create_runtime_state(&self) -> RuntimeState {
        RuntimeState::with_providers(self.provider_registry.clone())
    }
}

impl EngineBuilder {
    pub fn with_provider(mut self, provider: Box<dyn CapabilityProvider>) -> Self {
        self.provider_registry.register(provider);
        self
    }
    
    pub fn with_stdio_capabilities(mut self) -> Self {
        self.provider_registry.register(Box::new(StdioProvider::new()));
        self
    }
    
    pub fn with_fs_capabilities(mut self) -> Self {
        self.provider_registry.register(Box::new(FsProvider::new()));
        self
    }
    
    pub fn with_network_capabilities(mut self) -> Self {
        self.provider_registry.register(Box::new(NetworkProvider::new()));
        self
    }
}
```

### Step 4: Update Capability Resolution

**File:** `crates/ash-interp/src/capability_resolution.rs`

```rust
impl CapabilityResolver {
    pub fn resolve(
        &self,
        capability: &str,
        state: &RuntimeState,
    ) -> Result<&dyn CapabilityProvider, ResolutionError> {
        // Try to get from configured providers first
        if let Some(provider) = state.get_provider(capability) {
            return Ok(provider);
        }
        
        // Fall back to built-in providers if allowed
        if self.allow_builtin_fallback {
            return self.get_builtin_provider(capability);
        }
        
        Err(ResolutionError::ProviderNotConfigured {
            capability: capability.to_string(),
        })
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-engine --test provider_wiring_test` passes
- [ ] Custom providers can be configured and reach runtime
- [ ] `cargo test -p ash-interp` passes with provider integration
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Working provider registry in RuntimeState
- EngineBuilder providers reaching interpreter

Required by:
- Full capability enforcement at runtime
- Custom provider functionality

## Notes

**Critical Issue**: This is a fundamental embedding contract violation. The Engine API advertises provider configuration that doesn't actually affect execution.

**Risk Assessment**: High - affects all capability provider usage through the embedding API.

**Implementation Strategy**:
1. First: Add provider registry to RuntimeState
2. Second: Wire EngineBuilder to RuntimeState
3. Third: Update capability resolution to use registry
4. Fourth: Add regression tests

**Backwards Compatibility**: `RuntimeState::new()` remains available but creates an empty registry. New code should use `RuntimeState::with_providers()`.
