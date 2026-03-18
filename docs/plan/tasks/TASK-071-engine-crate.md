# TASK-071: Create ash-engine Crate

## Status: 🟢 Complete

## Description

Create the ash-engine crate with the unified Engine type and builder API.

## Specification Reference

- SPEC-010: Embedding API

## Requirements

### Functional Requirements

1. Create crates/ash-engine/
2. Engine and EngineBuilder types
3. Error type

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_engine_creates() {
    let engine = Engine::new().build();
    assert!(engine.is_ok());
}
```

### Step 2: Create Crate (Green)

Create crates/ash-engine/ with proper Cargo.toml

## Completion Checklist

- [ ] Crate created
- [ ] Engine type defined
- [ ] Added to workspace

## Estimated Effort

3 hours

## Dependencies

None

## Blocked By

Nothing

## Blocks

- TASK-072 (Engine::parse)
- TASK-077 (REPL uses Engine)
