# TASK-1591: Evaluate core CPS values and terms

**Status:** ✅ Complete
**Phase:** [PLAN-159](../PLAN-159-CPS-IR-INTERPRETER.md)
**Owner:** Phase 159

## Description

Implement the first interpreter slice for `LetVal`, `LetPrim`, `LetCont`, `Jump`, and `Call`, using `Lam` and `Cont` values from TASK-1590.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)

## Dependencies

- 📝 TASK-1590: Define CPS IR core data structures.

## Requirements

### Functional Requirements

1. Evaluate atoms through the runtime environment.
2. Bind values with `LetVal`.
3. Evaluate primitive operations only through `LetPrim`.
4. Install label continuations with `LetCont`.
5. Invoke continuations with `Jump` and ordinary CPS functions with `Call`.
6. Preserve fixed answer-type assumptions in test fixtures even if the runtime does not carry an `Ans` value.

### Property Requirements

- Evaluator results are deterministic for closed Phase 1 programs.
- Calling a non-lambda or jumping to a non-continuation traps fail-closed.

## TDD Steps

### Step 1: Write tests (Red)

**Files:** `crates/ash-interp/tests/task_1591_cps_ir.rs`

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

- Produces the evaluator substrate required by conditionals, handlers, recursion, and differential tests.

## Notes

Keep examples normalized CPS IR. Values must be bound with `LetVal`; primitive computations must be bound with `LetPrim`; branch bodies must be `Term`s.
