# TASK-083: Error Display Improvements

## Status: ✅ Complete

## Description

Improve error display in REPL with syntax highlighting and helpful messages.

## Specification Reference

- SPEC-011: REPL - Section 6 Error Display

## Requirements

### Functional Requirements

1. Syntax error highlighting with line numbers
2. Type error context
3. Suggestions for fixes

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_error_display() {
    let display = format_error("1 +", &parse_error);
    assert!(display.contains("expected expression"));
}
```

### Step 2: Implement (Green)

```rust
fn format_error(source: &str, error: &ParseError) -> String {
    // Format with line numbers and highlighting
}
```

## Completion Checklist

- [ ] Line numbers in errors
- [ ] Context highlighting
- [ ] Helpful suggestions

## Estimated Effort

3 hours

## Dependencies

- TASK-077

## Blocked By

- TASK-077

## Blocks

None (completes Phase 12)
