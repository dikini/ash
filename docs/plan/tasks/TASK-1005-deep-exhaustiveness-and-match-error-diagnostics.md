# TASK-1005: Match exhaustiveness and missing-witness diagnostics

## Status: 📝 Planned

## Description

Harden ordinary `match` exhaustiveness diagnostics and close audit-discovered gaps in nested/product coverage without expanding beyond ordinary ADTs.

## Specification Reference

- [SPEC-076](../../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
- [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)

## Dependencies

- ✅ TASK-1000 packet exists
- 📝 TASK-1001 audit gate must complete and patch this task before implementation starts

## Requirements

1. Preserve SPEC-076 non-goals and decision gates.
2. Add RED tests for missing constructors, wildcard/default coverage, wildcard-only matches over open or non-ADT scrutinees, blocked constructor-specific canonicalization, wrong constructor identity, nested/product coverage gaps, duplicate or unreachable arms where currently supported, and nested field witness rendering identified by the audit.
3. Preserve SPEC-068 alias/projection behavior and universal wildcard/default behavior.
4. Do not infer constructors under neutral or rigid type heads or treat constructor-specific coverage over blocked universes as exhaustive by guesswork.
5. Ensure diagnostics include scrutinee type, missing witness or blocked reason, span, and likely fix.

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
  - false # TASK-1001 must replace this guard with exact focused non-zero commands before TASK-1005 implementation starts
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

Match/exhaustiveness semantics
