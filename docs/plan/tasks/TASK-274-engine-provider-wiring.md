# TASK-274: Wire Engine Capability Providers to Runtime

## Status: 📝 Planned

## Description

Fix the critical issue where EngineBuilder stores capability providers but they are never passed to RuntimeState during execution. This renders the Embedding API non-functional as configured providers have no effect at runtime.

## Specification Reference

- SPEC-010: Embedding API Specification
- SPEC-004: Runtime Semantics - Capability Provider Integration

## Dependencies

- ✅ TASK-071: Create ash-engine crate structure
- ✅ TASK-075: Standard capability providers

## Requirements

### Functional Requirements

1. EngineBuilder must pass configured providers to RuntimeState during Engine::execute()
2. RuntimeState must store and expose the provider registry
3. Capability operations must resolve providers from RuntimeState
4. Standard providers (stdio, fs) must work when configured via Embedding API

### Current State (Broken)

**File:** `crates/ash-engine/src/lib.rs`

```rust
// EngineBuilder stores providers
pub struct EngineBuilder {
    providers: HashMap<String, Arc<dyn CapabilityProvider>>,
    // ...
}

impl EngineBuilder {
    pub fn with_provider(mut self, name: &str, provider: Arc<dyn CapabilityProvider>) -> Self {
        self.providers.insert(name.to_string(), provider);
        self
    }
}

// But execute() only passes RuntimeState without providers
pub fn execute(&self, workflow: &Workflow, input: Value) -> Result<ExecutionResult, Error> {
    let state = RuntimeState::new(); // No providers passed!
    // ...
}
```

### Target State (Fixed)

```rust
// RuntimeState needs provider registry
pub struct RuntimeState {
    // existing fields...
    providers: HashMap<String, Arc<dyn CapabilityProvider>>,
}

impl RuntimeState {
    pub fn with_providers(self, providers: HashMap<String, Arc<dyn CapabilityProvider>>) -> Self {
        Self { providers, ..self }
    }
    
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn CapabilityProvider>> {
        self.providers.get(name).cloned()
    }
}

// Engine::execute passes providers
pub fn execute(&self, workflow: &Workflow, input: Value) -> Result<ExecutionResult, Error> {
    let state = RuntimeState::new()
        .with_providers(self.providers.clone());
    // ...
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-engine/tests/provider_wiring_test.rs`

```rust
//! Tests for capability provider wiring

use ash_engine::{Engine, ExecutionResult};
use ash_core::Value;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

struct TestProvider {
    was_called: AtomicBool,
}

#[test]
fn test_configured_provider_is_used() {
    let provider = Arc::new(TestProvider {
        was_called: AtomicBool::new(false),
    });
    
    let engine = Engine::builder()
        .with_provider("test", provider.clone())
        .build();
    
    // Execute a workflow that uses the provider
    let result = engine.execute(/* workflow using test provider */);
    
    // Provider should have been called
    assert!(provider.was_called.load(Ordering::SeqCst));
}

#[test]
fn test_provider_registry_passed_to_runtime() {
    let engine = Engine::builder()
        .with_stdio_capabilities()
        .build();
    
    // Verify RuntimeState has providers
    let state = engine.create_runtime_state();
    assert!(state.get_provider("stdin").is_some());
    assert!(state.get_provider("stdout").is_some());
}
```

### Step 2: Implement RuntimeState Provider Storage

**File:** `crates/ash-interp/src/context.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;
use ash_core::capability::CapabilityProvider;

pub struct RuntimeState {
    // existing fields...
    providers: HashMap<String, Arc<dyn CapabilityProvider>>,
}

impl RuntimeState {
    pub fn new() -> Self {
        Self {
            // existing initialization...
            providers: HashMap::new(),
        }
    }
    
    pub fn with_provider(mut self, name: &str, provider: Arc<dyn CapabilityProvider>) -> Self {
        self.providers.insert(name.to_string(), provider);
        self
    }
    
    pub fn with_providers(mut self, providers: HashMap<String, Arc<dyn CapabilityProvider>>) -> Self {
        self.providers.extend(providers);
        self
    }
    
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn CapabilityProvider>> {
        self.providers.get(name).cloned()
    }
}
```

### Step 3: Update Engine to Pass Providers

**File:** `crates/ash-engine/src/lib.rs`

```rust
impl Engine {
    pub fn execute(&self, workflow: &Workflow, input: Value) -> Result<ExecutionResult, Error> {
        let mut state = RuntimeState::new()
            .with_providers(self.providers.clone());
        
        // Set up input binding if provided
        if input != Value::Null {
            state.bind_input(input);
        }
        
        let result = self.interpreter.execute(workflow, state)?;
        Ok(ExecutionResult::from(result))
    }
}
```

### Step 4: Update Capability Operations

**File:** `crates/ash-interp/src/capability.rs`

Ensure capability operations use `RuntimeState.get_provider()` instead of creating fresh providers.

## Verification Steps

- [ ] `cargo test -p ash-engine --test provider_wiring_test` passes
- [ ] `cargo test -p ash-interp --lib` passes
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Working provider registry in RuntimeState
- Functional Embedding API

Required by:
- TASK-275: Enable obligation checking (uses provider state)

## Notes

**Critical Issue**: This is a hard spec violation - the Embedding API contract promises configurable providers but they are non-functional.

**Design Decision**: Keep provider storage in RuntimeState (not Interpreter) to maintain clean separation between execution logic and runtime context.

**Edge Cases**:
- Empty provider registry should still work (use default providers)
- Provider name collisions - last configured wins
- Thread safety - providers are Arc<dyn>, state is per-execution
