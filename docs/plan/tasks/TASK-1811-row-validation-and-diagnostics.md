# TASK-1811: Validate row syntax and emit fail-closed diagnostics

## Status: ✅ Complete

## Description

Add validation diagnostics for parsed computation rows before Core lowering. This task enforces the Phase 177 source rules that keep row syntax unambiguous and authority-neutral.

## Specification Reference

- [PLAN-177](../PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [NOTE-021](../../notes/NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md)

## Dependencies

- TASK-1809 surface row parser carriers complete.
- TASK-1810 operation identity resolution complete or explicitly scoped.

## Requirements

### Functional Requirements

1. Reject callable declarations that specify both inline row syntax and expanded `where row { ... }`.
2. Reject more than one expanded `where row { ... }` block per callable.
3. Reject row tails except as the final row entry.
4. Reject duplicate row tails.
5. Reject raw predicate/law/contract bodies inside row syntax for Phase 177; require evidence/fact references or fail closed.
6. Reject unsupported row item families before Core lowering with item-specific diagnostics.
7. Add diagnostics that point to both duplicate row spellings when possible.
8. Add regression tests for each validation rule.

### Property Requirements

- Validation must happen before Core lowering.
- Diagnostics should identify row syntax as requirements, not grants.
- Existing legacy syntax should not be accidentally reinterpreted as target row syntax unless explicitly parsed by TASK-1809.

## TDD Steps

### Step 1: Write failing validation tests

Add tests for duplicate inline/expanded row spelling, duplicate expanded row blocks, misplaced `| r`, duplicate tails, and raw predicate row bodies.

### Step 2: Verify RED

Run focused parser/engine/typechecker tests and confirm currently accepted or wrong diagnostics fail.

### Step 3: Implement validation

Add validation at the earliest layer that has full callable row context and spans.

### Step 4: Verify GREEN

Run focused tests and affected crate tests.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file, rust-analyzer]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-typeck --test task_1811_row_validation_and_diagnostics
  - cargo test -p ash-parser
  - cargo test -p ash-engine
  - cargo test -p ash-typeck
  - git diff --check
checklist:
  - [x] Duplicate row spelling is rejected.
  - [x] Row tail placement is validated.
  - [x] Unsupported predicate bodies fail closed.
  - [x] Diagnostics include useful spans.
```

## Dependencies for Next Task

This task feeds TASK-1814.
