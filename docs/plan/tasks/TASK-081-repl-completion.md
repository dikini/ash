# TASK-081: Tab Completion

## Status: ✅ Complete

## Description

Implement tab completion for keywords, builtins, and user-defined names.

## Specification Reference

- SPEC-011: REPL - Section 5.3 Tab Completion

## Requirements

### Functional Requirements

1. Complete keywords (workflow, action, capability, etc.)
2. Complete builtin functions
3. Complete previously defined names

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_complete_keyword() {
    let completer = AshCompleter::new();
    let candidates = completer.complete("wor");
    assert!(candidates.contains(&"workflow".to_string()));
}
```

### Step 2: Implement (Green)

```rust
impl Completer for AshCompleter {
    fn complete(&self, line: &str, pos: usize) -> Vec<Pair> {
        // Return matching keywords
    }
}
```

## Completion Checklist

- [ ] Keyword completion
- [ ] Builtin completion
- [ ] rustyline integration

## Estimated Effort

4 hours

## Dependencies

- TASK-077

## Blocked By

- TASK-077

## Blocks

None
