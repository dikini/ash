# TASK-100: Type Error Reporting

## Status: 🔴 Not Started

## Description

Improve error messages for type mismatches with context and suggestions.

## Specification Reference

- SPEC-015: Typed Providers - Section 6 Error Messages

## Requirements

### Functional Requirements

1. `ExecError::TypeMismatch` with structured fields
2. Display trait for clear error messages
3. Context: which provider, which field (for records)
4. Path tracking for nested validation

### Property Requirements

```rust
let err = ExecError::TypeMismatch {
    provider: "sensor:temp".into(),
    expected: "Int".into(),
    actual: "String(\"hello\")".into(),
    path: Some("value".into()),
};

assert!(err.to_string().contains("sensor:temp"));
assert!(err.to_string().contains("expected Int, got String"));
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_error_display_simple() {
    let err = ExecError::TypeMismatch {
        provider: "sensor:temp".into(),
        expected: "Int".into(),
        actual: "String(\"hello\")".into(),
        path: None,
    };
    
    let msg = err.to_string();
    assert!(msg.contains("sensor:temp"));
    assert!(msg.contains("Int"));
    assert!(msg.contains("String"));
}

#[test]
fn test_error_display_with_path() {
    let err = ExecError::TypeMismatch {
        provider: "sensor:temp".into(),
        expected: "Int".into(),
        actual: "String(\"x\")".into(),
        path: Some("point.x".into()),
    };
    
    let msg = err.to_string();
    assert!(msg.contains("point.x"));
}
```

### Step 2: Verify RED

Expected: FAIL - error type not updated

### Step 3: Implement (Green)

```rust
#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("type mismatch in provider '{provider}'{path}: expected {expected}, got {actual}")]
    TypeMismatch {
        provider: String,
        expected: String,
        actual: String,
        path: Option<String>,
    },
    // ... other variants
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: improved type error reporting"
```

## Completion Checklist

- [ ] Structured error type
- [ ] Display implementation
- [ ] Path tracking for nested errors
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

2 hours

## Dependencies

- TASK-099 (Runtime validation)

## Blocked By

- TASK-099

## Blocks

None (completes typed providers)
