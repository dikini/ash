# TASK-1550: Parser Let Destructors

## Status: 📝 Planned

## Description

Add parser support for `let { ... } = ...` (record destructuring) and `let ( ... ) = ...` (tuple destructuring).

## Specification Reference

- [SPEC-091: Let Destructors](../../spec/SPEC-091-LET-DESTRUCTORS.md)
- [PLAN-155: Let Destructors](../PLAN-155-LET-DESTRUCTORS.md)

## Acceptance Criteria

- [ ] Parser accepts `let { field1, field2 } = expr;`
- [ ] Parser accepts `let { field1: var1, field2: var2 } = expr;`
- [ ] Parser accepts `let (a, b) = expr;`
- [ ] Parser rejects `let { field1, field1 } = expr;` (duplicate field)
- [ ] Parser produces clear AST nodes for destructuring
- [ ] No regressions in existing parsing tests

## Syntax

```ash
-- Record destructor
let { gen, shrink } = strategy;
let { gen: g, shrink: s } = strategy;
let { gen } = strategy;  -- Partial

-- Tuple destructor
let (a, b) = pair;
let (a, b, c) = triple;
```

## Verification

- `cargo test -p ash-parser` passes
- New parser tests for all destructor forms pass
- New parser tests for error cases pass
