# TASK-1796: Classify every `Value::List` reference before removal

## Status: ✅ Complete

## Description

Build a precise removal map for `Value::List`. Each reference must be classified as semantic authority, compatibility shim, test fixture, docs/changelog history, or dead code before TASK-1797 edits the enum.

## Specification Reference

- [PLAN-176: Deferred Cleanup after Target-Language Redesign](../PLAN-176-DEFERRED-CLEANUP-AFTER-TARGET-REDESIGN.md)
- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-157: List Migration Hardening](../PLAN-157-LIST-MIGRATION-HARDENING.md)

## Dependencies

- ✅ TASK-1795 readiness audit complete

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| TASK-1570 | PLAN-157 | High-risk `Value::List` removal with hundreds of refs | Unknown until audit | Re-evaluate in Phase 176 | Reference classification and removal tests |

## Requirements

### Functional Requirements

1. Produce a checked reference inventory for `Value::List` and related list conversion helpers.
2. Identify the smallest compatibility layer needed to preserve user-facing list literal/JSON behavior through `Cons`/`Nil`.
3. Add failing guards for any semantic reference that still requires design work.
4. Patch TASK-1797 with the exact file list and test targets.

### Property Requirements

- Retired bridges must have both positive visibility tests and negative leakage tests.
- If a prerequisite is still absent, the task must fail closed with a current blocker instead of preserving stale completion language.

## TDD Steps

### Step 1: Inventory references

Run exact searches for `Value::List`, list literal conversion, JSON conversion, pattern matching, stdlib list helpers, and test constructors.

### Step 2: Classify and assign owners

Write a table grouping references by crate and semantic role.

### Step 3: Create RED tests or guards

Add or specify tests that fail while `Value::List` is still semantic authority, unless TASK-1795 found removal blocked.

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
  - python3 -c 'from pathlib import Path; s=Path("docs/plan/tasks/TASK-1797-remove-value-list-runtime-variant.md").read_text(); assert "TASK-1796" in s'
  - git diff --check
checklist:
  - [x] Reference inventory complete
  - [x] TASK-1797 patched with exact file owners
```

## Dependencies for Next Task

This task feeds the following Phase 176 tasks according to the dependency table in PLAN-176.

## Notes

The original TASK-1570 estimated hundreds of references. Do not start removal until this map is current.

Completion evidence: `docs/audit/PHASE-176-deferred-cleanup-readiness.md` records the live classification. After the first TASK-1797 slice, Rust source contains 201 `Value::List(` references: 200 constructor-position compatibility calls and one serde helper enum arm, with no pattern-position semantic references remaining.
