# TASK-1795: Audit deferred cleanup candidates and prerequisite substrate

## Status: ✅ Complete

## Description

Audit the live code and docs for each deferred cleanup candidate before any implementation task changes Rust code. The audit must decide which items are now unblocked by the target-language redesign and which still need separate prerequisite work.

## Specification Reference

- [PLAN-176: Deferred Cleanup after Target-Language Redesign](../PLAN-176-DEFERRED-CLEANUP-AFTER-TARGET-REDESIGN.md)
- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-157: List Migration Hardening](../PLAN-157-LIST-MIGRATION-HARDENING.md)

## Dependencies

- ✅ TASK-1794 planning packet exists

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| TASK-1570 | PLAN-157 | High-risk `Value::List` removal with hundreds of refs | Unknown until audit | Re-evaluate in Phase 176 | Reference classification and removal tests |
| TASK-1580 | PLAN-158 | Needed power-tower lifting / pure-vs-Act distinction | Unknown until audit | Re-evaluate after target redesign | Closure lookup positive and effect-leakage negative tests |
| TASK-1511 recursive combinators | PLAN-151/TASK-1511 | Self-referential values and closure/language limits | Unknown until audit | Implement or re-scope | Final-surface QuickCheck combinator fixtures |
| Phase 152 status drift | PLAN-152 vs PLAN-INDEX | Historical status drift | Audit only | Reconcile after code decisions | Plan/task/index consistency check |

## Requirements

### Functional Requirements

1. Search and classify live references for `Value::List`, module-level callable lookup inside closures, QuickCheck recursive combinators, and Phase 152 status drift.
2. Create an audit artifact under `docs/audit/` or patch PLAN-176 with a compact readiness table.
3. Patch TASK-1796 through TASK-1801 if the live substrate differs from this plan.
4. Do not implement cleanup code in this task.

### Property Requirements

- Retired bridges must have both positive visibility tests and negative leakage tests.
- If a prerequisite is still absent, the task must fail closed with a current blocker instead of preserving stale completion language.

## TDD Steps

### Step 1: Run read-only searches

Use `search_files`/LSP and targeted reads to locate real call sites. Record counts and owners.

### Step 2: Classify prerequisites

For each candidate, mark substrate present, absent, or ambiguous. Name the exact blocker for absent/ambiguous cases.

### Step 3: Patch downstream task scope

If a candidate is still blocked, patch its task with a split/defer gate instead of leaving implementation instructions that cannot pass.

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
checklist:
  - [x] Readiness table exists
  - [x] Every downstream cleanup task has an audit disposition
  - [x] No Rust code changed in the audit task unless explicitly justified
```

## Dependencies for Next Task

This task feeds the following Phase 176 tasks according to the dependency table in PLAN-176.

## Notes

This task is the guardrail against repeating historical overclaims.

Completion evidence: `docs/audit/PHASE-176-deferred-cleanup-readiness.md` records the readiness table and downstream dispositions. The audit was performed before and alongside the first TASK-1797 migration slice; Rust changes belong to TASK-1797, not this audit task.
