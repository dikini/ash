# TASK-1845: Review Phase 182 consistency and cross-references

## Description

Review the phase for consistency, stale language, and cross-reference health before closeout.

## Requirements

- Scan for target docs that still imply tower-first target semantics.
- Check PLAN/TASK status consistency.
- Check spec/note indexes.

## Completion criteria

- [x] Review findings are addressed or explicitly recorded as future work.
- [x] PLAN/TASK/index status surfaces are consistent.

## Evidence

- Reviewed `SPEC-095b`, `SPEC-098c`, `NOTE-019`, `SPEC-INDEX`, `NOTE-INDEX`, `PLAN-182`, `PLAN-INDEX`, and TASK-1837 through TASK-1846 for stale tower-first wording and status drift.
- Addressed stale ambient-monad/bind wording in `NOTE-019` and explicit `do:K` target-foundation wording in `SPEC-095b` and `SPEC-098c`.
- Verification: `python3 tools/docs/validate_orientation_indexes.py --self-test` passed.

## Depends on

- TASK-1839 through TASK-1844.
