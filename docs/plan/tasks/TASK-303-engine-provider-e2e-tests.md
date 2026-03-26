# TASK-303: Engine Provider E2E Tests

## Status: 📝 Planned

## Description

Create end-to-end tests that verify capability providers registered via `EngineBuilder` are actually used during workflow execution. The provider wiring was implemented in TASK-289, but we need E2E coverage.

## Current Implementation (from TASK-289)

```rust
// EngineBuilder stores providers
EngineBuilder::new()
    .with_custom_provider(MyProvider::new())
    .build();

// RuntimeState holds providers in Arc<Mutex<HashMap>>

// execute_with_bindings_in_state() creates CapabilityContext from providers
let cap_ctx = runtime_state.create_capability_context().await;
```

## Test Scenarios

### Scenario 1: Custom Provider Registration
```rust
// Register a custom mock provider
let mut provider = MockProvider::new("sensor");
provider.expect_observe().return_value(Value::Int(42));

let engine = EngineBuilder::new()
    .with_custom_provider(provider)
    .build();

// Workflow observes from the provider
workflow test {
    observe sensor { value }
    ret value;
}

// Verify the workflow receives 42 from our mock
```

### Scenario 2: Provider Override
```rust
// Register provider that overrides default behavior
let engine = EngineBuilder::new()
    .with_stdio_capabilities()  // Default stdout
    .with_custom_provider(LoggingProvider::new())  // Override
    .build();

// Verify LoggingProvider is used, not default
```

### Scenario 3: Multiple Providers
```rust
// Multiple providers for different capabilities
let engine = EngineBuilder::new()
    .with_custom_provider(TemperatureSensor::new())
    .with_custom_provider(HumiditySensor::new())
    .build();

// Workflow uses both
workflow test {
    observe temp_sensor { temp }
    observe humidity_sensor { humidity }
    ret (temp, humidity);
}
```

### Scenario 4: Provider Not Found
```rust
// No provider registered for "unknown"
// Workflow trying to observe should get CapabilityNotAvailable error
```

## Test Requirements

1. **Mock Providers**: Create test doubles that record invocations
2. **Real Execution**: Tests must actually execute workflows, not just unit test internals
3. **Verification**: Confirm provider methods are called with correct arguments
4. **Async**: Tests must handle async execution properly

## Files to Create

- `crates/ash-engine/tests/provider_e2e_test.rs`

## Completion Checklist

- [ ] Custom provider registration E2E test
- [ ] Provider override E2E test
- [ ] Multiple providers E2E test
- [ ] Provider not found error test
- [ ] All E2E tests pass
- [ ] Tests verify actual provider invocation
- [ ] `cargo test -p ash-engine --test provider_e2e_test` passes
- [ ] `cargo clippy` clean
