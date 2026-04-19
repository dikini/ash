# TASK-632: Update CHANGELOG.md and PLAN-INDEX

## Status: ✅ Complete

## Description
Update planning and changelog surfaces to reflect completed non-deferred Phase 92 Track E/F work. This task should only close once the implementation tasks it reports are actually complete and verified.

## Specification Reference
- PLAN-BUILTIN-FN
- PLAN-INDEX workflow policy
- Common Changelog policy in AGENTS.md

## Dependencies
- ✅ TASK-627
- ✅ TASK-628
- ✅ TASK-629
- ✅ TASK-630
- ✅ TASK-631A

## Requirements
1. Add/update `CHANGELOG.md` entries for regex builtin migration and cleanup.
2. Update `docs/plan/PLAN-INDEX.md` task rows and phase status honestly.
3. Keep blocked/deferred tasks explicitly blocked/deferred.
4. Ensure task file statuses/checklists match implementation reality.

## Verification Steps
- [x] changelog entries present
- [x] PLAN-INDEX rows updated consistently
- [x] no overclaiming for blocked/deferred tasks
- [x] task-file statuses match reality

## Notes
This reconciliation task is complete once the documentation surfaces honestly
match the already-complete Track E and TASK-631A state. TASK-633 remains the
separate full-workspace verification gate, and TASK-631B remains blocked.
