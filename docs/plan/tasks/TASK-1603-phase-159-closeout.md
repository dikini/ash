# TASK-1603: Close out Phase 159

**Status:** 📝 Planned
**Phase:** [PLAN-159](../PLAN-159-CPS-IR-INTERPRETER.md)
**Owner:** Phase 159

## Description

Close Phase 159 by reconciling plan/task/changelog status, running focused and broad verification for the isolated prototype interpreter, auditing reference/docs drift, and recording honest remaining deferrals.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)

## Dependencies

- 📝 TASK-1590 through TASK-1602 complete or explicitly deferred with user approval.

## Requirements

### Functional Requirements

1. Re-read PLAN-159, PLAN-INDEX, all TASK-1590 through TASK-1602 files, SPEC-098b, SPEC-096b, and SPEC-097b.
2. Run focused CPS IR tests and broad affected-crate gates.
3. Run docs link/trailing-whitespace checks over changed plan/spec/task files.
4. Reconcile task statuses, PLAN-159 milestones, PLAN-INDEX progress row, and CHANGELOG.
5. Request or run independent review before marking the phase complete.
6. Preserve explicit deferrals for bytecode, JIT, legacy AST lowering, Lean 4 differential testing, mutual recursion, row polymorphism, effect aliases, and full discharge unless implemented in a later phase.

### Property Requirements

- Status surfaces must agree: task files, PLAN-159, PLAN-INDEX, and CHANGELOG cannot contradict each other.
- No closeout claim may rely on a zero-test run.
- Closeout must not claim legacy lowering or Lean differential testing was implemented by Phase 159.

## TDD Steps

### Step 1: Write tests (Red)

**Files:** Focused CPS IR test commands plus docs validation commands recorded during closeout.

Write focused tests before implementation. Tests must include at least one positive example and one negative or boundary example for this task's contract.

### Step 2: Implement (Green)

**Files:** `docs/plan/PLAN-159-CPS-IR-INTERPRETER.md`, `docs/plan/PLAN-INDEX.md`, `docs/plan/tasks/TASK-1590-*.md` through `docs/plan/tasks/TASK-1602-*.md`, `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md`, `CHANGELOG.md`

Implement only the slice named by this task. Preserve the SPEC-098b `Atom` / `Value` / `Term` boundary and avoid direct-style convenience nodes.

### Step 3: Integrate

Update all status surfaces in one pass after final verification evidence is available.

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
  - git diff --check -- docs/plan/PLAN-159-CPS-IR-INTERPRETER.md docs/plan/PLAN-INDEX.md docs/plan/tasks CHANGELOG.md
checklist:
  - [ ] Focused CPS tests pass and execute non-zero cases
  - [ ] Broad affected-crate gates pass or pre-existing failures are classified
  - [ ] Plan/task/index/changelog status surfaces agree
  - [ ] Legacy lowering and Lean differential testing remain explicitly deferred
  - [ ] Independent review completed after final diff
```

## Dependencies for Next Task

- Produces the final Phase 159 closeout evidence and status reconciliation.

## Notes

Do not mark Phase 159 complete while any background verification is still running or while a review finding remains unresolved.
