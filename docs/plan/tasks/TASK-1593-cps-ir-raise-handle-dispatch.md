# TASK-1593: Implement Raise and Handle dispatch

**Status:** ✅ Complete
**Phase:** [PLAN-159](../PLAN-159-CPS-IR-INTERPRETER.md)
**Owner:** Phase 159

## Description

Implement operation-typed `Raise` and `Handle` evaluation with explicit handler chain walking, matching SPEC-098b handler clause shape and row-field meaning.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)

## Dependencies

- 📝 TASK-1591: Evaluate core CPS values and terms.
- 📝 TASK-1592: Evaluate conditionals and structured data.

## Requirements

### Functional Requirements

1. Represent `EffectOp` with item, argument types, and result type.
2. Represent `HandlerClause { op, params, resume, body, row }`.
3. Represent `Handle { clause, body, cont, row }`.
4. Dispatch `Raise` by walking the active handler/provider chain.
5. Trap with `UnhandledEffect(op)` when no matching frame exists.
6. Preserve `Raise.row` as operation-local row and `Handle.row` as residual local row.

### Property Requirements

- An unhandled operation always traps.
- The nearest matching frame handles before an outer matching frame.

## TDD Steps

### Step 1: Write tests (Red)

**Files:** `crates/ash-interp/tests/task_1593_cps_ir.rs`

Write focused tests before implementation. Tests must include at least one positive example and one negative or boundary example for this task's contract.

### Step 2: Implement (Green)

**Files:** `crates/ash-core/src/cps.rs`, `crates/ash-interp/src/cps.rs`

Implement only the slice named by this task. Preserve the SPEC-098b `Atom` / `Value` / `Term` boundary and avoid direct-style convenience nodes.

### Step 3: Integrate

Wire the new slice through crate exports and the Phase 159 `.cps` fixture path without replacing the existing workflow interpreter.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-core -p ash-interp
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
checklist:
  - [ ] Focused tests execute non-zero cases
  - [ ] `.cps` fixtures parse or are explicitly deferred by this task
  - [ ] CHANGELOG.md updated when this task is completed
```

## Dependencies for Next Task

- Provides handler dispatch substrate for TASK-1594 and TASK-1595.

## Notes

Keep examples normalized CPS IR. Values must be bound with `LetVal`; primitive computations must be bound with `LetPrim`; branch bodies must be `Term`s.
