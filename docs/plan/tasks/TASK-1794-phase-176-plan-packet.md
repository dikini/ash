# TASK-1794: Create the Phase 176 deferred-cleanup planning packet

## Status: ✅ Complete

## Description

Create and register the Phase 176 planning packet for deferred cleanup candidates after the target-language redesign. This is documentation/planning only and does not implement Rust behavior.

## Specification Reference

- [PLAN-176: Deferred Cleanup after Target-Language Redesign](../PLAN-176-DEFERRED-CLEANUP-AFTER-TARGET-REDESIGN.md)
- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-157: List Migration Hardening](../PLAN-157-LIST-MIGRATION-HARDENING.md)

## Dependencies

- ✅ Phase 175 closeout committed on `main`
- ✅ Existing Phase 155 number is already occupied by let destructors; this packet uses the next available phase number, 176.

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| TASK-1570 | PLAN-157 | High-risk `Value::List` removal with hundreds of refs | Unknown until audit | Re-evaluate in Phase 176 | Reference classification and removal tests |
| TASK-1580 | PLAN-158 | Needed power-tower lifting / pure-vs-Act distinction | Unknown until audit | Re-evaluate after target redesign | Closure lookup positive and effect-leakage negative tests |
| TASK-1511 recursive combinators | PLAN-151/TASK-1511 | Self-referential values and closure/language limits | Unknown until audit | Implement or re-scope | Final-surface QuickCheck combinator fixtures |
| Phase 152 status drift | PLAN-152 vs PLAN-INDEX | Historical status drift | Audit only | Reconcile after code decisions | Plan/task/index consistency check |

## Requirements

### Functional Requirements

1. Create `PLAN-176-DEFERRED-CLEANUP-AFTER-TARGET-REDESIGN.md` with scope, gates, tasks, and verification baseline.
2. Create TASK-1794 through TASK-1802 with dependencies, deferral reconciliation, dispatch metadata, and verification commands.
3. Register Phase 176 in PLAN-INDEX progress and detail sections.
4. Add a CHANGELOG planning entry under `[Unreleased]`.

### Property Requirements

- Retired bridges must have both positive visibility tests and negative leakage tests.
- If a prerequisite is still absent, the task must fail closed with a current blocker instead of preserving stale completion language.

## TDD Steps

### Step 1: Inspect current planning state

Read PLAN-151, PLAN-152, PLAN-157, PLAN-158, PLAN-175, PLAN-INDEX, and CHANGELOG before assigning globally unique task IDs.

### Step 2: Write planning artifacts

Create the plan and task files with conservative audit-first scope.

### Step 3: Register planning surfaces

Update PLAN-INDEX and CHANGELOG after all task files exist.

### Step 4: Verify structure

Run structural checks that every task link resolves and PLAN-INDEX/CHANGELOG mention Phase 176.

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
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] Plan file exists
  - [x] Task files TASK-1794 through TASK-1802 exist
  - [x] PLAN-INDEX row and phase section exist
  - [x] CHANGELOG entry exists
```

## Dependencies for Next Task

This task feeds the following Phase 176 tasks according to the dependency table in PLAN-176.

## Notes

This task deliberately does not reuse Phase 155 because `PLAN-155-LET-DESTRUCTORS.md` is already complete.
