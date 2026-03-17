# TASK-007: Shared Testing Utilities

## Status: 🟢 Complete

## Description

Create shared testing utilities and fixtures for consistent testing across crates.

## Specification Reference

- AGENTS.md - Testing guidelines

## Requirements

### Functional Requirements

1. Test fixtures:
   - `TestContext` - Setup/teardown for tests
   - `TempWorkspace` - Temporary directory management
   - `MockCapabilityProvider` - Test double for capabilities

2. Assertion helpers:
   - `assert_effect_eq!` - Effect comparison with good errors
   - `assert_value_eq!` - Value comparison
   - `assert_workflow_eq!` - Workflow AST comparison

3. Snapshot testing setup:
   - insta configuration
   - Named snapshots for different test cases

4. Property test configuration:
   - Standard test count settings
   - Timeout configuration
   - Failure persistence

5. Test data builders:
   - `WorkflowBuilder` - Fluent API for building workflows
   - `ValueBuilder` - Easy value construction
   - `PatternBuilder` - Pattern construction helpers

### Code Organization

```
crates/ash-core/src/test_helpers.rs  # #[cfg(test)] only
crates/ash-core/src/test_fixtures.rs
tests/integration/                 # Integration test fixtures
```

## TDD Steps

### Step 1: Create Test Context (Green)

```rust
pub struct TestContext {
    temp_dir: TempDir,
    // other test resources
}

impl TestContext {
    pub fn new() -> Self { ... }
    pub fn temp_path(&self) -> &Path { ... }
}

impl Drop for TestContext {
    fn drop(&mut self) { /* cleanup */ }
}
```

### Step 2: Create Builders (Green)

```rust
pub struct WorkflowBuilder { ... }

impl WorkflowBuilder {
    pub fn observe(capability: &str) -> Self { ... }
    pub fn then(self, next: Workflow) -> Self { ... }
    pub fn build(self) -> Workflow { ... }
}
```

### Step 3: Add Assertion Macros (Green)

```rust
#[macro_export]
macro_rules! assert_effect_eq {
    ($left:expr, $right:expr) => {
        assert_eq!($left, $right, "Effects not equal: {:?} vs {:?}", $left, $right);
    };
}
```

### Step 4: Configure insta (Green)

Add `tests/snapshots/` and insta configuration.

### Step 5: Document Usage (Green)

Add examples in module docs.

## Completion Checklist

- [ ] TestContext with temp directory
- [ ] WorkflowBuilder fluent API
- [ ] ValueBuilder helper
- [ ] PatternBuilder helper
- [ ] Assertion macros
- [ ] insta configuration
- [ ] proptest configuration
- [ ] Documentation with examples
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

4 hours

## Dependencies

- TASK-003 (Workflow needed for builders)
- TASK-005 (Pattern needed for builders)

## Blocked By

- TASK-003
- TASK-005

## Blocks

- TASK-060 (Integration tests)
- All future test writing
