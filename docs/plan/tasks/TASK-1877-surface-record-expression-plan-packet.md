# TASK-1877: Surface Record Expression Plan Packet

**Status:** Complete
**Plan:** [PLAN-187](../PLAN-187-SURFACE-RECORD-EXPRESSIONS.md)

## Description

Create the Phase 187 planning packet for structural record expressions in function-first Ash.

## Requirements

- Add a Phase 187 plan with scope, non-goals, tasks, acceptance criteria, and RED evidence.
- Add task files before implementation work begins.
- Index Phase 187 in `PLAN-INDEX.md`.
- Keep language aligned with target Ash: records are ordinary expressions, rows are requirements,
  and workflow syntax remains compatibility/runtime-profile material.

## TDD Steps

1. RED: Record the current CLI parse failure for a structural record expression fixture.
2. GREEN: Create the plan/task packet and index it before implementation.

## Completion Checklist

- [x] Plan file added.
- [x] Task files added.
- [x] PLAN-INDEX updated.
- [x] RED evidence recorded.

## Evidence

- RED: `cargo run -q -p ash-cli -- check` on the structural record fixture failed with `parse error: Parsing Error: ContextError`.
