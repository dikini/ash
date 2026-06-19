# TASK-1598: Implement row representation and local-total validation scaffold

**Status:** 📝 Planned
**Phase:** [PLAN-159](../PLAN-159-CPS-IR-INTERPRETER.md)
**Owner:** Phase 159

## Description

Add the row substrate needed by the interpreter phase without overclaiming the full SPEC-096b/SPEC-097b type-system implementation.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)

## Dependencies

- 📝 TASK-1590: Define CPS IR core data structures.
- 📝 TASK-1592: Evaluate conditionals and structured data.
- 📝 TASK-1593: Implement Raise and Handle dispatch.
- 📝 TASK-1596: Implement single-binding LetRec recursion.
- 📝 TASK-1597: Implement RecordDischarge and Trap.

## Requirements

### Functional Requirements

1. Define typed row carriers and namespaced effect item identities for the CPS IR slice.
2. Compute local and total rows for `Jump`, `Call`, `Raise`, `Handle`, `LetVal`, `LetRec`, `LetPrim`, `LetCont`, `If`, `RecordDischarge`, and `Trap`.
3. Eliminate exact duplicate row items.
4. Fail closed when residual rows contradict the task-scoped checker rules.
5. Document that transparent alias/group expansion, public module-summary export, and complete kind-specific discharge remain follow-up work unless implemented with tests in this task.

### Property Requirements

- Local/total row equations match SPEC-098b §2.4.
- `cap fs` and `cap fs.read` remain distinct unless a later capability-interface rule expands them.

## TDD Steps

### Step 1: Write tests (Red)

**Files:** `crates/ash-interp/tests/task_1598_cps_ir.rs`

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

- Provides row validation evidence for handler and closeout tasks.

## Notes

Keep examples normalized CPS IR. Values must be bound with `LetVal`; primitive computations must be bound with `LetPrim`; branch bodies must be `Term`s.
