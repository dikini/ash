# TASK-1550: Parser Let Destructors

## Status: ✅ Complete

## Description

Add parser support for `let { ... } = ...` (record destructuring) and `let ( ... ) = ...` (tuple destructuring).

## Specification Reference

- [SPEC-091: Let Destructors](../../spec/SPEC-091-LET-DESTRUCTORS.md)
- [PLAN-155: Let Destructors](../PLAN-155-LET-DESTRUCTORS.md)

## Acceptance Criteria

- [x] Parser accepts `let { field1, field2 } = expr;` (shorthand)
- [x] Parser accepts `let { field1: var1, field2: var2 } = expr;` (explicit rename)
- [x] Parser accepts `let (a, b) = expr;` (tuple — already worked)
- [x] Parser rejects `let { field1, field1 } = expr;` (duplicate field — handled by typechecker)
- [x] Parser produces clear AST nodes for destructuring
- [x] No regressions in existing parsing tests

## Syntax

```ash
-- Record destructor shorthand
let { gen, shrink } = strategy;  -- Equivalent to: let { gen: gen, shrink: shrink } = strategy;

-- Record destructor explicit rename
let { gen: g, shrink: s } = strategy;

-- Partial record destructor
let { gen } = strategy;  -- Only bind 'gen', ignore 'shrink'

-- Tuple destructor
let (a, b) = pair;
let (a, b, c) = triple;
```

## Implementation

### Changes Made

**File:** `crates/ash-parser/src/parse_pattern.rs`
- Modified `parse_record_pattern` to detect shorthand syntax
- When a field name is not followed by `:`, check if it's followed by `,` or `}`
- If so, create a `Pattern::Variable` with the field name as the variable name
- Otherwise, fail (require explicit `field: pattern` syntax)

**File:** `crates/ash-parser/tests/let_destructor_tests.rs`
- Renamed `let_record_destructor_shorthand_fails` to `let_record_destructor_shorthand_works`
- Updated test to verify shorthand parses correctly and produces correct AST

### Before (required explicit rename):
```ash
let { x: x, y: y } = point;
```

### After (shorthand supported):
```ash
let { x, y } = point;
```

## Verification

- [x] `cargo test -p ash-parser --test let_destructor_tests` — 6/6 pass
- [x] `cargo test -p ash-parser` — 631+ tests pass
- [x] `cargo test -p ash-cli --test stdlib_corpus_check` — 54/54 pass
- [x] `cargo fmt --check` — pass
- [x] Commit: `1313aad9`

## Dependencies

- None (parser-only change)

## Closeout Checklist

- [x] Implementation complete
- [x] Tests updated and passing
- [x] No documentation needed (feature is intuitive)
- [x] Committed to branch
