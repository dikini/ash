# TASK-073: Implement Engine::check

## Status: ✅ Complete

## Description

Implement the type check method for the Engine type.

## Specification Reference

- SPEC-010: Embedding API - Section 2.1 Engine Type

## Requirements

### Functional Requirements

1. `Engine::check(workflow: &Workflow) -> Result<(), EngineError>`
2. Convert type errors to EngineError

### Property Requirements

```rust
// Valid workflow passes check
engine.check(&valid_workflow).is_ok()

// Type errors return error
engine.check(&invalid_workflow).is_err()
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_engine_check_valid() {
    let engine = Engine::new().build().unwrap();
    let workflow = engine.parse(r#"
        workflow test {
            action a {
                effect: operational;
                body: || -> 42;
            }
        }
    "#).unwrap();
    
    assert!(engine.check(&workflow).is_ok());
}
```

### Step 2: Implement (Green)

```rust
use ash_typeck::check;

impl Engine {
    pub fn check(&self, workflow: &Workflow) -> Result<(), EngineError> {
        check(workflow).map_err(|e| EngineError::Type(e.to_string()))
    }
}
```

## Completion Checklist

- [ ] Engine::check implemented
- [ ] Type errors converted properly
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

2 hours

## Dependencies

- TASK-071 (ash-engine crate)

## Blocked By

- TASK-071

## Blocks

- TASK-074 (Engine::execute)
