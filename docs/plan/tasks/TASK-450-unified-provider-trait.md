# TASK-450: Add Unified CapabilityProvider Trait

## Status: 📝 Planned

## Description

Add a unified `CapabilityProvider` trait to `ash_core` that both primitive and user-defined capabilities implement. This replaces the dual-trait system and eliminates the need for an adapter layer.

## Specification Reference

- [DESIGN-015: Unified Action System](../../design/DESIGN-015-UNIFIED-ACTION-SYSTEM.md) - Decision 2, Decision 3
- [SPEC-001: IR](../../spec/SPEC-001-IR.md) - Core types
- [SPEC-017: Capability Integration](../../spec/SPEC-017-CAPABILITY-INTEGRATION.md) - Provider interface

## Dependencies

- ✅ [TASK-449](TASK-449-action-vec-value.md): Action with Vec<Value>

## Requirements

### Functional Requirements

1. Add `crates/ash-core/src/capability.rs` with unified `CapabilityProvider` trait
2. Define `CapabilityError` enum (unifies ExecError/ProviderError concepts)
3. Keep `observe` signature unchanged: `observe(&[Constraint])`
4. Change `execute` signature to: `execute(&Action)` where Action has `Vec<Value>`
5. No adapter layer needed
6. Treat this task as Phase 1 follow-on work after parser/lowering and ACT execution were already absorbed into TASK-449, not as the place where AST/execution coherence is restored

### Property Requirements

```rust
// All providers should satisfy these properties:
property_provider_has_name(provider: CapabilityProvider) -> String
property_provider_has_effect(provider: CapabilityProvider) -> Effect
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-core/src/capability.rs` (NEW FILE)

**Current State:**
```rust
// File does not exist yet
```

**Target State:**
```rust
//! Unified capability provider trait and error types

use crate::{Constraint, Effect, Name, Value, Action};
use async_trait::async_trait;

/// Unified error type for all capability operations
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CapabilityError {
    #[error("Capability '{0}' not available")]
    NotAvailable(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

/// Unified capability provider trait
///
/// Both primitive and user-defined capabilities implement this trait.
#[async_trait]
pub trait CapabilityProvider: Send + Sync + std::fmt::Debug {
    /// Get the provider name
    fn name(&self) -> &str;
    
    /// Get the effect level of this provider
    fn effect(&self) -> Effect;
    
    /// Observe/read from this capability
    ///
    /// Uses unevaluated constraints (delayed evaluation).
    /// Constraints are evaluated by the provider as needed.
    async fn observe(&self, constraints: &[Constraint]) -> Result<Value, CapabilityError>;
    
    /// Execute an action on this capability
    ///
    /// Arguments are already evaluated (eager evaluation).
    async fn execute(&self, action: &Action) -> Result<Value, CapabilityError>;
}
```

**Key Changes:**
1. New `capability.rs` module in `ash-core`
2. Unified `CapabilityProvider` trait with `observe(&[Constraint])` and `execute(&Action)`
3. Unified `CapabilityError` enum

**Test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;

    #[derive(Debug)]
    struct MockProvider {
        name: &'static str,
        effect: Effect,
    }

    #[async_trait]
    impl CapabilityProvider for MockProvider {
        fn name(&self) -> &str { self.name }
        fn effect(&self) -> Effect { self.effect }
        
        async fn observe(&self, _constraints: &[Constraint]) -> Result<Value, CapabilityError> {
            Ok(Value::Null)
        }
        
        async fn execute(&self, action: &Action) -> Result<Value, CapabilityError> {
            Ok(Value::String(format!("executed: {}", action.name)))
        }
    }

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider {
            name: "test",
            effect: Effect::Operational,
        };
        
        assert_eq!(provider.name(), "test");
        assert_eq!(provider.effect(), Effect::Operational);
        
        let action = Action {
            name: "do_something".to_string(),
            arguments: vec![],
        };
        
        let result = provider.execute(&action).await.unwrap();
        assert_eq!(result, Value::String("executed: do_something".to_string()));
    }

    #[test]
    fn test_capability_error_display() {
        let err = CapabilityError::NotAvailable("test".to_string());
        assert_eq!(err.to_string(), "Capability 'test' not available");
    }
}
```

### Step 2: Implement (Green)

**File:** `crates/ash-core/src/capability.rs`

Implementation: Add the module with trait and error type as shown above.

### Step 3: Integration (Green)

Update `crates/ash-core/src/lib.rs`:
```rust
pub mod capability;  // Add this
```

### Step 4: Property Tests (Verify)

**File:** `tests/task_450_proptest.rs`

```rust
//! Property-based tests for TASK-450

use ash_core::{CapabilityProvider, CapabilityError, Effect, Value, Action, Constraint};
use proptest::prelude::*;

proptest! {
    #[test]
    fn provider_name_is_consistent(
        name in "[a-z]+",
        effect in prop_oneof![Just(Effect::Epistemic), Just(Effect::Operational)]
    ) {
        #[derive(Debug)]
        struct TestProvider {
            name: String,
            effect: Effect,
        }
        
        #[async_trait]
        impl CapabilityProvider for TestProvider {
            fn name(&self) -> &str { &self.name }
            fn effect(&self) -> Effect { self.effect }
            async fn observe(&self, _: &[Constraint]) -> Result<Value, CapabilityError> {
                Ok(Value::Null)
            }
            async fn execute(&self, _: &Action) -> Result<Value, CapabilityError> {
                Ok(Value::Null)
            }
        }
        
        let provider = TestProvider { name: name.clone(), effect };
        prop_assert_eq!(provider.name(), name);
        prop_assert_eq!(provider.effect(), effect);
    }

    #[test]
    fn capability_error_round_trip(
        err in prop_oneof![
            Just(CapabilityError::NotAvailable("test".to_string())),
            Just(CapabilityError::ExecutionFailed("fail".to_string())),
            Just(CapabilityError::ValidationFailed("invalid".to_string())),
        ]
    ) {
        let display = err.to_string();
        prop_assert!(!display.is_empty());
    }
}
```

## Verification Steps

- [ ] `cargo test --package ash-core --lib capability` passes
- [ ] Property tests pass (100+ iterations)
- [ ] `cargo clippy --package ash-core` clean
- [ ] `cargo fmt --check` clean
- [ ] Documentation builds: `cargo doc --package ash-core --no-deps`

## Dependencies for Next Task

This task outputs:
- Unified `CapabilityProvider` trait
- Unified `CapabilityError` enum

Required by:
- Follow-on task file to migrate `CapabilityContext` to the unified trait
- Follow-on task files to migrate `FsProvider`, `StdioProvider`, and `McpProvider`

## Notes

**Important considerations:**
- `observe` keeps `&[Constraint]` (unevaluated) - this is intentional
- `execute` uses `&Action` (with `Vec<Value>`) - arguments are pre-evaluated
- Phase misalignment is documented and accepted
- Error types are unified, reducing duplication
- Parser/lowering and ACT execution are intentionally out of scope here because they must be completed in `TASK-449` before this trait can be adopted cleanly

**Edge cases to consider:**
- Empty constraints in `observe`
- Empty arguments in `execute`
- Provider returning `None` vs explicit error

**Ready to Implement Checklist:**
- [x] File paths specified (`crates/ash-core/src/capability.rs`)
- [x] Current code shown (file doesn't exist)
- [x] Target code shown
- [x] Tests defined
- [x] Dependencies listed (TASK-449)
- [x] Edge cases mentioned (empty constraints/arguments)
- [x] Error cases considered (all CapabilityError variants)
