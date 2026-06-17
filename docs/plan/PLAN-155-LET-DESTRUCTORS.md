# PLAN-155: Let Destructors for Records and Tuples

**Status:** 📝 Planned
**Spec:** [SPEC-091: Let Destructors](../spec/SPEC-091-LET-DESTRUCTORS.md)
**Amends:** [PLAN-151](PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md) (TASK-1511)
**Task range:** TASK-1550 through TASK-1559

## Goal

Add `let` destructor syntax for record and tuple types. This is group assignment — not pattern matching — providing a convenient way to bind multiple variables from a structured value.

## Core Design

```ash
let { gen, shrink } = strategy;  -- Record destructor
let (a, b) = pair;               -- Tuple destructor
let { gen: g, shrink: s } = strategy;  -- Explicit renaming
```

**Semantics:**
- Records: order-independent, field-name matching, partial OK
- Tuples: order-dependent, position-based, length must match
- Variants (sum types): not supported, use `match`
- No ellision syntax — simply omit fields you don't need

## Non-Goals

- No deep destructuring (e.g., `let { a: { b } } = nested`)
- No destructuring in function parameters (deferred)
- No destructuring in `for` loops (deferred)
- No pattern guards in `let` (use `match` for conditions)

## Decision Gates

| Gate | Decision | Owner task |
|---|---|---|
| D1 | Parser accepts `let { ... } = ...` and `let ( ... ) = ...` | TASK-1550 |
| D2 | AST has representation for destructuring | TASK-1551 |
| D3 | Typechecker verifies fields exist, types match | TASK-1552 |
| D4 | Interpreter evaluates destructuring correctly | TASK-1553 |
| D5 | Error messages are informative and helpful | TASK-1554 |
| D6 | Documentation updated | TASK-1555-TASK-1558 |
| D7 | Closeout with verification | TASK-1559 |

## Task Table

| Task | Description | Status |
|---|---|---|
| [TASK-1550](tasks/TASK-1550-parser-let-destructors.md) | Add parser support for `let { ... } = ...` and `let ( ... ) = ...` | 📝 Planned |
| [TASK-1551](tasks/TASK-1551-ast-destructure-representation.md) | Add AST representation for `let` destructuring | 📝 Planned |
| [TASK-1552](tasks/TASK-1552-typecheck-destructors.md) | Typecheck destructuring: verify fields, types, duplicates | 📝 Planned |
| [TASK-1553](tasks/TASK-1553-interpreter-destructors.md) | Evaluate destructuring in interpreter | 📝 Planned |
| [TASK-1554](tasks/TASK-1554-destructor-diagnostics.md) | Add error messages for all destructor failure modes | 📝 Planned |
| [TASK-1555](tasks/TASK-1555-reference-let-destructors.md) | Update `reference/language/functions/local-and-anonymous.md` | 📝 Planned |
| [TASK-1556](tasks/TASK-1556-reference-record-destructors.md) | Update `reference/language/types/records.md` with destructor examples | 📝 Planned |
| [TASK-1557](tasks/TASK-1557-reference-tuple-destructors.md) | Update `reference/language/types/tuples.md` with destructor examples | 📝 Planned |
| [TASK-1558](tasks/TASK-1558-cookbook-destructor-patterns.md) | Add destructor examples to cookbook | 📝 Planned |
| [TASK-1559](tasks/TASK-1559-phase-155-closeout.md) | Close out Phase 155 with verification and documentation | 📝 Planned |

## Implementation Order

1. TASK-1550: Parser changes (foundation)
2. TASK-1551: AST changes (depends on parser)
3. TASK-1552: Typechecker (depends on AST)
4. TASK-1553: Interpreter (depends on typechecker)
5. TASK-1554: Diagnostics (parallel with interpreter)
6. TASK-1555-TASK-1558: Documentation (parallel, no dependencies)
7. TASK-1559: Closeout

## Verification Strategy

Every task must include:
- Focused Rust tests for the changed component
- Integration tests for end-to-end destructuring
- Negative tests for all error conditions
- `cargo fmt --check`, `cargo test`, `cargo clippy` gates
- `git diff --check`

## Closeout Criteria

- All TASK-1550 through TASK-1558 tasks complete
- SPEC-091, PLAN-155, and PLAN-INDEX agree on scope/status
- No regressions in existing tests
- CHANGELOG.md records the feature
- Phase 151 tasks updated with new dependencies

## Notes

This phase unblocks Phase 151's TASK-1511 by enabling:
- `let { gen, shrink } = strategy` for combinator implementation
- Cleaner extraction of record fields without verbose field access

The risk is low: changes are localized to parser, AST, typechecker, and interpreter. No runtime value representation changes.

## Documentation Tasks

| Task | File | Content |
|------|------|---------|
| TASK-1555 | `reference/language/functions/local-and-anonymous.md` | `let` destructor syntax, record and tuple forms, renaming, partial matching |
| TASK-1556 | `reference/language/types/records.md` | Record destructors, order independence, field name matching, error conditions |
| TASK-1557 | `reference/language/types/tuples.md` | Tuple destructors, order dependence, length matching, error conditions |
| TASK-1558 | `reference/cookbook/destructuring.md` | Common patterns: extracting fields, swapping variables, nested access |

## What This Unblocks

| Combinator | Previously | Now |
|------------|-----------|-----|
| `map` | `let g = s.gen; let sh = s.shrink;` | `let { gen, shrink } = s;` |
| `map_with_shrink` | Field access | Direct destructuring |
| `map2` | Field access on both | `let { gen: g1 } = sa; let { gen: g2 } = sb;` |
| `with_shrink` | Field access | Direct destructuring |
