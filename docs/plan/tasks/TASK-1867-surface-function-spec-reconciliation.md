# TASK-1867: Surface Function Spec Reconciliation

**Status:** Complete
**Plan:** [PLAN-185](../PLAN-185-SURFACE-FUNCTION-LANGUAGE.md)

## Description

Reconcile specs and orientation indexes after adding executable `fn main` entry support.

## Requirements

- Update target grammar/lowering/IR/semantics specs if they imply `workflow` remains the target core language path.
- Update `SPEC-INDEX.md` and `NOTE-INDEX.md` read paths.
- Avoid legacy authority vocabulary and tower-as-core wording.

## TDD Steps

1. Search target specs and notes for stale workflow/core-language wording.
2. Update only current target docs and orientation indexes.
3. Run orientation index and docs gate checks.

## Completion Checklist

- [x] Specs/indexes reconciled.
- [x] Docs gate passes.
- [x] Evidence recorded.

## Verification Evidence

- Updated `SPEC-095b`, `SPEC-098c`, `SPEC-INDEX.md`, and `NOTE-INDEX.md` to route Phase 185 through function-first entry syntax and workflow compatibility/profile handling.
