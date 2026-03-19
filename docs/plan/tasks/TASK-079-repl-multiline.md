# TASK-079: Multi-line Input Detection

## Status: ✅ Complete

## Description

Implement smart multi-line input detection based on parse errors.

## Specification Reference

- SPEC-011: REPL - Section 3.3 Multiline Input

## Requirements

### Functional Requirements

1. Detect incomplete input (continue to next line)
2. Detect syntax errors (show error immediately)
3. Smart indentation continuation

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_multiline_incomplete() {
    // Input that needs continuation
    assert!(is_incomplete("workflow test {"));
}

#[test]
fn test_multiline_complete() {
    // Complete expression
    assert!(!is_incomplete("42"));
}
```

### Step 2: Implement (Green)

```rust
fn is_incomplete(input: &str) -> bool {
    match engine.parse(input) {
        Ok(_) => false,
        Err(EngineError::Parse(e)) => e.contains("incomplete"),
        Err(_) => false,
    }
}
```

## Completion Checklist

- [ ] Incomplete detection works
- [ ] Error detection works
- [ ] Tests pass

## Estimated Effort

4 hours

## Dependencies

- TASK-077

## Blocked By

- TASK-077

## Blocks

None
