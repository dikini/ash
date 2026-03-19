# TASK-113: Read/Write Type Checking

## Status: ✅ Complete

## Description

Implement type checking for capability read (input) and write (output) schemas.

## Specification Reference

- SPEC-017: Capability Integration - Section 7 Type Safety

## Requirements

### Functional Requirements

1. Check observe/receive values match read schema
2. Check set/send values match write schema
3. Bidirectional providers have separate schemas
4. Error on type mismatch

### Property Requirements

```rust
// Read type mismatch
let result = typecheck(&observe);
assert!(matches!(result, Err(TypeError::InputMismatch { .. })));

// Write type mismatch
let result = typecheck(&set);
assert!(matches!(result, Err(TypeError::OutputMismatch { .. })));

// Correct types pass
let result = typecheck(&correct);
assert!(result.is_ok());
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_observe_type_mismatch() {
    -- Schema: sensor:temp returns Int
    -- But pattern expects String
    let observe = parse("observe sensor:temp as s"); -- s used as String
    
    let result = typecheck(&observe, &registry);
    assert!(result.is_err());
}

#[test]
fn test_set_type_mismatch() {
    -- Schema: hvac:target accepts Int
    let set = parse("set hvac:target = \"hot\""); -- String instead of Int
    
    let result = typecheck(&set, &registry);
    assert!(matches!(result, Err(TypeError::OutputMismatch { .. })));
}

#[test]
fn test_bidirectional_different_types() {
    -- Provider with different read/write types
    let schema = CapabilitySchema {
        read: Some(Type::Int),
        write: Some(Type::String),
    };
    
    -- Observe returns Int
    -- Set accepts String
    -- Both should typecheck correctly
}
```

### Step 2: Verify RED

Expected: FAIL - type checking not implemented

### Step 3: Implement (Green)

```rust
pub fn typecheck_capability(
    op: &CapabilityOperation,
    registry: &CapabilityRegistry,
) -> Result<Type, TypeError> {
    let schema = registry.get_schema(&op.capability, &op.channel)
        .ok_or(TypeError::UnknownCapability)?;
    
    match op.direction {
        Direction::Input => {
            let value_type = infer_type(&op.value)?;
            if let Some(read_schema) = &schema.read {
                if !read_schema.matches_type(&value_type) {
                    return Err(TypeError::InputMismatch {
                        capability: op.capability.clone(),
                        expected: read_schema.clone(),
                        actual: value_type,
                    });
                }
            }
        }
        Direction::Output => {
            let value_type = infer_type(&op.value)?;
            if let Some(write_schema) = &schema.write {
                if !write_schema.matches_type(&value_type) {
                    return Err(TypeError::OutputMismatch {
                        capability: op.capability.clone(),
                        expected: write_schema.clone(),
                        actual: value_type,
                    });
                }
            }
        }
    }
    
    Ok(schema.clone())
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: read/write type checking for capabilities"
```

## Completion Checklist

- [ ] Input type checking
- [ ] Output type checking
- [ ] Bidirectional schema support
- [ ] Error messages
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

4 hours

## Dependencies

- TASK-097 (Schema validation)

## Blocked By

- TASK-097

## Blocks

None (completes Phase 15)
