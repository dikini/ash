# TASK-1004: Workflow and operational binder irrefutability enforcement

## Status: 📝 Planned

## Description

Enforce irrefutable patterns for workflow-level binders such as workflow `let`, observe result binding, spawn/split binding, and loop element binding where the live audit confirms typed binders exist.

## Specification Reference

- [SPEC-076](../../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
- [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)

## Dependencies

- ✅ TASK-1000 packet exists
- 📝 TASK-1001 audit gate must complete and patch this task before implementation starts
- 📝 TASK-1002 shared irrefutability API must be implemented before this task wires workflow/operational binders

## Requirements

1. Preserve SPEC-076 non-goals and decision gates.
2. Add RED tests for each live source-level, lowered-only, and core-only binder using a refutable sum/list/literal pattern, including yield-arm lowering and any core spawn/split patterns identified by TASK-1001.
3. Wire checks at the semantic type-checking boundary, not only parser or lowering.
4. Keep runtime pattern-failure variants defensive but unreachable for checked binder cases, using TASK-1001's exact refreshed names.
5. Document any binder whose type is unavailable and add a blocked/deferred diagnostic instead of guessing.

## File Targets

- Modify: exact files to be patched by TASK-1001 audit
- Test: exact focused test target to be patched by TASK-1001 audit

## TDD / Execution Steps

1. Stop if this file still contains the fail-closed TASK-1001 verification guard.
2. Write RED tests named by TASK-1001.
3. Implement the smallest semantic change for this task only.
4. Run focused tests and required workspace checks.
5. Request independent review before marking complete.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - false # TASK-1001 must replace this guard with exact focused non-zero commands before TASK-1004 implementation starts
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [ ] TASK-1001 replaced the fail-closed guard
  - [ ] RED tests fail before implementation and pass after implementation
  - [ ] Scope did not expand beyond SPEC-076
  - [ ] Diagnostics are asserted where required
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Workflow/typeck binder semantics
