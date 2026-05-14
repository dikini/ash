# TASK-886: DESIGN-034 Gap Ownership and PLAN-106 Reconciliation

## Status: ✅ Complete

## Description

Reconcile DESIGN-034 §16.9 after SPEC-A through SPEC-H implementation by distinguishing closed substrate from explicit future packets, repairing PLAN-106 status drift, registering the deferred DESIGN-034 gap backlog, and adding a changelog entry.

## Specification Reference

- [DESIGN-034 §16.9](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#169-cross-packet-implementation-gaps-to-plan-explicitly)
- [PLAN-106](../PLAN-106-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
- [PLAN-113](../PLAN-113-DESIGN-034-DEFERRED-TYPE-COMPUTATION-GAPS.md)

## Dependencies

- ✅ TASK-884: Phase 116 review remediation
- ✅ TASK-885: Local gate resource hygiene

## Requirements

1. Preserve DESIGN-034's historical §16.9 gap list while adding current ownership/status after Phases 109-116.
2. Reconcile PLAN-106 task rows and completion checklist with completed task files and PLAN-INDEX status.
3. Add explicit future task owners for promoted data kinds, type holes/partial application, constructor-kinded/HKT binders, and pattern/exhaustiveness alias canonicalization.
4. Register the new backlog in PLAN-INDEX.
5. Update CHANGELOG.md.
6. Run scoped docs validation and git status checks.

## Dispatch

```
agent: hermes
reasoning: medium
max_turns: 12
toolsets: [terminal, file]
```

## Verification

```
strictness: clean
commands:
  - git diff --check
  - inline scoped docs/task/link validation over DESIGN-034, PLAN-106, PLAN-113, PLAN-INDEX, TASK-886 through TASK-891, and CHANGELOG.md
  - cargo test -p ash-parser --test task_874_proposition_surface task_874_parses_multi_argument_interface_bound_proposition_tail
  - cargo test -p ash-typeck --test task_875_proposition_environment task_875_lowers_multi_argument_interface_bound_proposition_terms
checklist:
  - [x] DESIGN-034 status note added without deleting historical gap list
  - [x] PLAN-106 task table and checklist reconciled
  - [x] PLAN-113 backlog created
  - [x] TASK-887 through TASK-890 future owners created
  - [x] TASK-891 test-hardening owner created and implemented
  - [x] CHANGELOG.md updated
```

## Notes

This task closes ownership/documentation gaps only. It deliberately does not implement promoted data constructors, source type holes, HKT binders, user-defined Monad, or pattern/exhaustiveness canonicalization rollout.
