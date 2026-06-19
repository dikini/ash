# TASK-1597: Implement RecordDischarge and Trap

**Status:** ✅ Complete
**Phase:** [PLAN-159](../PLAN-159-CPS-IR-INTERPRETER.md)
**Owner:** Phase 159

## Description

Implement administrative `RecordDischarge` and bottom-typed diagnostic `Trap` behavior for the interpreter slice.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)

## Dependencies

- 📝 TASK-1591: Evaluate core CPS values and terms.

TASK-1597 must remain independent of TASK-1598. It preserves discharge metadata and trap diagnostics, but row computation for those terms belongs to TASK-1598.

## Requirements

### Functional Requirements

1. Treat `RecordDischarge` as an administrative runtime no-op that preserves discharge metadata for evidence/audit paths.
2. Treat `Trap` as an unrecoverable abort with diagnostic `TrapReason`.
3. Keep recoverable failures modeled as `Raise { item: Failure(...) }`, not `Trap`.
4. Add tests for contract-discharge metadata and trap abort behavior.

### Property Requirements

- `Trap` never resumes.
- `RecordDischarge` does not alter the value produced by its body.

## TDD Steps

### Step 1: Write tests (Red)

**Files:** `crates/ash-interp/tests/task_1597_cps_ir.rs`

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

- Provides advanced administrative terms for Phase 5 completion.

## Notes

Keep examples normalized CPS IR. Values must be bound with `LetVal`; primitive computations must be bound with `LetPrim`; branch bodies must be `Term`s.
