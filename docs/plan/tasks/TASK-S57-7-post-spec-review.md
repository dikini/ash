# TASK-S57-7: Post-SPEC-Update Review of Phase 57B Tasks

## Status: ✅ Complete

## Description

After Phase 57A SPEC updates completed, review all Phase 57B implementation tasks for validity against the updated specifications. Syntax and semantics changes in 57A invalidated specific task content, assumptions, and acceptance criteria in a subset of the downstream 57B plans.

## Scope

Review each 57B task for alignment with completed 57A SPEC updates:

### Review Checklist

**For each 57B task (359-369):**

- [x] **Syntax validity**: Does task use normative syntax from 57A?
- [x] **API alignment**: Does task reference correct runtime APIs (Engine, not fictional Runtime)?
- [x] **Spec citations**: Are "Spec:" fields updated from MCE-001 to actual SPEC citations?
- [x] **Type consistency**: Do types match across tasks (especially RuntimeError style)?
- [x] **Acceptance criteria**: Are tests implementable with current AST/surface?
- [x] **Crate names**: Does task reference existing crates (ash-std package under `std/`, not fictional locations)?

### Review Outcome

| Task | Outcome | Notes |
|------|---------|-------|
| TASK-359 | Updated | Corrected stdlib source paths and replaced stale capability-usage examples with `cap Args` plus explicit observation |
| TASK-360 | Updated | Corrected stdlib source paths to `std/src/...` |
| TASK-361 | Updated | Replaced stale method-style/interface assumptions with the canonical `cap Args` / `observe Args 0` model |
| TASK-362 | Updated | Replaced stale `capability Args` parameter usage with `cap Args` and corrected stdlib paths |
| TASK-363a | Updated | Corrected stdlib-loading wording and example entry signature to match S57-4..S57-6 |
| TASK-363b | Updated | Replaced stale `capability X` wording with `cap X` and made the return contract exact |
| TASK-363c | Updated | Added missing imports in example snippets while preserving the existing bootstrap design |
| TASK-364 | Updated | Added missing imports in example snippets while preserving the S57-6-aligned contract |
| TASK-365 | Updated | Added missing imports in example snippets while preserving the S57-2/S57-3/S57-6-aligned exit behavior |
| TASK-366 | Updated | Replaced stale capability-injection wording with `cap Args` and stdlib-root language |
| TASK-367 | Reviewed, no change | Already anchored to SPEC-005/SPEC-021/SPEC-003/022 |
| TASK-368a | Updated | Replaced stale method-style `Args` tests with explicit `observe Args` examples |
| TASK-368b | Updated | Replaced stale `capability Stdout` parameter usage and method-style output example |
| TASK-369 | Updated | Replaced stale tuple-style `RuntimeError` example with record-style wording |

### Validation Gates

Before any 57B task begins implementation:

1. **SPEC completion verified**: All blocking 57A tasks show ✅ Complete
2. **This review task complete**: TASK-S57-7 shows ✅ Complete
3. **Task-specific validation**: Implementer verifies task content against updated SPEC

### Deliverables

- [x] Review report: List of tasks requiring updates
- [x] Updated task files: All stale 57B tasks aligned with 57A SPEC
- [x] Validation checklist: Signed off by reviewer

## Related

- All TASK-S57-*: SPEC updates that may affect 57B
- All TASK-3[5-9]*: Implementation tasks to review

## Est. Hours: 2-3

## Blocking

- All 57B tasks should verify this task is complete before starting.
- Individual 57B task ordering and implementation dependencies still apply after this review.

## Completion Summary

This review audited TASK-359 through TASK-369 against the completed S57-1 through S57-6 specs.
The review corrected stale usage-site capability syntax, method-style capability examples,
entry-workflow signature assumptions, stdlib source-path references, and outdated 57A-blocked
status language where necessary.

After these updates, the remaining 57B tasks are aligned to the normative SPEC set and may be
implemented according to their ordinary dependency order rather than waiting on unresolved 57A
spec questions.
