# TASK-1592: Evaluate conditionals and structured data

**Status:** ✅ Complete
**Phase:** [PLAN-159](../PLAN-159-CPS-IR-INTERPRETER.md)
**Owner:** Phase 159

## Description

Add `If`, `Record`, and `Tuple` support while preserving the CPS layering: records and tuples are values bound by `LetVal`, and branches are terms.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)

## Dependencies

- 📝 TASK-1590: Define CPS IR core data structures.
- 📝 TASK-1591: Evaluate core CPS values and terms.

## Requirements

### Functional Requirements

1. Evaluate `If` from an atom condition.
2. Evaluate `Record` and `Tuple` as `Value` constructors only.
3. Reject `.cps` examples that place `record` or `tuple` directly as branch terms.
4. Add tests for normalized branch bodies that bind a structured value and then `Jump`.

### Property Requirements

- Branch choice depends only on the condition atom.
- Record field order normalization must be specified and tested if equality or serialization depends on order.

## TDD Steps

### Step 1: Write tests (Red)

**Files:** `crates/ash-interp/tests/task_1592_cps_ir.rs`

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

- Provides structured value support for handler examples and recursive programs.

## Notes

Keep examples normalized CPS IR. Values must be bound with `LetVal`; primitive computations must be bound with `LetPrim`; branch bodies must be `Term`s.
